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
    // A higher-order builtin is kent now; a callback that isnae a function fails at run time.
    let out = run_full("blether sieve([1, 2, 3], 0)\n");
    let error = out.error.expect("calling 0 is a runtime error");
    assert!(error.contains("'0' isnae a function"), "{error}");
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
fn non_constant_default_is_evaluated_in_the_callee() {
    let source = "ken base = 1\ndae f(a, b = base) {\n    gie b\n}\nblether f(1)\n";
    assert_eq!(run(source), "1\n");
}

#[test]
fn top_level_variable_used_inside_function_reads_the_global() {
    assert_eq!(
        run("ken limit = 10\ndae f() {\n    gie limit\n}\nblether f()\n"),
        "10\n"
    );
}

#[test]
fn undeclared_names_are_compile_errors() {
    let read = compile_error("blether zzz\n");
    assert!(read.contains("'zzz'"), "{read}");
    let assign = compile_error("zzz = 1\n");
    assert!(assign.contains("'zzz'"), "{assign}");
    // A dae used as a value is fine now.
    let function_as_value = run("dae f() {\n    gie 1\n}\nken g = f\nblether g()\n");
    assert_eq!(function_as_value, "1\n");
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
            err.contains(&format!("Type error at line 1: {want}")),
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
    // Higher-order builtins are kent now; their arity is still checked up front.
    let message = compile_error("blether gaun([1, 2])\n");
    assert!(message.contains("'gaun'"), "{message}");
}

#[test]
fn user_function_shadows_a_builtin() {
    let src = "dae len(x) {\n    gie 99\n}\nblether len([1])\n";
    assert_eq!(run(src), "99\n");
}

// MDH-BUG-FLU-00117: a builtin call must not convert or scan the whole argument when
// it only needs its size, or the loop below is quadratic.
#[test]
fn len_in_a_loop_over_a_big_list_is_not_quadratic() {
    let source = "ken xs = []\nken i = 0\nwhiles i < 20000 { shove(xs, i)\ni = i + 1 }\n\
                  ken total = 0\nken j = 0\nwhiles j < 20000 { total = total + len(xs)\nj = j + 1 }\n\
                  blether total\n";
    let start = std::time::Instant::now();
    assert_eq!(run(source), "400000000\n");
    assert!(
        start.elapsed() < Duration::from_secs(20),
        "len(xs) loop took {:?}",
        start.elapsed()
    );
}

#[test]
fn len_and_bum_shortcuts_match_the_interpreter() {
    let source = "blether len([1, 2, 3])\nblether len({\"a\": 1})\nblether len(\"héllo\")\n\
                  blether len(creel([1, 2, 2]))\nblether bum([4, 5, 6])\nblether len(\"\")\n";
    assert_eq!(run(source), "3\n1\n5\n2\n6\n0\n");
    let out = run_full("blether bum([])\n");
    assert!(
        out.error.is_some(),
        "bum of an empty list must still be an error"
    );
}

// MDH-BUG-FLU-00119: the host store holds creels.
#[test]
fn creels_returned_by_builtins_live_in_the_host_store() {
    let source = "ken s = creel([3, 1, 2, 3])\nblether s\nblether len(s)\n\
                  blether is_in_creel(s, 2)\nblether is_in_creel(s, 9)\n\
                  blether creel([1, 2]) == creel([2, 1])\nblether creel([1]) == creel([2])\n\
                  blether [creel([1])] == [creel([1])]\n\
                  gin creel([]) { blether \"truthy\" } ither { blether \"falsy\" }\n";
    assert_eq!(
        run(source),
        "creel{\"1\", \"2\", \"3\"}\n3\naye\nnae\naye\nnae\naye\nfalsy\n"
    );
}

#[test]
fn creels_survive_reclamation_in_loops() {
    let source = "ken i = 0\nken n = 0\nwhiles i < 3000 { ken s = creel([i, i + 1])\n\
                  n = n + len(s)\ni = i + 1 }\nblether n\n";
    let out = run_full(source);
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "6000\n");
    assert!(out.peak_store_size < 2000, "peak {}", out.peak_store_size);
}

// ---------------------------------------------------------------------------
// hae_a_bash, hurl, mak_siccar, fer, keek, indexing, slices, f-strings,
// ternaries an' list spreads
// ---------------------------------------------------------------------------

/// The interpreter's output fer a program (on a big stack, like the CLI).
fn interpreted(source: &str) -> String {
    let source = source.to_string();
    let lines = thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(move || mdhavers::run_with_output(&source).map(|(_, out)| out))
        .expect("spawn")
        .join()
        .expect("interpreter panicked")
        .expect("interpreter error");
    lines.iter().map(|l| format!("{l}\n")).collect()
}

