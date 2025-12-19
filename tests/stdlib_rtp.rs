use mdhavers::{parse, Interpreter};

#[test]
fn stdlib_rtp_build_parse() {
    let code = r#"
fetch "stdlib/rtp"

ken payload = bytes(4)
bytes_set(payload, 0, 1)
bytes_set(payload, 1, 2)
bytes_set(payload, 2, 3)
bytes_set(payload, 3, 4)

ken pkt = rtp_packet(payload, 10, 100, 12345, 96, 1)
ken info = rtp_parse(pkt)

ken ok = info["ok"] an info["seq"] == 10 an info["timestamp"] == 100 an info["ssrc"] == 12345 an info["payload_type"] == 96 an info["marker"] an info["payload"] == payload

gin ok { blether "rtp_ok" } ither { blether "rtp_fail" }
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "rtp_ok");
}
