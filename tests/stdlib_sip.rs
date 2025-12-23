#![cfg(feature = "native")]

use mdhavers::{parse, Interpreter};

#[test]
fn stdlib_sip_parse_build_resolve() {
    let code = r#"
fetch "stdlib/sip"

ken msg = "INVITE sip:alice@example.com SIP/2.0\r\nVia: SIP/2.0/UDP host\r\nContent-Length: 4\r\n\r\nTest"
ken parsed = sip_parse_message(msg)
ken parse_ok = parsed["type"] == "request" an parsed["method"] == "INVITE" an parsed["uri"] == "sip:alice@example.com" an parsed["headers"]["via"] == "SIP/2.0/UDP host" an parsed["body"] == "Test"

ken msg_bytes = bytes_from_string(msg)
ken parsed_bytes = sip_parse_message(msg_bytes)
ken parse_bytes_ok = parsed_bytes["type"] == "request" an parsed_bytes["method"] == "INVITE" an parsed_bytes["uri"] == "sip:alice@example.com"

ken built = sip_build_request("INVITE", "sip:alice@example.com", {"Via": "SIP/2.0/UDP host"}, "Test")
ken parsed2 = sip_parse_message(built)
ken build_ok = parsed2["headers"]["via"] == "SIP/2.0/UDP host" an parsed2["headers"]["content-length"] == "4"

ken resolved = sip_resolve("localhost", "udp")
ken resolve_ok = nae

gin len(resolved) > 0 {
    resolve_ok = resolved[0]["port"] == 5060
}

gin parse_ok { blether "parse_ok" } ither { blether "parse_fail" }
gin parse_bytes_ok { blether "parse_bytes_ok" } ither { blether "parse_bytes_fail" }
gin build_ok { blether "build_ok" } ither { blether "build_fail" }
gin resolve_ok { blether "resolve_ok" } ither { blether "resolve_fail" }
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "parse_ok\nparse_bytes_ok\nbuild_ok\nresolve_ok");
}