/// Run under WASM; the output must be the interpreter's an' `want`. Returns the run.
fn assert_parity(source: &str, want: &str) -> mdhavers::wasm_runner::WasmRunOutput {
    let out = run_full(source);
    assert_eq!(out.error, None, "stdout: {}", out.stdout);
    assert_eq!(
        out.stdout,
        interpreted(source),
        "WASM differs fae the interpreter"
    );
    assert_eq!(out.stdout, want);
    out
}

#[test]
fn try_catch_binds_the_error_and_unwinds_brak_haud_gie_and_nesting() {
    let src = r#"dae safe_div(a, b) {
    hae_a_bash {
        gie a / b
    } gin_it_gangs_wrang e {
        gie "oops"
    }
}
blether safe_div(6, 3)
blether safe_div(1, 0)
ken n = 0
whiles aye {
    n = n + 1
    hae_a_bash {
        gin n == 3 {
            brak
        }
        gin n == 1 {
            haud
        }
        blether n
    } gin_it_gangs_wrang e {
        blether "nae"
    }
}
hae_a_bash {
    hurl "boom"
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    mak_siccar 1 == 2, "sums"
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    hae_a_bash {
        blether [1][5]
    } gin_it_gangs_wrang inner {
        blether "inner: " + inner
        hurl "again"
    }
} gin_it_gangs_wrang outer {
    blether "outer: " + outer
}
blether safe_div(8, 2)
"#;
    assert_parity(
        src,
        "2\noops\n2\nHurled at line 26: boom\nMak siccar failed at line 31! sums\n\
         inner: Hoachin'! Index 5 is oot o' bounds (size is 1) at line 37\n\
         outer: Hurled at line 40: again\n4\n",
    );
    // Every `hae_a_bash` left (by brak, haud, gie or a catch) was closed: an error
    // ootside them still stops the program.
    let out = run_full(&format!("{src}blether 1 / 0\nblether \"efter\"\n"));
    assert!(out.stdout.ends_with("again\n4\n"), "{}", out.stdout);
    assert!(
        out.error
            .as_deref()
            .is_some_and(|e| e.contains("divide by zero")),
        "{:?}",
        out.error
    );
}

#[test]
fn try_catch_frees_the_scopes_an_unwind_abandons() {
    // The error is raised six calls deep; each call's scopes are left open by the
    // unwind an' closed by the host at the catch, keeping only what is still reachable.
    let src = r#"dae deep(n) {
    ken junk = [n, "x" + n]
    gin n == 0 {
        gie 1 / 0
    }
    gie deep(n - 1) + 1
}
ken box = {"v": naething}
ken caught = 0
ken i = 0
whiles i < 2000 {
    hae_a_bash {
        box.v = [i, [i * 2]]
        ken r = deep(5)
        blether "unreachable"
    } gin_it_gangs_wrang e {
        caught = caught + 1
    }
    i = i + 1
}
blether caught
blether box.v
"#;
    let out = assert_parity(src, "2000\n[1999, [3998]]\n");
    assert!(out.peak_store_size < 1000, "peak {}", out.peak_store_size);
}

#[test]
fn hurl_and_mak_siccar_outside_a_catch_stop_the_program() {
    let out = run_full("blether 1\nhurl \"crivvens\"\nblether 2\n");
    assert_eq!(out.stdout, "1\n");
    assert_eq!(out.error.as_deref(), Some("Hurled at line 2: crivvens"));
    let out = run_full("mak_siccar 1 > 2\nblether 2\n");
    assert_eq!(
        out.error.as_deref(),
        Some("Mak siccar failed at line 1! Assertion failed")
    );
}

#[test]
fn fer_loops_walk_ranges_lists_and_strings() {
    let src = r#"ken total = 0
fer i in 1..=4 {
    total = total + i
}
blether total
ken xs = [1, 2, 3]
fer x in xs {
    shove(xs, x * 10)
    gin x == 2 {
        haud
    }
    blether x
}
blether xs
fer c in "abc" {
    gin c == "c" {
        brak
    }
    blether c
}
fer i in 0..2 {
    fer j in 0..2 {
        blether i * 10 + j
    }
}
dae count_to(n) {
    ken out = []
    fer i in 0..n {
        gin i == 3 {
            gie out
        }
        shove(out, i)
    }
    gie out
}
blether count_to(2)
blether count_to(9)
fer i in 5..2 {
    blether "never"
}
"#;
    assert_parity(
        src,
        "10\n1\n3\n[1, 2, 3, 10, 20, 30]\na\nb\n0\n1\n10\n11\n[0, 1]\n[0, 1, 2]\n",
    );
}

