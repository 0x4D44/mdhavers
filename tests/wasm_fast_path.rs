//! Semantic and host-call checks for the WASM small-integer and inline-bytes fast paths.
#![cfg(feature = "wasm_runner")]

use mdhavers::wasm_compiler::compile_to_wat;
use mdhavers::wasm_runner::{run_wat_captured, WasmRunOutput};
use mdhavers::{parse, Interpreter};
use std::thread;

fn run_wasm(source: &str) -> WasmRunOutput {
    let wat = compile_to_wat(source).unwrap_or_else(|e| panic!("compile failed: {e}\n{source}"));
    run_wat_captured(&wat).unwrap_or_else(|e| panic!("module failed to load: {e}\n{wat}"))
}

fn run_interpreter(source: &str) -> Result<String, String> {
    let source = source.to_owned();
    thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(move || {
            let program = parse(&source).map_err(|e| e.to_string())?;
            let mut interpreter = Interpreter::new();
            interpreter.interpret(&program).map_err(|e| e.to_string())?;
            Ok(interpreter
                .get_output()
                .iter()
                .map(|line| format!("{line}\n"))
                .collect())
        })
        .expect("spawn interpreter thread")
        .join()
        .expect("interpreter panicked")
}

fn assert_stdout_parity(source: &str) -> WasmRunOutput {
    let expected = run_interpreter(source).unwrap_or_else(|e| panic!("interpreter failed: {e}"));
    let out = run_wasm(source);
    assert_eq!(out.error, None, "WASM error; stdout: {}", out.stdout);
    assert_eq!(out.stdout, expected, "WASM differs from the interpreter");
    out
}

fn assert_error_parity(source: &str) -> String {
    let expected = run_interpreter(source).expect_err("interpreter should fail");
    let out = run_wasm(source);
    assert_eq!(out.error.as_deref(), Some(expected.as_str()));
    expected
}

#[test]
fn small_integer_edges_arithmetic_and_mixed_numbers_match_interpreter() {
    let source = r#"
ken max = 9223372036854775807
ken min = -max - 1
blether 2305843009213693951 + 1
blether -2305843009213693952 - 1
blether 4611686018427387903 + 1
blether -4611686018427387904 - 1
blether -(-2305843009213693952)
blether -2305843009213693952 * 1
blether 1152921504606846976 * 2
blether 3037000499 * 3037000499
blether 9007199254740992 + 1
blether max
blether min
blether 7 / 2
blether 7 / -2
blether -7 / 2
blether -7 / -2
blether 7 % 2
blether 7 % -2
blether -7 % 2
blether -7 % -2
blether 1 + 2.5
blether 2.5 + 1
blether 5 - 2.5
blether 2.5 * 2
blether 5.0 / 2
blether 3 == 3
blether 3 != 4
blether 3 < 4
blether 3 <= 3
blether 4 > 3
blether 4 >= 4
blether 1 == 1.0
blether 1 < 1.5
blether 2.5 > 2
blether 9007199254740993 == 9007199254740992.0
blether 9007199254740993 > 9007199254740992.0
blether 9007199254740994 > 9007199254740992.0
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(
        out.stdout,
        concat!(
            "2305843009213693952\n",
            "-2305843009213693953\n",
            "4611686018427387904\n",
            "-4611686018427387905\n",
            "2305843009213693952\n",
            "-2305843009213693952\n",
            "2305843009213693952\n",
            "9223372030926249001\n",
            "9007199254740993\n",
            "9223372036854775807\n",
            "-9223372036854775808\n",
            "3\n-3\n-3\n3\n",
            "1\n1\n-1\n-1\n",
            "3.5\n3.5\n2.5\n5\n2.5\n",
            "aye\naye\naye\naye\naye\naye\naye\naye\naye\n",
            "aye\nnae\naye\n"
        )
    );
}

