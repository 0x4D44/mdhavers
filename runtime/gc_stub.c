/**
 * gc_stub.c - the single allocator for compiled mdhavers programs, and reference counting
 *
 * Compiled code and the runtimes allocate only through these functions (see
 * "Allocation" in mdh_runtime.h). Memory management is reference counting only
 * (wrk_docs DEC, round 2, F6 Q7): there is no garbage collector behind this file.
 *
 * Every object carries an MdhObjHeader in front of its payload (DES F6 3.2). Stage R1
 * (wrk_docs "2026.10.25 - DES - R1 refcount insertion plan") keeps counts; stage R2 frees:
 * an object whose count reaches zero has its children released and is then freed with
 * the buffers it owns. MDH_RC_NOFREE=1 at run time keeps R1's checker instead: the object
 * is marked DEAD and kept, and any later retain or release of it aborts, naming the kind
 * and the line it died at, so every would-be use-after-free shows up without a crash.
 *
 * Per-kind counters (allocated, died) are kept while counting is on (a program compiled
 * with MDH_RC=1, see __mdh_rc_enable); MDH_RC_REPORT=1 prints the live
 * objects per kind at exit. Build with -DMDH_ALLOC_STATS to count buffers too.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "mdh_runtime.h"

_Static_assert(sizeof(MdhObjHeader) == 16, "the object header keeps payloads 16-byte aligned");

/* ========== Counters ========== */

static int64_t __mdh_rc_allocated[MDH_KIND_COUNT];
static int64_t __mdh_rc_died[MDH_KIND_COUNT];

static void __mdh_alloc_note(uint32_t kind) {
#ifndef MDH_ALLOC_STATS
    /* Only a program compiled with counting pays for the counters. */
    if (__mdh_rc_kinds == 0) return;
#endif
    if (kind >= MDH_KIND_COUNT) kind = MDH_KIND_BUFFER;
    __atomic_fetch_add(&__mdh_rc_allocated[kind], 1, __ATOMIC_RELAXED);
}

int64_t __mdh_alloc_count(uint32_t kind) {
    if (kind >= MDH_KIND_COUNT) return -1;
#ifndef MDH_ALLOC_STATS
    if (kind == MDH_KIND_BUFFER) return -1;
#endif
    return __atomic_load_n(&__mdh_rc_allocated[kind], __ATOMIC_RELAXED);
}

int64_t __mdh_rc_live(uint32_t kind) {
    if (kind == MDH_KIND_BUFFER || kind >= MDH_KIND_COUNT) return -1;
    return __atomic_load_n(&__mdh_rc_allocated[kind], __ATOMIC_RELAXED) -
           __atomic_load_n(&__mdh_rc_died[kind], __ATOMIC_RELAXED);
}

/* ========== Allocation ========== */

static MdhObjHeader *__mdh_obj_header(const void *payload) {
    return (MdhObjHeader *)((char *)payload - sizeof(MdhObjHeader));
}

void *__mdh_obj_alloc(uint32_t kind, size_t size) {
    /* Counting off (__mdh_rc_kinds is fixed before main allocates anything): nothing reads
     * a heap object's header, so it gets none, as before reference counting existed. */
    if (__mdh_rc_kinds == 0) {
        __mdh_alloc_note(kind);
        return malloc(size);
    }
    if (size > SIZE_MAX - sizeof(MdhObjHeader)) return NULL;
    MdhObjHeader *h = (MdhObjHeader *)malloc(sizeof(MdhObjHeader) + size);
    if (!h) return NULL;
    h->refcount = 1;
    h->kind = kind;
    h->flags = MDH_OBJ_MAGIC;
    __mdh_alloc_note(kind);
    return h + 1;
}

static const char *__mdh_rc_kind_name(uint32_t kind) {
    static const char *const names[MDH_KIND_COUNT] = {
        "buffer", "string", "list", "dict", "closure", "instance", "bytes", "bound method", "native",
    };
    return kind < MDH_KIND_COUNT ? names[kind] : "unknown";
}

