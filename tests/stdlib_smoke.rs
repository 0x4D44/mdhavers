//! Smoke tests for the standard library: every `stdlib/*.braw` module must
//! load through the interpreter wi'oot error, an' the functions fixed under
//! MDH-BUG-FLU-00056 must actually work.

use mdhavers::{parse, Interpreter};
use std::path::PathBuf;

/// Modules that cannae be loaded in a plain test run, wi' the reason.
/// Keep this list short an' justified - every entry is a gap in the smoke test.
const SKIP: &[(&str, &str)] = &[
    // (module name, reason) - filled in only where a load genuinely cannae work
    // here (needs a terminal, a network, or a feature this build lacks).
];

fn stdlib_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("stdlib")
}

fn run(code: &str) -> Result<Vec<String>, String> {
    let program = parse(code).map_err(|e| format!("parse: {e}"))?;
    let mut interp = Interpreter::new();
    interp.set_current_dir(env!("CARGO_MANIFEST_DIR"));
    interp
        .interpret(&program)
        .map_err(|e| format!("run: {e}"))?;
    Ok(interp.get_output().to_vec())
}

#[test]
fn every_stdlib_module_loads() {
    let mut names: Vec<String> = std::fs::read_dir(stdlib_dir())
        .expect("read stdlib dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "braw").unwrap_or(false))
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .collect();
    names.sort();
    assert!(
        names.len() > 30,
        "expected the stdlib modules, got {names:?}"
    );

    let mut failures = Vec::new();
    for name in &names {
        if SKIP.iter().any(|(skipped, _)| skipped == name) {
            continue;
        }
        let code = format!("fetch \"stdlib/{name}\"\n");
        if let Err(e) = run(&code) {
            failures.push(format!("{name}: {e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "stdlib modules that failed tae load:\n{}",
        failures.join("\n")
    );
}

#[test]
fn collections_group_by_works() {
    let out = run(r#"
fetch "stdlib/collections"
ken g = group_by([1, 2, 3, 4, 5], |x| x % 2)
blether len(g["1"])
blether len(g["0"])
"#)
    .expect("group_by should run");
    // The module prints a load banner first; check only what the test printed.
    assert_eq!(out[out.len() - 2..], ["3".to_string(), "2".to_string()]);
}

#[test]
fn functional_sort_by_works() {
    // sort_by builds `|item| {"item": ..., "key": ...}` - a lambda returning a
    // dict literal, which didnae parse afore MDH-BUG-FLU-00066.
    let out = run(r#"
fetch "stdlib/functional"
blether join(sort_by(["ccc", "a", "bb"], |s| len(s)), ",")
"#)
    .expect("sort_by should run");
    assert_eq!(out.last().map(String::as_str), Some("a,bb,ccc"));
}

#[test]
fn uuid_slugify_trims_trailing_dash() {
    let out = run(r#"
fetch "stdlib/uuid"
blether slugify("Hello World ")
"#)
    .expect("slugify should run");
    assert_eq!(out.last().map(String::as_str), Some("hello-world"));
}

#[test]
fn swarm_symphony_json_output_uses_json_stringify() {
    // swarm_symphony/main.braw called the undefined `tae_json` on its JSON
    // output path (MDH-BUG-FLU-00056). The path is chosen by SWARM_JSON, read
    // at load time; nothing else in this test binary reads SWARM_*.
    std::env::set_var("SWARM_JSON", "1");
    let out = run(r#"
fetch "swarm_symphony/main"
ken w = make_world()
output_state(w)
"#);
    std::env::remove_var("SWARM_JSON");
    let out = out.expect("swarm JSON output should run");
    let last = out.last().map(String::as_str).unwrap_or("");
    assert!(
        last.starts_with('{') && last.contains("\"agent_count\""),
        "expected a JSON state line, got {last:?}"
    );
}

/// MDH-BUG-FLU-00060: in the installed layout (`<home>/bin/mdhavers` with
/// `<home>/lib/*.braw`) neither the source tree nor `stdlib/` is reachable, so
/// the prelude and `fetch` must be found through `../lib`.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn installed_layout_finds_prelude_and_modules() {
    use std::process::{Command, Stdio};

    let home = tempfile::tempdir().expect("temp home");
    let bin_dir = home.path().join("bin");
    let lib_dir = home.path().join("lib");
    let work = home.path().join("work");
    for d in [&bin_dir, &lib_dir, &work] {
        std::fs::create_dir_all(d).expect("mkdir");
    }
    let exe = bin_dir.join("mdhavers");
    std::fs::copy(env!("CARGO_BIN_EXE_mdhavers"), &exe).expect("copy binary");
    for f in ["prelude.braw", "collections.braw"] {
        std::fs::copy(stdlib_dir().join(f), lib_dir.join(f)).expect("copy stdlib file");
    }
    std::fs::write(
        work.join("t.braw"),
        "fetch \"collections\"\nblether is_nowt(naething)\nblether len(group_by([1, 2, 3], |x| x % 2))\n",
    )
    .expect("write script");

    let out = Command::new(&exe)
        .arg("t.braw")
        .current_dir(&work)
        .env_remove("MDHAVERS_HOME")
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .output()
        .expect("run installed binary");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "installed-layout run failed: {stderr}\n{stdout}"
    );
    assert!(
        !stderr.contains("cannae find stdlib/prelude"),
        "prelude not found in installed layout: {stderr}"
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines[lines.len() - 2..],
        ["aye", "2"],
        "prelude fn and fetched module should both work: {stdout}"
    );
}
