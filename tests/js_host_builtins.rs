//! JS backend: builtins the interpreter has but JS deliberately doesnae (MDH-BUG-FLU-00114).
//!
//! Sockets, TLS/DTLS/SRTP, event loops, threads an' the shell are interpreter/native-only: in
//! JS each raises a catchable "isnae available in JS" error (never a ReferenceError). File an'
//! environment builtins need Node an' say so in a browser.

use std::io::Write;
use std::process::{Command, Stdio};

fn node_available() -> bool {
    Command::new("node").arg("--version").output().is_ok()
}

fn run_js(js: &str, wrap: &str) -> String {
    let mut child = Command::new("node")
        .arg("-e")
        .arg(wrap)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start node");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(js.as_bytes())
        .expect("write js");
    let out = child.wait_with_output().expect("node output");
    assert!(
        out.status.success(),
        "node failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Run compiled JS under node as normal.
fn run_plain(source: &str) -> String {
    let js = mdhavers::compile_to_js(source).expect("compile");
    run_js(&js, "eval(require('fs').readFileSync(0,'utf8'))")
}

/// Run compiled JS as a browser would see it: nae `process`, nae `require`.
fn run_browser_like(source: &str) -> String {
    let js = mdhavers::compile_to_js(source).expect("compile");
    run_js(
        &js,
        "new Function('process','require', require('fs').readFileSync(0,'utf8'))(undefined, undefined)",
    )
}

const NO_JS_NAMES: &[&str] = &[
    "socket_udp",
    "socket_tcp",
    "socket_bind",
    "socket_close",
    "udp_send_to",
    "tcp_send",
    "dns_lookup",
    "tls_client_new",
    "tls_connect",
    "dtls_server_new",
    "srtp_create",
    "event_loop_new",
    "timer_after",
    "thread_spawn",
    "mutex_new",
    "condvar_new",
    "atomic_new",
    "chan_new",
    "shell",
    "shell_status",
];

#[test]
fn unsupported_builtins_raise_a_clear_error() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    let mut source = String::new();
    for name in NO_JS_NAMES {
        source.push_str(&format!(
            "hae_a_bash {{\n    {name}(1)\n}} gin_it_gangs_wrang err {{\n    blether err\n}}\n"
        ));
    }
    let out = run_plain(&source);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), NO_JS_NAMES.len(), "output was: {out}");
    for (name, line) in NO_JS_NAMES.iter().zip(lines) {
        assert_eq!(
            line,
            format!(
                "Jings! Something went awfy wrang: {name}() isnae available in JS - it's fer the interpreter an' native builds only"
            )
        );
    }
}

#[test]
fn unsupported_builtins_can_be_shadowed() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    let out = run_plain("dae shell(x) {\n    gie x + 1\n}\nblether shell(1)\n");
    assert_eq!(out.trim(), "2");
}

#[test]
fn file_and_env_builtins_say_so_in_a_browser() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    let source = "hae_a_bash {\n    read_file(\"x\")\n} gin_it_gangs_wrang err {\n    blether err\n}\n\
                  hae_a_bash {\n    env_get(\"HOME\")\n} gin_it_gangs_wrang err {\n    blether err\n}\n\
                  blether len(bytes(4))\n\
                  blether regex_test(\"abc\", \"b\")\n\
                  blether path_join(\"a\", \"b\")\n";
    let out = run_browser_like(source);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 5, "output was: {out}");
    assert!(lines[0].contains("read_file() needs Node"), "{}", lines[0]);
    assert!(lines[1].contains("env_get() needs Node"), "{}", lines[1]);
    assert_eq!(&lines[2..], ["4", "aye", "a/b"]);
}
