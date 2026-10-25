#![cfg(feature = "llvm")]
//! Native backend: memory-layout prerequisites for reference counting (stage R0 of
//! `wrk_docs/2026.10.23 - DES - LLVM 18-19 upgrade and reference counting.md`, 3.8).
//! R0.1 one allocator.

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

/// Compiles natively, runs, and returns the trimmed stdout lines of a clean exit.
fn run_ok(source: &str) -> Vec<String> {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("layout_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        output.status.code(),
        Some(0),
        "program failed: stdout={stdout:?} stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    stdout.trim().lines().map(str::to_string).collect()
}

fn ir_of(source: &str) -> String {
    let program = parse(source).expect("parse");
    LLVMCompiler::new().compile_to_ir(&program).expect("ir")
}

// ---------- R0.1: every allocation goes through the single allocator ----------

#[test]
fn generated_code_allocates_only_through_the_runtime_allocator() {
    // Strings (concat, self-append, case, trim, chars, split), lists (literal, spread,
    // slice, push), dicts, closures, bound methods and instances.
    let src = r#"
kin P {
    dae init(x) { masel.x = x }
    dae get() { gie masel.x }
}
ken p = P(1)
p.y = 2
ken m = p.get
ken s = "a" + tae_string(p.x)
s = s + "b"
ken u = upper(s) + lower(s) + wheesht(" t ")
ken cs = chars(s)
ken parts = split("a,b", ",")
ken xs = [1, 2, 3]
ken ys = [...xs, 4]
ken zs = xs[0:2]
shove(xs, 5)
ken d = {"k": 1}
ken n = 3
ken f = |a| a + n
blether len(u) + len(cs) + len(parts) + len(ys) + len(zs) + d["k"] + f(1) + m()
"#;
    let ir = ir_of(src);
    for libc in ["malloc", "realloc", "calloc", "strdup"] {
        assert!(
            !ir.contains(&format!("@{libc}(")),
            "generated code calls libc `{libc}` directly"
        );
    }
    for alloc in ["@__mdh_obj_alloc(", "@__mdh_buf_alloc("] {
        assert!(ir.contains(alloc), "expected a call to {alloc}");
    }
    assert_eq!(run_ok(src), vec!["24"]);
}