static void __mdh_rc_fail(const char *op, const MdhObjHeader *h) {
    fprintf(
        stderr,
        "mdhavers rc: %s o' a deid %s object %p (it dee'd at line %lld), at line %lld\n",
        op,
        __mdh_rc_kind_name(h->kind),
        (const void *)(h + 1),
        (long long)h->refcount,
        (long long)__mdh_rc_line()
    );
    fflush(stderr);
    abort();
}

static void __mdh_rc_no_header(const char *op, const void *payload) {
    fprintf(
        stderr,
        "mdhavers rc: %s o' %p, which has nae object header (static or interior pointer), at "
        "line %lld\n",
        op,
        payload,
        (long long)__mdh_rc_line()
    );
    fflush(stderr);
    abort();
}

void *__mdh_obj_realloc(void *obj, size_t size) {
    if (!obj) return __mdh_obj_alloc(MDH_KIND_STRING, size);
    if (__mdh_rc_kinds == 0) return realloc(obj, size); /* headerless, see __mdh_obj_alloc */
    if (size > SIZE_MAX - sizeof(MdhObjHeader)) return NULL;
    MdhObjHeader *h = __mdh_obj_header(obj);
    if ((h->flags & MDH_OBJ_MAGIC_MASK) != MDH_OBJ_MAGIC) __mdh_rc_no_header("realloc", obj);
    if (h->flags & MDH_OBJ_DEAD) __mdh_rc_fail("realloc", h);
    if (h->flags & MDH_OBJ_IMMORTAL) {
        fprintf(stderr, "mdhavers rc: realloc o' an immortal object %p\n", obj);
        fflush(stderr);
        abort();
    }
    /* Growing in place is only legal for an object nobody else holds (DES F6 3.7):
     * realloc may move it, leaving any other holder pointing at freed memory. */
    int64_t count = __atomic_load_n(&h->refcount, __ATOMIC_ACQUIRE);
    if (h->kind < MDH_KIND_COUNT && (__mdh_rc_kinds & (1u << h->kind)) && count != 1) {
        fprintf(
            stderr,
            "mdhavers rc: realloc o' a shared %s object %p (count %lld), at line %lld\n",
            __mdh_rc_kind_name(h->kind),
            obj,
            (long long)count,
            (long long)__mdh_rc_line()
        );
        fflush(stderr);
        abort();
    }
    MdhObjHeader *grown = (MdhObjHeader *)realloc(h, sizeof(MdhObjHeader) + size);
    return grown ? grown + 1 : NULL;
}

char *__mdh_obj_strdup(const char *s) {
    size_t n = strlen(s) + 1;
    char *out = (char *)__mdh_obj_alloc(MDH_KIND_STRING, n);
    if (out) memcpy(out, s, n);
    return out;
}

void *__mdh_buf_alloc(size_t size) {
#ifdef MDH_ALLOC_STATS
    __mdh_alloc_note(MDH_KIND_BUFFER);
#endif
    return malloc(size);
}

void *__mdh_buf_realloc(void *buf, size_t size) {
#ifdef MDH_ALLOC_STATS
    if (!buf) __mdh_alloc_note(MDH_KIND_BUFFER);
#endif
    return realloc(buf, size);
}

void __mdh_buf_free(void *buf) {
    free(buf);
}

char *__mdh_buf_strdup(const char *s) {
    size_t n = strlen(s) + 1;
    char *out = (char *)__mdh_buf_alloc(n);
    if (out) memcpy(out, s, n);
    return out;
}

/* ========== Reference counting ========== */

/* Kinds whose objects are counted (bit 1 << kind). Retain and release ignore the others,
 * so an uncounted container never releases its children. None until a program compiled
 * with counting (MDH_RC=1) calls __mdh_rc_enable: code compiled without it takes no
 * references, so counting there would only kill objects it still uses. */
uint32_t __mdh_rc_kinds = 0;

/* Counted kinds whose objects are freed on reaching zero (stage R2); the other counted
 * kinds are only marked DEAD, as in R1. Native objects are never freed: the tri runtime
 * keeps them in a table keyed by address. */
uint32_t __mdh_rc_free_kinds = 0;

