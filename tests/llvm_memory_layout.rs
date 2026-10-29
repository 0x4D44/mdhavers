#![cfg(feature = "llvm")]
//! Native backend: memory-layout prerequisites for reference counting (stage R0 of
//! `wrk_docs/2026.10.23 - DES - LLVM 18-19 upgrade and reference counting.md`, 3.8).
//! R0.1 one allocator, R0.2 stable instance identity, R0.3 immortal static strings.

use std::fmt::Write as _;
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

// ---------- R0.2: an instance keeps its identity while it grows ----------

#[test]
fn instance_stays_one_object_while_fields_are_added() {
    // The instance is reachable from a list, a dict, a closure and a second variable.
    // Adding many fields (through masel and from outside) must not split it into copies.
    let mut src = String::from(
        r#"
kin Bag {
    dae init() { masel.n = 0 }
    dae add(v) {
        masel.n = masel.n + v
        gie masel
    }
}
ken b = Bag()
ken alias = b
ken xs = [b]
ken d = {"b": b}
ken peek = || b.n
"#,
    );
    for i in 0..40 {
        writeln!(src, "b.f{i} = {i}").unwrap();
        writeln!(src, "xs[0].add(1)").unwrap();
    }
    src.push_str(
        r#"
alias.extra = "late"
blether b.n
blether alias.n
blether xs[0].n
blether d["b"].n
blether peek()
blether xs[0].f39
blether d["b"].f0 + d["b"].f39
blether b.extra
blether xs[0].extra
blether d["b"].add(2).n
blether b.n
"#,
    );
    let out = run_ok(&src);
    assert_eq!(
        out,
        vec!["40", "40", "40", "40", "40", "39", "39", "late", "late", "42", "42"]
    );
}

// ---------- R0.3: string literals and class names are immortal objects ----------

/// `{ i64 refcount = IMMORTAL, i32 kind = String, i32 flags, chars }` (DES 3.2).
fn immortal_string_constant(text: &str, flags: u32) -> String {
    format!(
        "{{ i64 {}, i32 1, i32 {flags}, [{} x i8] c\"{text}\\00\" }}",
        i64::MAX / 2,
        text.len() + 1
    )
}

#[test]
fn string_literals_and_class_names_carry_an_immortal_header() {
    let src = r#"
kin Wee {
    dae init() { masel.k = "field" }
}
ken w = Wee()
ken s = "literal"
s = s + "!"
blether s
blether "literal"
blether w
blether w.k
blether f"interp {s}"
"#;
    // Flags are IMMORTAL; with counting on (the default) they also carry the header
    // magic, OBJ_FLAG_MAGIC | OBJ_FLAG_IMMORTAL.
    for (rc, flags) in [(false, 1), (true, 0x4D44_0001)] {
        let program = parse(src).expect("parse");
        let ir = LLVMCompiler::new()
            .with_refcounting(rc)
            .compile_to_ir(&program)
            .expect("ir");
        for text in ["literal", "Wee", "field", "interp "] {
            let constant = immortal_string_constant(text, flags);
            let line = ir
                .lines()
                .find(|l| l.contains(&constant))
                .unwrap_or_else(|| {
                    panic!("rc={rc}: no immortal constant for {text:?}: want {constant}")
                });
            assert!(
                line.contains(" constant "),
                "rc={rc}: {text:?} is not read-only: {line}"
            );
            assert!(
                line.contains("align 16"),
                "rc={rc}: {text:?} is not 16-byte aligned: {line}"
            );
        }
        // Values point at the chars, 16 bytes past the header.
        assert!(
            ir.contains("i64 16)"),
            "rc={rc}: no pointer to the chars after a header"
        );
    }
    assert_eq!(
        run_ok(src),
        vec![
            "literal!",
            "literal",
            "<Wee instance>",
            "field",
            "interp literal!"
        ]
    );
}
