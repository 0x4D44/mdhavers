//! MDH-BUG-FLU-00160: heave_oot / dict_remove in a loop must not be quadratic.
//!
//! MDH-BUG-FLU-00198: the check is a scaling ratio inside one process, not a
//! wall-clock limit. A fixed limit failed on a loaded box although the work is
//! well under a second of CPU. Removing N keys costs time proportional to N when
//! linear, so a 4x bigger dict takes about 4x as long; a quadratic delete takes
//! about 16x. Each size is timed best-of-three, which shrugs off a stall, and the
//! test fails only above 8x.

use mdhavers::{parse, Interpreter};
use std::time::{Duration, Instant};

fn program(n: usize) -> String {
    format!(
        r#"
ken d = {{}}
ken i = 0
whiles i < {n} {{
    d[tae_string(i)] = i
    i = i + 1
}}
ken j = 0
whiles j < {n} {{
    heave_oot(d, tae_string(j))
    j = j + 1
}}
blether len(d)
"#
    )
}

/// Run the fill-then-remove program for `n` keys; returns (best time, output).
fn best_of_three(n: usize) -> (Duration, String) {
    let program = parse(&program(n)).expect("parse");
    let mut best = Duration::MAX;
    let mut out = String::new();
    for _ in 0..3 {
        let mut interp = Interpreter::new();
        let start = Instant::now();
        interp.interpret(&program).expect("run");
        best = best.min(start.elapsed());
        out = interp.get_output().join("\n");
    }
    (best, out)
}

#[test]
fn heave_oot_scales_linearly_not_quadratically() {
    let (small, small_out) = best_of_three(10_000);
    let (large, large_out) = best_of_three(40_000);
    assert_eq!(small_out.trim(), "0");
    assert_eq!(large_out.trim(), "0");
    let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-6);
    assert!(
        ratio < 8.0,
        "heave_oot looks quadratic: 4x the keys took {ratio:.1}x as long ({small:?} -> {large:?}); linear is ~4x, quadratic ~16x"
    );
}
