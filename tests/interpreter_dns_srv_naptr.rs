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