#[test]
fn fer_loop_over_an_unwalkable_value_is_the_interpreters_error() {
    let src = "hae_a_bash {\n    fer x in 5 {\n        blether x\n    }\n} \
               gin_it_gangs_wrang e {\n    blether e\n}\n";
    assert_parity(
        src,
        "That's pure mince! Type error at line 2: Cannae iterate ower a integer\n",
    );
}

#[test]
fn fer_loop_garbage_is_swept_each_iteration() {
    let src =
        "ken last = naething\nfer i in 0..20000 {\n    last = [i, \"s\" + i]\n}\nblether last\n";
    let out = run_full(src);
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, interpreted(src));
    assert!(out.peak_store_size < 1000, "peak {}", out.peak_store_size);
}

#[test]
fn keek_matches_literals_ranges_bindings_and_wildcards() {
    let src = r#"dae describe(v) {
    ken out = ""
    keek v {
        whan 1 -> { out = "one" }
        whan "hi" -> { out = "greeting" }
        whan 2..5 -> { out = "small" }
        whan naething -> { out = "nowt" }
        whan other -> { out = "other " + other }
    }
    gie out
}
blether describe(1)
blether describe("hi")
blether describe(4)
blether describe(5)
blether describe(naething)
keek 7 {
    whan _ -> { blether "wild" }
}
hae_a_bash {
    keek 3 {
        whan 1 -> { blether "no" }
    }
} gin_it_gangs_wrang e {
    blether e
}
"#;
    assert_parity(
        src,
        "one\ngreeting\nsmall\nother 5\nnowt\nwild\n\
         That's pure mince! Type error at line 21: Nae match found fer 3\n",
    );
}

#[test]
fn indexing_reads_and_writes_like_the_interpreter() {
    let src = r#"ken xs = [10, 20, [30]]
ken d = {"a": 1}
blether xs[0]
blether xs[-1]
blether "braw"[1]
blether "braw"[-1]
blether d["a"]
ken inner = xs[2]
inner[0] = 99
blether xs
xs[-3] = 7
d["b"] = [1]
d["b"][0] = 2
blether xs
blether d
fer probe in [5, -9] {
    hae_a_bash {
        blether xs[probe]
    } gin_it_gangs_wrang e {
        blether e
    }
}
hae_a_bash {
    blether d["zz"]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether xs[1.5]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    ken s = "abc"
    s[0] = "z"
} gin_it_gangs_wrang e {
    blether e
}
"#;
    assert_parity(
        src,
        "10\n[30]\nr\nw\n1\n[10, 20, [99]]\n[7, 20, [99]]\n{\"a\": 1, \"b\": [2]}\n\
         Hoachin'! Index 5 is oot o' bounds (size is 3) at line 18\n\
         Hoachin'! Index -9 is oot o' bounds (size is 3) at line 18\n\
         Awa' an bile yer heid! 'zz' hasnae been defined yet at line 24\n\
         That's pure mince! Type error at line 29: Cannae index a list wi' a float\n\
         That's pure mince! Type error at line 35: Cannae set index on a string wi' a integer\n",
    );
}

#[test]
fn index_set_keeps_young_values_in_old_containers_alive() {
    // The list an' dict are older than every loop scope; the young values stored in
    // them survive the sweeps only through the write barrier.
    let src = r#"ken xs = [0, 0, 0]
ken d = {}
fer i in 0..3000 {
    xs[i % 3] = [i, "v" + i]
    d["k" + (i % 2)] = "w" + i
}
blether xs
blether d
"#;
    let out = run_full(src);
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, interpreted(src));
    assert!(out.stdout.starts_with("[[2997, "), "{}", out.stdout);
    assert!(out.peak_store_size < 1000, "peak {}", out.peak_store_size);
}

