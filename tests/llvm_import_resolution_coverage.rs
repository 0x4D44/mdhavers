#![cfg(all(feature = "llvm", coverage))]

use std::fs;
use std::path::{Path, PathBuf};

use mdhavers::{llvm::LLVMCompiler, parse};

fn write_braw(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write braw module");
}

#[test]
fn llvm_import_resolution_exercises_source_path_and_lib_prefix_branches_for_coverage() {
    let dir = tempfile::Builder::new()
        .prefix("import_resolve")
        .tempdir_in("target")
        .expect("tempdir");

    let project = dir.path().join("project");
    let src_nested = project.join("src").join("nested");

    // A module found relative to the source file's parent directory.
    write_braw(
        &src_nested.join("relmod.braw"),
        r#"
dae rel() { gie 1 }
"#,
    );

    // A module found via ancestor walk (candidate exists).
    write_braw(
        &project.join("from_ancestor.braw"),
        r#"
ken A
"#,
    );

    // Modules resolved through the "lib/..." -> "stdlib/..." stripping behavior.
    // - Parent stdlib path: <parent>/stdlib/<name>.braw
    write_braw(
        &src_nested.join("stdlib").join("parent_mod.braw"),
        r#"
ken PARENT_ONLY
"#,
    );
    // - Grandparent stdlib path: <grandparent>/stdlib/<name>.braw
    write_braw(
        &project
            .join("src")
            .join("stdlib")
            .join("grand_mod.braw"),
        r#"
ken GRAND_ONLY
"#,
    );
    // - Ancestor walk stdlib path: <ancestor>/stdlib/<name>.braw
    write_braw(
        &project.join("stdlib").join("ancestor_mod.braw"),
        r#"
ken ANCESTOR_ONLY
"#,
    );

    // Include a few imports that exercise different resolution branches and both
    // extension/non-extension paths.
    let program = parse(
        r#"
fetch "relmod"
fetch "from_ancestor"
fetch "lib/parent_mod"
fetch "lib/grand_mod"
fetch "lib/ancestor_mod"
fetch "bytes"
fetch "bytes.braw"
fetch "hello"
log_blether "hiya"
"#,
    )
    .expect("parse");

    // Provide a nested source path so resolve_import_path() explores parent/grandparent/ancestor logic.
    let source_path: PathBuf = project.join("src").join("nested").join("main.braw");
    let obj = dir.path().join("out.o");
    LLVMCompiler::new()
        .with_optimization(0)
        .compile_to_object_with_source(&program, &obj, Some(&source_path))
        .expect("compile");
    assert!(obj.exists());
}

#[test]
fn llvm_import_prefix_sanitizes_non_alphanumeric_module_names_for_coverage() {
    let dir = tempfile::Builder::new()
        .prefix("import_prefix")
        .tempdir_in("target")
        .expect("tempdir");

    let module_path = dir.path().join("weird-name.braw");
    write_braw(
        &module_path,
        r#"
ken UNINITIALIZED

dae f() { gie 1 }
"#,
    );

    let src = format!(
        r#"
fetch "{}" tae m
blether m["f"]()
"#,
        module_path.display()
    );
    let program = parse(&src).expect("parse");
    let ir = LLVMCompiler::new().compile_to_ir(&program).expect("compile");
    assert!(!ir.is_empty());
}

#[test]
fn llvm_import_resolution_uses_cwd_stripped_lib_fallback_for_coverage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src_path = dir.path().join("main.braw");
    let obj = dir.path().join("out.o");

    // With a non-workspace `source_path`, resolution should fall back to CWD/stdlib for "lib/..." imports.
    let program = parse(
        r#"
fetch "lib/colors"
blether 1
"#,
    )
    .expect("parse");
    LLVMCompiler::new()
        .with_optimization(0)
        .compile_to_object_with_source(&program, &obj, Some(&src_path))
        .expect("compile");
    assert!(obj.exists());
}

#[test]
fn llvm_import_resolution_falls_back_to_exe_dir_for_coverage() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    let exe = std::env::current_exe().expect("current_exe");
    let exe_dir = exe.parent().expect("exe parent");

    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let direct_name = format!("__exe_cov_direct_{n}_{}", std::process::id());
    let stdlib_name = format!("__exe_cov_stdlib_{n}_{}", std::process::id());
    let stripped_name = format!("__exe_cov_stripped_{n}_{}", std::process::id());

    let direct_path = exe_dir.join(format!("{direct_name}.braw"));
    let stdlib_dir = exe_dir.join("stdlib");
    let stdlib_path = stdlib_dir.join(format!("{stdlib_name}.braw"));
    let stripped_path = stdlib_dir.join(format!("{stripped_name}.braw"));

    fs::create_dir_all(&stdlib_dir).expect("create exe stdlib dir");
    write_braw(&direct_path, "dae f() { gie 1 }\n");
    write_braw(&stdlib_path, "dae g() { gie 2 }\n");
    write_braw(&stripped_path, "dae h() { gie 3 }\n");

    let src = format!(
        r#"
fetch "{direct_name}"
fetch "{stdlib_name}"
fetch "lib/{stripped_name}"
blether f()
blether g()
blether h()
"#
    );
    let program = parse(&src).expect("parse");
    let ir = LLVMCompiler::new().compile_to_ir(&program).expect("compile");
    assert!(!ir.is_empty());

    let _ = fs::remove_file(&direct_path);
    let _ = fs::remove_file(&stdlib_path);
    let _ = fs::remove_file(&stripped_path);
}
