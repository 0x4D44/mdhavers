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

#[test]
fn interpreter_thread_spawn_and_join() {
    let code = r#"
ken h = thread_spawn(len, [[1, 2, 3]])
ken v = thread_join(h)
ken h2 = thread_spawn(len, [[1]])
thread_detach(h2)
blether v
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "3");
}

#[test]
fn interpreter_thread_spawn_rejects_non_native_functions() {
    let code = r#"
hae_a_bash {
    thread_spawn(|x| x, [1])
    blether "nope"
} gin_it_gangs_wrang e {
    blether "caught"
}
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "caught");
}

#[test]
fn interpreter_atomic_compare_and_swap_covers_success_and_failure_paths() {
    let code = r#"
ken a = atomic_new(1)
blether atomic_cas(a, 1, 2)
blether atomic_load(a)
blether atomic_cas(a, 1, 3)
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "aye\n2\nnae");
}
