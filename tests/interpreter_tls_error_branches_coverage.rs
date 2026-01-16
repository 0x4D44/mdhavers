#![cfg(all(feature = "native", unix))]

use std::cell::RefCell;
use std::rc::Rc;

use mdhavers::value::{DictValue, NativeFunction};
use mdhavers::{Interpreter, Value};
use rcgen::generate_simple_self_signed;

fn native(interp: &Interpreter, name: &str) -> Rc<NativeFunction> {
    let exports = interp.globals.borrow().get_exports();
    exports
        .into_iter()
        .find_map(|(n, v)| {
            if n == name {
                match v {
                    Value::NativeFunction(native) => Some(native),
                    other => panic!("expected native function {name}, got {other:?}"),
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("native function not found: {name}"))
}

fn bytes(data: &[u8]) -> Value {
    Value::Bytes(Rc::new(RefCell::new(data.to_vec())))
}

fn result_ok_int(value: Value) -> i64 {
    let Value::Dict(d) = value else {
        panic!("expected dict result, got {value:?}");
    };
    let dict = d.borrow();
    assert_eq!(
        dict.get(&Value::String("ok".to_string())),
        Some(&Value::Bool(true)),
        "expected ok=true result"
    );
    dict.get(&Value::String("value".to_string()))
        .and_then(|v| v.as_integer())
        .expect("expected ok.value integer")
}

fn assert_result_err_contains(value: Value, needle: &str) {
    let Value::Dict(d) = value else {
        panic!("expected dict result, got {value:?}");
    };
    let dict = d.borrow();
    assert_eq!(
        dict.get(&Value::String("ok".to_string())),
        Some(&Value::Bool(false)),
        "expected ok=false result"
    );
    let Value::String(msg) = dict
        .get(&Value::String("error".to_string()))
        .cloned()
        .unwrap_or(Value::String(String::new()))
    else {
        panic!("expected string error");
    };
    assert!(
        msg.contains(needle),
        "expected error to contain '{needle}', got: {msg}"
    );
}

#[test]
fn interpreter_tls_send_and_recv_require_connected_session_for_coverage() {
    let interp = Interpreter::new();

    let tls_client_new = native(&interp, "tls_client_new");
    let tls_send = native(&interp, "tls_send");
    let tls_recv = native(&interp, "tls_recv");
    let tls_close = native(&interp, "tls_close");

    let tls_id = result_ok_int((tls_client_new.func)(vec![Value::Nil]).unwrap());

    let send_res = (tls_send.func)(vec![Value::Integer(tls_id), bytes(b"hi")]).unwrap();
    assert_result_err_contains(send_res, "TLS not connected");

    let recv_res = (tls_recv.func)(vec![Value::Integer(tls_id), Value::Integer(1)]).unwrap();
    assert_result_err_contains(recv_res, "TLS not connected");

    let _ = (tls_close.func)(vec![Value::Integer(tls_id)]).unwrap();
}

#[test]
fn interpreter_tls_client_new_defaults_empty_server_name_to_localhost_for_coverage() {
    let interp = Interpreter::new();

    let tls_client_new = native(&interp, "tls_client_new");
    let tls_close = native(&interp, "tls_close");

    let mut dict = DictValue::new();
    dict.set(
        Value::String("mode".to_string()),
        Value::String("client".to_string()),
    );
    dict.set(
        Value::String("server_name".to_string()),
        Value::String(String::new()),
    );
    dict.set(Value::String("insecure".to_string()), Value::Bool(true));

    let tls_id = result_ok_int(
        (tls_client_new.func)(vec![Value::Dict(Rc::new(RefCell::new(dict)))]).unwrap(),
    );
    let _ = (tls_close.func)(vec![Value::Integer(tls_id)]).unwrap();
}

#[test]
fn interpreter_tls_client_new_treats_wrongly_typed_config_fields_as_absent_for_coverage() {
    let interp = Interpreter::new();

    let tls_client_new = native(&interp, "tls_client_new");
    let tls_close = native(&interp, "tls_close");

    let mut dict = DictValue::new();
    dict.set(Value::String("mode".to_string()), Value::Integer(1));
    dict.set(Value::String("server_name".to_string()), Value::Integer(2));
    dict.set(Value::String("insecure".to_string()), Value::Integer(3));
    dict.set(Value::String("ca_pem".to_string()), Value::Integer(4));
    dict.set(Value::String("cert_pem".to_string()), Value::Integer(5));
    dict.set(Value::String("key_pem".to_string()), Value::Integer(6));

    let tls_id = result_ok_int(
        (tls_client_new.func)(vec![Value::Dict(Rc::new(RefCell::new(dict)))]).unwrap(),
    );
    let _ = (tls_close.func)(vec![Value::Integer(tls_id)]).unwrap();
}

#[test]
fn interpreter_tls_client_new_rejects_invalid_ca_pem_for_coverage() {
    let interp = Interpreter::new();
    let tls_client_new = native(&interp, "tls_client_new");

    let invalid_ca = "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n".to_string();
    let mut dict = DictValue::new();
    dict.set(
        Value::String("mode".to_string()),
        Value::String("client".to_string()),
    );
    dict.set(
        Value::String("server_name".to_string()),
        Value::String("localhost".to_string()),
    );
    dict.set(
        Value::String("ca_pem".to_string()),
        Value::String(invalid_ca),
    );

    let err = (tls_client_new.func)(vec![Value::Dict(Rc::new(RefCell::new(dict)))])
        .expect_err("expected invalid CA to error");
    assert!(
        err.contains("No valid CA certificates found"),
        "unexpected error: {err}"
    );
}

#[test]
fn interpreter_tls_server_config_exercises_rsa_key_fallback_branch_for_coverage() {
    let interp = Interpreter::new();
    let tls_client_new = native(&interp, "tls_client_new");

    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();

    // Not a PKCS8 key, so the server-config builder will fall back to RSA parsing, then hit the
    // invalid-base64 error path for coverage.
    let rsa_key =
        "-----BEGIN RSA PRIVATE KEY-----\nNOT_BASE64\n-----END RSA PRIVATE KEY-----\n".to_string();

    let mut dict = DictValue::new();
    dict.set(
        Value::String("mode".to_string()),
        Value::String("server".to_string()),
    );
    dict.set(
        Value::String("cert_pem".to_string()),
        Value::String(cert_pem),
    );
    dict.set(Value::String("key_pem".to_string()), Value::String(rsa_key));

    let err = (tls_client_new.func)(vec![Value::Dict(Rc::new(RefCell::new(dict)))])
        .expect_err("expected invalid RSA key to error");
    assert!(
        err.contains("Invalid server key"),
        "unexpected error: {err}"
    );
}

#[test]
fn interpreter_tls_server_config_maps_with_single_cert_error_for_coverage() {
    let interp = Interpreter::new();
    let tls_client_new = native(&interp, "tls_client_new");

    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();

    // Base64-valid, but not a valid PKCS8 private key; this should fail at with_single_cert().
    let invalid_der_key =
        "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n".to_string();

    let mut dict = DictValue::new();
    dict.set(
        Value::String("mode".to_string()),
        Value::String("server".to_string()),
    );
    dict.set(
        Value::String("cert_pem".to_string()),
        Value::String(cert_pem),
    );
    dict.set(
        Value::String("key_pem".to_string()),
        Value::String(invalid_der_key),
    );

    let err = (tls_client_new.func)(vec![Value::Dict(Rc::new(RefCell::new(dict)))])
        .expect_err("expected invalid private key to error");
    assert!(
        err.contains("Invalid server TLS config"),
        "unexpected error: {err}"
    );
}

#[test]
fn interpreter_tls_server_config_requires_key_pem_for_coverage() {
    let interp = Interpreter::new();
    let tls_client_new = native(&interp, "tls_client_new");

    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();

    let mut dict = DictValue::new();
    dict.set(
        Value::String("mode".to_string()),
        Value::String("server".to_string()),
    );
    dict.set(
        Value::String("cert_pem".to_string()),
        Value::String(cert_pem),
    );

    let err = (tls_client_new.func)(vec![Value::Dict(Rc::new(RefCell::new(dict)))])
        .expect_err("expected missing key_pem to error");
    assert!(
        err.contains("Server key_pem is required"),
        "unexpected error: {err}"
    );
}

#[test]
fn interpreter_tls_server_config_requires_private_key_for_coverage() {
    let interp = Interpreter::new();
    let tls_client_new = native(&interp, "tls_client_new");

    let cert = generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();

    let mut dict = DictValue::new();
    dict.set(
        Value::String("mode".to_string()),
        Value::String("server".to_string()),
    );
    dict.set(
        Value::String("cert_pem".to_string()),
        Value::String(cert_pem.clone()),
    );
    // Provide a PEM value that does not contain a private key to hit the missing-key branch.
    dict.set(
        Value::String("key_pem".to_string()),
        Value::String(cert_pem),
    );

    let err = (tls_client_new.func)(vec![Value::Dict(Rc::new(RefCell::new(dict)))])
        .expect_err("expected missing private key to error");
    assert!(
        err.contains("did not contain a private key"),
        "unexpected error: {err}"
    );
}
