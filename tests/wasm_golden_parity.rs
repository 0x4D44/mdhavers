//! WASM backend differential harness (mirrors tests/js_golden_parity.rs).
//!
//! Every golden program (tests/golden/**/*.braw wi' a .expected file) is compiled
//! tae WAT, run in the wasm_runner host, an' its stdout compared wi' the .expected
//! file (which is the interpreter's output). A compile error, a runtime trap, a
//! host panic or a timeout counts as a failure, same as the JS harness counts a
//! non-zero node exit. Error-output goldens get nae special treatment: only stdout
//! is compared, as in the JS harness.
//!
//! Cases that are known tae diverge are listed in tests/wasm_parity_allowlist.txt,
//! grouped by cause (`# == cause ==` header, then one path per line). The list may
//! only shrink: the test fails on ony mismatch no' in the list AND on ony listed
//! case that now passes.
//!
//! Skipped (no' run at all): programs marked SKIP_INTERPRETER (their .expected is
//! no' the interpreter's output), programs marked SKIP_WASM, an' programs that
//! `fetch` other modules.
//!
//! Tae regenerate the allowlist from the current results (then review the diff):
//!     MDH_WASM_PARITY_BLESS=1 cargo test --features wasm_runner --test wasm_golden_parity
#![cfg(feature = "wasm_runner")]

use mdhavers::wasm_compiler::compile_to_wat;
use mdhavers::wasm_runner::run_wat_captured;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const GOLDEN_DIR: &str = "tests/golden";
const ALLOWLIST: &str = "tests/wasm_parity_allowlist.txt";
const CASE_BASE_SECS: u64 = 60;

/// Goldens that need far more than the usual limit (showcase_blether_lisp runs ~53e9
/// instructions under the debug wasm-run). The limit stays finite so a real hang fails.
const HEAVY_GOLDENS: &[&str] = &["showcase_blether_lisp"];
const HEAVY_FACTOR: u64 = 10;

/// The per-case time limit: `base_secs`, times `HEAVY_FACTOR` for a heavy golden, times
/// `MDH_GOLDEN_TIMEOUT_SCALE` (default 1) for a loaded box (MDH-BUG-FLU-00182).
fn case_timeout(name: &str, base_secs: u64) -> Duration {
    let raw = std::env::var("MDH_GOLDEN_TIMEOUT_SCALE").ok();
    case_timeout_scaled(name, base_secs, timeout_scale(raw.as_deref()))
}

/// `MDH_GOLDEN_TIMEOUT_SCALE` as a number; anything unusable means 1.
fn timeout_scale(raw: Option<&str>) -> f64 {
    raw.and_then(|v| v.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(1.0)
}

fn case_timeout_scaled(name: &str, base_secs: u64, scale: f64) -> Duration {
    let heavy = if HEAVY_GOLDENS.contains(&name) {
        HEAVY_FACTOR
    } else {
        1
    };
    Duration::from_secs_f64((base_secs * heavy) as f64 * scale)
}

/// MDH-BUG-FLU-00182: the limit scales with case weight and with the box's load.
#[test]
fn case_timeout_scales_with_weight_and_load_scale() {
    let plain = case_timeout_scaled("some_light_golden", 60, 1.0);
    let heavy = case_timeout_scaled("showcase_blether_lisp", 60, 1.0);
    assert_eq!(plain, Duration::from_secs(60));
    assert!(
        heavy >= plain * 10,
        "a heavy golden must get a far larger limit ({heavy:?} vs {plain:?})"
    );
    assert_eq!(
        case_timeout_scaled("some_light_golden", 60, 3.0),
        plain * 3,
        "MDH_GOLDEN_TIMEOUT_SCALE multiplies the limit"
    );
    assert_eq!(
        case_timeout_scaled("showcase_blether_lisp", 60, 2.0),
        heavy * 2
    );
    assert_eq!(timeout_scale(Some("2.5")), 2.5);
    assert_eq!(timeout_scale(Some("junk")), 1.0);
    assert_eq!(timeout_scale(Some("-4")), 1.0);
    assert_eq!(timeout_scale(None), 1.0);
}

fn discover_tests(dir: &Path, tests: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            discover_tests(&path, tests);
        } else if path.extension().is_some_and(|e| e == "braw")
            && path.with_extension("expected").exists()
        {
            tests.push(path);
        }
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn skip_reason(source: &str) -> Option<&'static str> {
    if source.contains("SKIP_INTERPRETER") {
        Some("expected output is no' the interpreter's")
    } else if source.contains("SKIP_WASM") {
        Some("marked SKIP_WASM")
    } else if source
        .lines()
        .any(|line| line.trim_start().starts_with("fetch "))
    {
        Some("imports a module")
    } else {
        None
    }
}

/// Collapse a message intae a stable cause: first line, digits an' quoted bits dropped.
fn normalise(msg: &str) -> String {
    let first = msg
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("no message");
    let mut out = String::new();
    let mut in_quote = false;
    for c in first.chars() {
        match c {
            '\'' | '`' | '"' => {
                if in_quote {
                    out.push('_');
                }
                in_quote = !in_quote;
            }
            _ if in_quote => {}
            c if c.is_ascii_digit() => {
                if !out.ends_with('N') {
                    out.push('N');
                }
            }
            '#' => {}
            c => out.push(c),
        }
    }
    out.chars().take(100).collect::<String>().trim().to_string()
}