#[test]
fn slices_fstrings_ternaries_and_spreads_match_the_interpreter() {
    let src = r#"ken xs = [0, 1, 2, 3, 4, 5]
blether xs[1:4]
blether xs[:2]
blether xs[4:]
blether xs[::2]
blether xs[::-1]
blether xs[-2:]
blether "bonnie"[1:3]
blether "bonnie"[::-1]
ken name = "Morag"
ken n = 3
blether f"{name} has {n * 2} sheep"
blether f"{xs[0]}-{xs[1]}"
blether gin n > 2 than "many" ither "few"
blether [...xs[:2], 9, ...[7]]
blether [..."ab"][1]
blether xs[1:5:2]
hae_a_bash {
    blether xs[::0]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether 5[1:2]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether [...5]
} gin_it_gangs_wrang e {
    blether e
}
"#;
    assert_parity(
        src,
        "[1, 2, 3]\n[0, 1]\n[4, 5]\n[0, 2, 4]\n[5, 4, 3, 2, 1, 0]\n[4, 5]\non\neinnob\n\
         Morag has 6 sheep\n0-1\nmany\n[0, 1, 9, 7]\nb\n[1, 3]\n\
         That's pure mince! Type error at line 19: Slice step cannae be zero, ya dafty!\n\
         That's pure mince! Type error at line 24: Cannae slice a integer, ya numpty!\n\
         That's pure mince! Type error at line 29: Cannae skail (spread) somethin' that isnae a list or string!\n",
    );
}

#[test]
fn unsupported_nodes_are_named_in_the_compile_error() {
    for (src, kind) in [
        ("ken f = speir \"x\"\n", "The Input expression"),
        ("thing P { x }\n", "The Struct statement"),
        ("ken r = 1..3\n", "The Range expression"),
    ] {
        let err = compile_to_wat(src).expect_err(src).to_string();
        assert!(err.contains(kind), "{src}: {err}");
    }
}

// ---------------------------------------------------------------------------
// function values, closures, higher-order builtins and pipes
// (wrk_docs/2026.10.24 - DES - WASM function values and closures.md)
// ---------------------------------------------------------------------------

#[test]
fn daes_and_builtins_are_values_and_pipes_call_them() {
    let src = r#"dae twice(x) {
    gie x * 2
}
ken f = twice
blether f(4)
blether 5 |> twice
blether "  hi  " |> wheesht |> upper
ken l = len
blether l([1, 2, 3])
blether (twice)(6)
blether 3 |> |n| n + 1
blether f
"#;
    assert_parity(src, "8\n10\nHI\n3\n12\n4\n<dae twice>\n");
}

#[test]
fn closures_share_mutable_variables_with_their_creator() {
    let src = r#"ken x = 1
ken bump = |y| {
    x = x + y
    gie x
}
blether bump(2)
x = 10
blether bump(1)
blether x
dae counter() {
    ken n = 0
    dae step() {
        n = n + 1
        gie n
    }
    gie step
}
ken c = counter()
ken d = counter()
c()
c()
blether c()
blether d()
dae make_adder(a) {
    gie |b| |c| a + b + c
}
blether make_adder(1)(2)(3)
"#;
    assert_parity(src, "3\n11\n11\n3\n1\n6\n");
}

#[test]
fn fer_closures_capture_each_iteration_decision_f3() {
    let src = r#"ken fs = []
fer i in 0..3 {
    shove(fs, || i)
    i = i * 10
}
blether gaun(fs, |f| f())
ken ws = []
ken k = 0
whiles k < 3 {
    ken snap = k
    shove(ws, || snap)
    k = k + 1
}
blether gaun(ws, |f| f())
"#;
    assert_parity(src, "[0, 10, 20]\n[0, 1, 2]\n");
}

#[test]
fn nested_daes_recurse_and_shadow_top_level_daes() {
    let src = r#"dae greet() {
    gie "global"
}
dae outer(n) {
    dae greet() {
        gie "nested"
    }
    dae fact(x) {
        gin x <= 1 {
            gie 1
        }
        gie x * fact(x - 1)
    }
    blether greet()
    gie fact(n)
}
blether outer(5)
blether greet()
"#;
    assert_parity(src, "nested\n120\nglobal\n");
}

#[test]
fn higher_order_builtins_match_the_interpreter() {
    let src = r#"ken xs = [1, 2, 3, 4, 5]
blether gaun(xs, |x| x * x)
blether sieve(xs, |x| x % 2 == 0)
blether tumble(xs, 100, |acc, x| acc - x)
ken seen = []
blether ilk(xs, |x| shove(seen, x + 1))
blether seen
blether hunt(xs, |x| x > 2)
blether hunt(xs, |x| x > 9)
blether ony(xs, |x| x > 4)
blether ony([], |x| aye)
blether aw(xs, |x| x > 0)
blether aw(xs, |x| x > 1)
blether grup_up(xs, |x| gin x % 2 == 0 than "even" ither "odd")
blether pairt_by(xs, |x| x < 3)
dae is_odd(x) {
    gie x % 2 == 1
}
blether sieve(xs, is_odd)
blether gaun(["a", "b"], upper)
"#;
    assert_parity(
        src,
        "[1, 4, 9, 16, 25]\n[2, 4]\n85\nnaething\n[2, 3, 4, 5, 6]\n3\nnaething\naye\nnae\naye\nnae\n\
         {\"odd\": [1, 3, 5], \"even\": [2, 4]}\n[[1, 2], [3, 4, 5]]\n[1, 3, 5]\n[A, B]\n",
    );
}

