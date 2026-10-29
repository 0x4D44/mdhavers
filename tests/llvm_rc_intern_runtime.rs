//! RC7 D: exercise the intern table directly, including races and unpublished blocks.
#![cfg(all(feature = "llvm", unix))]

use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Stdio};

const DRIVER: &str = r#"
#include "mdh_runtime.h"
#include <assert.h>
#include <pthread.h>
#include <sched.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>

extern MdhValue __mdh_chr(MdhValue);
extern MdhValue __mdh_char_at(MdhValue, MdhValue);
extern MdhValue __mdh_char_at_interned(MdhValue, MdhValue);
extern int64_t __mdh_intern_live_blocks(void);

static MdhValue sources[1024];
static int ready;
static void *hammer(void *unused) {
    (void)unused;
    __atomic_fetch_add(&ready, 1, __ATOMIC_ACQ_REL);
    while (__atomic_load_n(&ready, __ATOMIC_ACQUIRE) != 8) sched_yield();
    for (int i = 0; i < 10000; i++) {
        MdhValue c = __mdh_char_at_interned(sources[0], __mdh_make_int(0));
        assert(strcmp(__mdh_get_string(c), __mdh_get_string(sources[0])) == 0);
    }
    for (int repeat = 0; repeat < 8; repeat++) {
        for (int i = 0; i < 1024; i++) {
            MdhValue c = __mdh_char_at_interned(sources[i], __mdh_make_int(0));
            assert(strcmp(__mdh_get_string(c), __mdh_get_string(sources[i])) == 0);
        }
    }
    return NULL;
}

static void error_text(MdhValue s, MdhValue idx, int interned, char *out) {
    jmp_buf env;
    __mdh_try_push(&env);
    if (setjmp(env) == 0) {
        if (interned) __mdh_char_at_interned(s, idx); else __mdh_char_at(s, idx);
        assert(!"expected a caught char_at error");
    } else {
        snprintf(out, 256, "%s", __mdh_get_string(__mdh_get_last_error()));
    }
    __mdh_try_pop();
}

int main(int argc, char **argv) {
    struct rlimit cap = {1048576, 1048576};
    setrlimit(RLIMIT_FSIZE, &cap);
    assert(argc == 2);
    __mdh_rc_enable();
    __mdh_rc_init();
    if (strcmp(argv[1], "threads") == 0) {
        for (int i = 0; i < 1024; i++) sources[i] = __mdh_chr(__mdh_make_int(0x20000 + i));
        /* As at first spawn: disable non-atomic fast counts before starting threads. */
        __mdh_line_threaded = 1;
        __mdh_rc_fast = 0;
        pthread_t threads[8];
        for (int i = 0; i < 8; i++) assert(pthread_create(&threads[i], NULL, hammer, NULL) == 0);
        for (int i = 0; i < 8; i++) assert(pthread_join(threads[i], NULL) == 0);
        /* Four pages and exactly 1024 entries: CAS losers must have freed their blocks. */
        assert(__mdh_intern_live_blocks() == 1028);
        for (int i = 0; i < 1024; i++) __mdh_release(sources[i]);
        puts("8 threads, 1024 characters, 1028 blocks");
    } else if (strcmp(argv[1], "errors") == 0) {
        const char *bad[] = {"\xc0\xaf", "\xe2\x82", "\xf4\x90\x80\x80", "a\xff"};
        for (int i = 0; i < 4; i++) {
            MdhValue s = __mdh_make_string(bad[i]);
            char old[256], interned[256];
            error_text(s, __mdh_make_int(-1), 0, old);
            error_text(s, __mdh_make_int(-1), 1, interned);
            assert(strcmp(old, "Invalid UTF-8 in string") == 0);
            assert(strcmp(old, interned) == 0);
            __mdh_release(s);
        }
        MdhValue s = __mdh_make_string("a");
        MdhValue inputs[] = {s, __mdh_make_int(1), s};
        MdhValue indices[] = {__mdh_make_int(5), __mdh_make_int(0), s};
        for (int i = 0; i < 3; i++) {
            char old[256], interned[256];
            error_text(inputs[i], indices[i], 0, old);
            error_text(inputs[i], indices[i], 1, interned);
            assert(strcmp(old, interned) == 0);
        }
        __mdh_release(s);
        /* Clear the counted final error; the OOM error is immortal. */
        jmp_buf env;
        __mdh_try_push(&env);
        if (setjmp(env) == 0) __mdh_raise_out_of_memory();
        __mdh_try_pop();
        puts("7 errors match legacy char_at");
    } else if (strcmp(argv[1], "oom") == 0) {
        MdhValue s = __mdh_make_string("\xce\xbb");
        jmp_buf env;
        __mdh_try_push(&env);
        if (setjmp(env) == 0) {
            __mdh_char_at_interned(s, __mdh_make_int(0));
            assert(!"expected the injected allocation failure");
        } else {
            assert(strstr(__mdh_get_string(__mdh_get_last_error()), "ower muckle"));
        }
        __mdh_try_pop();
        const char *mode = getenv("MDH_TEST_FAIL_INTERN_ALLOC");
        assert(__mdh_intern_live_blocks() == (strcmp(mode, "page") == 0 ? 0 : 1));
        MdhValue c = __mdh_char_at_interned(s, __mdh_make_int(0));
        assert(strcmp(__mdh_get_string(c), "\xce\xbb") == 0);
        assert(__mdh_intern_live_blocks() == 2);
        __mdh_release(s);
        puts("caught ower muckle, retry works, 2 blocks");
    } else if (strcmp(argv[1], "immortal") == 0) {
        MdhValue s = __mdh_make_string("x");
        MdhValue c = __mdh_char_at_interned(s, __mdh_make_int(0));
        MdhObjHeader *h = (MdhObjHeader *)(intptr_t)c.data - 1;
        assert(h->flags == (MDH_OBJ_MAGIC | MDH_OBJ_IMMORTAL));
        assert(h->kind == MDH_KIND_STRING);
        MdhValue list = __mdh_make_list(1);
        __mdh_list_push(list, __mdh_make_nil());
        for (int i = 0; i < 1000000; i++) {
            __mdh_list_set(list, 0, c);
            assert(h->refcount == MDH_REFCOUNT_IMMORTAL);
            __mdh_list_set(list, 0, __mdh_make_nil());
        }
        __mdh_release(list);
        __mdh_release(s);
        __mdh_rc_check(c, "interned use after a million stores");
        puts(__mdh_get_string(c));
    } else {
        abort();
    }
    assert(__mdh_rc_live(MDH_KIND_STRING) == 0);
    assert(__mdh_rc_live(MDH_KIND_LIST) == 0);
    return 0;
}
"#;

