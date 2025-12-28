#![cfg(coverage)]

use mdhavers::{parse, run, HaversError, Interpreter, Value};
use mdhavers::interpreter::TraceMode;

#[test]
fn interpreter_instantiation_sweep_runs_for_coverage() {
    let result = run(
        r#"
mak_siccar 1 < 2
mak_siccar 1 <= 1
mak_siccar 2 > 1
mak_siccar 2 >= 2

mak_siccar 1.0 < 2.0
mak_siccar 1 < 2.0
mak_siccar 1.0 < 2

mak_siccar 1.0 <= 2.0
mak_siccar 1.0 <= 2
mak_siccar 1 <= 2.0

mak_siccar 2.0 > 1.0
mak_siccar 2.0 > 1
mak_siccar 2 > 1.0

mak_siccar 2.0 >= 2.0
mak_siccar 2.0 >= 2
mak_siccar 2 >= 2.0

mak_siccar "a" < "b"
mak_siccar "a" <= "a"
mak_siccar "b" > "a"
mak_siccar "b" >= "b"

// Bool logic
mak_siccar aye an aye
mak_siccar nae or aye

// Ternary
ken x = gin 1 < 2 than 3 ither 4
mak_siccar x == 3

// Match
keek 2 {
    whan 1 -> { mak_siccar nae }
    whan 2 -> { mak_siccar aye }
}

// Try/catch
hae_a_bash { hurl "boom" } gin_it_gangs_wrang e {
    mak_siccar e != naething
}

// Struct construction + property get
thing Pair { a, b }
ken p = Pair(1, 2)
mak_siccar p.a == 1
mak_siccar p.b == 2

// Class + method call + masel property
kin Foo {
    dae init(x) { masel.x = x }
    dae get() { gie masel.x }
}
ken f = Foo(5)
mak_siccar f.get() == 5

// For loops over ranges and lists
ken sum = 0
fer i in 1..=3 { sum = sum + i }
mak_siccar sum == 6

ken sum2 = 0
fer i in [1, 2, 3] { sum2 = sum2 + i }
mak_siccar sum2 == 6

42
"#,
    )
    .unwrap();

    assert_eq!(result, Value::Integer(42));
}

#[test]
fn interpreter_trace_modes_execute_for_instantiation_coverage_in_dependency_instance() {
    let program = parse(
        r#"
ken x = 1
x
"#,
    )
    .expect("parse");
    let mut interp = Interpreter::new();
    interp.set_trace_mode(TraceMode::Verbose);
    let value = interp.interpret(&program).expect("interpret");
    interp.set_trace_mode(TraceMode::Off);
    assert_eq!(value, Value::Integer(1));
}

#[test]
fn interpreter_execute_stmt_control_flow_arms_execute_for_instantiation_coverage_in_dependency_instance(
) {
    let program = parse("gie 7").expect("parse");
    let mut interp = Interpreter::new();
    let value = interp.interpret(&program).expect("interpret");
    assert_eq!(value, Value::Integer(7));

    let program = parse("brak").expect("parse");
    let mut interp = Interpreter::new();
    match interp.interpret(&program) {
        Ok(value) => panic!("expected break-outside-loop error, got: {value:?}"),
        Err(HaversError::BreakOutsideLoop { .. }) => {}
        Err(err) => panic!("unexpected error: {err:?}"),
    }

    let program = parse("haud").expect("parse");
    let mut interp = Interpreter::new();
    match interp.interpret(&program) {
        Ok(value) => panic!("expected continue-outside-loop error, got: {value:?}"),
        Err(HaversError::ContinueOutsideLoop { .. }) => {}
        Err(err) => panic!("unexpected error: {err:?}"),
    }
}