#[test]
fn a_callback_error_unwinds_to_the_catch_and_frees_the_callback_scopes() {
    let src = r#"ken xs = [1, 2, 0, 4]
ken total = 0
fer round in 0..200 {
    hae_a_bash {
        blether gaun(xs, |x| 10 / x)
    } gin_it_gangs_wrang e {
        total = total + 1
    }
}
blether total
ken safe = |x| {
    hae_a_bash {
        gie 10 / x
    } gin_it_gangs_wrang e {
        gie "oops"
    }
}
blether gaun(xs, safe)
"#;
    let out = assert_parity(src, "200\n[10, 5, oops, 2]\n");
    assert!(out.store_size < 200, "store kept {} slots", out.store_size);
}

#[test]
fn a_callback_error_without_a_catch_is_a_runtime_error() {
    let out = run_full("blether gaun([1, 0], |x| 1 / x)\nblether \"never\"\n");
    let error = out.error.expect("division by zero");
    assert!(error.contains("divide by zero"), "{error}");
    assert_eq!(out.stdout, "");
}

#[test]
fn calling_a_function_value_with_the_wrong_count_is_a_runtime_error() {
    let out = run_full("ken f = |a, b| a + b\nblether f(1)\n");
    let error = out.error.expect("arity error");
    assert!(
        error.contains("expects 2 arguments but ye gave it 1"),
        "{error}"
    );
    let out = run_full("ken n = 3\nblether n(1)\n");
    let error = out.error.expect("not callable");
    assert!(error.contains("'3' isnae a function"), "{error}");
}

#[test]
fn a_loop_making_closures_keeps_the_host_store_bounded() {
    let out = run_full(
        "ken total = 0\nfer i in 0..20000 {\n    ken add = |x| x + i\n    total = total + gaun([1, 2], add)[1]\n}\nblether total\n",
    );
    assert_eq!(out.error, None);
    assert_eq!(out.stdout, "200030000\n");
    assert!(
        out.peak_store_size < 5000,
        "store peaked at {}",
        out.peak_store_size
    );
}

#[test]
fn a_dict_field_holding_a_function_can_be_called() {
    let src = r#"dae make() {
    gie {"f": |x| x + 1, "g": len}
}
ken d = make()
blether d.f(4)
blether d.g("abc")
"#;
    assert_parity(src, "5\n3\n");
}

// MDH-BUG-FLU-00122: a dict shows its keys as the interpreter does, whatever their type.
#[test]
fn dict_display_matches_the_interpreter_for_every_key_type() {
    let src = r#"ken d = {}
d[1] = "int"
d[2.5] = "float"
d[aye] = "bool"
d[nae] = "bool2"
d[naething] = "nil"
d["s"] = "str"
ken k = [1, 2]
d[k] = "list"
dae f() { gie 1 }
d[f] = "fn"
blether d
blether grup_up([1, 2], |x| x % 2)
"#;
    assert_parity(src, &interpreted(src));
}

// MDH-BUG-FLU-00120: host-raised errors name the line, in blocks, loops and callees.
#[test]
fn host_errors_carry_the_statement_line() {
    let src = r#"ken max = 9223372036854775807
dae boom(n) {
    ken pad = 1

    gie n + 9223372036854775807
}
hae_a_bash {
    blether boom(1) + 1
} gin_it_gangs_wrang e {
    blether e
}
ken i = 0
whiles i < 2 {
    i = i + 1
    hae_a_bash {
        blether 1 / 0
    } gin_it_gangs_wrang e {
        blether e
    }
}
hae_a_bash {
    blether 1 < "a"
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether "a" - 1
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    ken r = gaun([1], |x| x)
    blether max * 2
} gin_it_gangs_wrang e {
    blether e
}
"#;
    let expected = interpreted(src);
    assert!(expected.contains("at line 5"), "{expected}");
    assert_parity(src, &expected);
}

