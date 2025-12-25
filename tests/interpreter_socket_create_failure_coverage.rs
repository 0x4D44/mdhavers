#![cfg(all(feature = "native", unix))]

use std::fs::File;
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

struct RlimitGuard {
    prev: libc::rlimit,
}

impl Drop for RlimitGuard {
    fn drop(&mut self) {
        unsafe {
            libc::setrlimit(libc::RLIMIT_NOFILE, &self.prev);
        }
    }
}

#[test]
fn interpreter_socket_udp_tcp_creation_error_branches_cover_emfile_for_coverage() {
    let interp = Interpreter::new();
    let socket_udp = native(&interp, "socket_udp");
    let socket_tcp = native(&interp, "socket_tcp");

    let mut prev = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let rc = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut prev) };
    assert_eq!(rc, 0, "getrlimit failed");

    let guard = RlimitGuard { prev };

    let new_soft = std::cmp::min(guard.prev.rlim_cur as u64, 64_u64) as libc::rlim_t;
    let next = libc::rlimit {
        rlim_cur: new_soft,
        rlim_max: guard.prev.rlim_max,
    };
    let rc = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &next) };
    assert_eq!(rc, 0, "setrlimit failed");

    let mut files = Vec::new();
    loop {
        match File::open("/dev/null") {
            Ok(file) => files.push(file),
            Err(_) => break,
        }
    }

    assert_result_err((socket_udp.func)(vec![]).unwrap());
    assert_result_err((socket_tcp.func)(vec![]).unwrap());

    drop(files);
    drop(guard);
}

