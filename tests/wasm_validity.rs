//! WASM backend validity tests (MDH-BUG-FLU-00044..00047).
//!
//! - every module `compile_to_wat` accepts must be a valid WASM module;
//! - the golden corpus is compiled, validated, run in the host runner and compared with
//!   its `.expected` output (known mismatches live in `wasm_validity_allowlist.txt`);
//! - unit tests for each bug.
//!
//! Needs the host runner: `cargo test --features wasm_runner --test wasm_validity`.
#![cfg(feature = "wasm_runner")]

use mdhavers::wasm_compiler::{compile_to_wat, AUDIO_IMPORTS};
use mdhavers::wasm_runner::run_wat_captured;
use mdhavers::HaversError;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Parse the WAT and run the full WASM validator over it.
fn validate_wat(wat_text: &str) -> Result<(), String> {
    let bytes = wat::parse_str(wat_text).map_err(|e| format!("WAT parse failed: {e}"))?;
    let engine = wasmtime::Engine::default();
    wasmtime::Module::validate(&engine, &bytes).map_err(|e| format!("validation failed: {e:#}"))
}

/// Compile, validate and run a program; returns captured stdout.
fn run(source: &str) -> String {
    let out = run_full(source);
    if let Some(err) = &out.error {
        panic!("unexpected runtime error: {err}\nstdout: {}", out.stdout);
    }
    out.stdout
}

fn run_full(source: &str) -> mdhavers::wasm_runner::WasmRunOutput {
    let wat_text = compile_to_wat(source).expect("compile_to_wat");
    validate_wat(&wat_text).unwrap_or_else(|e| panic!("{e}\n{wat_text}"));
    run_wat_captured(&wat_text).expect("module should load")
}

fn compile_error(source: &str) -> String {
    match compile_to_wat(source) {
        Err(HaversError::CompileError(message)) => message,
        Err(other) => panic!("expected CompileError for {source:?}, got {other:?}"),
        Ok(_) => panic!("expected CompileError for {source:?}, got Ok"),
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn discover(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            discover(&path, out);
        } else if path.extension().is_some_and(|e| e == "braw") {
            out.push(path);
        }
    }
}

fn braw_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    discover(dir, &mut files);
    files.sort();
    files
}

// ---------------------------------------------------------------------------
// MDH-BUG-FLU-00044: compile_to_wat never returns Ok for an invalid module
// ---------------------------------------------------------------------------

#[test]
fn functions_only_program_exports_a_real_main() {
    let source = "dae add(a, b) {\n    gie a + b\n}\n";
    let wat_text = compile_to_wat(source).unwrap();
    assert!(wat_text.contains("(func $main"), "{wat_text}");
    validate_wat(&wat_text).unwrap();
    assert_eq!(run(source), "");
}

#[test]
fn empty_program_is_a_valid_module() {
    let wat_text = compile_to_wat("").unwrap();
    validate_wat(&wat_text).unwrap();
}

#[test]
fn user_function_named_main_does_not_collide_with_exported_main() {
    let source = "dae main() {\n    gie 7\n}\nblether main()\n";
    assert_eq!(run(source), "7\n");
}

#[test]
fn user_function_named_like_a_host_import_is_mangled() {
    let source = "dae mdh_add(a, b) {\n    gie a * b\n}\nblether mdh_add(6, 7)\n";
    assert_eq!(run(source), "42\n");
}

#[test]
fn default_parameters_are_filled_in_at_the_call_site() {
    let source = "dae add(a, b = 2) {\n    gie a + b\n}\nblether add(40)\nblether add(1, 1)\n";
    assert_eq!(run(source), "42\n2\n");
}

#[test]
fn unknown_callee_is_a_clear_compile_error_naming_the_function() {
    let message = compile_error("blether sieve([1, 2, 3], 0)\n");
    assert!(message.contains("'sieve'"), "{message}");
    let message = compile_error("blether no_such_function(1)\n");
    assert!(message.contains("'no_such_function'"), "{message}");
}

#[test]
fn wrong_argument_count_is_a_compile_error() {
    let too_many = compile_error("dae f(a) {\n    gie a\n}\nblether f(1, 2)\n");
    assert!(too_many.contains("'f'"), "{too_many}");
    let too_few = compile_error("dae f(a) {\n    gie a\n}\nblether f()\n");
    assert!(
        too_few.contains("'f'") && too_few.contains("'a'"),
        "{too_few}"
    );
    let audio = compile_error("soond_stairt(1)\n");
    assert!(audio.contains("'soond_stairt'"), "{audio}");
}