// MDH-BUG-FLU-00121: bytes live in the host store and round-trip through builtins.
#[test]
fn bytes_round_trip_through_the_host_store() {
    let src = r#"ken b = bytes(3)
bytes_set(b, 0, 7)
bytes_set(b, 2, 9)
blether b
blether bytes_get(b, 2)
blether bytes_len(bytes_slice(b, 1, 3))
ken c = bytes(3)
bytes_set(c, 0, 7)
bytes_set(c, 2, 9)
blether b == c
blether b == bytes(3)
blether [b, c]
gin bytes(0) { blether "full" } ither { blether "empty" }
gin b { blether "full" } ither { blether "empty" }
"#;
    assert_parity(src, &interpreted(src));
}

// ---- Top-level variables used inside daes: one host cell per global ----

#[test]
fn daes_read_and_write_top_level_variables() {
    let src = r#"ken limit = 10
ken count = 0
ken items = []
dae bump(n) {
    count = count + n
    shove(items, n)
    gie limit + count
}
dae shadow(limit) {
    ken count = 99
    gie limit + count
}
blether bump(1)
limit = 100
blether bump(2)
ken limit = 1000
blether bump(3)
blether shadow(1)
blether count
blether items
ken peek = || count + limit
count = 5
blether peek()
"#;
    assert_parity(src, "11\n103\n1006\n100\n6\n[1, 2, 3]\n1005\n");
}

#[test]
fn a_global_keeps_a_container_built_inside_a_dae() {
    // The value is born in the dae's scopes; only the cell's write barrier keeps it.
    let src = r#"ken keep = naething
dae build(n) {
    ken inner = [n, "kept " + n]
    keep = {"k": inner, "n": [n, n]}
}
build(7)
fer i in 0..3000 {
    ken junk = ["junk", i, [i]]
}
blether keep
dae churn() {
    fer i in 0..3000 {
        keep = [i, "x" + i]
    }
}
churn()
blether keep
"#;
    let out = assert_parity(src, "{\"k\": [7, kept 7], \"n\": [7, 7]}\n[2999, x2999]\n");
    assert!(
        out.peak_store_size < 2000,
        "store peaked at {}",
        out.peak_store_size
    );
}

// ---- Classes: instances in the host store, methods as guest functions ----

const ACCOUNT_CLASSES: &str = r#"ken bonus = 10
kin Account {
    dae init(owner, balance = bonus) {
        masel.owner = owner
        masel.balance = balance
        masel.log = []
    }
    dae deposit(amount, note = "dep " + masel.owner) {
        masel.balance = masel.balance + amount
        shove(masel.log, note)
        gie masel
    }
    dae __pit_thegither__(other) {
        gie Account(masel.owner + "&" + other.owner, masel.balance + other.balance)
    }
    dae __same_as__(other) {
        gie masel.balance == other.balance
    }
    dae __wee_er__(other) {
        gie masel.balance < other.balance
    }
}
kin Savings fae Account {
    dae deposit(amount) {
        gie masel.add_interest(amount * 2)
    }
    dae add_interest(amount, rate = 1) {
        masel.balance = masel.balance + amount * rate
        gie masel
    }
}
dae open(name) {
    gie Account(name)
}
"#;

#[test]
fn classes_construct_dispatch_inherit_and_display_like_the_interpreter() {
    let src = format!(
        "{ACCOUNT_CLASSES}{}",
        r#"ken a = open("Ada")
ken b = Account("Bob", 5)
a.deposit(3).deposit(4, "fower")
blether a.balance
blether a.log
blether b.log
ken s = Savings("Sal")
s.deposit(5)
blether s.balance
ken dep = b.deposit
dep(1)
blether b.balance
blether dep
blether a
blether Account
blether [a, s]
blether tae_string(s)
ken c = a + b
blether c.owner
blether c.balance
blether a == b
blether a == a
blether b < a
blether a != b
ken d = {"acct": a}
d.acct.balance = 99
blether a.balance
blether gaun(["x", "y"], Account)[1].owner
blether whit_kind(a)
ken make = || |x| x + 1
kin Box {
    dae init() { masel.f = |x| x + 100 }
    dae f(x) { gie x + 1 }
    dae getter() { gie || masel.f(1) }
}
blether Box().f(3)
ken f = Box().f
blether f(3)
blether Box().getter()()
"#
    );
    assert_parity(
        &src,
        "17\n[dep Ada, fower]\n[]\n20\n6\n<dae deposit>\n<Account instance>\n<kin Account>\n\
         [<Account instance>, <Savings instance>]\n<Savings instance>\nAda&Bob\n23\nnae\naye\naye\naye\n\
         99\ny\ninstance\n4\n103\n2\n",
    );
}