fn run_driver(mode: &str, hook: &str, expected: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let scratch = root.join(".scratch");
    fs::create_dir_all(&scratch).unwrap();
    let dir = tempfile::Builder::new()
        .prefix("intern-runtime-")
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
    assert!(
        build.status.success(),
        "C driver build: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let stdout = dir.path().join("stdout");
    let stderr = dir.path().join("stderr");
    let status = Command::new("deltic")
        .args(["timeout", "30"])
        .arg(&executable)
        .arg(mode)
        .env("MDH_RC", "1")
        .env("MDH_RC_NOFREE", "1")
        .env("MDH_RC_REPORT", "1")
        .env("MDH_TEST_FAIL_INTERN_ALLOC", hook)
        .stdin(Stdio::null())
        .stdout(File::create(&stdout).unwrap())
        .stderr(File::create(&stderr).unwrap())
        .status()
        .unwrap();
    assert!(fs::metadata(&stdout).unwrap().len() + fs::metadata(&stderr).unwrap().len() < 1048576);
    let stderr = fs::read_to_string(stderr).unwrap();
    assert!(
        status.success(),
        "intern driver {mode}/{hook}: {status}\n{stderr}"
    );
    assert_eq!(fs::read_to_string(stdout).unwrap(), expected);
    assert!(stderr.contains(" total=0 "), "{stderr}");
}

#[test]
fn d2_eight_threads_publish_one_entry_per_character() {
    run_driver(
        "threads",
        "count",
        "8 threads, 1024 characters, 1028 blocks\n",
    );
}

#[test]
fn d4_invalid_utf8_and_type_errors_match_the_legacy_helper() {
    run_driver("errors", "count", "7 errors match legacy char_at\n");
}

#[test]
fn d5_page_failure_frees_the_unpublished_block() {
    run_driver("oom", "page", "caught ower muckle, retry works, 2 blocks\n");
}

#[test]
fn d5_entry_failure_frees_the_unpublished_block() {
    run_driver(
        "oom",
        "entry",
        "caught ower muckle, retry works, 2 blocks\n",
    );
}

#[test]
fn d6_interned_headers_stay_immortal_during_a_million_list_stores() {
    run_driver("immortal", "count", "x\n");
}
