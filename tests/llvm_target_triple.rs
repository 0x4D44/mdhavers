//! Emitted IR must carry the host target triple (MDH-BUG-FLU-00154).

#![cfg(feature = "llvm")]

use inkwell::targets::{InitializationConfig, Target, TargetMachine};
use mdhavers::{parse, LLVMCompiler};

#[test]
fn emitted_ir_uses_the_host_default_triple() {
    Target::initialize_native(&InitializationConfig::default()).unwrap();
    let host = TargetMachine::get_default_triple();
    let host = host.as_str().to_str().unwrap().to_string();

    let program = parse("blether 1").unwrap();
    let ir = LLVMCompiler::new().compile_to_ir(&program).unwrap();

    let line = ir
        .lines()
        .find(|l| l.starts_with("target triple"))
        .expect("target triple line");
    assert_eq!(line, format!("target triple = \"{host}\""));
    assert!(ir.lines().any(|l| l.starts_with("target datalayout")));
}