#[test]
fn non_constant_default_is_a_compile_error() {
    let message = compile_error("ken base = 1\ndae f(a, b = base) {\n    gie a\n}\nblether f(1)\n");
    assert!(message.contains("constant"), "{message}");
}

#[test]
fn top_level_variable_used_inside_function_is_a_compile_error() {
    let message = compile_error("ken limit = 10\ndae f() {\n    gie limit\n}\nblether f()\n");
    assert!(message.contains("'limit'"), "{message}");
}

#[test]
fn undeclared_names_are_compile_errors() {
    let read = compile_error("blether zzz\n");
    assert!(read.contains("'zzz'"), "{read}");
    let assign = compile_error("zzz = 1\n");
    assert!(assign.contains("'zzz'"), "{assign}");
    let function_as_value = compile_error("dae f() {\n    gie 1\n}\nken g = f\nblether 1\n");
    assert!(function_as_value.contains("'f'"), "{function_as_value}");
}

#[test]
fn brak_and_haud_outside_a_loop_are_compile_errors() {
    assert!(compile_error("brak\n").contains("brak"));
    assert!(compile_error("haud\n").contains("haud"));
    assert!(compile_error("dae f() {\n    brak\n}\nf()\n").contains("brak"));
    // ...but are fine inside one.
    let source = "ken i = 0\nwhiles i < 5 {\n    i = i + 1\n    gin i == 2 {\n        haud\n    }\n    gin i == 4 {\n        brak\n    }\n    blether i\n}\n";
    assert_eq!(run(source), "1\n3\n");
}

#[test]
fn duplicate_functions_and_parameters_are_compile_errors() {
    let dup_fn = compile_error("dae f() {\n    gie 1\n}\ndae f() {\n    gie 2\n}\n");
    assert!(dup_fn.contains("'f'"), "{dup_fn}");
    let dup_param = compile_error("dae f(a, a) {\n    gie a\n}\n");
    assert!(dup_param.contains("'a'"), "{dup_param}");
}

#[test]
fn recursion_and_forward_calls_are_valid() {
    let source = "dae fact(n) {\n    gin n <= 1 {\n        gie 1\n    }\n    gie n * fact(n - 1)\n}\nblether later()\ndae later() {\n    gie fact(10)\n}\n";
    assert_eq!(run(source), "3628800\n");
}

// ---------------------------------------------------------------------------
// MDH-BUG-FLU-00045: non-ASCII string literals
// ---------------------------------------------------------------------------

#[test]
fn non_ascii_string_literals_are_emitted_as_utf8_bytes() {
    let wat_text = compile_to_wat("blether \"£5\"\n").unwrap();
    assert!(wat_text.contains("\\c2\\a35\\00"), "{wat_text}");
}

#[test]
fn non_ascii_string_literals_round_trip_through_the_runner() {
    let source = "blether \"£5\"\nblether \"café\"\nblether \"€\"\nblether \"日本語 😀\"\n";
    assert_eq!(run(source), "£5\ncafé\n€\n日本語 😀\n");
}

// ---------------------------------------------------------------------------
// MDH-BUG-FLU-00046: single audio import table
// ---------------------------------------------------------------------------

#[test]
fn audio_table_matches_the_interpreter_names() {
    let names: Vec<&str> = AUDIO_IMPORTS.iter().map(|(n, _)| *n).collect();
    assert!(names.contains(&"midi_pit_rin_roond"));
    assert!(!names.contains(&"midi_pit_tune"));
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len(), "duplicate audio import");
}

#[test]
fn every_audio_import_instantiates_and_runs_in_the_host_runner() {
    for (name, arity) in AUDIO_IMPORTS {
        let args = vec!["1"; *arity].join(", ");
        let source = format!("{name}({args})\n");
        let wat_text = compile_to_wat(&source).unwrap_or_else(|e| panic!("{name}: {e}"));
        validate_wat(&wat_text).unwrap_or_else(|e| panic!("{name}: {e}"));
        let out = run_wat_captured(&wat_text).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(out.error, None, "{name}");
    }
}

