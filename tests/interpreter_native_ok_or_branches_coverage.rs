#![cfg(all(feature = "native", unix))]

use std::cell::RefCell;
use std::rc::Rc;

use mdhavers::value::NativeFunction;
use mdhavers::{Interpreter, Value};

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

fn assert_result_err(value: Value) {
    let Value::Dict(d) = value else {
        panic!("expected dict result, got {value:?}");
    };
    let dict = d.borrow();
    assert_eq!(
        dict.get(&Value::String("ok".to_string())),
        Some(&Value::Bool(false)),
        "expected ok=false result"
    );
}

fn bytes(data: &[u8]) -> Value {
    Value::Bytes(Rc::new(RefCell::new(data.to_vec())))
}

#[test]
fn interpreter_covers_selected_ok_or_error_branches_for_coverage() {
    let interp = Interpreter::new();

    // tls_* handle/socket validation + registry miss.
    let socket_tcp = native(&interp, "socket_tcp");
    let socket_close = native(&interp, "socket_close");
    let tls_client_new = native(&interp, "tls_client_new");
    let tls_connect = native(&interp, "tls_connect");
    let tls_send = native(&interp, "tls_send");
    let tls_recv = native(&interp, "tls_recv");
    let tls_close = native(&interp, "tls_close");

    assert!((tls_client_new.func)(vec![Value::String("nope".to_string())]).is_err());
    let tls_id = result_ok_int((tls_client_new.func)(vec![Value::Nil]).unwrap());
    let tcp_id = result_ok_int((socket_tcp.func)(vec![]).unwrap());

    assert!((tls_connect.func)(vec![Value::Nil, Value::Integer(tcp_id)]).is_err());
    assert!((tls_connect.func)(vec![Value::Integer(tls_id), Value::Nil]).is_err());
    assert!((tls_connect.func)(vec![Value::Integer(tls_id), Value::Integer(999_999)]).is_err());
    let unknown_tls = (tls_connect.func)(vec![Value::Integer(999_999), Value::Integer(tcp_id)]).unwrap();
    assert_result_err(unknown_tls);

    assert!((tls_send.func)(vec![Value::Nil, bytes(b"hi")]).is_err());
    let send_unknown = (tls_send.func)(vec![Value::Integer(999_999), bytes(b"hi")]).unwrap();
    assert_result_err(send_unknown);

    assert!((tls_recv.func)(vec![Value::Nil, Value::Integer(1)]).is_err());
    assert!((tls_recv.func)(vec![Value::Integer(tls_id), Value::Nil]).is_err());
    let recv_neg = (tls_recv.func)(vec![Value::Integer(tls_id), Value::Integer(-1)]).unwrap();
    assert_result_err(recv_neg);

    assert!((tls_close.func)(vec![Value::Nil]).is_err());

    // dtls_handshake argument validation for socket/dtls handles.
    let dtls_handshake = native(&interp, "dtls_handshake");
    assert!((dtls_handshake.func)(vec![Value::Nil, Value::Integer(0)]).is_err());
    assert!((dtls_handshake.func)(vec![Value::Integer(0), Value::Nil]).is_err());
    assert!((dtls_handshake.func)(vec![Value::Integer(0), Value::Integer(999_999)]).is_err());

    // srtp_* handle validation.
    let srtp_protect = native(&interp, "srtp_protect");
    let srtp_unprotect = native(&interp, "srtp_unprotect");
    let pkt = bytes(b"pkt");
    assert!((srtp_protect.func)(vec![Value::Nil, pkt.clone()]).is_err());
    assert!((srtp_unprotect.func)(vec![Value::Nil, pkt]).is_err());

    // Event-loop watcher/socket validations.
    let event_loop_new = native(&interp, "event_loop_new");
    let event_loop_stop = native(&interp, "event_loop_stop");
    let event_watch_read = native(&interp, "event_watch_read");
    let event_watch_write = native(&interp, "event_watch_write");
    let event_unwatch = native(&interp, "event_unwatch");
    let timer_cancel = native(&interp, "timer_cancel");

    let loop_id = match (event_loop_new.func)(vec![]).unwrap() {
        Value::Integer(id) => id,
        other => panic!("expected loop id integer, got {other:?}"),
    };
    assert!((event_loop_stop.func)(vec![Value::Integer(999_999)]).is_err());
    assert!((event_watch_read.func)(vec![Value::Integer(loop_id), Value::Nil, Value::Nil]).is_err());
    assert!((event_watch_write.func)(vec![Value::Integer(loop_id), Value::Nil, Value::Nil]).is_err());
    assert!((event_unwatch.func)(vec![Value::Integer(loop_id), Value::Nil]).is_err());
    assert!((timer_cancel.func)(vec![Value::Integer(loop_id), Value::Nil]).is_err());

    // condvar_* mutex-handle validation (need a real condvar id so the first ok_or passes).
    let condvar_new = native(&interp, "condvar_new");
    let condvar_wait = native(&interp, "condvar_wait");
    let condvar_timed_wait = native(&interp, "condvar_timed_wait");
    let condvar_id = match (condvar_new.func)(vec![]).unwrap() {
        Value::Integer(id) => id,
        other => panic!("expected condvar id integer, got {other:?}"),
    };
    assert!((condvar_wait.func)(vec![Value::Integer(condvar_id), Value::Nil]).is_err());
    assert!(
        (condvar_timed_wait.func)(vec![Value::Integer(condvar_id), Value::Nil, Value::Integer(0)])
            .is_err()
    );

    // Nested argument-validation branches in common stdlib-style natives.
    let range = native(&interp, "range");
    assert!((range.func)(vec![Value::Integer(0), Value::String("nope".to_string())]).is_err());

    let jammy = native(&interp, "jammy");
    assert!((jammy.func)(vec![Value::Integer(0), Value::String("nope".to_string())]).is_err());

    let chynge = native(&interp, "chynge");
    assert!((chynge.func)(vec![
        Value::List(Rc::new(RefCell::new(vec![Value::Integer(1)]))),
        Value::String("nope".to_string()),
        Value::Integer(2)
    ])
    .is_err());

    let dicht = native(&interp, "dicht");
    assert!((dicht.func)(vec![
        Value::List(Rc::new(RefCell::new(vec![Value::Integer(1)]))),
        Value::String("nope".to_string())
    ])
    .is_err());

    let chunks = native(&interp, "chunks");
    assert!((chunks.func)(vec![
        Value::List(Rc::new(RefCell::new(vec![Value::Integer(1)]))),
        Value::String("nope".to_string())
    ])
    .is_err());

    let random_int = native(&interp, "random_int");
    assert!((random_int.func)(vec![Value::Integer(1), Value::String("nope".to_string())]).is_err());

    let date_format = native(&interp, "date_format");
    assert!((date_format.func)(vec![
        Value::Integer(i64::MAX),
        Value::String("%Y".to_string())
    ])
    .is_err());

    let date_add = native(&interp, "date_add");
    assert!((date_add.func)(vec![
        Value::Integer(0),
        Value::String("nope".to_string()),
        Value::String("seconds".to_string())
    ])
    .is_err());
    assert!((date_add.func)(vec![
        Value::Integer(i64::MAX),
        Value::Integer(1),
        Value::String("seconds".to_string())
    ])
    .is_err());

    let date_diff = native(&interp, "date_diff");
    assert!((date_diff.func)(vec![
        Value::Integer(0),
        Value::String("nope".to_string()),
        Value::String("seconds".to_string())
    ])
    .is_err());

    // Clean up OS resources created above.
    let _ = (socket_close.func)(vec![Value::Integer(tcp_id)]);
}

