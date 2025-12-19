use mdhavers::{parse, Interpreter};

#[test]
fn interpreter_atomic_and_channel() {
    let code = r#"
ken a = atomic_new(1)
atomic_add(a, 2)
blether atomic_load(a)

ken ch = chan_new(0)
chan_send(ch, 42)
blether chan_recv(ch)
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "3\n42");
}
