#![cfg(all(feature = "native", unix))]

use mdhavers::{parse, Interpreter};
use rcgen::generate_simple_self_signed;
use rustls::{
    Certificate, ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection,
    ServerName, StreamOwned,
};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn generate_cert() -> (String, String) {
    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    let key_pem = cert.serialize_private_key_pem();
    (cert_pem, key_pem)
}

fn build_server_config(cert_pem: &str, key_pem: &str) -> Arc<ServerConfig> {
    let mut cert_reader = std::io::Cursor::new(cert_pem.as_bytes());
    let certs = certs(&mut cert_reader).unwrap();
    let certs = certs.into_iter().map(Certificate).collect::<Vec<_>>();

    let mut key_reader = std::io::Cursor::new(key_pem.as_bytes());
    let mut keys = pkcs8_private_keys(&mut key_reader).unwrap();
    if keys.is_empty() {
        let mut key_reader = std::io::Cursor::new(key_pem.as_bytes());
        keys = rsa_private_keys(&mut key_reader).unwrap();
    }
    let key = keys.into_iter().next().expect("missing private key");

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, rustls::PrivateKey(key))
        .unwrap();
    Arc::new(config)
}

fn build_client_config(cert_pem: &str) -> Arc<ClientConfig> {
    let mut roots = RootCertStore::empty();
    let mut reader = std::io::Cursor::new(cert_pem.as_bytes());
    let certs = certs(&mut reader).unwrap();
    let (added, _ignored) = roots.add_parsable_certificates(&certs);
    assert!(added > 0, "no certs added");

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();
    Arc::new(config)
}

fn escape_for_braw(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[test]
fn interpreter_tls_client_to_rust_server() {
    let (cert_pem, key_pem) = generate_cert();
    let server_config = build_server_config(&cert_pem, &key_pem);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_thread = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut stream = StreamOwned::new(ServerConnection::new(server_config).unwrap(), stream);
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"ping");
        stream.write_all(b"pong").unwrap();
        let _ = stream.flush();
    });

    let cert_escaped = escape_for_braw(&cert_pem);
    let code = format!(
        r#"
ken s = socket_tcp()
ken result = "tls_fail"

gin s["ok"] {{
    ken sock = s["value"]
    ken c = socket_connect(sock, "127.0.0.1", {port})
    gin c["ok"] {{
        ken cfg = {{"mode": "client", "server_name": "localhost", "ca_pem": "{cert_escaped}"}}
        ken t = tls_client_new(cfg)
        gin t["ok"] {{
            ken tls = t["value"]
            ken h = tls_connect(tls, sock)
            gin h["ok"] {{
                ken sent = tls_send(tls, bytes_from_string("ping"))
                ken recv = tls_recv(tls, 4)
                ken ok = sent["ok"] an recv["ok"] an recv["value"] == bytes_from_string("pong")
                gin ok {{ result = "tls_ok" }}
            }}
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
    assert_eq!(out.trim(), "tls_ok");

    server_thread.join().unwrap();
}

#[test]
fn interpreter_tls_server_to_rust_client() {
    let (cert_pem, key_pem) = generate_cert();
    let cert_escaped = escape_for_braw(&cert_pem);
    let key_escaped = escape_for_braw(&key_pem);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let (tx, rx) = std::sync::mpsc::channel();
    let server_thread = thread::spawn(move || {
        let code = format!(
            r#"
ken s = socket_tcp()
ken result = "server_fail"

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
                    gin h["ok"] {{
                        ken recv = tls_recv(tls, 4)
                        gin recv["ok"] an recv["value"] == bytes_from_string("ping") {{
                            ken sent = tls_send(tls, bytes_from_string("pong"))
                            gin sent["ok"] {{ result = "server_ok" }}
                        }}
                    }}
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
        tx.send(out).unwrap();
    });

    // Wait for server to be ready
    let mut stream = None;
    for _ in 0..40 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let stream = stream.expect("failed to connect to TLS server");

    let client_config = build_client_config(&cert_pem);
    let server_name = ServerName::try_from("localhost").unwrap();
    let mut tls = StreamOwned::new(
        ClientConnection::new(client_config, server_name).unwrap(),
        stream,
    );
    tls.write_all(b"ping").unwrap();
    tls.flush().unwrap();
    let mut buf = [0u8; 4];
    tls.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"pong");

    server_thread.join().unwrap();
    let out = rx.recv().unwrap();
    assert_eq!(out.trim(), "server_ok");
}

#[test]
fn interpreter_tls_connect_twice_returns_result_err_for_coverage() {
    let (cert_pem, key_pem) = generate_cert();
    let server_config = build_server_config(&cert_pem, &key_pem);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server_thread = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut stream = StreamOwned::new(ServerConnection::new(server_config).unwrap(), stream);
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"ping");
        stream.write_all(b"pong").unwrap();
        let _ = stream.flush();
    });

    let cert_escaped = escape_for_braw(&cert_pem);
    let code = format!(
        r#"
ken s = socket_tcp()
ken result = "tls_fail"

gin s["ok"] {{
    ken sock = s["value"]
    ken c = socket_connect(sock, "127.0.0.1", {port})
    gin c["ok"] {{
        ken cfg = {{"mode": "client", "server_name": "localhost", "ca_pem": "{cert_escaped}"}}
        ken t = tls_client_new(cfg)
        gin t["ok"] {{
            ken tls = t["value"]
            ken h1 = tls_connect(tls, sock)
            gin h1["ok"] {{
                ken sent = tls_send(tls, bytes_from_string("ping"))
                ken recv = tls_recv(tls, 4)
                gin sent["ok"] an recv["ok"] an recv["value"] == bytes_from_string("pong") {{
                    ken h2 = tls_connect(tls, sock)
                    gin nae h2["ok"] {{
                        result = h2["error"]
                    }}
                }}
            }}
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
        out.contains("TLS session already connected"),
        "unexpected output: {out}"
    );

    server_thread.join().unwrap();
}

#[test]
fn interpreter_tls_send_rejects_non_bytes_argument_for_coverage() {
    let program = parse("tls_send(1, 2)").unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected tls_send() type error");
    let s = format!("{err:?}");
    assert!(s.contains("tls_send() expects bytes"), "unexpected error: {s}");
}