#[test]
fn class_errors_are_catchable_and_match_the_interpreter() {
    let src = format!(
        "{ACCOUNT_CLASSES}{}",
        r#"ken a = Account("Ada")
hae_a_bash {
    blether a.nope
} gin_it_gangs_wrang err {
    blether "caught nope: " + err
}
hae_a_bash {
    a.deposit()
} gin_it_gangs_wrang err {
    blether "caught deposit: " + err
}
hae_a_bash {
    blether a.missing_method(1)
} gin_it_gangs_wrang err {
    blether "caught missing: " + err
}
blether a.balance
"#
    );
    let want = interpreted(&src);
    assert!(
        want.contains("caught nope: Awa' an bile yer heid! 'nope' hasnae been defined yet at line"),
        "{want}"
    );
    assert!(
        want.contains("caught deposit: That's pure mince! Type error at line"),
        "{want}"
    );
    assert!(
        want.ends_with("hasnae been defined yet at line 47\n10\n"),
        "{want}"
    );
    assert_parity(&src, &want);
    let out = run_full(&format!(
        "{ACCOUNT_CLASSES}Savings(\"S\").add_interest(1, 2, 3)\n"
    ));
    let error = out.error.expect("arity error");
    assert!(
        error.contains("Function 'add_interest' expects 1 tae 2 arguments but ye gave it 3"),
        "{error}"
    );
    let out = run_full("ken x = 3\nkin B fae x {\n    dae init() {}\n}\n");
    let error = out.error.expect("parent is no' a class");
    assert!(error.contains("x isnae a class"), "{error}");
    let message = compile_error("dae f() {\n    gie masel\n}\nblether f()\n");
    assert!(message.contains("'masel'"), "{message}");
}

#[test]
fn instance_fields_survive_store_churn_and_loops_stay_bounded() {
    // The fields are born in `init`'s scopes; only the instance (remembered by the
    // write barrier, then traced through `push_children`) keeps them.
    let src = r#"kin Box {
    dae init(n) {
        masel.items = [n, "item " + n]
        masel.meta = {"n": [n, n]}
    }
    dae show() {
        gie masel.items
    }
}
ken boxes = []
fer i in 0..50 {
    shove(boxes, Box(i))
}
fer j in 0..3000 {
    ken junk = ["junk", j, [j]]
}
blether boxes[7].show()
blether boxes[49].meta
ken total = 0
fer i in 0..20000 {
    ken b = Box(i)
    total = total + b.items[0]
}
blether total
"#;
    let out = assert_parity(src, "[7, item 7]\n{\"n\": [49, 49]}\n199990000\n");
    assert!(
        out.peak_store_size < 5000,
        "store peaked at {}",
        out.peak_store_size
    );
}

// MDH-BUG-FLU-00124: creel members print quoted, as the interpreter shows them.
#[test]
fn creel_display_quotes_every_member() {
    let src = "ken c = creel([\"1\", 2, \"b\"])\nblether c\nblether [creel([])]\n";
    assert_parity(src, "creel{\"1\", \"2\", \"b\"}\n[creel{}]\n");
}

// ---------------------------------------------------------------------------
// residual divergences (MDH-BUG-FLU-00125, MDH-BUG-FLU-00123)
// ---------------------------------------------------------------------------

#[test]
fn dae_defaults_are_evaluated_in_the_callee() {
    let src = r#"ken base = 10
dae f(a, b = a + base, c = [a]) {
    gie [a, b, c]
}
blether f(1)
blether f(1, 2)
blether f(1, 2, 3)
base = 20
blether f(1)
ken g = f
blether g(2)
blether gaun([1, 2], f)
dae outer(x) {
    dae inner(a, b = x * 2) {
        gie a + b
    }
    x = x + 1
    gie inner
}
ken h = outer(5)
blether h(1)
blether h(1, 1)
hae_a_bash {
    g()
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    h(1, 2, 3)
} gin_it_gangs_wrang e {
    blether e
}
"#;
    let want = interpreted(src);
    assert!(
        want.starts_with(
            "[1, 11, [1]]\n[1, 2, [1]]\n[1, 2, 3]\n[1, 21, [1]]\n[2, 22, [2]]\n\
             [[1, 21, [1]], [2, 22, [2]]]\n13\n2\n"
        ),
        "{want}"
    );
    assert!(
        want.contains("Function 'f' expects 1 tae 3 arguments but ye gave it 0"),
        "{want}"
    );
    assert_parity(src, &want);
}

