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
fn interpreter_atomic_store_float_cas_and_channel_capacity_for_coverage() {
    let code = r#"
ken a = atomic_new(0)
atomic_store(a, 1.5)
blether atomic_load(a)
blether atomic_cas(a, 1.0, 2.0)
blether atomic_load(a)
blether atomic_cas(a, 1.0, 3.0)

ken ch = chan_new(1)
blether chan_send(ch, 1)
blether chan_send(ch, 2)
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "1\naye\n2\nnae\naye\nnae");
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

#[test]
fn interpreter_channel_try_recv_and_is_closed_branches_for_coverage() {
    let code = r#"
ken ch = chan_new(0)
blether chan_is_closed(ch)
blether chan_try_recv(ch)
chan_close(ch)
blether chan_is_closed(ch)
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "nae\nnaething\naye");
}

#[test]
fn interpreter_mutex_and_condvar_builtins_cover_basic_paths_for_coverage() {
    let code = r#"
ken m = mutex_new()
blether mutex_try_lock(m)
blether mutex_try_lock(m)
mutex_unlock(m)
blether mutex_try_lock(m)
mutex_unlock(m)

ken cv = condvar_new()
blether condvar_wait(cv, m)
blether condvar_timed_wait(cv, m, 0.0)
condvar_signal(cv)
condvar_broadcast(cv)
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "aye\nnae\naye\naye\naye");
}
