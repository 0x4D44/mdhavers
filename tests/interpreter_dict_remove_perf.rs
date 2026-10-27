//! MDH-BUG-FLU-00160: heave_oot / dict_remove in a loop must not be quadratic.
//!
//! MDH-BUG-FLU-00198: the check is a scaling ratio inside one process, not a
//! wall-clock limit. A fixed limit failed on a loaded box although the work is
//! well under a second of CPU. Removing N keys costs time proportional to N when
//! linear, so a 4x bigger dict takes about 4x as long; a quadratic delete takes
//! about 16x. The small size is timed best-of-three, the large one gets two tries, and the
//! test fails only above 10x.

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

fn run_once(n: usize) -> (Duration, String) {
    let program = parse(&program(n)).expect("parse");
    let mut interp = Interpreter::new();
    let start = Instant::now();
    interp.interpret(&program).expect("run");
    (start.elapsed(), interp.get_output().join("\n"))
}

#[test]
fn heave_oot_scales_linearly_not_quadratically() {
    let mut small = Duration::MAX;
    for _ in 0..3 {
        let (took, out) = run_once(5_000);
        assert_eq!(out.trim(), "0");
        small = small.min(took);
    }
    // The 4x run happens on a thread so a quadratic delete (minutes) fails at the
    // limit instead of running to the end. Two tries absorb a scheduling stall.
    let limit = small.mul_f64(10.0);
    for attempt in 0..2 {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(run_once(20_000));
        });
        if let Ok((took, out)) = rx.recv_timeout(limit) {
            assert_eq!(out.trim(), "0");
            eprintln!("20k keys took {took:?} vs 5k {small:?} (attempt {attempt})");
            return;
        }
    }
    panic!(
        "heave_oot looks quadratic: 4x the keys took over {limit:?} (10x the {small:?} for 5k keys); linear is ~4x, quadratic ~16x"
    );
}
