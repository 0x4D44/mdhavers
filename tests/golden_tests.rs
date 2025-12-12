//! Golden tests for mdhavers language
//!
//! Runs .braw files and compares output against .expected files.
//! Tests both interpreter and LLVM native compilation.

use std::fs;
use std::path::{Path, PathBuf};

/// Discover all golden tests in a directory recursively
fn discover_tests(dir: &Path) -> Vec<PathBuf> {
    let mut tests = Vec::new();

    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    tests.extend(discover_tests(&path));
                } else if path.extension().map_or(false, |e| e == "braw") {
                    // Check if corresponding .expected file exists
                    let expected = path.with_extension("expected");
                    if expected.exists() {
                        tests.push(path);
                    }
                }
            }
        }
    }

    tests.sort();
    tests
}

/// Check if a test should be skipped for a given mode
fn should_skip(source: &str, mode: &str) -> bool {
    match mode {
        "native" => source.contains("// SKIP_NATIVE") || source.contains("# SKIP_NATIVE"),
        "interpreter" => {
            source.contains("// SKIP_INTERPRETER") || source.contains("# SKIP_INTERPRETER")
        }
        _ => false,
    }
}

/// Compare actual output with expected output
fn compare_output(actual: &str, expected: &str) -> Result<(), String> {
    let actual_trimmed = actual.trim();
    let expected_trimmed = expected.trim();

    if actual_trimmed == expected_trimmed {
        return Ok(());
    }

    let actual_lines: Vec<&str> = actual_trimmed.lines().collect();
    let expected_lines: Vec<&str> = expected_trimmed.lines().collect();

    let mut diff = String::new();
    diff.push_str("Output mismatch:\n");
    diff.push_str("--- Expected ---\n");
    for line in &expected_lines {
        diff.push_str(&format!("  {}\n", line));
    }
    diff.push_str("--- Actual ---\n");
    for line in &actual_lines {
        diff.push_str(&format!("  {}\n", line));
    }
    Err(diff)
}

/// Run a single golden test with interpreter
fn run_interpreter_test(braw_path: &Path) -> Result<String, String> {
    let source = fs::read_to_string(braw_path)
        .map_err(|e| format!("Failed to read {}: {}", braw_path.display(), e))?;

    let (_value, output) =
        mdhavers::run_with_output(&source).map_err(|e| format!("Interpreter error: {:?}", e))?;

    Ok(output.join("\n"))
}

/// Run a single golden test with LLVM native compilation
#[cfg(feature = "llvm")]
fn run_native_test(braw_path: &Path) -> Result<String, String> {
    use std::process::Command;
    use tempfile::tempdir;

    let source = fs::read_to_string(braw_path)
        .map_err(|e| format!("Failed to read {}: {}", braw_path.display(), e))?;

    let program = mdhavers::parse(&source).map_err(|e| format!("Parse error: {:?}", e))?;

    let dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let exe_path = dir.path().join("test_exe");

    let compiler = mdhavers::LLVMCompiler::new();
    compiler
        .compile_to_native(&program, &exe_path, 2)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !output.status.success() {
        return Err(format!("Exit code: {:?}", output.status.code()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Main test function - runs all golden tests with interpreter
#[test]
fn golden_tests_interpreter() {
    let golden_dir = Path::new("tests/golden");
    let tests = discover_tests(golden_dir);

    if tests.is_empty() {
        println!("No golden tests found in {}", golden_dir.display());
        return;
    }

    let mut failures = Vec::new();
    let mut skipped = 0;

    for test_path in &tests {
        let source = fs::read_to_string(test_path).unwrap();

        if should_skip(&source, "interpreter") {
            skipped += 1;
            continue;
        }

        let expected_path = test_path.with_extension("expected");
        let expected = fs::read_to_string(&expected_path).expect("Failed to read expected file");

        match run_interpreter_test(test_path) {
            Ok(actual) => {
                if let Err(diff) = compare_output(&actual, &expected) {
                    failures.push((test_path.clone(), diff));
                }
            }
            Err(e) => {
                failures.push((test_path.clone(), e));
            }
        }
    }

    if !failures.is_empty() {
        let mut msg = format!("\n{} golden tests failed:\n\n", failures.len());
        for (path, error) in &failures {
            msg.push_str(&format!("FAIL: {}\n{}\n\n", path.display(), error));
        }
        panic!("{}", msg);
    }

    println!(
        "\n✓ {} golden tests passed (interpreter), {} skipped",
        tests.len() - skipped,
        skipped
    );
}

/// Run all golden tests with LLVM native compilation
#[test]
#[cfg(feature = "llvm")]
fn golden_tests_native() {
    let golden_dir = Path::new("tests/golden");
    let tests = discover_tests(golden_dir);

    if tests.is_empty() {
        println!("No golden tests found in {}", golden_dir.display());
        return;
    }

    let mut failures = Vec::new();
    let mut skipped = 0;

    for test_path in &tests {
        let source = fs::read_to_string(test_path).unwrap();

        if should_skip(&source, "native") {
            skipped += 1;
            continue;
        }

        let expected_path = test_path.with_extension("expected");
        let expected = fs::read_to_string(&expected_path).expect("Failed to read expected file");

        match run_native_test(test_path) {
            Ok(actual) => {
                if let Err(diff) = compare_output(&actual, &expected) {
                    failures.push((test_path.clone(), diff));
                }
            }
            Err(e) => {
                failures.push((test_path.clone(), e));
            }
        }
    }

    if !failures.is_empty() {
        let mut msg = format!("\n{} golden tests failed:\n\n", failures.len());
        for (path, error) in &failures {
            msg.push_str(&format!("FAIL: {}\n{}\n\n", path.display(), error));
        }
        panic!("{}", msg);
    }

    println!(
        "\n✓ {} golden tests passed (native), {} skipped",
        tests.len() - skipped,
        skipped
    );
}
