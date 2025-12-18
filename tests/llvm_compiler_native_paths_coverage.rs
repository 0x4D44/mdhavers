#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, llvm::LLVMCompiler};

#[test]
fn llvm_compiler_object_and_native_paths_are_covered() {
    let program = parse("blether 1").expect("parse");
    let dir = tempfile::tempdir().expect("tempdir");

    // Exercise the optimization-level mapping and object emission paths.
    for level in [0u8, 1u8, 2u8, 3u8] {
        let obj = dir.path().join(format!("out_{level}.o"));
        LLVMCompiler::new()
            .with_optimization(level)
            .compile_to_object_with_source(&program, &obj, None)
            .unwrap();
        assert!(obj.exists(), "expected object file for -O{level}");
    }

    // Exercise the `compile_to_object` convenience wrapper.
    {
        let obj = dir.path().join("wrapper.o");
        LLVMCompiler::new().compile_to_object(&program, &obj).unwrap();
        assert!(obj.exists());
    }

    // Exercise the `compile_to_native` wrapper and the full link path.
    {
        let exe = dir.path().join("out_exe");
        LLVMCompiler::default()
            .compile_to_native(&program, &exe, 3)
            .unwrap();
        assert!(exe.exists(), "expected native executable output");
    }
}

