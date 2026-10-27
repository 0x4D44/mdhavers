#![cfg(all(feature = "llvm", unix))]
//! Reference counting, stage R1 (`wrk_docs/2026.10.25 - DES - R1 refcount insertion plan.md`):
//! the allocator's header, retain/release, DEAD marking, the underflow and header checks,
//! the staging mask and the MDH_RC_REPORT harness, tested on `runtime/gc_stub.c` alone.
//! Stage R2 frees at zero; MDH_RC_NOFREE=1 keeps R1's DEAD marking for the checker tests.
//! A small C driver links against it with stand-in runtime hooks (lists only).

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tempfile::{tempdir, TempDir};

const DRIVER: &str = r#"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include "mdh_runtime.h"

int64_t test_line = 7;
int64_t __mdh_rc_line(void) { return test_line; }
/* thread_spawn sets this before its first thread; from then on counts are atomic. */
int32_t __mdh_line_threaded = 0;
/* The checked allocators' out-of-memory raise (mdh_runtime.c). */
void __mdh_raise_out_of_memory(void) { abort(); }

void __mdh_rc_children(uint32_t kind, void *payload, MdhRcVisit visit, void *ctx) {
    if (kind != MDH_KIND_LIST) return;
    MdhList *list = (MdhList *)payload;
    for (int64_t i = 0; i < list->length; i++) visit(list->items[i], ctx);
}

/* Counts the objects the free path tore down, per kind. */
int64_t freed[MDH_KIND_COUNT];
void __mdh_rc_free_buffers(uint32_t kind, void *payload) {
    freed[kind]++;
    if (kind == MDH_KIND_LIST) __mdh_buf_free(((MdhList *)payload)->items);
}

static MdhValue str(const char *s) {
    MdhValue v; v.tag = MDH_TAG_STRING; v.data = (int64_t)(intptr_t)__mdh_obj_strdup(s); return v;
}

static MdhValue list_of(int64_t n, MdhValue *items) {
    MdhList *l = (MdhList *)__mdh_obj_alloc(MDH_KIND_LIST, sizeof(MdhList));
    l->items = (MdhValue *)__mdh_buf_alloc(sizeof(MdhValue) * (size_t)(n > 0 ? n : 1));
    for (int64_t i = 0; i < n; i++) { l->items[i] = items[i]; }
    l->length = n; l->capacity = n;
    MdhValue v; v.tag = MDH_TAG_LIST; v.data = (int64_t)(intptr_t)l; return v;
}

static int64_t count_of(MdhValue v) {
    return ((MdhObjHeader *)((char *)(intptr_t)v.data - 16))->refcount;
}

/* Each thread takes and drops references to the same objects, as threads sharing a
 * string and a list do. */
static MdhValue shared_str, shared_list;
static void *hammer(void *arg) {
    (void)arg;
    for (int k = 0; k < 1000000; k++) {
        __mdh_retain(shared_str); __mdh_retain(shared_list);
        __mdh_release(shared_list); __mdh_release(shared_str);
    }
    return NULL;
}

