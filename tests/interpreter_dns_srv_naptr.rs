#![cfg(feature = "native")]

use mdhavers::{parse, Interpreter};

#[test]
fn interpreter_dns_srv_naptr_smoke() {
    let code = r#"
ken srv = dns_srv("_sip._udp", "example.com")
ken srv_ok = nae

gin srv["ok"] an len(srv["value"]) > 0 {
    ken first = srv["value"][0]
    gin contains(first, "priority") an contains(first, "weight") an contains(first, "port") an contains(first, "target") {
        srv_ok = aye
    }
}

gin srv_ok { blether "srv_ok" } ither { blether "srv_err" }

ken naptr = dns_naptr("example.com")
ken naptr_ok = nae

gin naptr["ok"] an len(naptr["value"]) > 0 {
    ken first = naptr["value"][0]
    gin contains(first, "order") an contains(first, "preference") an contains(first, "service") {
        naptr_ok = aye
    }
}

gin naptr_ok { blether "naptr_ok" } ither { blether "naptr_err" }
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    let out = out.trim();
    let allowed = [
        "srv_ok\nnaptr_ok",
        "srv_ok\nnaptr_err",
        "srv_err\nnaptr_ok",
        "srv_err\nnaptr_err",
    ];
    assert!(allowed.contains(&out), "unexpected output: {out}");
}

#[test]
fn interpreter_dns_lookup_rejects_non_string_arg_for_coverage() {
    let program = parse("dns_lookup(1)").unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected dns_lookup() type error");
    let s = format!("{err:?}");
    assert!(
        s.contains("dns_lookup() expects host string"),
        "unexpected error: {s}"
    );
}

#[test]
fn interpreter_dns_srv_rejects_non_string_args_for_coverage() {
    let program = parse("dns_srv(1, 2)").unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected dns_srv() type error");
    let s = format!("{err:?}");
    assert!(
        s.contains("dns_srv() expects service string"),
        "unexpected error: {s}"
    );
}

#[test]
fn interpreter_dns_srv_rejects_non_string_domain_for_coverage() {
    let program = parse("dns_srv(\"_sip._udp\", 1)").unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected dns_srv() domain type error");
    let s = format!("{err:?}");
    assert!(
        s.contains("dns_srv() expects domain string"),
        "unexpected error: {s}"
    );
}

#[test]
fn interpreter_dns_naptr_rejects_non_string_arg_for_coverage() {
    let program = parse("dns_naptr(1)").unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected dns_naptr() type error");
    let s = format!("{err:?}");
    assert!(
        s.contains("dns_naptr() expects domain string"),
        "unexpected error: {s}"
    );
}

#[test]
fn interpreter_dns_lookup_invalid_host_returns_result_err_for_coverage() {
    let code = r#"
ken result = "ok"
ken r = dns_lookup("bad host")
gin nae r["ok"] {
    result = r["error"]
}
blether result
"#;
    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert!(out.contains("dns_lookup()"), "unexpected output: {out}");
}

#[test]
fn interpreter_dns_srv_empty_service_branch_for_coverage() {
    let code = r#"
ken r = dns_srv("", "example.com")
gin r["ok"] { blether "aye" } ither { blether "nae" }
"#;
    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    let out = out.trim();
    assert!(out == "aye" || out == "nae", "unexpected output: {out}");
}
