//! Emitted IR must carry the host target triple (MDH-BUG-FLU-00154).

#![cfg(feature = "llvm")]

use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;
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

    // The data layout must be the one LLVM computes for the host target, not a fixed x86
    // string: a layout-only regression (old x86 layout on arm64) fails here.
    let triple = TargetMachine::get_default_triple();
    let machine = Target::from_triple(&triple)
        .unwrap()
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .expect("host target machine");
    let layout = machine.get_target_data().get_data_layout();
    let layout = layout.as_str().to_str().unwrap().to_string();
    assert!(!layout.is_empty());
    let layout_line = ir
        .lines()
        .find(|l| l.starts_with("target datalayout"))
        .expect("target datalayout line");
    assert_eq!(layout_line, format!("target datalayout = \"{layout}\""));
}