int main(int argc, char **argv) {
    const char *mode = argc > 1 ? argv[1] : "";
    if (strcmp(mode, "off") != 0) __mdh_rc_enable();
    __mdh_rc_init();
    if (strcmp(mode, "off") == 0) {
        /* A program compiled without counting never calls __mdh_rc_enable. */
        MdhValue s = str("hello");
        __mdh_release(s); __mdh_release(s);
        static struct { MdhObjHeader h; char c[8]; } bare = {
            { MDH_REFCOUNT_IMMORTAL, MDH_KIND_STRING, MDH_OBJ_IMMORTAL }, "bare" };
        MdhValue b; b.tag = MDH_TAG_STRING; b.data = (int64_t)(intptr_t)bare.c;
        __mdh_retain(b); __mdh_release(b); __mdh_rc_check(b, "read");
        /* Heap objects get no header at all: realloc must not look for one either. */
        char *grown = (char *)__mdh_obj_realloc((void *)(intptr_t)s.data, 64);
        strcat(grown, ", world");
        printf("%s live %lld\n", grown, (long long)__mdh_rc_live(MDH_KIND_STRING));
        return 0;
    }
    if (strcmp(mode, "counts") == 0) {
        MdhValue s = str("hello");
        printf("born %lld\n", (long long)count_of(s));
        __mdh_retain(s);
        printf("retained %lld\n", (long long)count_of(s));
        __mdh_release(s);
        printf("released %lld live %lld\n", (long long)count_of(s), (long long)__mdh_rc_live(MDH_KIND_STRING));
        test_line = 12;
        __mdh_release(s);
        MdhObjHeader *h = (MdhObjHeader *)((char *)(intptr_t)s.data - 16);
        printf("dead %d line %lld live %lld chars %s\n", (h->flags & MDH_OBJ_DEAD) != 0,
               (long long)h->refcount, (long long)__mdh_rc_live(MDH_KIND_STRING), (char *)(intptr_t)s.data);
        MdhValue i; i.tag = MDH_TAG_INT; i.data = 5;
        __mdh_retain(i); __mdh_release(i);
        printf("immediates ok\n");
        return 0;
    }
    if (strcmp(mode, "retain-dead") == 0 || strcmp(mode, "release-dead") == 0) {
        MdhValue s = str("x");
        __mdh_release(s);
        test_line = 9;
        if (mode[2] == 't') __mdh_retain(s); else __mdh_release(s);
        printf("not reached\n");
        return 0;
    }
    if (strcmp(mode, "check-dead") == 0) {
        MdhValue s = str("x");
        __mdh_rc_check(s, "blether");
        __mdh_release(s);
        __mdh_rc_check(s, "blether");
        printf("not reached\n");
        return 0;
    }
    if (strcmp(mode, "no-header") == 0) {
        static char text[64] = "abcdefghijklmnopqrstuvwxyz";
        MdhValue v; v.tag = MDH_TAG_STRING; v.data = (int64_t)(intptr_t)(text + 20);
        __mdh_retain(v);
        printf("not reached\n");
        return 0;
    }
    if (strcmp(mode, "immortal") == 0) {
        static struct { MdhObjHeader h; char c[8]; } lit = {
            { MDH_REFCOUNT_IMMORTAL, MDH_KIND_STRING, MDH_OBJ_MAGIC | MDH_OBJ_IMMORTAL }, "lit" };
        MdhValue v; v.tag = MDH_TAG_STRING; v.data = (int64_t)(intptr_t)lit.c;
        for (int k = 0; k < 3; k++) __mdh_release(v);
        __mdh_retain(v);
        printf("immortal %d\n", lit.h.refcount == MDH_REFCOUNT_IMMORTAL && lit.h.flags == (MDH_OBJ_MAGIC | MDH_OBJ_IMMORTAL));
        return 0;
    }
    if (strcmp(mode, "children") == 0) {
        MdhValue a = str("a"), b = str("b");
        MdhValue items[3] = { a, b, a };
        __mdh_retain(a); __mdh_retain(b); __mdh_retain(a);
        MdhValue l = list_of(3, items);
        __mdh_release(a); __mdh_release(b); /* the list now holds a twice and b once */
        printf("a %lld b %lld\n", (long long)count_of(a), (long long)count_of(b));
        __mdh_release(l);
        printf("live strings %lld lists %lld freed %lld %lld\n",
               (long long)__mdh_rc_live(MDH_KIND_STRING), (long long)__mdh_rc_live(MDH_KIND_LIST),
               (long long)freed[MDH_KIND_STRING], (long long)freed[MDH_KIND_LIST]);
        return 0;
    }
    if (strcmp(mode, "deep") == 0) {
        MdhValue inner = str("leaf");
        for (int k = 0; k < 1000000; k++) {
            MdhValue one[1] = { inner };
            inner = list_of(1, one); /* takes the previous object's only reference */
        }
        __mdh_release(inner);
        printf("live %lld %lld freed %lld %lld\n", (long long)__mdh_rc_live(MDH_KIND_STRING),
               (long long)__mdh_rc_live(MDH_KIND_LIST), (long long)freed[MDH_KIND_STRING],
               (long long)freed[MDH_KIND_LIST]);
        return 0;
    }
    if (strcmp(mode, "free-kinds") == 0) {
        printf("free kinds 0x%x\n", __mdh_rc_free_kinds);
        return 0;
    }
    if (strcmp(mode, "free-reuse") == 0) {
        /* Freed memory goes back to malloc: churning more objects than the RSS cap would
         * hold if nothing were freed stays small, and nothing is left live. */
        for (int k = 0; k < 2000000; k++) {
            MdhValue items[2] = { str("churn"), str("more churn") };
            __mdh_release(list_of(2, items));
        }
        struct rusage ru;
        getrusage(RUSAGE_SELF, &ru);
        long long peak_kb = (long long)ru.ru_maxrss;
#ifdef __APPLE__
        peak_kb /= 1024; /* bytes on macOS, kilobytes elsewhere */
#endif
        printf("live %lld %lld freed %lld %lld under 64 MB %d\n",
               (long long)__mdh_rc_live(MDH_KIND_STRING), (long long)__mdh_rc_live(MDH_KIND_LIST),
               (long long)freed[MDH_KIND_STRING], (long long)freed[MDH_KIND_LIST],
               peak_kb < 64 * 1024);
        return 0;
    }
    if (strcmp(mode, "threads") == 0) {
        shared_str = str("shared");
        MdhValue items[1] = { shared_str };
        __mdh_retain(shared_str);
        shared_list = list_of(1, items);
        __mdh_line_threaded = 1; /* what thread_spawn does before creating its first thread */
        pthread_t t[4];
        for (int k = 0; k < 4; k++) pthread_create(&t[k], NULL, hammer, NULL);
        for (int k = 0; k < 4; k++) pthread_join(t[k], NULL);
        printf("string %lld list %lld\n", (long long)count_of(shared_str), (long long)count_of(shared_list));
        __mdh_release(shared_str);
        __mdh_release(shared_list);
        printf("live %lld %lld\n", (long long)__mdh_rc_live(MDH_KIND_STRING),
               (long long)__mdh_rc_live(MDH_KIND_LIST));
        return 0;
    }
    if (strcmp(mode, "report") == 0) {
        MdhValue keep = str("kept");
        MdhValue gone = str("gone");
        __mdh_release(gone);
        (void)keep;
        return 0;
    }
    return 2;
}
"#;