#[test]
fn integer_overflow_and_zero_divisors_keep_interpreter_errors() {
    for source in [
        "ken max = 9223372036854775807\nmax + 1\n",
        "ken max = 9223372036854775807\nken min = -max - 1\nmin - 1\n",
        "3037000500 * 3037000500\n",
        "ken max = 9223372036854775807\nken min = -max - 1\nmin / -1\n",
        "ken max = 9223372036854775807\nken min = -max - 1\nmin % -1\n",
        "ken max = 9223372036854775807\nken min = -max - 1\n-min\n",
        "7 / 0\n",
        "7 % 0\n",
        "1152921504606846976 * 16\n",
        "-1152921504606846976 * 16\n",
    ] {
        let error = assert_error_parity(source);
        assert!(
            error.contains("overflowed") || error.contains("divide by zero"),
            "unexpected arithmetic error: {error}"
        );
    }
}

#[test]
fn zero_truthiness_matches_in_gin_whiles_logical_operators_and_not() {
    let source = r#"
gin 0 { blether "bad integer zero" } ither { blether "integer zero is false" }
gin 0.0 { blether "bad float zero" } ither { blether "float zero is false" }
gin -0.0 { blether "bad negative float zero" } ither { blether "negative float zero is false" }
gin -1 { blether "negative integer is true" }
gin 1.5 { blether "nonzero float is true" }
ken n = 0
whiles 0 { n = n + 1 }
whiles 0.0 { n = n + 1 }
whiles n < 2 { n = n + 1 }
blether n
blether 0 an "bad and"
blether 0.0 an "bad float and"
blether 7 an "and right"
blether 0 or "or right"
blether 0.0 or "float or right"
blether 7 or "bad or"
blether !0
blether !0.0
blether !-0.0
blether !1
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(
        out.stdout,
        concat!(
            "integer zero is false\n",
            "float zero is false\n",
            "negative float zero is false\n",
            "negative integer is true\n",
            "nonzero float is true\n",
            "2\n",
            "0\n0\nand right\nor right\nfloat or right\n7\n",
            "aye\naye\naye\nnae\n"
        )
    );
}

#[test]
fn inline_bytes_match_builtin_semantics_and_errors() {
    let source = r#"
ken b = bytes(3)
bytes_set(b, 0, 10)
bytes_set(b, 1, 42.9)
bytes_set(b, -1, 30)
blether bytes_len(b)
blether bytes_get(b, 0)
blether bytes_get(b, 1)
blether bytes_get(b, -1)
blether bytes_get(b, -3)
hae_a_bash { bytes_get(b, 3) } gin_it_gangs_wrang e { blether e }
hae_a_bash { bytes_get(b, -4) } gin_it_gangs_wrang e { blether e }
hae_a_bash { bytes_get(b, 1.0) } gin_it_gangs_wrang e { blether e }
hae_a_bash { bytes_get(1, 0) } gin_it_gangs_wrang e { blether e }
hae_a_bash { bytes_set(b, 1.0, 4) } gin_it_gangs_wrang e { blether e }
hae_a_bash { bytes_set(1, 0, 1) } gin_it_gangs_wrang e { blether e }
hae_a_bash { bytes_set(b, 0, 256) } gin_it_gangs_wrang e { blether e }
hae_a_bash { bytes_set(b, 0, -1) } gin_it_gangs_wrang e { blether e }
hae_a_bash { bytes_len(1) } gin_it_gangs_wrang e { blether e }
"#;
    let out = assert_stdout_parity(source);
    assert!(out.stdout.contains("3\n10\n42\n30\n10\n"));
    assert!(out.stdout.contains("bytes_get() index oot o' bounds"));
    assert!(out.stdout.contains("bytes_get() expects integer index"));
    assert!(out.stdout.contains("bytes_set() expects integer index"));
    assert!(out
        .stdout
        .contains("bytes_set() value must be between 0 and 255"));
    assert!(out.stdout.contains("bytes_len() expects bytes"));

    assert_error_parity("ken b = bytes(1)\nbytes_get(b, 1)\n");
    assert_error_parity("ken b = bytes(1)\nbytes_set(b, 0, 256)\n");
}

