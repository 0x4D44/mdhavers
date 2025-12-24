#![cfg(all(feature = "native", unix))]

use mdhavers::{parse, Interpreter};

#[test]
fn interpreter_srtp_roundtrip() {
    let code = r#"
ken result = "srtp_fail"

dae make_bytes_seq(n, start) {
    ken b = bytes(n)
    ken i = 0
    whiles i < n {
        bytes_set(b, i, start + i)
        i = i + 1
    }
    gie b
}

dae make_rtp_packet() {
    ken b = bytes(16)
    bytes_set(b, 0, 128)
    bytes_set(b, 1, 0)
    bytes_set(b, 2, 0)
    bytes_set(b, 3, 1)
    bytes_set(b, 4, 0)
    bytes_set(b, 5, 0)
    bytes_set(b, 6, 0)
    bytes_set(b, 7, 1)
    bytes_set(b, 8, 0)
    bytes_set(b, 9, 0)
    bytes_set(b, 10, 0)
    bytes_set(b, 11, 1)
    bytes_set(b, 12, 16)
    bytes_set(b, 13, 17)
    bytes_set(b, 14, 18)
    bytes_set(b, 15, 19)
    gie b
}

ken key = make_bytes_seq(16, 1)
ken salt = make_bytes_seq(14, 50)
ken cfg = {"profile": "SRTP_AES128_CM_SHA1_80", "master_key": key, "master_salt": salt}
ken ctx = srtp_create(cfg)
gin ctx["ok"] {
    ken pkt = make_rtp_packet()
    ken prot = srtp_protect(ctx["value"], pkt)
    gin prot["ok"] {
        ken unp = srtp_unprotect(ctx["value"], prot["value"])
        gin unp["ok"] an unp["value"] == pkt {
            result = "srtp_ok"
        }
    }
}

blether result
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "srtp_ok");
}

#[test]
fn interpreter_srtp_client_server_keys() {
    let code = r#"
dae make_bytes_seq(n, start) {
    ken b = bytes(n)
    ken i = 0
    whiles i < n {
        bytes_set(b, i, start + i)
        i = i + 1
    }
    gie b
}

ken ck = make_bytes_seq(16, 1)
ken cs = make_bytes_seq(14, 50)
ken sk = make_bytes_seq(16, 2)
ken ss = make_bytes_seq(14, 60)

ken cfg_client = {"profile": "SRTP_AES128_CM_SHA1_80", "client_key": ck, "client_salt": cs, "server_key": sk, "server_salt": ss, "role": "client"}
ken cfg_server = {"profile": "SRTP_AES128_CM_SHA1_80", "client_key": ck, "client_salt": cs, "server_key": sk, "server_salt": ss, "role": "server"}

ken a = srtp_create(cfg_client)
ken b = srtp_create(cfg_server)
gin a["ok"] an b["ok"] { blether "keys_ok" } ither { blether "keys_fail" }
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "keys_ok");
}

#[test]
fn interpreter_srtp_create_rejects_non_dict_config_for_coverage() {
    let program = parse("srtp_create(1)").unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected srtp_create() type error");
    let s = format!("{err:?}");
    assert!(s.contains("srtp_create"), "unexpected error: {s}");
}

#[test]
fn interpreter_srtp_error_paths_return_result_err_for_coverage() {
    let code = r#"
dae make_bytes_seq(n, start) {
    ken b = bytes(n)
    ken i = 0
    whiles i < n {
        bytes_set(b, i, start + i)
        i = i + 1
    }
    gie b
}

ken k16a = make_bytes_seq(16, 1)
ken k16b = make_bytes_seq(16, 2)
ken s14a = make_bytes_seq(14, 50)
ken s14b = make_bytes_seq(14, 60)
ken kbad = make_bytes_seq(1, 1)

ken unsupported = srtp_create({"profile": "NOPE", "master_key": k16a, "master_salt": s14a})
blether unsupported["error"]

ken missing_send_key = srtp_create({"profile": "SRTP_AES128_CM_SHA1_80"})
blether missing_send_key["error"]

ken missing_send_salt = srtp_create({"profile": "SRTP_AES128_CM_SHA1_80", "send_key": k16a})
blether missing_send_salt["error"]

ken missing_recv_key = srtp_create({"profile": "SRTP_AES128_CM_SHA1_80", "send_key": k16a, "send_salt": s14a, "recv_salt": s14b})
blether missing_recv_key["error"]

ken missing_recv_salt = srtp_create({"profile": "SRTP_AES128_CM_SHA1_80", "send_key": k16a, "send_salt": s14a, "recv_key": k16b})
blether missing_recv_salt["error"]

ken send_err = srtp_create({"profile": "SRTP_AES128_CM_SHA1_80", "send_key": kbad, "send_salt": s14a, "recv_key": k16a, "recv_salt": s14a})
blether send_err["error"]

ken recv_err = srtp_create({"profile": "SRTP_AES128_CM_SHA1_80", "send_key": k16a, "send_salt": s14a, "recv_key": kbad, "recv_salt": s14a})
blether recv_err["error"]

ken pkt = bytes(16)
ken prot = srtp_protect(999999, pkt)
blether prot["error"]

ken unp = srtp_unprotect(999999, pkt)
blether unp["error"]
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");

    assert!(
        out.contains("Unsupported SRTP profile"),
        "missing unsupported profile error: {out}"
    );
    assert!(
        out.contains("Missing SRTP send_key"),
        "missing send_key error: {out}"
    );
    assert!(
        out.contains("Missing SRTP send_salt"),
        "missing send_salt error: {out}"
    );
    assert!(
        out.contains("Missing SRTP recv_key"),
        "missing recv_key error: {out}"
    );
    assert!(
        out.contains("Missing SRTP recv_salt"),
        "missing recv_salt error: {out}"
    );
    assert!(
        out.contains("SRTP send session error"),
        "missing send session error: {out}"
    );
    assert!(
        out.contains("SRTP recv session error"),
        "missing recv session error: {out}"
    );
    assert!(
        out.contains("Unknown SRTP handle"),
        "missing unknown handle error: {out}"
    );
}

#[test]
fn interpreter_srtp_protect_rejects_non_bytes_packet_for_coverage() {
    let program = parse("srtp_protect(1, 1)").unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected srtp_protect() type error");
    let s = format!("{err:?}");
    assert!(
        s.contains("srtp_protect() expects bytes"),
        "unexpected error: {s}"
    );
}

#[test]
fn interpreter_srtp_unprotect_rejects_non_bytes_packet_for_coverage() {
    let program = parse("srtp_unprotect(1, 1)").unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected srtp_unprotect() type error");
    let s = format!("{err:?}");
    assert!(
        s.contains("srtp_unprotect() expects bytes"),
        "unexpected error: {s}"
    );
}
