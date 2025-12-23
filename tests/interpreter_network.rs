#![cfg(all(feature = "native", unix))]

use mdhavers::{parse, Interpreter};

#[test]
fn interpreter_network_udp_tcp_dns() {
    let code = r#"
ken dns = dns_lookup("localhost")
gin dns["ok"] an len(dns["value"]) > 0 {
    blether "dns_ok"
} ither {
    blether "dns_fail"
}

# UDP loopback echo
ken udp_sock = naething
ken udp_port = -1
fer p in 40000..40100 {
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
    blether "udp_fail"
} ither {
    ken sender_res = socket_udp()
    gin sender_res["ok"] {
        ken sender = sender_res["value"]
        ken msg = bytes_from_string("ping")
        ken sent = udp_send_to(sender, msg, "127.0.0.1", udp_port)
        gin sent["ok"] {
            ken recv = udp_recv_from(udp_sock, 16)
            gin recv["ok"] an len(recv["value"]["buf"]) > 0 {
                blether "udp_ok"
            } ither {
                blether "udp_fail"
            }
        } ither {
            blether "udp_fail"
        }
        socket_close(sender)
    } ither {
        blether "udp_fail"
    }
    socket_close(udp_sock)
}

# TCP loopback echo
ken tcp_sock = naething
ken tcp_port = -1
fer p in 41000..41100 {
    ken s = socket_tcp()
    gin nae s["ok"] { haud }
    ken sock = s["value"]
    socket_set_reuseaddr(sock, aye)
    ken r = socket_bind(sock, "127.0.0.1", p)
    gin r["ok"] {
        ken l = socket_listen(sock, 4)
        gin l["ok"] {
            tcp_sock = sock
            tcp_port = p
            brak
        } ither {
            socket_close(sock)
        }
    } ither {
        socket_close(sock)
    }
}

gin tcp_port < 0 {
    blether "tcp_fail"
} ither {
    ken client_res = socket_tcp()
    gin nae client_res["ok"] {
        blether "tcp_fail"
    } ither {
        ken client = client_res["value"]
        ken c = socket_connect(client, "127.0.0.1", tcp_port)
        gin nae c["ok"] {
            blether "tcp_fail"
            socket_close(client)
        } ither {
            ken a = socket_accept(tcp_sock)
            gin a["ok"] {
                ken server = a["value"]["sock"]
                ken msg = bytes_from_string("pong")
                ken sent = tcp_send(client, msg)
                gin sent["ok"] {
                    ken recv = tcp_recv(server, 16)
                    gin recv["ok"] an len(recv["value"]) > 0 {
                        blether "tcp_ok"
                    } ither {
                        blether "tcp_fail"
                    }
                } ither {
                    blether "tcp_fail"
                }
                socket_close(server)
            } ither {
                blether "tcp_fail"
            }
            socket_close(client)
        }
    }
    socket_close(tcp_sock)
}
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "dns_ok\nudp_ok\ntcp_ok");
}

#[test]
fn interpreter_socket_option_setters() {
    let code = r#"
ken udp = socket_udp()
gin udp["ok"] {
    ken sock = udp["value"]
    socket_set_nonblocking(sock, aye)
    socket_set_ttl(sock, 64)
    socket_set_rcvbuf(sock, 4096)
    socket_set_sndbuf(sock, 4096)
    socket_close(sock)
}

ken tcp = socket_tcp()
gin tcp["ok"] {
    ken sock = tcp["value"]
    socket_set_reuseaddr(sock, aye)
    socket_set_reuseport(sock, aye)
    socket_set_nodelay(sock, aye)
    socket_close(sock)
}

blether "opts_ok"
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "opts_ok");
}
