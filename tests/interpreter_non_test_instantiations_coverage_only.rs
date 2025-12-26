#![cfg(coverage)]

use mdhavers::{parse, Interpreter, Value};

fn interpret_ok(source: &str) -> (Value, Vec<String>) {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let mut interp = Interpreter::new();
    let value = interp
        .interpret(&program)
        .unwrap_or_else(|e| panic!("interpret failed for:\n{source}\nerr={e:?}"));
    (value, interp.get_output().to_vec())
}

fn interpret_err(source: &str) -> String {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let mut interp = Interpreter::new();
    match interp.interpret(&program) {
        Ok(value) => panic!("expected interpreter error for:\n{source}\n\nbut got Ok: {value:?}"),
        Err(err) => format!("{err:?}"),
    }
}

#[test]
fn log_span_in_success_executes_non_test_with_current_interpreter_paths_for_coverage() {
    let (value, _out) = interpret_ok(
        r#"
dae f() { gie 123 }
ken span = log_span("x")
log_span_in(span, f)
"#,
    );
    assert_eq!(value, Value::Integer(123));
}

#[test]
fn char_at_success_executes_non_test_map_closure_for_coverage() {
    let (value, _out) = interpret_ok(r#"char_at("hi", 0)"#);
    assert_eq!(value, Value::String("h".to_string()));
}

#[test]
fn char_at_out_of_bounds_executes_non_test_ok_or_else_for_coverage() {
    let err = interpret_err(r#"char_at("hi", 99)"#);
    assert!(
        err.contains("Index 99") && err.contains("oot o' bounds"),
        "unexpected error: {err}"
    );
}

#[test]
fn ord_empty_string_executes_non_test_error_closure_for_coverage() {
    let err = interpret_err(r#"ord("")"#);
    assert!(err.contains("empty string"), "unexpected error: {err}");
}

#[test]
fn zip_success_executes_non_test_pair_map_closure_for_coverage() {
    let (value, _out) = interpret_ok(r#"len(zip([1, 2], [3, 4]))"#);
    assert_eq!(value, Value::Integer(2));
}

#[test]
fn read_file_missing_path_executes_non_test_map_err_for_coverage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("definitely_missing_read_file.txt");
    let source = format!(r#"read_file("{}")"#, missing.to_string_lossy());
    let err = interpret_err(&source);
    assert!(err.contains("Couldnae read"), "unexpected error: {err}");
}

#[test]
fn read_lines_missing_path_executes_non_test_map_err_for_coverage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("definitely_missing_read_lines.txt");
    let source = format!(r#"read_lines("{}")"#, missing.to_string_lossy());
    let err = interpret_err(&source);
    assert!(err.contains("Couldnae read"), "unexpected error: {err}");
}

#[test]
fn tri_import_requires_alias_error_executes_non_test_closure_for_coverage() {
    let err = interpret_err(r#"fetch "tri""#);
    assert!(err.contains("tri import requires an alias"), "unexpected error: {err}");
}

#[test]
fn for_loop_over_string_executes_non_test_string_iterable_path_for_coverage() {
    let (_value, out) = interpret_ok(
        r#"
fer c in "ab" { blether c }
0
"#,
    );
    assert_eq!(out, ["a".to_string(), "b".to_string()]);
}

#[test]
fn destructure_string_executes_non_test_string_destructure_path_for_coverage() {
    let (_value, out) = interpret_ok(
        r#"
ken [a, b, c] = "abc"
blether a
blether b
blether c
0
"#,
    );
    assert_eq!(
        out,
        ["a".to_string(), "b".to_string(), "c".to_string()]
    );
}

#[test]
fn list_index_out_of_bounds_executes_non_test_ok_or_else_for_coverage() {
    let err = interpret_err(
        r#"
ken xs = [1, 2]
xs[99]
"#,
    );
    assert!(
        err.contains("IndexOutOfBounds") || err.contains("index"),
        "unexpected error: {err}"
    );
}

#[test]
fn float_and_string_comparisons_execute_non_test_compare_closures_for_coverage() {
    let (value, _out) = interpret_ok(
        r#"
ken g = 1 <= 2
ken a = 1.0 < 2.0
ken b = 1.0 <= 2.0
ken c = 2.0 > 1.0
ken d = "a" < "b"
ken e = "a" <= "a"
ken f = "b" > "a"
len([a, b, c, d, e, f, g])
"#,
    );
    assert_eq!(value, Value::Integer(7));
}

#[cfg(all(feature = "native", unix))]
#[test]
fn tls_client_new_default_mode_executes_non_test_unwrap_or_else_for_coverage() {
    let (value, _out) = interpret_ok(
        r#"
ken r = tls_client_new({"server_name": "localhost"})
r["ok"]
"#,
    );
    assert_eq!(value, Value::Bool(true));
}

#[cfg(all(feature = "native", unix))]
#[test]
fn socket_bind_ipv6_only_host_executes_non_test_no_ipv4_branch_for_coverage() {
    let (value, _out) = interpret_ok(
        r#"
ken s = socket_tcp()
ken sock = s["value"]
ken err = ""

hae_a_bash {
    socket_bind(sock, "::1", 0)
} gin_it_gangs_wrang e {
    err = e
}

socket_close(sock)
err
"#,
    );

    let err = value.as_string().expect("expected string error message");
    assert!(
        err.contains("socket_bind() No IPv4 address found"),
        "unexpected: {err}"
    );
}

#[cfg(all(feature = "native", unix))]
#[test]
fn event_watch_update_executes_non_test_find_closures_for_coverage() {
    let (value, _out) = interpret_ok(
        r#"
ken loop = event_loop_new()
ken s = socket_tcp()
ken sock = s["value"]

dae cb(ev) {
    # no-op
}

event_watch_read(loop, sock, cb)
event_watch_read(loop, sock, cb)

event_watch_write(loop, sock, cb)
event_watch_write(loop, sock, cb)

socket_close(sock)
0
"#,
    );
    assert_eq!(value, Value::Integer(0));
}

#[test]
fn shadow_stack_poison_executes_recovery_paths_for_non_test_coverage() {
    mdhavers::interpreter::poison_shadow_stack_for_coverage();

    mdhavers::interpreter::set_stack_file("non-test-coverage");
    mdhavers::interpreter::push_stack_frame("f", 1);
    mdhavers::interpreter::pop_stack_frame();
    let _ = mdhavers::interpreter::get_stack_trace();
    mdhavers::interpreter::clear_stack_trace();
}

#[test]
fn interpreter_default_with_dir_and_set_current_dir_execute_non_test_instantiations_for_coverage() {
    mdhavers::interpreter::exercise_interpreter_dir_instantiations_for_coverage();

    let tmp = tempfile::tempdir().expect("tempdir");
    let tmp_path = tmp.path();
    let tmp_buf = tmp_path.to_path_buf();
    let tmp_str = tmp_path.to_string_lossy().to_string();

    let mut interp = Interpreter::default();
    interp.set_current_dir(tmp_path);
    interp.set_current_dir(tmp_buf.clone());
    interp.set_current_dir(tmp_str.as_str());

    let _a = Interpreter::with_dir(tmp_path);
    let _b = Interpreter::with_dir(tmp_buf);
    let _c = Interpreter::with_dir(tmp_str);
}

#[cfg(all(feature = "native", unix))]
#[test]
fn tls_insecure_handshake_executes_insecure_verifier_for_non_test_coverage() {
    use std::net::TcpListener;
    use std::time::Duration;

    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    let key_pem = cert.serialize_private_key_pem();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port() as i64;

    let server_handle = std::thread::spawn(move || {
        use rustls::{Certificate, PrivateKey, ServerConfig, ServerConnection};
        use rustls_pemfile::{certs, pkcs8_private_keys};

        let (mut stream, _) = listener.accept().unwrap();
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

        let mut cert_reader = std::io::Cursor::new(cert_pem.as_bytes());
        let certs = certs(&mut cert_reader).unwrap();
        let certs = certs.into_iter().map(Certificate).collect::<Vec<_>>();

        let mut key_reader = std::io::Cursor::new(key_pem.as_bytes());
        let mut keys = pkcs8_private_keys(&mut key_reader).unwrap();
        let key = PrivateKey(keys.remove(0));

        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .unwrap();

        let mut conn = ServerConnection::new(std::sync::Arc::new(config)).unwrap();
        while conn.is_handshaking() {
            conn.complete_io(&mut stream).unwrap();
        }
    });

    let (value, _out) = interpret_ok(&format!(
        r#"
ken s = socket_tcp()
ken sock = s["value"]
socket_connect(sock, "127.0.0.1", {port})

ken t = tls_client_new({{
    "mode": "client",
    "server_name": "localhost",
    "insecure": aye
}})
ken tls = t["value"]

ken res = tls_connect(tls, sock)
tls_close(tls)
socket_close(sock)
res["ok"]
"#
    ));

    server_handle.join().unwrap();

    assert_eq!(value, Value::Bool(true));
}
