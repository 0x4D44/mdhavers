use mdhavers::{parse, HaversError, Interpreter, Value};

fn run(source: &str) -> Result<Value, HaversError> {
    let program = parse(source).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program)
}

#[test]
fn interpreter_compare_type_error_branches_are_covered() {
    for src in ["blether aye < 1", "blether aye <= 1", "blether aye > 1", "blether aye >= 1"] {
        let err = run(src).unwrap_err();
        match err {
            HaversError::TypeError { .. } => {}
            other => panic!("expected TypeError, got: {other:?}"),
        }
    }
}

