use mdhavers::{parse, Interpreter};

#[test]
fn interpreter_logging_extras_and_callback_cover_more_branches() {
    let code = r#"
dae cb(payload) {
    blether payload["message"]
}

log_init({"sinks": [
    {"kind": "memory", "max": 10},
    {"kind": "callback", "fn": cb},
]})

log_blether "no extras"
log_blether "structured fields", {"user": "md", "ok": aye}
log_blether "targeted", "tests.logging"
log_blether "both", {"a": 1}, "tests.logging"

hae_a_bash {
    log_blether "bad extras", 123
} gin_it_gangs_wrang e {
    blether "caught"
}
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(
        out.trim(),
        ["no extras", "structured fields", "targeted", "both", "caught"].join("\n")
    );
}

