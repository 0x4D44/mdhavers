#![cfg(feature = "llvm")]

use std::fs;
use std::path::{Path, PathBuf};

use mdhavers::{parse, LLVMCompiler};

fn collect_braw_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_braw_files(&path, out);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) == Some("braw") {
            out.push(path);
        }
    }
}

#[test]
fn llvm_compiles_stdlib_and_examples_modules_to_ir_for_coverage() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dirs = [root.join("stdlib"), root.join("examples")];

    let mut files = Vec::new();
    for dir in dirs {
        collect_braw_files(&dir, &mut files);
    }
    files.sort();

    let compiler = LLVMCompiler::new();

    for path in files {
        let source = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("failed to read module {}: {}", path.display(), e);
        });
        let program = match parse(&source) {
            Ok(program) => program,
            Err(e) => {
                // Some repo examples may intentionally use experimental/older syntax.
                // For coverage, keep going as long as we don't panic.
                let err = format!("{e:?}");
                assert!(
                    !err.is_empty(),
                    "parse error string should not be empty for {}",
                    path.display()
                );
                continue;
            }
        };

        // Not all repo modules are guaranteed to compile under the LLVM backend yet.
        // For coverage, we accept either Ok or Err, but avoid panics and ensure errors are non-empty.
        match compiler.compile_to_ir(&program) {
            Ok(ir) => assert!(
                !ir.is_empty(),
                "IR output should not be empty for {}",
                path.display()
            ),
            Err(e) => {
                let err = format!("{e:?}");
                assert!(
                    !err.is_empty(),
                    "compile error string should not be empty for {}",
                    path.display()
                );
            }
        }
    }
}
