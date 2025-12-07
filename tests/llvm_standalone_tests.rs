//! Integration tests for LLVM standalone executable compilation
//!
//! These tests verify that mdhavers produces standalone executables
//! that only depend on system libraries (libc, ld-linux).

#![cfg(feature = "llvm")]

use std::process::Command;

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

/// Helper to compile source code and run the resulting executable
fn compile_and_run(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;

    let dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let exe_path = dir.path().join("test_exe");

    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_native(&program, &exe_path, 2)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Executable failed with exit code: {:?}",
            output.status.code()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Helper to verify that an executable only has system dependencies
fn verify_standalone(source: &str) -> Result<(), String> {
    let program = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;

    let dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let exe_path = dir.path().join("test_exe");

    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_native(&program, &exe_path, 2)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    // Run ldd on the executable
    let output = Command::new("ldd")
        .arg(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run ldd: {}", e))?;

    let ldd_output = String::from_utf8_lossy(&output.stdout);

    // Check that we only have system libraries
    for line in ldd_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Allowed dependencies: libc, ld-linux, linux-vdso, libm
        let allowed = [
            "libc.so",
            "ld-linux",
            "linux-vdso",
            "libm.so",
        ];

        let is_allowed = allowed.iter().any(|lib| line.contains(lib));

        if !is_allowed {
            return Err(format!("Found unexpected dependency: {}", line));
        }
    }

    Ok(())
}

#[test]
fn test_simple_print() {
    let source = r#"
        ken x = 42
        blether x
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "42");
}

#[test]
fn test_string_print() {
    let source = r#"
        blether "Hello, standalone!"
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "Hello, standalone!");
}

#[test]
fn test_arithmetic_add() {
    let source = r#"
        ken a = 10
        ken b = 5
        blether a + b
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "15");
}

#[test]
fn test_arithmetic_sub() {
    let source = r#"
        ken a = 10
        ken b = 3
        blether a - b
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "7");
}

#[test]
fn test_arithmetic_mul() {
    let source = r#"
        ken a = 6
        ken b = 7
        blether a * b
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "42");
}

#[test]
fn test_arithmetic_div() {
    let source = r#"
        ken a = 20
        ken b = 4
        blether a / b
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "5");
}

#[test]
fn test_arithmetic_mod() {
    let source = r#"
        ken a = 17
        ken b = 5
        blether a % b
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "2");
}

#[test]
fn test_conditional_true() {
    let source = r#"
        ken x = 10
        gin x > 5 {
            blether "big"
        } ither {
            blether "wee"
        }
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "big");
}

#[test]
fn test_conditional_false() {
    let source = r#"
        ken x = 3
        gin x > 5 {
            blether "big"
        } ither {
            blether "wee"
        }
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "wee");
}

#[test]
fn test_while_loop() {
    let source = r#"
        ken i = 0
        ken sum = 0
        whiles i < 5 {
            sum = sum + i
            i = i + 1
        }
        blether sum
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "10"); // 0+1+2+3+4 = 10
}

#[test]
fn test_function_call() {
    let source = r#"
        dae add(a, b) {
            gie a + b
        }

        blether add(3, 4)
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "7");
}

#[test]
fn test_recursive_function() {
    let source = r#"
        dae factorial(n) {
            gin n <= 1 {
                gie 1
            }
            gie n * factorial(n - 1)
        }

        blether factorial(5)
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "120");
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        dae double(x) {
            gie x * 2
        }

        dae add_one(x) {
            gie x + 1
        }

        ken result = double(add_one(5))
        blether result
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "12"); // (5+1)*2 = 12
}

#[test]
fn test_negative_numbers() {
    let source = r#"
        ken x = 5
        ken y = 0 - x
        blether y
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "-5");
}

#[test]
fn test_bool_true() {
    let source = r#"
        blether aye
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "aye");
}

#[test]
fn test_bool_false() {
    let source = r#"
        blether nae
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "nae");
}

#[test]
fn test_nil() {
    let source = r#"
        blether naething
    "#;

    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "naething");
}

#[test]
fn test_standalone_no_custom_deps() {
    let source = r#"
        ken x = 42
        blether x
    "#;

    verify_standalone(source).expect("Executable has unexpected dependencies");
}

#[test]
fn test_complex_standalone() {
    let source = r#"
        dae fibonacci(n) {
            gin n <= 1 {
                gie n
            }
            gie fibonacci(n - 1) + fibonacci(n - 2)
        }

        blether fibonacci(10)
    "#;

    // First verify it's standalone
    verify_standalone(source).expect("Executable has unexpected dependencies");

    // Then verify the output
    let output = compile_and_run(source).expect("Compilation failed");
    assert_eq!(output.trim(), "55");
}