#[test]
fn midi_pit_rin_roond_runs() {
    let out = run_wat_captured(&compile_to_wat("midi_pit_rin_roond(1, 1)\n").unwrap()).unwrap();
    assert_eq!(out.error, None);
}

// ---------------------------------------------------------------------------
// MDH-BUG-FLU-00047: host runtime semantics
// ---------------------------------------------------------------------------

#[test]
fn integer_arithmetic_matches_the_interpreter() {
    let source = "blether 7 / 2\nblether -7 / 2\nblether 7 % 3\nblether 9007199254740993 + 0\nblether 9007199254740993 - 1\nblether 3 * 4\nblether 7 / 2.0\nblether 1.5 + 1\n";
    assert_eq!(
        run(source),
        "3\n-3\n1\n9007199254740993\n9007199254740992\n12\n3.5\n2.5\n"
    );
}

#[test]
fn string_arithmetic_matches_the_interpreter() {
    let source = "blether \"ab\" * 3\nblether \"n=\" + 4\nblether 4 + \"x\"\n";
    assert_eq!(run(source), "ababab\nn=4\n4x\n");
}

#[test]
fn integer_overflow_is_a_runtime_error() {
    for expr in [
        "9223372036854775807 + 1",
        "-9223372036854775807 - 2",
        "9223372036854775807 * 2",
    ] {
        let out = run_full(&format!("blether 1\nblether {expr}\nblether 2\n"));
        assert_eq!(out.stdout, "1\n", "{expr}");
        let error = out.error.unwrap_or_else(|| panic!("{expr}: no error"));
        assert!(error.contains("overflowed"), "{expr}: {error}");
    }
}

#[test]
fn division_and_modulo_by_zero_are_runtime_errors() {
    for expr in ["1 / 0", "1 % 0", "1.5 / 0", "1 / 0.0"] {
        let out = run_full(&format!("blether 1\nblether {expr}\nblether 2\n"));
        assert_eq!(out.stdout, "1\n", "{expr}");
        let error = out.error.unwrap_or_else(|| panic!("{expr}: no error"));
        assert!(error.contains("divide by zero"), "{expr}: {error}");
    }
}

#[test]
fn type_errors_are_runtime_errors_not_silent_nil() {
    let cases = [
        ("1 - \"a\"", "Cannae subtract"),
        ("\"a\" - 1", "Cannae subtract"),
        ("[1] * 2", "Cannae multiply"),
        ("aye / 2", "Cannae divide"),
        ("1 % 2.5", "Cannae get remainder"),
        ("-\"a\"", "Cannae negate"),
        ("naething + 1", "Cannae add"),
    ];
    for (expr, expected) in cases {
        let out = run_full(&format!("blether 1\nblether {expr}\nblether 2\n"));
        assert_eq!(out.stdout, "1\n", "{expr}");
        let error = out.error.unwrap_or_else(|| panic!("{expr}: no error"));
        assert!(error.contains(expected), "{expr}: {error}");
    }
}

#[test]
fn negating_i64_min_overflows() {
    let out = run_full("blether -(-9223372036854775807 - 1)\n");
    assert!(out.error.unwrap().contains("overflowed"));
}

#[test]
fn dicts_keep_insertion_order() {
    let source = "ken d = {\"c\": 3, \"a\": 1, \"b\": 2}\nd.z = 26\nd.a = 10\nblether d\n";
    for _ in 0..20 {
        assert_eq!(run(source), "{\"c\": 3, \"a\": 10, \"b\": 2, \"z\": 26}\n");
    }
}

#[test]
fn lists_compare_structurally_and_concatenate() {
    let source = "blether [1, 2] == [1, 2]\nblether [1, 2] == [1, 3]\nblether [1] + [2, 3]\n";
    assert_eq!(run(source), "aye\nnae\n[1, 2, 3]\n");
}

#[test]
fn dicts_compare_structurally_decision_f1() {
    let source = "blether {\"a\": 1, \"b\": [1]} == {\"b\": [1], \"a\": 1}\n\
                  blether {\"a\": 1} == {\"a\": 2}\n\
                  blether {\"a\": 1} == {\"a\": 1, \"b\": 2}\n\
                  blether {\"a\": {\"x\": 1}} != {\"a\": {\"x\": 1}}\n\
                  blether {} == {}\n";
    assert_eq!(run(source), "aye\nnae\nnae\nnae\naye\n");
}