#[test]
fn bytes_equality_truthiness_and_display_match_interpreter() {
    let source = r#"
ken empty = bytes(0)
ken a = bytes(2)
ken b = bytes(2)
bytes_set(a, 0, 17)
bytes_set(a, 1, 201)
bytes_set(b, 0, 17)
bytes_set(b, 1, 201)
blether a == b
blether [a] == [b]
bytes_set(b, 1, 202)
blether a == b
blether [a] == [b]
blether empty
blether a
blether [a]
gin empty { blether "bad empty bytes truthiness" } ither { blether "empty bytes are false" }
gin a { blether "nonempty bytes are true" }
blether !empty
blether !a
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(
        out.stdout,
        concat!(
            "aye\naye\nnae\nnae\n",
            "bytes[0]\nbytes[2]\n[bytes[2]]\n",
            "empty bytes are false\nnonempty bytes are true\n",
            "aye\nnae\n"
        )
    );
}

#[test]
fn specialized_function_parameters_keep_dynamic_and_captured_fallbacks() {
    let source = r#"
dae dynamic_add(a, b) {
    ken i = 0
    whiles i < 1 {
        i = i + 1
    }
    gie a + b
}
dae combine(a, b = 2) {
    gie a + b
}
dae mutate(start) {
    ken i = 0
    whiles i < 1 {
        i = i + 1
    }
    dae bump() {
        start = start + 1
    }
    bump()
    gie start
}
blether dynamic_add(40, 2)
blether dynamic_add("prefix", 1)
blether dynamic_add(1, "tail")
blether dynamic_add(40, 2.5)
blether combine(40)
blether combine(40, 2.5)
blether combine("default=")
blether mutate(40)
blether mutate("captured=")
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(
        out.stdout,
        "42\nprefix1\n1tail\n42.5\n42\n42.5\ndefault=2\n41\ncaptured=1\n"
    );
    assert!(
        out.peak_store_size < 100,
        "small function calls unexpectedly grew the value store to {} slots",
        out.peak_store_size
    );
}

#[test]
fn block_expression_type_changes_keep_loop_scope_collection() {
    let source = r#"
dae grow_text(a, b) {
    ken changed = {
        a = "x"
        b = "y"
        0
    }
    ken i = 0
    whiles i < 1000 {
        a = a + b
        i = i + 1
    }
    gie len(a)
}
blether grow_text(1, 2)
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(out.stdout, "1001\n");
    // The runner sweeps after 256 pending slots; the live roots leave an observed peak of 262.
    assert!(
        out.peak_store_size <= 300,
        "string allocations in the loop escaped scope collection: {} slots",
        out.peak_store_size
    );
}

#[test]
fn inline_bytes_survive_host_builtin_aliasing_and_memory_growth() {
    let source = r#"
ken b = bytes(3)
bytes_set(b, 1, 77)
ken large = bytes(200000)
bytes_set(b, 2, 88)
ken copied = bytes_slice(b, 0, 3)
blether bytes_len(large)
blether bytes_get(large, 199999)
blether bytes_get(copied, 1)
blether bytes_get(copied, 2)
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(out.stdout, "200000\n0\n77\n88\n");
}

#[test]
fn nested_argument_growth_keeps_pending_bytes_arguments_valid() {
    let source = r#"
ken b = bytes(2)
bytes_set(b, 0, 41)
bytes_set(b, bytes_len(bytes(200000)) - 200000, 73)
ken read = bytes_get(b, bytes_len(bytes(200000)) - 200000)
blether bytes_get(b, 0)
blether read
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(out.stdout, "73\n73\n");
}

#[test]
fn builtin_value_write_is_visible_to_inline_reads_and_host_slice() {
    let source = r#"
ken setter = bytes_set
ken b = bytes(2)
setter(b, 0, 61)
blether bytes_get(b, 0)
ken copied = bytes_slice(b, 0, 1)
blether bytes_get(copied, 0)
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(out.stdout, "61\n61\n");
}

#[test]
fn bytes_append_updates_existing_alias_after_payload_reallocation() {
    let source = r#"
ken b = bytes(2)
bytes_set(b, 0, 3)
bytes_set(b, 1, 7)
ken alias = b
bytes_append(b, b)
blether bytes_len(alias)
blether bytes_get(alias, 0)
blether bytes_get(alias, 1)
blether bytes_get(alias, 2)
blether bytes_get(alias, 3)
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(out.stdout, "4\n3\n7\n3\n7\n");
}