/* MDH_RC_DEFAULT_KINDS are counted; MDH_RC_KINDS (a number, 0x prefix for hex) overrides
 * it for debugging. Of those, MDH_RC_DEFAULT_FREE_KINDS are freed at zero.
 * MDH_RC_NOFREE=1 frees nothing: every object that reaches zero is marked DEAD and kept,
 * so a later use aborts with the line it died at (R1's checker). MDH_RC_FREE_KINDS
 * narrows the freed kinds for bisecting. */
void __mdh_rc_enable(void) {
    const char *kinds = getenv("MDH_RC_KINDS");
    __mdh_rc_kinds = (kinds && *kinds) ? (uint32_t)strtoul(kinds, NULL, 0) : MDH_RC_DEFAULT_KINDS;
    const char *nofree = getenv("MDH_RC_NOFREE");
    const char *free_kinds = getenv("MDH_RC_FREE_KINDS");
    uint32_t freed = (free_kinds && *free_kinds) ? (uint32_t)strtoul(free_kinds, NULL, 0)
                                                 : MDH_RC_DEFAULT_FREE_KINDS;
    if (nofree && strcmp(nofree, "1") == 0) freed = 0;
    __mdh_rc_free_kinds = freed & __mdh_rc_kinds & ~(1u << MDH_KIND_NATIVE);
}

static int __mdh_rc_report_enabled = 0;

static void __mdh_rc_report(void) {
    int64_t total = 0;
    fprintf(stderr, "mdhavers rc: live at exit:");
    for (uint32_t kind = 1; kind < MDH_KIND_COUNT; kind++) {
        int64_t live = __mdh_rc_live(kind);
        total += live;
        fprintf(stderr, " %s=%lld", __mdh_rc_kind_name(kind), (long long)live);
    }
    fprintf(
        stderr,
        " total=%lld (counted kinds 0x%x, freed kinds 0x%x)\n",
        (long long)total,
        __mdh_rc_kinds,
        __mdh_rc_free_kinds
    );
    fflush(stderr);
}

void __mdh_rc_init(void) {
    const char *report = getenv("MDH_RC_REPORT");
    if (!__mdh_rc_report_enabled && report && strcmp(report, "1") == 0) {
        __mdh_rc_report_enabled = 1;
        atexit(__mdh_rc_report);
    }
}

/* Counts are plain until the program spawns its first thread and atomic from then on
 * (DES F6 3.5, stage R3). __mdh_thread_spawn sets __mdh_line_threaded before it creates
 * the thread, and never clears it, so thread creation orders every plain update before
 * the first atomic one. Retain is relaxed (taking a reference needs one already held);
 * release is acq_rel, so whatever a thread did with an object happens before the thread
 * that drops the last reference releases its children and frees it. */
static inline void __mdh_rc_inc(MdhObjHeader *h) {
    if (__builtin_expect(__mdh_line_threaded != 0, 0)) {
        __atomic_fetch_add(&h->refcount, 1, __ATOMIC_RELAXED);
    } else {
        h->refcount++;
    }
}

/* Drop one reference; true when it was the last one. */
static inline int __mdh_rc_dec(MdhObjHeader *h) {
    if (__builtin_expect(__mdh_line_threaded != 0, 0)) {
        return __atomic_sub_fetch(&h->refcount, 1, __ATOMIC_ACQ_REL) == 0;
    }
    return --h->refcount == 0;
}

/* The header of a counted object, or NULL when retain/release must leave it alone
 * (immortal, or a kind not counted). Aborts on a DEAD object or a missing header. */
static MdhObjHeader *__mdh_rc_counted(const char *op, int64_t data) {
    /* Counting off: static strings compiled without it carry no magic. */
    if (data == 0 || __mdh_rc_kinds == 0) return NULL;
    MdhObjHeader *h = __mdh_obj_header((const void *)(intptr_t)data);
    uint32_t flags = h->flags;
    if ((flags & MDH_OBJ_MAGIC_MASK) != MDH_OBJ_MAGIC) {
        __mdh_rc_no_header(op, (const void *)(intptr_t)data);
    }
    if (flags & MDH_OBJ_IMMORTAL) return NULL;
    if (flags & MDH_OBJ_DEAD) __mdh_rc_fail(op, h);
    if (h->kind >= MDH_KIND_COUNT || !(__mdh_rc_kinds & (1u << h->kind))) return NULL;
    return h;
}