/// Run one golden case under WASM. Ok(()) on a match, Err(cause) otherwise.
fn run_case(braw: &Path) -> Result<(), String> {
    let source = fs::read_to_string(braw).map_err(|e| format!("read failed: {e}"))?;
    let expected = fs::read_to_string(braw.with_extension("expected"))
        .map_err(|e| format!("read expected failed: {e}"))?;

    let case_timeout = case_timeout(
        braw.file_stem().and_then(|s| s.to_str()).unwrap_or(""),
        CASE_BASE_SECS,
    );

    // Worker thread: a runaway or panicking program mustnae hang or kill the suite.
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = std::panic::catch_unwind(|| -> Result<String, String> {
            let wat = compile_to_wat(&source)
                .map_err(|e| format!("compile error: {}", normalise(&e.to_string())))?;
            let out = run_wat_captured(&wat)
                .map_err(|e| format!("module did not load: {}", normalise(&e)))?;
            match out.error {
                Some(err) => Err(format!("runtime error: {}", normalise(&err))),
                None => Ok(out.stdout),
            }
        });
        let _ = tx.send(match result {
            Ok(r) => r,
            Err(p) => {
                let msg = p
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default();
                Err(format!("HOST PANIC: {}", normalise(&msg)))
            }
        });
    });
    match rx.recv_timeout(case_timeout) {
        Err(_) => Err(format!("timed out after {}s", case_timeout.as_secs())),
        Ok(Err(e)) => Err(e),
        Ok(Ok(actual)) if actual.trim() == expected.trim() => Ok(()),
        Ok(Ok(_)) => Err("output mismatch".to_string()),
    }
}

fn read_allowlist() -> BTreeSet<String> {
    let text = fs::read_to_string(ALLOWLIST).unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split('#').next().unwrap_or(l).trim().to_string())
        .collect()
}

fn write_allowlist(failures: &BTreeMap<String, String>) {
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (path, cause) in failures {
        groups.entry(cause).or_default().push(path);
    }
    let mut ordered: Vec<_> = groups.into_iter().collect();
    ordered.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));

    let mut text = String::from(
        "# Golden programs whose WASM output does no' yet match the interpreter.\n\
         # Checked by tests/wasm_golden_parity.rs. This list may only shrink: fix a case,\n\
         # then delete its line (the test fails while a listed case passes).\n\
         # Grouped by cause, biggest group first.\n\
         # Regenerate: MDH_WASM_PARITY_BLESS=1 cargo test --features wasm_runner --test wasm_golden_parity\n",
    );
    for (cause, paths) in ordered {
        text.push_str(&format!("\n# == {cause} ({}) ==\n", paths.len()));
        for p in paths {
            text.push_str(p);
            text.push('\n');
        }
    }
    fs::write(ALLOWLIST, text).expect("write allowlist");
}

#[test]
fn wasm_golden_parity() {
    let mut tests = Vec::new();
    discover_tests(Path::new(GOLDEN_DIR), &mut tests);
    tests.sort();
    assert!(!tests.is_empty(), "no golden tests found in {GOLDEN_DIR}");

    let mut skipped = 0;
    let mut passed = Vec::new();
    let mut failures: BTreeMap<String, String> = BTreeMap::new();
    for braw in &tests {
        let key = display_path(braw);
        let source = fs::read_to_string(braw).expect("read golden source");
        if skip_reason(&source).is_some() {
            skipped += 1;
            continue;
        }
        match run_case(braw) {
            Ok(()) => passed.push(key),
            Err(reason) => {
                failures.insert(key, reason);
            }
        }
    }

    if std::env::var_os("MDH_WASM_PARITY_BLESS").is_some() {
        write_allowlist(&failures);
        println!(
            "Blessed {ALLOWLIST}: {} failing, {} passing, {} skipped",
            failures.len(),
            passed.len(),
            skipped
        );
        return;
    }

    let allowlist = read_allowlist();
    let unexpected: Vec<String> = failures
        .iter()
        .filter(|(path, _)| !allowlist.contains(*path))
        .map(|(path, reason)| format!("  {path}: {reason}"))
        .collect();
    let stale: Vec<String> = allowlist
        .iter()
        .filter(|path| !failures.contains_key(*path))
        .map(|path| format!("  {path}"))
        .collect();

    let mut msg = String::new();
    if !unexpected.is_empty() {
        msg.push_str(&format!(
            "{} golden programs give different output under WASM (no' in {ALLOWLIST}):\n{}\n",
            unexpected.len(),
            unexpected.join("\n")
        ));
    }
    if !stale.is_empty() {
        msg.push_str(&format!(
            "{} entries in {ALLOWLIST} now pass (or nae longer exist/run) - delete them:\n{}\n",
            stale.len(),
            stale.join("\n")
        ));
    }
    assert!(msg.is_empty(), "\n{msg}");

    println!(
        "WASM golden parity: {} passing, {} allowlisted, {} skipped",
        passed.len(),
        failures.len(),
        skipped
    );
}