#[test]
fn endian_builtin_writes_are_visible_to_inline_reads() {
    let source = r#"
ken b = bytes(6)
bytes_write_u16be(b, 0, 4660)
bytes_write_u32be(b, 2, 2309737967)
blether bytes_get(b, 0)
blether bytes_get(b, 1)
blether bytes_get(b, 2)
blether bytes_get(b, 3)
blether bytes_get(b, 4)
blether bytes_get(b, 5)
blether bytes_read_u16be(b, 0)
blether bytes_read_u32be(b, 2)
"#;
    let out = assert_stdout_parity(source);
    assert_eq!(out.stdout, "18\n52\n137\n171\n205\n239\n4660\n2309737967\n");
}

fn retained_bytes_and_reused_slots_program() -> String {
    const RETAINED: usize = 180;
    let mut source = String::new();
    for i in 0..RETAINED {
        source.push_str(&format!("ken b{i} = bytes(1)\nbytes_set(b{i}, 0, {i})\n"));
    }
    source.push_str(
        "ken i = 0\nwhiles i < 180 {\n    ken transient = bytes(1)\n    bytes_set(transient, 0, i % 256)\n    ken other = [i]\n    i = i + 1\n}\nken wrong = [1]\nhae_a_bash {\n    bytes_get(wrong, 0)\n} gin_it_gangs_wrang e {\n    blether e\n}\n",
    );
    for i in [0, 63, 64, 127, 128, 179] {
        source.push_str(&format!("blether bytes_get(b{i}, 0)\n"));
    }
    source
}

#[test]
fn retained_bytes_survive_descriptor_growth_and_reused_handle_slots() {
    let out = assert_stdout_parity(&retained_bytes_and_reused_slots_program());
    assert!(
        out.stdout.contains("bytes_get() expects bytes"),
        "a reused non-bytes slot kept a stale bytes tag: {}",
        out.stdout
    );
    assert!(out.stdout.ends_with("0\n63\n64\n127\n128\n179\n"));
}

fn numeric_loop(iterations: usize) -> String {
    format!(
        "dae sum(limit) {{\n    ken i = 0\n    ken total = 0\n    whiles i < limit {{\n        i = i + 1\n        total = total + (i % 7)\n    }}\n    gie total\n}}\nblether sum({iterations})\n"
    )
}

fn bytes_loop(iterations: usize) -> String {
    format!(
        "dae fill(limit, b) {{\n    ken i = 0\n    ken total = 0\n    whiles i < limit {{\n        bytes_set(b, 0, i % 251)\n        total = total + bytes_get(b, 0)\n        i = i + 1\n    }}\n    gie total\n}}\nblether fill({iterations}, bytes(1))\n"
    )
}

#[test]
fn numeric_inner_loop_adds_no_host_calls_per_iteration() {
    let numeric_zero = assert_stdout_parity(&numeric_loop(0));
    let numeric_many = assert_stdout_parity(&numeric_loop(128));
    assert_eq!(numeric_zero.error, None);
    assert_eq!(numeric_many.error, None);
    assert_eq!(numeric_zero.stdout, "0\n");
    assert_eq!(numeric_many.stdout, "381\n");
    assert_eq!(
        numeric_many.host_calls, numeric_zero.host_calls,
        "numeric loop added host calls"
    );
}

#[test]
fn inline_bytes_inner_loop_adds_no_host_calls_per_iteration() {
    let bytes_zero = assert_stdout_parity(&bytes_loop(0));
    let bytes_many = assert_stdout_parity(&bytes_loop(128));
    assert_eq!(bytes_zero.error, None);
    assert_eq!(bytes_many.error, None);
    assert_eq!(bytes_zero.stdout, "0\n");
    assert_eq!(bytes_many.stdout, "8128\n");
    assert_eq!(
        bytes_many.host_calls, bytes_zero.host_calls,
        "inline bytes loop added host calls"
    );
}
