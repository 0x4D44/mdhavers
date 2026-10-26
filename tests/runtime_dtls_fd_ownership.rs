//! `dtls_handshake` hands its dup'd socket fd to the Rust runtime, which must own and close
//! it on every path, with the C caller closing nothing (MDH-BUG-FLU-00131).
//!
//! Neither half shows in a native program: a double close of a just-closed fd number is a
//! harmless EBADF unless another thread reused it. So two C drivers check each half:
//!
//! * driver A links the C runtime against a stub `__mdh_rs_dtls_handshake` that closes the
//!   fd and opens `/dev/null` (which takes the same number), as another thread might. The
//!   C wrapper must leave that new fd alone.
//! * driver B links the real Rust runtime archive and calls `__mdh_rs_dtls_handshake`
//!   with invalid DTLS handles; the fd it was given must be closed afterwards.

#![cfg(all(feature = "llvm", unix))]

use std::path::Path;
use std::process::{Command, Stdio};

const OUT_DIR: &str = env!("OUT_DIR");

const COMMON: &str = r#"
#include "mdh_runtime.h"
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

typedef struct {
    uint8_t ok;
    MdhValue value;
    MdhValue error;
} MdhRsResult;

static int fd_is_open(int fd) { return fcntl(fd, F_GETFD) != -1; }
"#;

const DRIVER_A: &str = r#"
static int reused_fd = -1;

MdhRsResult __mdh_rs_dtls_handshake(MdhValue dtls, MdhValue sock) {
    (void)dtls;
    close((int)sock.data);
    reused_fd = open("/dev/null", O_RDONLY);
    MdhRsResult r;
    memset(&r, 0, sizeof r);
    r.ok = 0;
    r.error = __mdh_make_string("handshake failed");
    return r;
}

int main(void) {
    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    MdhValue res = __mdh_dtls_handshake(__mdh_make_int(7), __mdh_make_int(sock));
    (void)res;
    printf("reused_fd_still_open=%d\n", fd_is_open(reused_fd));
    printf("original_still_open=%d\n", fd_is_open(sock));
    return 0;
}
"#;

const DRIVER_B: &str = r#"
extern MdhRsResult __mdh_rs_dtls_handshake(MdhValue dtls, MdhValue sock);

static void probe(const char *label, int64_t handle) {
    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    int owned = dup(sock);
    MdhRsResult r = __mdh_rs_dtls_handshake(__mdh_make_int(handle), __mdh_make_int(owned));
    printf("%s ok=%d fd_closed=%d original_open=%d\n", label, r.ok, !fd_is_open(owned), fd_is_open(sock));
    close(sock);
}

int main(void) {
    probe("unknown", 424242);
    probe("zero", 0);
    probe("negative", -5);
    return 0;
}
"#;

fn cc() -> String {
    std::env::var("CC").unwrap_or_else(|_| "cc".to_string())
}

fn compile(dir: &Path, name: &str, driver: &str, objects: &[String], extra: &[&str]) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = dir.join(format!("{name}.c"));
    std::fs::write(&src, format!("{COMMON}{driver}")).unwrap();
    let exe = dir.join(name);
    let out = Command::new(cc())
        .current_dir(root)
        .args(["-std=gnu17", "-O1", "-w", "-Iruntime", "-Iruntime/platform"])
        .arg(&src)
        .args(objects)
        .args(extra)
        .args(["-lm", "-pthread", "-o"])
        .arg(&exe)
        .stdin(Stdio::null())
        .output()
        .expect("run the C compiler");
    assert!(
        out.status.success(),
        "{name} build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&exe)
        .stdin(Stdio::null())
        .output()
        .expect("run the driver");
    assert!(run.status.success(), "{name} crashed: {:?}", run.status);
    String::from_utf8_lossy(&run.stdout).into_owned()
}

#[test]
fn the_c_wrapper_does_not_close_the_dup_fd_again() {
    let dir = tempfile::tempdir().unwrap();
    let unresolved = if cfg!(target_os = "macos") {
        "-Wl,-undefined,dynamic_lookup"
    } else {
        "-Wl,--unresolved-symbols=ignore-all"
    };
    let out = compile(
        dir.path(),
        "driver_a",
        DRIVER_A,
        &[
            "runtime/mdh_runtime.c".to_string(),
            "runtime/gc_stub.c".to_string(),
            "runtime/platform/platform_unix.c".to_string(),
        ],
        &[unresolved],
    );
    assert_eq!(
        out, "reused_fd_still_open=1\noriginal_still_open=1\n",
        "__mdh_dtls_handshake closed an fd it no longer owns"
    );
}

#[test]
fn the_rust_runtime_closes_the_fd_on_an_invalid_dtls_handle() {
    let dir = tempfile::tempdir().unwrap();
    let archives: Vec<String> = ["mdh_runtime_c.a", "mdh_runtime_rs.a", "gc_stub_c.a"]
        .iter()
        .map(|a| format!("{OUT_DIR}/{a}"))
        .collect();
    let mut extra = Vec::new();
    // The runtime archives call into each other: ELF linkers need a group.
    let mut objects = Vec::new();
    if cfg!(target_os = "macos") {
        objects.extend(archives);
    } else {
        objects.push("-Wl,--start-group".to_string());
        objects.extend(archives);
        objects.push("-Wl,--end-group".to_string());
        extra.push("-ldl");
    }
    let out = compile(dir.path(), "driver_b", DRIVER_B, &objects, &extra);
    assert_eq!(
        out,
        "unknown ok=0 fd_closed=1 original_open=1\n\
         zero ok=0 fd_closed=1 original_open=1\n\
         negative ok=0 fd_closed=1 original_open=1\n",
        "__mdh_rs_dtls_handshake left its fd open on an error path"
    );
}
