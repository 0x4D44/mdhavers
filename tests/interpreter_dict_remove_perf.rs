//! MDH-BUG-FLU-00160: heave_oot / dict_remove in a loop must not be quadratic.

use mdhavers::{parse, Interpreter};

#[test]
fn heave_oot_100k_keys_finishes_quickly() {
    let code = r#"
ken d = {}
ken i = 0
whiles i < 100000 {
    d[tae_string(i)] = i
    i = i + 1
}
ken j = 0
whiles j < 100000 {
    heave_oot(d, tae_string(j))
    j = j + 1
}
blether len(d)
"#;
    let program = parse(code).expect("parse");
    let mut interp = Interpreter::new();
    let start = std::time::Instant::now();
    interp.interpret(&program).expect("run");
    let took = start.elapsed();
    assert_eq!(interp.get_output().join("\n").trim(), "0");
    assert!(
        took < std::time::Duration::from_secs(30),
        "100k heave_oot took {took:?}"
    );
}