void __mdh_rc_retain_obj(int64_t data) {
    MdhObjHeader *h = __mdh_rc_counted("retain", data);
    if (h) __mdh_rc_inc(h);
}

/* Objects that reached zero and still have to release their children. An explicit work
 * list, so a long chain of nested containers cannot overflow the C stack. */
typedef struct {
    MdhObjHeader **items;
    size_t len;
    size_t cap;
    MdhObjHeader *inline_items[32];
} MdhRcWork;

static void __mdh_rc_work_push(MdhRcWork *w, MdhObjHeader *h) {
    if (w->len == w->cap) {
        size_t cap = w->cap * 2;
        MdhObjHeader **grown;
        if (w->items == w->inline_items) {
            grown = (MdhObjHeader **)malloc(cap * sizeof(*grown));
            if (grown) memcpy(grown, w->items, w->len * sizeof(*grown));
        } else {
            grown = (MdhObjHeader **)realloc(w->items, cap * sizeof(*grown));
        }
        if (!grown) {
            fprintf(stderr, "mdhavers rc: oot o' memory releasing objects\n");
            abort();
        }
        w->items = grown;
        w->cap = cap;
    }
    w->items[w->len++] = h;
}

/* Drop one reference to a child; queue it when that was the last one. */
static void __mdh_rc_drop_child(MdhValue child, void *ctx) {
    if (!MDH_RC_HEAP_TAG(child.tag)) return;
    MdhObjHeader *h = __mdh_rc_counted("release", child.data);
    if (h && __mdh_rc_dec(h)) __mdh_rc_work_push((MdhRcWork *)ctx, h);
}

/* `h` reached zero: release what it holds, then free it (and anything that reaches zero
 * in turn), or mark it DEAD when its kind is not freed (see __mdh_rc_free_kinds). The
 * work list keeps teardown iterative: a million-deep nested list frees in a loop. */
static void __mdh_rc_kill(MdhObjHeader *h) {
    MdhRcWork w;
    w.items = w.inline_items;
    w.len = 0;
    w.cap = sizeof(w.inline_items) / sizeof(w.inline_items[0]);
    __mdh_rc_work_push(&w, h);
    int64_t line = __mdh_rc_line();
    while (w.len > 0) {
        MdhObjHeader *dead = w.items[--w.len];
        uint32_t kind = dead->kind;
        /* DEAD before its children are released, so a miscounted path back to it aborts
         * instead of queueing it twice. */
        dead->flags |= MDH_OBJ_DEAD;
        dead->refcount = line; /* the death line, for the abort message */
        __atomic_fetch_add(&__mdh_rc_died[kind], 1, __ATOMIC_RELAXED);
        __mdh_rc_children(kind, dead + 1, __mdh_rc_drop_child, &w);
        if (__mdh_rc_free_kinds & (1u << kind)) {
            __mdh_rc_free_buffers(kind, dead + 1);
            free(dead);
        }
    }
    if (w.items != w.inline_items) free(w.items);
}

void __mdh_rc_release_obj(int64_t data) {
    MdhObjHeader *h = __mdh_rc_counted("release", data);
    if (h && __mdh_rc_dec(h)) __mdh_rc_kill(h);
}

void __mdh_retain(MdhValue v) {
    if (MDH_RC_HEAP_TAG(v.tag)) __mdh_rc_retain_obj(v.data);
}

void __mdh_release(MdhValue v) {
    if (MDH_RC_HEAP_TAG(v.tag)) __mdh_rc_release_obj(v.data);
}

void __mdh_rc_check(MdhValue v, const char *op) {
    if (MDH_RC_HEAP_TAG(v.tag) && v.data != 0 && __mdh_rc_kinds != 0) {
        MdhObjHeader *h = __mdh_obj_header((const void *)(intptr_t)v.data);
        if ((h->flags & MDH_OBJ_MAGIC_MASK) != MDH_OBJ_MAGIC) {
            __mdh_rc_no_header(op, (const void *)(intptr_t)v.data);
        }
        if (h->flags & MDH_OBJ_DEAD) __mdh_rc_fail(op, h);
    }
}
