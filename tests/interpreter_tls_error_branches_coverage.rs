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
fn interpreter_tls_client_new_rejects_invalid_ca_pem_for_coverage() {
    let interp = Interpreter::new();
    let tls_client_new = native(&interp, "tls_client_new");

    let invalid_ca = "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n".to_string();
    let mut dict = DictValue::new();
    dict.set(Value::String("mode".to_string()), Value::String("client".to_string()));
    dict.set(Value::String("server_name".to_string()), Value::String("localhost".to_string()));
    dict.set(Value::String("ca_pem".to_string()), Value::String(invalid_ca));

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

    // Not a PKCS8 key, so the server-config builder will fall back to RSA parsing.
    let rsa_key = "-----BEGIN RSA PRIVATE KEY-----\nAAAA\n-----END RSA PRIVATE KEY-----\n".to_string();

    let mut dict = DictValue::new();
    dict.set(Value::String("mode".to_string()), Value::String("server".to_string()));
    dict.set(Value::String("cert_pem".to_string()), Value::String(cert_pem));
    dict.set(Value::String("key_pem".to_string()), Value::String(rsa_key));

    let _ = (tls_client_new.func)(vec![Value::Dict(Rc::new(RefCell::new(dict)))])
        .expect_err("expected invalid RSA key to error");
}

