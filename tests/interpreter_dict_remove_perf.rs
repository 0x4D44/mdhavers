//! MDH-BUG-FLU-00160: heave_oot / dict_remove in a loop must not be quadratic.
//!
//! MDH-BUG-FLU-00198 / 00202: no wall-clock limit and no ratio across sizes. On a
//! loaded box even a plain interpreter loop scales superlinearly with size, so a
//! size-to-size ratio flaked both ways. Instead, one program times, at a single size,
//! the removal loop against a control loop doing the same work without the dict
//! (the same `tae_string` calls). Both run in the same process under the same
//! contention. Linear removal costs a small multiple of the control; a quadratic
//! removal of 20k keys costs hundreds of times more, so the test fails above 25x.
//! A quadratic removal also never finishes in time, which fails the run.

use mdhavers::{parse, Interpreter};
use std::sync::mpsc;
use std::time::Duration;

const KEYS: usize = 20_000;
const LIMIT_RATIO: f64 = 25.0;

fn program(n: usize) -> String {
    format!(
        r#"
ken d = {{}}
ken i = 0
whiles i < {n} {{
    d[tae_string(i)] = i
    i = i + 1
}}
ken t0 = mono_ms()
ken j = 0
whiles j < {n} {{
    heave_oot(d, tae_string(j))
    j = j + 1
}}
ken t1 = mono_ms()
ken k = 0
whiles k < {n} {{
    ken s = tae_string(k)
    k = k + 1
}}
ken t2 = mono_ms()
blether len(d)
blether t1 - t0
blether t2 - t1
"#
    )
}

#[test]
fn heave_oot_removal_costs_a_small_multiple_of_a_plain_loop() {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let program = parse(&program(KEYS)).expect("parse");
        let mut interp = Interpreter::new();
        interp.interpret(&program).expect("run");
        let out: Vec<String> = interp.get_output().iter().map(|l| l.to_string()).collect();
        let _ = tx.send(out);
    });
    // Linear removal of 20k keys takes well under a second of CPU; a quadratic one takes minutes.
    let out = rx
        .recv_timeout(Duration::from_secs(300))
        .expect("heave_oot of 20k keys did not finish in 300 s: looks quadratic");
    let nums: Vec<f64> = out
        .iter()
        .map(|l| l.trim().parse().expect("numeric output"))
        .collect();
    assert_eq!(nums[0], 0.0, "every key should be gone: {out:?}");
    let (removal_ms, control_ms) = (nums[1], nums[2].max(1.0));
    eprintln!("removal {removal_ms} ms, control loop {control_ms} ms");
    assert!(
        removal_ms < LIMIT_RATIO * control_ms,
        "heave_oot looks quadratic: removing {KEYS} keys took {removal_ms} ms against {control_ms} ms for the same loop without the dict (limit {LIMIT_RATIO}x)"
    );
}
