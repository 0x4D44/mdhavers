#![cfg(all(feature = "native", unix))]

use mdhavers::{parse, Interpreter};
use rcgen::generate_simple_self_signed;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn generate_cert() -> (String, String) {
    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    let key_pem = cert.serialize_private_key_pem();
    (cert_pem, key_pem)
}

fn escape_for_braw(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn allocate_port() -> u16 {
    UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn interpreter_dtls_unknown_srtp_profile_string_falls_back_to_default_for_coverage() {
    let program = parse(
        r#"
ken d = dtls_server_new({"srtp_profiles": ["NOPE"]})
blether d["ok"]
"#,
    )
    .unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "aye");
}

#[test]
fn interpreter_dtls_ignores_non_string_srtp_profiles_items_for_coverage() {
    let program = parse(
        r#"
ken d = dtls_server_new({"srtp_profiles": [1, "SRTP_AES128_CM_SHA1_80"]})
blether d["ok"]
"#,
    )
    .unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "aye");
}

#[test]
fn interpreter_dtls_requires_remote_host_or_connected_socket() {
    let (cert_pem, key_pem) = generate_cert();
    let cert_escaped = escape_for_braw(&cert_pem);
    let key_escaped = escape_for_braw(&key_pem);

    let code = format!(
        r#"
ken result = "nope"
ken s = socket_udp()

gin s["ok"] {{
    ken sock = s["value"]
    socket_set_reuseaddr(sock, aye)
    ken b = socket_bind(sock, "127.0.0.1", 0)
    gin b["ok"] {{
        ken cfg = {{
            "mode": "server",
            "server_name": "",
            "cert_pem": "{cert_escaped}",
            "key_pem": "{key_escaped}",
            "insecure": aye
        }}
        ken d = dtls_server_new(cfg)
        gin d["ok"] {{
            ken hs = dtls_handshake(d["value"], sock)
            gin nae hs["ok"] {{
                result = hs["error"]
            }}
        }}
    }}
    socket_close(sock)
}}

blether result
"#
    );

    let program = parse(&code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert!(
        out.contains("dtls_handshake requires remote_host/remote_port"),
        "unexpected output: {out}"
    );
}

#[test]
fn interpreter_dtls_server_new_rejects_non_dict_config() {
    let program = parse("dtls_server_new(1)").unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected dtls_server_new type error");
    let s = format!("{err:?}");
    assert!(s.contains("dtls_server_new"), "unexpected error: {s}");
}

#[test]
fn interpreter_dtls_handshake_keys() {
    let (cert_pem, key_pem) = generate_cert();
    let cert_escaped = escape_for_braw(&cert_pem);
    let key_escaped = escape_for_braw(&key_pem);

    let server_port = allocate_port();
    let client_port = allocate_port();

    let (server_tx, server_rx) = mpsc::channel();
    let cert_server = cert_escaped.clone();
    let key_server = key_escaped.clone();
    let server_thread = thread::spawn(move || {
        let code = format!(
            r#"
ken result = "dtls_fail"
ken s = socket_udp()

gin s["ok"] {{
    ken sock = s["value"]
    socket_set_reuseaddr(sock, aye)
    ken b = socket_bind(sock, "127.0.0.1", {server_port})
    gin b["ok"] {{
        ken cfg = {{
            "mode": "server",
            "cert_pem": "{cert_server}",
            "key_pem": "{key_server}",
            "remote_host": "127.0.0.1",
            "remote_port": {client_port},
            "srtp_profiles": ["SRTP_AES128_CM_SHA1_80"]
        }}
        ken d = dtls_server_new(cfg)
        gin d["ok"] {{
            ken hs = dtls_handshake(d["value"], sock)
            gin hs["ok"] an hs["value"]["key_len"] > 0 {{
                result = "dtls_ok"
            }}
        }}
    }}
    socket_close(sock)
}}

blether result
"#
        );
        let program = parse(&code).unwrap();
        let mut interp = Interpreter::new();
        interp.interpret(&program).unwrap();
        let out = interp.get_output().join("\n");
        server_tx.send(out).unwrap();
    });

    thread::sleep(Duration::from_millis(50));

    let (client_tx, client_rx) = mpsc::channel();
    let cert_client = cert_escaped.clone();
    let key_client = key_escaped.clone();
    let client_thread = thread::spawn(move || {
        let code = format!(
            r#"
ken result = "dtls_fail"
ken s = socket_udp()

gin s["ok"] {{
    ken sock = s["value"]
    socket_set_reuseaddr(sock, aye)
    ken b = socket_bind(sock, "127.0.0.1", {client_port})
    gin b["ok"] {{
        ken cfg = {{
            "mode": "client",
            "server_name": "localhost",
            "insecure": aye,
            "cert_pem": "{cert_client}",
            "key_pem": "{key_client}",
            "remote_host": "127.0.0.1",
            "remote_port": {server_port},
            "srtp_profiles": ["SRTP_AES128_CM_SHA1_80"]
        }}
        ken d = dtls_server_new(cfg)
        gin d["ok"] {{
            ken hs = dtls_handshake(d["value"], sock)
            gin hs["ok"] an hs["value"]["key_len"] > 0 {{
                result = "dtls_ok"
            }}
        }}
    }}
    socket_close(sock)
}}

blether result
"#
        );
        let program = parse(&code).unwrap();
        let mut interp = Interpreter::new();
        interp.interpret(&program).unwrap();
        let out = interp.get_output().join("\n");
        client_tx.send(out).unwrap();
    });

    let server_out = server_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("server timed out");
    let client_out = client_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("client timed out");

    server_thread.join().unwrap();
    client_thread.join().unwrap();

    assert_eq!(server_out.trim(), "dtls_ok");
    assert_eq!(client_out.trim(), "dtls_ok");
}
