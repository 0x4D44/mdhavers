#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{
    llvm::compiler::{
        exercise_build_status_dependency_paths_for_coverage, set_llvm_compiler_failpoint_for_coverage,
        link_native_executable_from_object_for_coverage, LLVMCompilerFailPoint,
    },
    llvm::LLVMCompiler,
    parse,
};

#[test]
fn llvm_compiler_dependency_error_paths_are_covered_via_injection() {
    exercise_build_status_dependency_paths_for_coverage();

    let program = parse("blether 1").expect("parse");
    let dir = tempfile::tempdir().expect("tempdir");
    let obj_path = dir.path().join("injected_dep.o");
    let exe_path = dir.path().join("injected_dep_exe");

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::InitializeNativeTarget);
        LLVMCompiler::new()
            .compile_to_object(&program, &obj_path)
            .unwrap_err()
    };
    assert!(err
        .to_string()
        .contains("coverage injected: initialize_native_target"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::TargetFromTriple);
        LLVMCompiler::new()
            .compile_to_object(&program, &obj_path)
            .unwrap_err()
    };
    assert!(err.to_string().contains("coverage injected: target_from_triple"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::CreateTargetMachine);
        LLVMCompiler::new()
            .compile_to_object(&program, &obj_path)
            .unwrap_err()
    };
    assert!(err.to_string().contains("Failed to create target machine"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::RunOptimizationPasses);
        LLVMCompiler::new()
            .compile_to_object(&program, &obj_path)
            .unwrap_err()
    };
    assert!(err
        .to_string()
        .contains("coverage injected: run_optimization_passes"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::WriteObjectFile);
        LLVMCompiler::new()
            .compile_to_object(&program, &obj_path)
            .unwrap_err()
    };
    assert!(err.to_string().contains("coverage injected: write_object_file"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::CreateRuntimeFile);
        link_native_executable_from_object_for_coverage(&obj_path, &exe_path).unwrap_err()
    };
    assert!(err.to_string().contains("coverage injected: create_runtime_file"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::WriteRuntimeFile);
        link_native_executable_from_object_for_coverage(&obj_path, &exe_path).unwrap_err()
    };
    assert!(err.to_string().contains("coverage injected: write_runtime_file"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::CreateRuntimeRsFile);
        link_native_executable_from_object_for_coverage(&obj_path, &exe_path).unwrap_err()
    };
    assert!(err
        .to_string()
        .contains("coverage injected: create_runtime_rs_file"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::WriteRuntimeRsFile);
        link_native_executable_from_object_for_coverage(&obj_path, &exe_path).unwrap_err()
    };
    assert!(err
        .to_string()
        .contains("coverage injected: write_runtime_rs_file"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::CreateGcStubFile);
        link_native_executable_from_object_for_coverage(&obj_path, &exe_path).unwrap_err()
    };
    assert!(err.to_string().contains("coverage injected: create_gc_stub_file"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::WriteGcStubFile);
        link_native_executable_from_object_for_coverage(&obj_path, &exe_path).unwrap_err()
    };
    assert!(err.to_string().contains("coverage injected: write_gc_stub_file"));

    let err = {
        let _guard = set_llvm_compiler_failpoint_for_coverage(LLVMCompilerFailPoint::LinkCommandStatus);
        link_native_executable_from_object_for_coverage(&obj_path, &exe_path).unwrap_err()
    };
    assert!(err.to_string().contains("coverage injected: link_command_status"));
}
