#![cfg(all(feature = "native", unix))]

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

fn bytes(data: &[u8]) -> Value {
    Value::Bytes(Rc::new(std::cell::RefCell::new(data.to_vec())))
}

fn result_ok_value(value: Value) -> Option<Value> {
    let Value::Dict(d) = value else {
        return None;
    };
    let dict = d.borrow();
    let ok = dict.get(&Value::String("ok".to_string()))?;
    if ok == &Value::Bool(true) {
        dict.get(&Value::String("value".to_string())).cloned()
    } else {
        None
    }
}

fn result_ok_int(value: Value) -> i64 {
    result_ok_value(value)
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

#[test]
fn interpreter_socket_and_io_builtins_cover_error_paths_for_coverage() {
    let interp = Interpreter::new();

    let socket_udp = native(&interp, "socket_udp");
    let socket_tcp = native(&interp, "socket_tcp");
    let socket_bind = native(&interp, "socket_bind");
    let socket_connect = native(&interp, "socket_connect");
    let socket_listen = native(&interp, "socket_listen");
    let socket_accept = native(&interp, "socket_accept");
    let socket_set_nonblocking = native(&interp, "socket_set_nonblocking");
    let socket_set_rcvbuf = native(&interp, "socket_set_rcvbuf");
    let socket_set_sndbuf = native(&interp, "socket_set_sndbuf");
    let socket_close = native(&interp, "socket_close");
    let udp_send_to = native(&interp, "udp_send_to");
    let udp_recv_from = native(&interp, "udp_recv_from");
    let tcp_send = native(&interp, "tcp_send");
    let tcp_recv = native(&interp, "tcp_recv");

    let tcp_id = result_ok_int((socket_tcp.func)(vec![]).unwrap());
    let udp_id = result_ok_int((socket_udp.func)(vec![]).unwrap());

    // socket_bind: host type error + port range error
    assert!(
        (socket_bind.func)(vec![Value::Integer(tcp_id), Value::Integer(1), Value::Integer(0)])
            .is_err()
    );
    assert!(
        (socket_bind.func)(vec![Value::Integer(tcp_id), Value::Nil, Value::Integer(70000)]).is_err()
    );

    // socket_bind: success (host=nil) then syscall error (bind twice on same socket)
    let bind_ok = (socket_bind.func)(vec![Value::Integer(tcp_id), Value::Nil, Value::Integer(0)])
        .unwrap();
    assert!(result_ok_value(bind_ok).is_some());
    let bind_err = (socket_bind.func)(vec![Value::Integer(tcp_id), Value::Nil, Value::Integer(0)])
        .unwrap();
    assert_result_err(bind_err);

    // socket_connect: host type error + port range error + syscall error (connect to port 0)
    assert!(
        (socket_connect.func)(vec![Value::Integer(tcp_id), Value::Nil, Value::Integer(0)]).is_err()
    );
    assert!(
        (socket_connect.func)(vec![
            Value::Integer(tcp_id),
            Value::String("127.0.0.1".to_string()),
            Value::Integer(70000)
        ])
        .is_err()
    );
    let conn_err = (socket_connect.func)(vec![
        Value::Integer(tcp_id),
        Value::String("127.0.0.1".to_string()),
        Value::Integer(0),
    ])
    .unwrap();
    assert_result_err(conn_err);

    // socket_listen: syscall error (listen on UDP socket)
    let listen_err = (socket_listen.func)(vec![Value::Integer(udp_id), Value::Integer(1)]).unwrap();
    assert_result_err(listen_err);

    // socket_accept: syscall error (accept on non-listening socket)
    let accept_err = (socket_accept.func)(vec![Value::Integer(tcp_id)]).unwrap();
    assert_result_err(accept_err);

    // socket_set_nonblocking: success path, then use udp_recv_from to hit recvfrom error quickly
    let nonblock_ok =
        (socket_set_nonblocking.func)(vec![Value::Integer(udp_id), Value::Bool(true)]).unwrap();
    assert!(result_ok_value(nonblock_ok).is_some());
    let recv_err = (udp_recv_from.func)(vec![Value::Integer(udp_id), Value::Integer(1)]).unwrap();
    assert_result_err(recv_err);

    // socket_set_rcvbuf/sndbuf: argument validation errors (no syscalls)
    assert!(
        (socket_set_rcvbuf.func)(vec![Value::Integer(tcp_id), Value::Integer(-1)]).is_err()
    );
    assert!(
        (socket_set_sndbuf.func)(vec![Value::Integer(tcp_id), Value::Integer(-1)]).is_err()
    );

    // udp_send_to: argument validation errors (avoid actual send)
    assert!((udp_send_to.func)(vec![
            Value::Integer(udp_id),
            Value::Nil,
            Value::String("127.0.0.1".to_string()),
            Value::Integer(0),
        ])
        .is_err());
    assert!((udp_send_to.func)(vec![
            Value::Integer(udp_id),
            bytes(b"hi"),
            Value::Nil,
            Value::Integer(0),
        ])
        .is_err());
    assert!((udp_send_to.func)(vec![
            Value::Integer(udp_id),
            bytes(b"hi"),
            Value::String("127.0.0.1".to_string()),
            Value::Integer(70000),
        ])
        .is_err());

    // tcp_send/tcp_recv: bytes arg validation + syscall errors on unconnected socket.
    assert!((tcp_send.func)(vec![Value::Integer(tcp_id), Value::Nil]).is_err());
    let send_err = (tcp_send.func)(vec![Value::Integer(tcp_id), bytes(b"hi")]).unwrap();
    assert_result_err(send_err);
    let recv_err = (tcp_recv.func)(vec![Value::Integer(tcp_id), Value::Integer(1)]).unwrap();
    assert_result_err(recv_err);

    // Clean up sockets (ignore close errors; coverage is the goal here).
    let _ = (socket_close.func)(vec![Value::Integer(tcp_id)]);
    let _ = (socket_close.func)(vec![Value::Integer(udp_id)]);
}
