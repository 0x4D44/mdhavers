use mdhavers::{parse, Interpreter};

#[test]
fn interpreter_event_loop_timer_and_read() {
    let code = r#"
# Timer event
ken loop = event_loop_new()

dae on_timer(ev) {
    # no-op
}

timer_after(loop, 5, on_timer)
ken events = event_loop_poll(loop, 50)
ken saw_timer = nae
fer ev in events {
    gin ev["kind"] == "timer" {
        saw_timer = aye
    }
}

gin saw_timer { blether "timer_ok" } ither { blether "timer_fail" }

# Read event (UDP)
ken rloop = event_loop_new()

dae on_read(ev) {
    # no-op
}

ken udp_sock = naething
ken udp_port = -1
fer p in 42000..42100 {
    ken s = socket_udp()
    gin nae s["ok"] { haud }
    ken sock = s["value"]
    ken r = socket_bind(sock, "127.0.0.1", p)
    gin r["ok"] {
        udp_sock = sock
        udp_port = p
        brak
    } ither {
        socket_close(sock)
    }
}

gin udp_port < 0 {
    blether "read_fail"
} ither {
    event_watch_read(rloop, udp_sock, on_read)
    ken sender_res = socket_udp()
    gin sender_res["ok"] {
        ken sender = sender_res["value"]
        ken msg = bytes_from_string("ping")
        udp_send_to(sender, msg, "127.0.0.1", udp_port)
        ken evs = event_loop_poll(rloop, 50)
        ken saw_read = nae
        fer ev in evs {
            gin ev["kind"] == "read" {
                saw_read = aye
            }
        }
        gin saw_read { blether "read_ok" } ither { blether "read_fail" }
        socket_close(sender)
    } ither {
        blether "read_fail"
    }
    socket_close(udp_sock)
}
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "timer_ok\nread_ok");
}
