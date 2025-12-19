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
