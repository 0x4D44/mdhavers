#![cfg(feature = "llvm")]
//! Native codegen is deterministic: compiling the same program twice gives the same IR
//! (MDH-BUG-FLU-00130). Each compile builds fresh HashMaps with fresh random seeds, so
//! any IR emitted in HashMap order shows up as a difference within a few compiles.

use mdhavers::{parse, LLVMCompiler};

/// Many Int shadows synced at loop exit, a lambda capturing many variables, and
/// `stopwatch`, whose name lookup walks every user function.
const SOURCE: &str = r#"
dae churn(n) {
    ken a = 0
    ken b = 1
    ken c = 2
    ken d = 3
    ken e = 4
    ken f = 5
    ken g = 6
    ken h = 7
    ken i = 0
    whiles i < n {
        a = a + i
        b = b + a
        c = c + b
        d = d + c
        e = e + d
        f = f + e
        g = g + f
        h = h + g
        i = i + 1
    }
    gie a + b + c + d + e + f + g + h
}

dae closures(n) {
    ken p = n
    ken q = n + 1
    ken r = n + 2
    ken s = n + 3
    ken t = n + 4
    ken u = n + 5
    ken add = |x| x + p + q + r + s + t + u
    gie add(1)
}

dae tick() {
    gie 1
}

blether churn(10)
blether closures(3)
ken timing = stopwatch(tick)
"#;

#[test]
fn native_ir_is_identical_across_compiles() {
    let program = parse(SOURCE).expect("parse");
    let first = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect("compile to IR");
    for run in 1..20 {
        let next = LLVMCompiler::new()
            .compile_to_ir(&program)
            .expect("compile to IR");
        assert!(
            next == first,
            "IR differs between compile 0 and compile {run}"
        );
    }
}