struct Driver {
    _dir: TempDir,
    exe: PathBuf,
}

fn build_driver(kinds: &str) -> Driver {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempdir().expect("temp dir");
    let driver_c = dir.path().join("driver.c");
    std::fs::write(&driver_c, DRIVER).expect("write driver");
    let exe = dir.path().join("rc_driver");
    let status = Command::new("cc")
        .arg("-O1")
        .arg(format!("-DMDH_RC_DEFAULT_KINDS={kinds}"))
        .arg("-I")
        .arg(root.join("runtime"))
        .arg(root.join("runtime/gc_stub.c"))
        .arg(&driver_c)
        .arg("-pthread")
        .arg("-o")
        .arg(&exe)
        .status()
        .expect("run cc");
    assert!(status.success(), "cc failed");
    Driver { _dir: dir, exe }
}

fn run(driver: &Driver, mode: &str, envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(&driver.exe);
    cmd.arg(mode).stdin(Stdio::null());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run driver")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

const ALL: &str = "0x1FE";
/// R1's checker: objects reaching zero are marked DEAD and kept, not freed.
const NOFREE: &[(&str, &str)] = &[("MDH_RC_NOFREE", "1")];
/// Free every counted kind, whatever the build's default.
const FREE_ALL: &[(&str, &str)] = &[("MDH_RC_FREE_KINDS", ALL)];

#[test]
fn objects_are_born_at_one_and_marked_dead_at_zero() {
    let d = build_driver(ALL);
    let out = run(&d, "counts", NOFREE);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "born 1\nretained 2\nreleased 1 live 1\ndead 1 line 12 live 0 chars hello\nimmediates ok\n"
    );
}

#[test]
fn touching_a_dead_object_aborts_with_its_kind_and_death_line() {
    let d = build_driver(ALL);
    for (mode, op) in [
        ("retain-dead", "retain"),
        ("release-dead", "release"),
        ("check-dead", "blether"),
    ] {
        let out = run(&d, mode, NOFREE);
        assert!(!out.status.success(), "{mode} did not abort");
        assert!(!stdout(&out).contains("not reached"), "{mode}");
        let err = stderr(&out);
        assert!(
            err.contains(&format!("mdhavers rc: {op} o' a deid string object"))
                && err.contains("(it dee'd at line 7), at line"),
            "{mode}: {err}"
        );
    }
}