#[test]
fn self_referential_dicts_compare_without_hanging_decision_f1() {
    let source = "ken a = {\"n\": 1}\na.me = a\nken b = {\"n\": 1}\nb.me = b\nblether a == b\n\
                  ken c = {\"n\": 2}\nc.me = c\nblether a == c\n";
    assert_eq!(run(source), "aye\nnae\n");
}

#[test]
fn empty_dict_is_falsy_decision_f2() {
    let source = "gin {} { blether \"truthy\" } ither { blether \"falsy\" }\nblether !{}\n";
    assert_eq!(run(source), "falsy\naye\n");
}

// ---------------------------------------------------------------------------
// corpus: everything the backend accepts is a valid module
// ---------------------------------------------------------------------------

#[test]
fn every_accepted_program_is_a_valid_module() {
    let mut files = braw_files(&repo_root().join("tests"));
    files.extend(braw_files(&repo_root().join("examples")));
    assert!(!files.is_empty());

    let mut accepted = 0usize;
    let mut unsupported = 0usize;
    let mut invalid = Vec::new();
    for path in &files {
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        match compile_to_wat(&source) {
            Err(_) => unsupported += 1,
            Ok(wat_text) => {
                accepted += 1;
                if let Err(e) = validate_wat(&wat_text) {
                    invalid.push(format!("{}: {e}", path.display()));
                }
            }
        }
    }
    println!("wasm corpus: {accepted} accepted, {unsupported} unsupported (CompileError)");
    assert!(
        invalid.is_empty(),
        "compile_to_wat returned Ok for invalid modules:\n{}",
        invalid.join("\n")
    );
}

// ---------------------------------------------------------------------------
// golden programs: compile, validate, run, compare with .expected
// ---------------------------------------------------------------------------

/// `path | reason` per line (path relative to the repo root); `#` starts a comment.
fn load_allowlist() -> BTreeMap<String, String> {
    let text = include_str!("wasm_validity_allowlist.txt");
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (path, reason) = line
            .split_once('|')
            .unwrap_or_else(|| panic!("allowlist line needs `path | reason`: {line}"));
        let (path, reason) = (path.trim(), reason.trim());
        assert!(!reason.is_empty(), "allowlist entry needs a reason: {line}");
        assert!(
            map.insert(path.to_string(), reason.to_string()).is_none(),
            "duplicate allowlist entry: {path}"
        );
    }
    map
}

enum GoldenOutcome {
    Unsupported,
    Pass,
    Mismatch(String),
    Invalid(String),
}

fn run_golden(source: String, expected: String) -> GoldenOutcome {
    let wat_text = match compile_to_wat(&source) {
        Ok(w) => w,
        Err(_) => return GoldenOutcome::Unsupported,
    };
    if let Err(e) = validate_wat(&wat_text) {
        return GoldenOutcome::Invalid(e);
    }
    // Run on a worker thread so one runaway program cannot hang the suite.
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(run_wat_captured(&wat_text));
    });
    match rx.recv_timeout(Duration::from_secs(30)) {
        Err(_) => GoldenOutcome::Mismatch("timed out".to_string()),
        Ok(Err(e)) => GoldenOutcome::Mismatch(format!("module did not load: {e}")),
        Ok(Ok(out)) => {
            if let Some(error) = out.error {
                GoldenOutcome::Mismatch(format!("runtime error: {error}"))
            } else if out.stdout.trim() == expected.trim() {
                GoldenOutcome::Pass
            } else {
                GoldenOutcome::Mismatch(format!(
                    "--- expected ---\n{}\n--- actual ---\n{}",
                    expected.trim(),
                    out.stdout.trim()
                ))
            }
        }
    }
}