#[test]
fn a_global_used_before_its_ken_is_the_interpreters_error() {
    let src = r#"dae show() {
    gie later
}
hae_a_bash {
    blether show()
} gin_it_gangs_wrang e {
    blether e
}
dae bump() {
    later = 1
}
hae_a_bash {
    bump()
} gin_it_gangs_wrang e {
    blether e
}
ken later = 5
blether show()
"#;
    assert_parity(
        src,
        "Awa' an bile yer heid! 'later' hasnae been defined yet at line 2\n\
         Awa' an bile yer heid! 'later' hasnae been defined yet at line 10\n5\n",
    );
    let out = run_full("dae f() {\n    gie x\n}\nblether f()\nken x = 1\n");
    let error = out.error.expect("read before ken");
    assert!(
        error.contains("'x' hasnae been defined yet at line 2"),
        "{error}"
    );
}

#[test]
fn block_bindings_shadow_without_touching_the_outer_binding() {
    let src = r#"ken x = 10
ken i = "outer"
dae show() {
    gie [x, i]
}
gin aye {
    ken x = 20
    blether x
    blether show()
}
fer i in 0..2 {
    blether show()
}
blether x
blether i
dae f(n) {
    ken total = n
    gin n > 0 {
        ken total = 99
        total = total + 1
    }
    fer n in [7] {
        blether n
    }
    gie [n, total]
}
blether f(1)
ken y = 1
gin aye {
    ken y = y + 1
    blether y
}
blether y
"#;
    assert_parity(
        src,
        "20\n[10, outer]\n[10, outer]\n[10, outer]\n10\nouter\n7\n[1, 1]\n2\n1\n",
    );
}

#[test]
fn a_second_ken_in_the_same_scope_rebinds_the_shared_cell() {
    let src = r#"ken n = 1
ken get = || n
ken n = 2
blether get()
dae f() {
    ken v = "a"
    ken read = || v
    ken v = "b"
    gie read()
}
blether f()
fer k in [1, 2] {
    ken w = k
    ken rw = || w
    ken w = w * 10
    blether rw()
}
"#;
    assert_parity(src, "2\nb\n10\n20\n");
}

#[test]
fn a_kin_inside_a_dae_captures_its_variables() {
    // The method envs live only in the class (an' a bound method): the churn reuses
    // their slots if `push_children` misses them.
    let src = r#"dae make(start) {
    ken step = 2
    kin Counter {
        dae init() {
            masel.n = start
        }
        dae next() {
            masel.n = masel.n + step
            gie masel.n
        }
        dae twin() {
            gie Counter()
        }
    }
    step = 3
    gie Counter()
}
ken c = make(10)
ken nx = c.next
fer j in 0..3000 {
    ken junk = [j, [j]]
}
blether c.next()
blether nx()
blether c.twin().next()
blether c
"#;
    assert_parity(src, "13\n16\n13\n<Counter instance>\n");
}

#[test]
fn an_error_after_an_operator_overload_names_the_statement_line() {
    let src = r#"kin V {
    dae init(x) {
        masel.x = x
    }
    dae __pit_thegither__(o) {
        ken t = 1
        gie V(masel.x + o.x)
    }
    dae __same_as__(o) {
        ken t = 2
        gie masel.x == o.x
    }
}
ken a = V(1)
ken z = [a + a, a == a, 1 / 0]
"#;
    let error = run_full(src).error.expect("division by zero");
    assert!(
        error.contains("Tryin' tae divide by zero at line 15"),
        "{error}"
    );
}

#[test]
fn destructuring_matches_the_interpreter() {
    let src = r#"ken [a, _, ...mid, z] = [1, 2, 3, 4, 5]
blether [a, mid, z]
ken [p, q] = "hi"
blether q + p
ken [...all] = []
blether all
hae_a_bash {
    ken [m, n, o] = [1]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    ken [m] = 5
} gin_it_gangs_wrang e {
    blether e
}
"#;
    let want = interpreted(src);
    assert!(want.starts_with("[1, [3, 4], 5]\nih\n[]\n"), "{want}");
    assert_parity(src, &want);
}

#[test]
fn block_expressions_free_their_garbage_and_keep_their_value() {
    let src = r#"ken out = []
fer i in 0..3000 {
    shove(out, [i, {
        ken t = [i, "x" + i]
        t[1]
    }])
}
blether len(out)
blether out[2999]
"#;
    let out = assert_parity(src, "3000\n[2999, x2999]\n");
    assert!(
        out.peak_store_size < 20000,
        "store peaked at {}",
        out.peak_store_size
    );
}