#[test]
fn a_heap_value_without_a_header_aborts() {
    let d = build_driver(ALL);
    let out = run(&d, "no-header", &[]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("which has nae object header"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn immortal_objects_are_never_written() {
    let d = build_driver(ALL);
    let out = run(&d, "immortal", &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "immortal 1\n");
}

#[test]
fn a_dead_container_releases_its_children() {
    let d = build_driver(ALL);
    let out = run(&d, "children", NOFREE);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "a 2 b 1\nlive strings 0 lists 0 freed 0 0\n");
    // Freeing: the list and both strings go through the free path.
    let out = run(&d, "children", FREE_ALL);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "a 2 b 1\nlive strings 0 lists 0 freed 2 1\n");
}

#[test]
fn releasing_a_million_deep_nest_does_not_recurse() {
    let d = build_driver(ALL);
    let out = run(&d, "deep", NOFREE);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "live 0 0 freed 0 0\n");
    // DES F6 3.9: freeing a 10^6-deep nested list does not overflow the stack.
    let out = run(&d, "deep", FREE_ALL);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "live 0 0 freed 1 1000000\n");
}

#[test]
fn counts_are_atomic_once_threads_exist() {
    // Four threads each take and drop a reference to one string and one list a million times
    // (stage R3). With plain counts, lost updates leave a wrong count or kill the shared
    // objects early (MDH_RC_NOFREE=1 then aborts on the next touch instead of crashing).
    let d = build_driver(ALL);
    let out = run(&d, "threads", NOFREE);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "string 2 list 1\nlive 0 0\n");
}

#[test]
fn freed_objects_give_their_memory_back() {
    // 2 million lists of two strings would be hundreds of MB if kept (MDH_RC_NOFREE=1 keeps them).
    let d = build_driver(ALL);
    let out = run(&d, "free-reuse", FREE_ALL);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "live 0 0 freed 4000000 2000000 under 64 MB 1\n"
    );
}

#[test]
fn the_freed_kinds_follow_the_switches() {
    let d = build_driver(ALL);
    let kinds = |envs: &[(&str, &str)]| stdout(&run(&d, "free-kinds", envs));
    // Every counted kind but native objects (the tri runtime keys them by address).
    assert_eq!(kinds(&[]), "free kinds 0xfe\n");
    assert_eq!(kinds(NOFREE), "free kinds 0x0\n");
    assert_eq!(kinds(FREE_ALL), "free kinds 0xfe\n");
    assert_eq!(kinds(&[("MDH_RC_FREE_KINDS", "0x4")]), "free kinds 0x4\n");
    // Only counted kinds are freed, and MDH_RC_NOFREE wins over MDH_RC_FREE_KINDS.
    assert_eq!(
        kinds(&[("MDH_RC_KINDS", "0x2"), ("MDH_RC_FREE_KINDS", ALL)]),
        "free kinds 0x2\n"
    );
    assert_eq!(
        kinds(&[("MDH_RC_NOFREE", "1"), ("MDH_RC_FREE_KINDS", ALL)]),
        "free kinds 0x0\n"
    );
}

#[test]
fn uncounted_kinds_are_left_alone() {
    // Strings only (the first staging step): the list never dies, so its children live on.
    let d = build_driver("0x2");
    let out = run(&d, "children", &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "a 2 b 1\nlive strings 2 lists 1 freed 0 0\n");
    // MDH_RC_KINDS overrides the build's default.
    let out = run(&d, "children", &[("MDH_RC_KINDS", ALL), FREE_ALL[0]]);
    assert_eq!(stdout(&out), "a 2 b 1\nlive strings 0 lists 0 freed 2 1\n");
}

#[test]
fn nothing_is_counted_until_a_program_enables_counting() {
    // Code compiled without MDH_RC=1 takes no references; even MDH_RC_KINDS cannot turn
    // counting on under it, its static strings (no magic) pass every check, and its heap
    // objects get no header (no cost over the runtime before counting).
    let d = build_driver(ALL);
    let out = run(&d, "off", &[("MDH_RC_KINDS", ALL)]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "hello, world live 0\n");
}

#[test]
fn rc_report_prints_live_objects_per_kind_at_exit() {
    let d = build_driver(ALL);
    let quiet = run(&d, "report", &[]);
    assert!(quiet.status.success());
    assert_eq!(stderr(&quiet), "");
    let out = run(&d, "report", &[("MDH_RC_REPORT", "1")]);
    assert!(out.status.success());
    assert_eq!(
        stderr(&out),
        "mdhavers rc: live at exit: string=1 list=0 dict=0 closure=0 instance=0 bytes=0 \
         bound method=0 native=0 total=1 (counted kinds 0x1fe, freed kinds 0xfe)\n"
    );
}
