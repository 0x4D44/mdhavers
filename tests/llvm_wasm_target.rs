#![cfg(feature = "llvm")]
//! `mdhavers build --target wasm32-emscripten`: the browser build.

use std::fs;
use std::path::Path;
use std::process::Command;

use mdhavers::{parse, BuildTarget, LLVMCompiler};

const PROGRAM: &str = r#"
fetch "helper"
ken words = split("braw bricht nicht", " ")
shove(words, "the nicht")
blether join(words, "-")
blether upper(words[0]) + " " + tae_string(len(words))
ken d = {"name": "Hamish", "age": 42}
d["toon"] = "Embra"
blether d
kin Beastie {
    dae init(name) {
        masel.name = name
    }
    dae hullo() {
        gie "Hullo fae " + masel.name
    }
}
blether Beastie("Nessie").hullo()
ken add = |a, b| a + b
blether gaun([1, 2, 3], |x| add(x, 10))
hae_a_bash {
    hurl "Och"
} gin_it_gangs_wrang err {
    blether "Caught: " + err
}
blether 3000000000 * 3
blether 7.0 / 2
blether twice(21)
"#;

const HELPER: &str = "dae twice(n) {\n    gie n * 2\n}\n";

fn write_program(dir: &Path) -> std::path::PathBuf {
    fs::write(dir.join("helper.braw"), HELPER).unwrap();
    let main = dir.join("main.braw");
    fs::write(&main, PROGRAM).unwrap();
    main
}

fn available(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn wasm_target_parses_its_names() {
    assert_eq!(
        BuildTarget::parse("wasm32-emscripten"),
        Ok(BuildTarget::Wasm32Emscripten)
    );
    assert_eq!(
        BuildTarget::parse("wasm32-unknown-emscripten"),
        Ok(BuildTarget::Wasm32Emscripten)
    );
    assert_eq!(BuildTarget::parse("native"), Ok(BuildTarget::Native));
    assert!(BuildTarget::parse("riscv")
        .unwrap_err()
        .contains("Dinnae ken"));
}

/// wasm32 IR: the wasm triple, and `size_t` narrowed to 32 bits where C has it.
#[test]
fn wasm_ir_has_the_wasm_triple_and_32_bit_size_t() {
    let dir = tempfile::tempdir().unwrap();
    let main = write_program(dir.path());
    let program = parse(PROGRAM).unwrap();
    let ir = LLVMCompiler::new()
        .with_target(BuildTarget::Wasm32Emscripten)
        .compile_to_ir_with_source(&program, Some(&main))
        .unwrap();
    assert!(
        ir.contains("target triple = \"wasm32-unknown-emscripten\""),
        "no wasm triple"
    );
    assert!(
        ir.contains("declare i32 @strlen(ptr"),
        "strlen not narrowed"
    );
    assert!(
        ir.contains("@__mdh_obj_alloc(i32, i32)") || ir.contains("@__mdh_obj_alloc(i32 "),
        "__mdh_obj_alloc not narrowed"
    );
    assert!(!ir.contains("size_t64("), "a wide declaration is left");

    // The native IR is untouched.
    let native = LLVMCompiler::new()
        .compile_to_ir_with_source(&program, Some(&main))
        .unwrap();
    assert!(!native.contains("wasm32"));
    assert!(native.contains("declare i64 @strlen(ptr"));
}

/// Built for wasm and run under node, a program prints what its native build prints.
/// Skips (passes) when emcc or node is missing.
#[test]
fn wasm_build_runs_under_node_like_native() {
    if !available("emcc") || !available("node") {
        eprintln!("skipping: emcc or node is not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let main = write_program(dir.path());
    let program = parse(PROGRAM).unwrap();

    let exe = dir.path().join("native_exe");
    LLVMCompiler::new()
        .compile_to_native_with_source(&program, &exe, 1, Some(&main))
        .unwrap();
    let native = Command::new(&exe).output().unwrap();
    assert!(native.status.success());

    let js = dir.path().join("main.js");
    LLVMCompiler::new()
        .compile_to_wasm_with_source(&program, &js, 1, Some(&main), &[])
        .unwrap();
    assert!(
        js.with_extension("wasm").exists(),
        "no .wasm beside the .js"
    );
    let wasm = Command::new("node").arg(&js).output().unwrap();
    assert!(
        wasm.status.success(),
        "node failed: {}",
        String::from_utf8_lossy(&wasm.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&wasm.stdout),
        String::from_utf8_lossy(&native.stdout)
    );
}

/// Unbounded recursion in the browser build raises the catchable stack error, as it does
/// natively, before the engine's own stack runs out (an uncatchable RangeError).
/// Skips (passes) when emcc or node is missing.
#[test]
fn wasm_deep_recursion_raises_the_stack_error() {
    if !available("emcc") || !available("node") {
        eprintln!("skipping: emcc or node is not on PATH");
        return;
    }
    const DEEP: &str = r#"
dae deep(n) {
    gie deep(n + 1) + 1
}
hae_a_bash {
    blether deep(0)
} gin_it_gangs_wrang err {
    blether "Caught: " + err
}
dae deepstr(n) {
    gie "x" + deepstr(n + 1)
}
hae_a_bash {
    blether deepstr(0)
} gin_it_gangs_wrang err {
    blether "Caught: " + err
}
blether "still standin"
"#;
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("deep.braw");
    fs::write(&main, DEEP).unwrap();
    let program = parse(DEEP).unwrap();
    let js = dir.path().join("deep.js");
    LLVMCompiler::new()
        .compile_to_wasm_with_source(&program, &js, 1, Some(&main), &[])
        .unwrap();
    let run = Command::new("node").arg(&js).output().unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run.status.success(),
        "node failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        stdout,
        "Caught: Stack's fair puggled! Too many nested calls at line 3 - yer recursion's gone radge!\n\
         Caught: Stack's fair puggled! Too many nested calls at line 11 - yer recursion's gone radge!\n\
         still standin\n"
    );
}
