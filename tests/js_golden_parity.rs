//! JS backend differential harness.
//!
//! Every golden program (tests/golden/**/*.braw wi' a .expected file) is compiled
//! tae JavaScript, run under node, an' its stdout compared wi' the .expected file
//! (which is the interpreter's output).
//!
//! Cases that are known tae diverge are listed in tests/js_parity_allowlist.txt,
//! one path per line followed by `# reason`. The list may only shrink: the test
//! fails on ony mismatch no' in the list AND on ony listed case that now passes.
//!
//! Skipped (no' run at all): programs marked SKIP_INTERPRETER (their .expected is
//! no' the interpreter's output), programs marked SKIP_JS, an' programs that
//! `fetch` other modules (the JS backend emits a plain `require` an' has nae
//! module loader fer .braw files).
//!
//! Tae regenerate the allowlist from the current results (then review the diff):
//!     MDH_JS_PARITY_BLESS=1 cargo test --test js_golden_parity

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const GOLDEN_DIR: &str = "tests/golden";
const ALLOWLIST: &str = "tests/js_parity_allowlist.txt";
const NODE_BASE_SECS: u64 = 40;

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
    } else if source.contains("SKIP_JS") {
        Some("marked SKIP_JS")
    } else if source
        .lines()
        .any(|line| line.trim_start().starts_with("fetch "))
    {
        Some("imports a module")
    } else {
        None
    }
}

/// First line o' node's stderr that names the error, trimmed fer the allowlist.
fn summarise_node_error(stderr: &str) -> String {
    let line = stderr
        .lines()
        .map(str::trim)
        .find(|l| l.contains("Error:") || l.starts_with("Error"))
        .or_else(|| stderr.lines().map(str::trim).find(|l| !l.is_empty()))
        .unwrap_or("no stderr");
    line.chars().take(100).collect::<String>().replace('#', "")
}

fn run_node(js_path: &Path, node_timeout: Duration) -> Result<String, String> {
    let mut child = Command::new("node")
        .arg(js_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start node: {e}"))?;

    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf);
        buf
    });

    let deadline = Instant::now() + node_timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(format!("waiting on node failed: {e}")),
        }
    };
    let out = String::from_utf8_lossy(&out_reader.join().unwrap_or_default()).to_string();
    let err = String::from_utf8_lossy(&err_reader.join().unwrap_or_default()).to_string();

    match status {
        None => Err(format!("timed out after {}s", node_timeout.as_secs())),
        Some(status) if !status.success() => Err(format!(
            "node exit {}: {}",
            status.code().unwrap_or(-1),
            summarise_node_error(&err)
        )),
        Some(_) => Ok(out),
    }
}

/// Run one golden case under JS. Ok(()) on a match, Err(short reason) otherwise.
fn run_case(braw: &Path, js_path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(braw).map_err(|e| format!("read failed: {e}"))?;
    let expected = fs::read_to_string(braw.with_extension("expected"))
        .map_err(|e| format!("read expected failed: {e}"))?;
    let js = mdhavers::compile_to_js(&source).map_err(|e| {
        let msg: String = e.to_string().chars().take(100).collect();
        format!("compile error: {}", msg.replace('#', ""))
    })?;
    fs::write(js_path, js).map_err(|e| format!("write failed: {e}"))?;
    let node_timeout = case_timeout(
        braw.file_stem().and_then(|s| s.to_str()).unwrap_or(""),
        NODE_BASE_SECS,
    );
    let actual = run_node(js_path, node_timeout)?;
    if actual.trim() == expected.trim() {
        Ok(())
    } else {
        Err("output mismatch".to_string())
    }
}

/// path -> reason
fn read_allowlist() -> BTreeMap<String, String> {
    let text = fs::read_to_string(ALLOWLIST).unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| match l.split_once('#') {
            Some((path, reason)) => (path.trim().to_string(), reason.trim().to_string()),
            None => (l.to_string(), String::new()),
        })
        .collect()
}

fn write_allowlist(failures: &BTreeMap<String, String>) {
    let mut text = String::from(
        "# Golden programs whose JS output does no' yet match the interpreter.\n\
         # Checked by tests/js_golden_parity.rs. This list may only shrink: fix a case,\n\
         # then delete its line (the test fails while a listed case passes).\n\
         # Regenerate: MDH_JS_PARITY_BLESS=1 cargo test --test js_golden_parity\n",
    );
    for (path, reason) in failures {
        text.push_str(&format!("{path} # {reason}\n"));
    }
    fs::write(ALLOWLIST, text).expect("write allowlist");
}

#[test]
fn js_golden_parity() {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!(
            "SKIPPED js_golden_parity: `node` is no' on PATH, so the JS backend cannae be checked"
        );
        return;
    }

    let mut tests = Vec::new();
    discover_tests(Path::new(GOLDEN_DIR), &mut tests);
    tests.sort();
    assert!(!tests.is_empty(), "no golden tests found in {GOLDEN_DIR}");

    let dir = tempfile::tempdir().expect("tempdir");
    let js_path = dir.path().join("golden_case.js");

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
        match run_case(braw, &js_path) {
            Ok(()) => passed.push(key),
            Err(reason) => {
                failures.insert(key, reason);
            }
        }
    }

    if std::env::var_os("MDH_JS_PARITY_BLESS").is_some() {
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
        .filter(|(path, _)| !allowlist.contains_key(*path))
        .map(|(path, reason)| format!("  {path}: {reason}"))
        .collect();
    let stale: Vec<String> = allowlist
        .keys()
        .filter(|path| !failures.contains_key(*path))
        .map(|path| format!("  {path}"))
        .collect();

    let mut msg = String::new();
    if !unexpected.is_empty() {
        msg.push_str(&format!(
            "{} golden programs give different output under JS (no' in {ALLOWLIST}):\n{}\n",
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
        "JS golden parity: {} passing, {} allowlisted, {} skipped",
        passed.len(),
        failures.len(),
        skipped
    );
}
