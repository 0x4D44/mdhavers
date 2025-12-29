#![cfg(all(feature = "native", unix))]

use mdhavers::{parse, Interpreter};
use rcgen::generate_simple_self_signed;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn escape_for_braw(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn generate_cert() -> (String, String) {
    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    let key_pem = cert.serialize_private_key_pem();
    (cert_pem, key_pem)
}

#[test]
fn interpreter_tls_connect_client_reports_handshake_failure_for_coverage() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_thread = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = stream.write_all(b"not tls");
        let _ = stream.flush();
    });

    let code = format!(
        r#"
ken s = socket_tcp()
ken result = "tls_fail"

gin s["ok"] {{
    ken sock = s["value"]
    ken c = socket_connect(sock, "127.0.0.1", {port})
    gin c["ok"] {{
        ken cfg = {{"mode": "client", "server_name": "localhost", "insecure": aye}}
        ken t = tls_client_new(cfg)
        gin t["ok"] {{
            ken tls = t["value"]
            ken h = tls_connect(tls, sock)
            gin nae h["ok"] {{ result = h["error"] }}
            tls_close(tls)
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
        out.contains("TLS handshake failed"),
        "unexpected output: {out}"
    );

    server_thread.join().unwrap();
}

#[test]
fn interpreter_tls_connect_server_reports_handshake_failure_for_coverage() {
    let (cert_pem, key_pem) = generate_cert();
    let cert_escaped = escape_for_braw(&cert_pem);
    let key_escaped = escape_for_braw(&key_pem);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let client_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(25));
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let _ = stream.write_all(b"not tls");
        let _ = stream.flush();
    });

    let code = format!(
        r#"
ken s = socket_tcp()
ken result = "tls_fail"

gin s["ok"] {{
    ken sock = s["value"]
    socket_set_reuseaddr(sock, aye)
    ken b = socket_bind(sock, "127.0.0.1", {port})
    gin b["ok"] {{
        ken l = socket_listen(sock, 1)
        gin l["ok"] {{
            ken a = socket_accept(sock)
            gin a["ok"] {{
                ken client = a["value"]["sock"]
                ken cfg = {{"mode": "server", "cert_pem": "{cert_escaped}", "key_pem": "{key_escaped}"}}
                ken t = tls_client_new(cfg)
                gin t["ok"] {{
                    ken tls = t["value"]
                    ken h = tls_connect(tls, client)
                    gin nae h["ok"] {{ result = h["error"] }}
                    tls_close(tls)
                }}
                socket_close(client)
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
        out.contains("TLS handshake failed"),
        "unexpected output: {out}"
    );

    client_thread.join().unwrap();
}