#[test]
fn golden_programs_the_wasm_backend_accepts_match_their_expected_output() {
    let root = repo_root();
    let allowlist = load_allowlist();
    let mut seen_allowlisted = Vec::new();

    let (mut passed, mut unsupported) = (0usize, 0usize);
    let mut failures = Vec::new();

    for path in braw_files(&root.join("tests").join("golden")) {
        let expected_path = path.with_extension("expected");
        let Ok(expected) = fs::read_to_string(&expected_path) else {
            continue;
        };
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");

        match run_golden(source, expected) {
            GoldenOutcome::Unsupported => unsupported += 1,
            GoldenOutcome::Invalid(e) => failures.push(format!("{rel}: INVALID MODULE: {e}")),
            GoldenOutcome::Pass => {
                passed += 1;
                if allowlist.contains_key(&rel) {
                    failures.push(format!(
                        "{rel}: now passes - remove it from tests/wasm_validity_allowlist.txt"
                    ));
                }
            }
            GoldenOutcome::Mismatch(detail) => {
                if allowlist.contains_key(&rel) {
                    seen_allowlisted.push(rel);
                } else {
                    failures.push(format!("{rel}: MISMATCH\n{detail}"));
                }
            }
        }
    }

    for rel in allowlist.keys() {
        if !root.join(rel).exists() {
            failures.push(format!("{rel}: allowlisted but the file no longer exists"));
        }
    }

    println!(
        "wasm golden corpus: {passed} passed, {} allowlisted mismatches, {unsupported} unsupported (CompileError)",
        seen_allowlisted.len()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

// ---- MDH-BUG-FLU-00047: ordering comparisons and host store growth ----

#[test]
fn ordering_comparisons_on_mismatched_types_raise_the_interpreter_type_error() {
    for (src, want) in [
        ("blether 1 < \"a\"\n", "Cannae compare integer wi' string"),
        ("blether \"a\" <= 1\n", "Cannae compare string wi' integer"),
        ("blether aye > 1\n", "Cannae compare bool wi' integer"),
        (
            "blether naething >= 1\n",
            "Cannae compare naething wi' integer",
        ),
        ("blether [1] < [2]\n", "Cannae compare list wi' list"),
    ] {
        let out = run_full(src);
        let err = out
            .error
            .unwrap_or_else(|| panic!("{src:?}: expected an error"));
        assert!(
            err.contains(&format!("Type error: {want}")),
            "{src:?}: got {err}"
        );
    }
}

#[test]
fn ordering_comparisons_keep_numeric_and_string_semantics() {
    assert_eq!(
        run("blether 1 < 2.5\nblether 3.0 >= 3\nblether \"a\" < \"b\"\nblether \"b\" <= \"a\"\nblether 2 > 1\n"),
        "aye\naye\naye\nnae\naye\n"
    );
}

#[test]
fn long_arithmetic_loop_keeps_the_host_store_bounded() {
    let out = run_full(
        "ken i = 0\nken f = 0.5\nwhiles i < 10000000 {\n    i = i + 1\n    f = f + 0.25\n}\nblether i\nblether f\n",
    );
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "10000000\n2500000.5\n");
    assert!(out.store_size < 1000, "store grew to {}", out.store_size);
}

#[test]
fn inline_numbers_round_trip_at_the_edges() {
    assert_eq!(
        run("blether 4611686018427387903 + 1\nblether -4611686018427387904 - 1\nblether 9223372036854775807\nblether 1.0e300 * 10.0 > 1.0e300\nblether 0.0 - 0.0\nblether -2.5 * 2\n"),
        "4611686018427387904\n-4611686018427387905\n9223372036854775807\naye\n0\n-5\n"
    );
}

// ---- MDH-BUG-FLU-00092: the host store frees what the guest can no longer reach ----

#[test]
fn long_string_building_loop_keeps_the_host_store_bounded() {
    let out = run_full(
        "ken i = 0\nken last = \"\"\nwhiles i < 20000 {\n    ken s = \"item \" + i\n    ken l = [s, i, {\"k\": s}]\n    last = s + \"!\"\n    i = i + 1\n}\nblether last\n",
    );
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "item 19999!\n");
    assert!(
        out.peak_store_size < 1000,
        "store peaked at {}",
        out.peak_store_size
    );
}

#[test]
fn numeric_statements_skip_their_host_scopes() {
    // Only the loop itself opens a scope; its numeric body and condition do not.
    let wat = compile_to_wat(
        "ken i = 0\nken f = 0.5\nwhiles i < 10 {\n    i = i + 1\n    f = f + 0.25\n}\nblether i\n",
    )
    .expect("compile_to_wat");
    assert_eq!(wat.matches("(call $mdh_scope_enter)").count(), 1, "{wat}");
    assert_eq!(wat.matches("(call $mdh_scope_next)").count(), 0, "{wat}");
}

