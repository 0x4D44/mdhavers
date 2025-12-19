use mdhavers::{parse, Interpreter};

#[test]
fn stdlib_rtcp_rr_build_parse() {
    let code = r#"
fetch "stdlib/rtcp"

ken report = {
    "ssrc": 111,
    "fraction_lost": 2,
    "cumulative_lost": 3,
    "highest_seq": 1000,
    "jitter": 5,
    "lsr": 0,
    "dlsr": 0
}

ken pkt = rtcp_rr(222, [report])
ken parsed = rtcp_parse_rr(pkt)

ken ok = parsed["ok"] an parsed["ssrc"] == 222 an len(parsed["reports"]) == 1 an parsed["reports"][0]["ssrc"] == 111 an parsed["reports"][0]["fraction_lost"] == 2 an parsed["reports"][0]["cumulative_lost"] == 3

gin ok { blether "rtcp_ok" } ither { blether "rtcp_fail" }
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "rtcp_ok");
}
