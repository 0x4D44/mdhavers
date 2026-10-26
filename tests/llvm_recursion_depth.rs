#![cfg(feature = "llvm")]
//! Native backend: unbounded recursion raises the interpreter's catchable "Stack's fair
//! puggled!" error instead of crashing, on the main thread and on spawned threads
//! (MDH-BUG-FLU-00164).

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_native(source: &str, refcounting: bool) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("recursion_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

const PUGGLED: &str =
    "Stack's fair puggled! Too many nested calls at line 1 - yer recursion's gone radge!";

#[test]
fn unbounded_recursion_raises_the_scots_error_and_exits_1() {
    for refcounting in [false, true] {
        let run = run_native(
            "dae r(n) { gie 1 + r(n + 1) }\nblether \"start\"\nblether r(0)\n",
            refcounting,
        );
        assert_eq!(run.code, Some(1), "rc={refcounting}: {}", run.stderr);
        assert_eq!(run.stdout.trim(), "start");
        assert!(
            run.stderr.contains(PUGGLED),
            "rc={refcounting}: {}",
            run.stderr
        );
    }
}

#[test]
fn the_error_is_catchable_again_and_again() {
    let src = r#"dae r(n) { gie 1 + r(n + 1) }
dae s(n) {
    ken label = "deep " + tae_string(n)
    gie len(label) + s(n + 1)
}
fer k in 0..3 {
    hae_a_bash {
        blether r(0)
    } gin_it_gangs_wrang e {
        blether e
    }
    hae_a_bash {
        blether s(0)
    } gin_it_gangs_wrang e {
        blether "caught s"
    }
}
blether "after"
"#;
    for refcounting in [false, true] {
        let run = run_native(src, refcounting);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        let mut expected = Vec::new();
        for _ in 0..3 {
            expected.push(PUGGLED);
            expected.push("caught s");
        }
        expected.push("after");
        assert_eq!(lines, expected, "rc={refcounting}");
    }
}

#[test]
fn a_spawned_thread_has_its_own_limit() {
    let src = r#"dae r(n) { gie 1 + r(n + 1) }
ken t = thread_spawn(|| {
    hae_a_bash {
        r(0)
    } gin_it_gangs_wrang e {
        gie "thread caught"
    }
}, [])
blether thread_join(t)
ken u = thread_spawn(|| r(0), [])
hae_a_bash {
    thread_join(u)
} gin_it_gangs_wrang e {
    blether "join raised"
}
"#;
    let run = run_native(src, false);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout.lines().collect::<Vec<_>>(),
        vec!["thread caught", "join raised"]
    );
}

#[test]
fn deep_but_finite_recursion_still_works() {
    let run = run_native(
        "dae d(n) {\n    gin n == 0 { gie 0 }\n    gie 1 + d(n - 1)\n}\nblether d(5000)\n",
        false,
    );
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout.trim(), "5000");
}

#[test]
fn formatting_a_too_deeply_nested_list_raises_the_error() {
    // The runtime formats nested values recursively: it stops with the stack error at a
    // checked headroom, from normal code, rather than running into the tripwire.
    let src = r#"ken l = []
fer i in 0..300000 {
    l = [l]
}
hae_a_bash {
    blether l
} gin_it_gangs_wrang e {
    blether "caught: " + e
}
hae_a_bash {
    blether json_stringify(l)
} gin_it_gangs_wrang e {
    blether "json caught: " + e
}
blether "after"
blether l
"#;
    let run = run_native(src, false);
    assert_eq!(run.code, Some(1), "{}", run.stderr);
    let puggled = |line: usize| {
        format!(
            "Stack's fair puggled! Too many nested calls at line {line} - yer recursion's gone radge!"
        )
    };
    assert_eq!(
        run.stdout.lines().collect::<Vec<_>>(),
        vec![
            format!("caught: {}", puggled(6)),
            format!("json caught: {}", puggled(11)),
            "after".to_string()
        ]
    );
    assert!(run.stderr.contains(&puggled(16)), "{}", run.stderr);
}

#[test]
fn json_of_a_too_deeply_nested_key_or_fallback_value_raises_the_error() {
    // json_stringify (Rust) formats a non-string dict key, or a value JSON has no form
    // for, with the C formatter: running too deep there comes back to Rust as an error,
    // not a raise across its frames.
    let src = r#"ken l = []
fer i in 0..300000 {
    l = [l]
}
ken d = {}
d["a"] = "x"
d[l] = 1
ken s = empty_creel()
toss_in(s, l)
ken small = {}
small[[1, [2]]] = [empty_creel()]
blether json_stringify(small)
fer i in 0..3 {
    hae_a_bash {
        blether json_stringify(d)
    } gin_it_gangs_wrang e {
        blether "key caught: " + e
    }
    hae_a_bash {
        blether json_stringify({"s": s})
    } gin_it_gangs_wrang e {
        blether "value caught: " + e
    }
}
blether json_stringify(small)
"#;
    let puggled = |line: usize| {
        format!(
            "Stack's fair puggled! Too many nested calls at line {line} - yer recursion's gone radge!"
        )
    };
    let small = r#"{"[1, [2]]": ["creel{}"]}"#.to_string();
    let mut expected = vec![small.clone()];
    for _ in 0..3 {
        expected.push(format!("key caught: {}", puggled(15)));
        expected.push(format!("value caught: {}", puggled(20)));
    }
    expected.push(small);
    for refcounting in [false, true] {
        let run = run_native(src, refcounting);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        assert_eq!(
            run.stdout.lines().collect::<Vec<_>>(),
            expected,
            "rc={refcounting}"
        );
    }
}

#[test]
fn recursion_that_calls_the_runtime_only_now_and_then_raises_the_error() {
    // Most levels call nothing out of compiled code; every k-th calls a runtime
    // function from wherever the stack has got to.
    let src = r#"dae r(n, k) {
    gin n % k == 0 {
        ken s = tae_string(n) + "!"
    }
    gie 1 + r(n + 1, k)
}
fer k in [1000, 997, 991, 983, 977, 971, 967, 953, 947, 941, 7, 3] {
    hae_a_bash {
        blether r(0, k)
    } gin_it_gangs_wrang e {
        blether "caught " + tae_string(k)
    }
}
blether "after"
"#;
    let ks = [1000, 997, 991, 983, 977, 971, 967, 953, 947, 941, 7, 3];
    let mut expected: Vec<String> = ks.iter().map(|k| format!("caught {k}")).collect();
    expected.push("after".to_string());
    for refcounting in [false, true] {
        let run = run_native(src, refcounting);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        assert_eq!(
            run.stdout.lines().collect::<Vec<_>>(),
            expected,
            "rc={refcounting}"
        );
    }
}

#[test]
fn a_function_with_a_frame_bigger_than_the_tripwire_raises_the_error() {
    // 400 hae_a_bash sites reserve about 200 KiB of handler buffers in one frame, more
    // than the tripwire: the frame is probed page by page, so it cannot step over it.
    let body: String = (0..400)
        .map(|i| format!("    hae_a_bash {{ x = x + {i} }} gin_it_gangs_wrang e{i} {{ x = 0 }}\n"))
        .collect();
    let src = format!(
        "dae big(n) {{\n    ken x = n\n{body}    gie 1 + big(n + 1)\n}}\n\
         hae_a_bash {{\n    blether big(0)\n}} gin_it_gangs_wrang e {{\n    blether \"big caught\"\n}}\n\
         blether \"after\"\n"
    );
    let run = run_native(&src, false);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout.lines().collect::<Vec<_>>(),
        vec!["big caught", "after"]
    );
}