#[test]
fn a_local_that_can_hold_a_string_keeps_its_scopes() {
    // `u` is assigned a string variable, so `u + t` can allocate and must be reclaimed
    // each iteration rather than piling up in the loop's scope.
    let out = run_full(
        "ken t = \"ab\"\nken u = 0\nken i = 0\nwhiles i < 5000 {\n    u = t\n    u = u + t\n    i = i + 1\n}\nblether u\n",
    );
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "abab\n");
    assert!(
        out.peak_store_size < 1000,
        "store peaked at {}",
        out.peak_store_size
    );
}

#[test]
fn reclamation_keeps_everything_still_reachable() {
    // Return values fae nested scopes, brak/haud/gie leaving scopes early, young values
    // stored intae an old dict (the write barrier), and wide ints re-interned after
    // their slots were freed.
    let src = "\
dae build(n) {
    ken acc = \"\"
    ken j = 0
    whiles aye {
        j = j + 1
        gin j > n {
            brak
        }
        ken big = 4611686018427387000 + j
        gin j % 2 == 0 {
            haud
        }
        gin big > 0 {
            acc = acc + \"x\" + j
        }
    }
    gie acc
}
dae find(n) {
    ken k = 0
    whiles k < 100 {
        gin k == n {
            gie \"found \" + k
        }
        k = k + 1
    }
    gie \"nane\"
}
ken box = {\"v\": \"start\", \"l\": [1]}
ken i = 0
whiles i < 300 {
    box.v = build(5) + find(3)
    box.l = [box.v, [find(i)]]
    i = i + 1
}
blether box.v
blether box.l
blether build(3) + \"/\" + find(200)
";
    let out = run_full(src);
    assert_eq!(out.error, None);
    assert_eq!(
        out.stdout,
        "x1x3x5found 3\n[x1x3x5found 3, [nane]]\nx1x3/nane\n"
    );
    assert!(
        out.peak_store_size < 1000,
        "store peaked at {}",
        out.peak_store_size
    );
}

// ---------------------------------------------------------------------------
// Builtins run the interpreter's ain natives through `__mdh_builtinN` host imports
// ---------------------------------------------------------------------------

#[test]
fn builtins_run_the_interpreters_natives() {
    let src = "\
ken d = {\"a\": 1, \"b\": [2]}
blether len([1, 2, 3])
blether len(\"hello\")
blether upper(\"braw\")
blether keys(d)
blether tae_string(42) + \"!\"
blether tae_string(d)
";
    assert_eq!(
        run(src),
        "3\n5\nBRAW\n[a, b]\n42!\n{\"a\": 1, \"b\": [2]}\n"
    );
}

#[test]
fn shove_mutates_the_callers_list_through_the_write_barrier() {
    // The list is older than every loop scope, so the young items shoved intae it
    // survive only if the write-back goes through `get_mut`. An alias sees the change.
    let src = "\
ken xs = []
ken alias = xs
ken i = 0
whiles i < 2000 {
    shove(xs, [i, tae_string(i) + \"!\"])
    i = i + 1
}
blether len(alias)
blether yank(xs)
blether len(xs)
blether heid(xs)
";
    let out = run_full(src);
    assert_eq!(out.error, None, "stdout: {}", out.stdout);
    assert_eq!(out.stdout, "2000\n[1999, 1999!]\n1999\n[0, 0!]\n");
}

#[test]
fn builtin_results_are_reclaimed_like_any_young_value() {
    let src = "\
ken i = 0
ken last = \"\"
whiles i < 20000 {
    last = upper(tae_string(i) + \"x\")
    i = i + 1
}
blether last
";
    let out = run_full(src);
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "19999X\n");
    assert!(
        out.peak_store_size < 1000,
        "store peaked at {}",
        out.peak_store_size
    );
}

#[test]
fn builtin_error_is_the_interpreters_message() {
    let out = run_full("blether \"afore\"\nblether upper(5)\nblether \"efter\"\n");
    assert_eq!(out.stdout, "afore\n");
    let err = out.error.expect("upper(5) should fail");
    assert!(err.contains("upper() expects a string"), "{err}");
}

#[test]
fn builtin_arity_and_higher_order_builtins_are_compile_errors() {
    let message = compile_error("blether len(1, 2)\n");
    assert!(message.contains("'len'"), "{message}");
    let message = compile_error("blether gaun([1, 2], 0)\n");
    assert!(message.contains("'gaun'"), "{message}");
}

#[test]
fn user_function_shadows_a_builtin() {
    let src = "dae len(x) {\n    gie 99\n}\nblether len([1])\n";
    assert_eq!(run(src), "99\n");
}
