//! `__mdh_chunks` must not overflow int64 for a huge size (MDH-BUG-FLU-00132).
//!
//! The old `(len + n - 1) / n` count overflowed for n near INT64_MAX, which is undefined
//! behaviour in C. It only wraps quietly today (the capacity falls back to 8), so a
//! native program cannot tell it from the fix. This builds the C runtime with `-ftrapv`
//! (a signed overflow aborts) around a tiny driver and calls `__mdh_chunks` directly.

#![cfg(unix)]

use std::path::Path;
use std::process::{Command, Stdio};

const DRIVER: &str = r#"
#include "mdh_runtime.h"
#include <stdio.h>
#include <stdint.h>

static void show(int64_t len, int64_t n) {
    MdhValue l = __mdh_make_list(4);
    for (int64_t i = 0; i < len; i++) {
        __mdh_list_push(l, __mdh_make_int(i + 1));
    }
    MdhValue r = __mdh_chunks(l, __mdh_make_int(n));
    MdhList *rl = __mdh_get_list(r);
    printf("len=%lld n=%lld chunks=%lld", (long long)len, (long long)n, (long long)rl->length);
    for (int64_t c = 0; c < rl->length; c++) {
        printf(" %lld", (long long)__mdh_get_list(rl->items[c])->length);
    }
    printf("\n");
}

int main(void) {
    show(3, INT64_MAX);
    show(3, INT64_MAX - 1);
    show(3, INT64_MAX - 2);
    show(3, INT64_MAX / 2 + 1);
    show(5, 2);
    show(4, 2);
    show(0, 3);
    show(3, 3);
    return 0;
}
"#;

#[test]
fn chunks_with_a_huge_size_never_overflows_int64() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().expect("temp dir");
    let driver = dir.path().join("driver.c");
    std::fs::write(&driver, DRIVER).unwrap();
    let exe = dir.path().join("driver");
    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    // The Rust runtime helpers are not needed here; leave their symbols unresolved.
    let unresolved = if cfg!(target_os = "macos") {
        "-Wl,-undefined,dynamic_lookup"
    } else {
        "-Wl,--unresolved-symbols=ignore-all"
    };
    let build = Command::new(&cc)
        .current_dir(root)
        .args([
            "-std=gnu17",
            "-O1",
            "-ftrapv",
            "-w",
            "-Iruntime",
            "-Iruntime/platform",
        ])
        .arg(&driver)
        .args([
            "runtime/mdh_runtime.c",
            "runtime/gc_stub.c",
            "runtime/platform/platform_unix.c",
            "-lm",
            "-lpthread",
            unresolved,
            "-o",
        ])
        .arg(&exe)
        .stdin(Stdio::null())
        .output()
        .expect("run the C compiler");
    assert!(
        build.status.success(),
        "driver build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe)
        .stdin(Stdio::null())
        .output()
        .expect("run the driver");
    assert!(
        run.status.success(),
        "__mdh_chunks overflowed int64 (trapped): {:?}\n{}",
        run.status,
        String::from_utf8_lossy(&run.stdout)
    );
    let big = i64::MAX;
    let expected = format!(
        "len=3 n={big} chunks=1 3\n\
         len=3 n={} chunks=1 3\n\
         len=3 n={} chunks=1 3\n\
         len=3 n={} chunks=1 3\n\
         len=5 n=2 chunks=3 2 2 1\n\
         len=4 n=2 chunks=2 2 2\n\
         len=0 n=3 chunks=0\n\
         len=3 n=3 chunks=1 3\n",
        big - 1,
        big - 2,
        big / 2 + 1
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
}
