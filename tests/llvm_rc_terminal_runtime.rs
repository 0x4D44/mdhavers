//! RC7 revision 5: the previous error may alias a rooted, borrowed element.
#![cfg(all(feature = "llvm", unix))]

use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};

const DRIVER: &str = r#"
#include "mdh_runtime.h"
#include <assert.h>
#include <setjmp.h>
#include <stdio.h>
#include <sys/resource.h>

static int64_t count(MdhValue value) {
    return ((MdhObjHeader *)(intptr_t)value.data - 1)->refcount;
}

/* This frame contains no handler. Its local takes no reference, just as a C slot
 * rooted in the caller-owned root would do. The caller's catch abandons this frame. */
static void borrowed_frame(MdhValue root) {
    MdhValue borrowed = ((MdhList *)(intptr_t)root.data)->items[0];
    assert(count(borrowed) == 1);
    __mdh_hurl(borrowed);
    __builtin_unreachable();
}

int main(void) {
    struct rlimit cap = {1048576, 1048576};
    setrlimit(RLIMIT_FSIZE, &cap);
    __mdh_rc_enable();
    __mdh_rc_init();
    MdhValue root = __mdh_make_list(1);
    MdhValue element = __mdh_make_string("root still holds the previous error");
    __mdh_list_push(root, element);
    __mdh_release(element);
    jmp_buf handler;
    __mdh_try_push(&handler);
    if (setjmp(handler) == 0) borrowed_frame(root);
    __mdh_try_pop();
    assert(__mdh_get_last_error().data == element.data);
    assert(count(element) == 2); /* root and the thread's previous error */

    /* The second raw raise releases exactly the previous-error reference. */
    __mdh_try_push(&handler);
    if (setjmp(handler) == 0) __mdh_hurl(__mdh_make_nil());
    __mdh_try_pop();
    __mdh_rc_check(element, "rooted previous error after second raw raise");
    assert(count(element) == 1);
    puts(__mdh_get_string(element));
    assert(__mdh_rc_live(MDH_KIND_STRING) == 1);
    __mdh_release(root);
    assert(__mdh_rc_live(MDH_KIND_STRING) == 0);
    assert(__mdh_rc_live(MDH_KIND_LIST) == 0);
    return 0;
}
"#;

#[test]
fn raw_hurl_releases_only_its_previous_error_reference() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let scratch = root.join(".scratch");
    fs::create_dir_all(&scratch).unwrap();
    let dir = tempfile::Builder::new()
        .prefix("terminal-runtime-")
        .tempdir_in(scratch)
        .unwrap();
    let driver = dir.path().join("driver.c");
    fs::write(&driver, DRIVER).unwrap();
    let executable = dir.path().join("driver");
    let unresolved = if cfg!(target_os = "macos") {
        "-Wl,-undefined,dynamic_lookup"
    } else {
        "-Wl,--unresolved-symbols=ignore-all"
    };
    let build = Command::new("deltic")
        .args(["timeout", "120"])
        .arg(std::env::var("CC").unwrap_or_else(|_| "cc".into()))
        .current_dir(root)
        .args(["-std=gnu17", "-O2", "-w", "-Iruntime", "-Iruntime/platform"])
        .arg(driver)
        .args([
            "runtime/mdh_runtime.c",
            "runtime/gc_stub.c",
            "runtime/platform/platform_unix.c",
            "-lm",
            "-lpthread",
            unresolved,
            "-o",
        ])
        .arg(&executable)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(build.status.success(), "{build:?}");
    for nofree in ["0", "1"] {
        let stdout = dir.path().join("stdout");
        let stderr = dir.path().join("stderr");
        let status = Command::new("deltic")
            .args(["timeout", "30"])
            .arg(&executable)
            .env("MDH_RC", "1")
            .env("MDH_RC_NOFREE", nofree)
            .env("MDH_RC_REPORT", "1")
            .stdin(Stdio::null())
            .stdout(File::create(&stdout).unwrap())
            .stderr(File::create(&stderr).unwrap())
            .status()
            .unwrap();
        let stderr = fs::read_to_string(stderr).unwrap();
        assert!(status.success(), "{status}: {stderr}");
        assert_eq!(
            fs::read_to_string(stdout).unwrap(),
            "root still holds the previous error\n"
        );
        // The runtime is unchanged from trunk. Both its baseline and this lane
        // must leave no counted error or container alive.
        assert!(stderr.contains(" total=0 "), "{stderr}");
    }
}
