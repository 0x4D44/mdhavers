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

/* Plain until the first thread is spawned, atomic after (as the counts are, see
 * __mdh_rc_inc): __mdh_thread_spawn sets __mdh_line_threaded before the thread exists. */
static inline void __mdh_counter_add(int64_t *counter) {
    if (__builtin_expect(__mdh_line_threaded != 0, 0)) {
        __atomic_fetch_add(counter, 1, __ATOMIC_RELAXED);
    } else {
        (*counter)++;
    }
}

static void __mdh_alloc_note(uint32_t kind) {
#ifndef MDH_ALLOC_STATS
    /* Only a program compiled with counting pays for the counters. */
    if (__mdh_rc_kinds == 0) return;
#endif
    if (kind >= MDH_KIND_COUNT) kind = MDH_KIND_BUFFER;
    __mdh_counter_add(&__mdh_rc_allocated[kind]);
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

#ifdef MDH_ALLOC_FAULTS
/* Test-only, compiled into debug builds of mdhavers (build.rs) and never into release:
 * MDH_TEST_ALLOC_FAIL_OVER=N at run time makes every object allocation or reallocation
 * of N bytes or more fail, to exercise the out-of-memory paths (MDH-BUG-FLU-00184).
 * MDH_TEST_BUF_FAIL_OVER=N does the same for buffers (__mdh_buf_alloc/_realloc: list
 * items, dict payloads and other backing arrays), MDH-BUG-FLU-00189. 0 or unset: off. */
static size_t __mdh_fault_limit(const char *name, size_t *limit, int *read) {
    if (!*read) {
        const char *s = getenv(name);
        *limit = s ? (size_t)strtoull(s, NULL, 10) : 0;
        *read = 1;
    }
    return *limit;
}
static size_t __mdh_alloc_fail_over(void) {
    static int read = 0;
    static size_t limit = 0;
    return __mdh_fault_limit("MDH_TEST_ALLOC_FAIL_OVER", &limit, &read);
}
static size_t __mdh_buf_fail_over(void) {
    static int read = 0;
    static size_t limit = 0;
    return __mdh_fault_limit("MDH_TEST_BUF_FAIL_OVER", &limit, &read);
}
#define MDH_ALLOC_FAULT(size) (__mdh_alloc_fail_over() != 0 && (size) >= __mdh_alloc_fail_over())
#define MDH_BUF_FAULT(size) (__mdh_buf_fail_over() != 0 && (size) >= __mdh_buf_fail_over())
#else
#define MDH_ALLOC_FAULT(size) 0
#define MDH_BUF_FAULT(size) 0
#endif

static MdhObjHeader *__mdh_obj_header(const void *payload) {
    return (MdhObjHeader *)((char *)payload - sizeof(MdhObjHeader));
}

/* ========== Small-object cache (R4) ==========
 *
 * Objects freed at zero are kept on per-size-class free lists and handed out again by
 * __mdh_obj_alloc, instead of a free() and a malloc() each: in allocation loops those
 * were most of reference counting's cost. A class is a block size (header included) in
 * 16-byte steps up to MDH_OBJ_CACHE_MAX; a cached object's block was malloc'd at its
 * class size, and the class (1-based) sits in bits 16-23 of the header's kind field
 * (MDH_HDR_KIND strips it). Each list keeps at most MDH_OBJ_CACHE_DEPTH blocks. Only
 * while the program is single-threaded: __mdh_obj_cache_drain frees every cached block
 * at the first thread_spawn, and from then on objects are malloc'd and freed as before.
 * Off under AddressSanitizer, so a use after free is still reported. */
#define MDH_OBJ_CACHE_MAX 256
#define MDH_OBJ_CACHE_CLASSES (MDH_OBJ_CACHE_MAX / 16)
#define MDH_OBJ_CACHE_DEPTH 256
#define MDH_HDR_CLASS_SHIFT 16
#define MDH_HDR_KIND(h) ((h)->kind & 0xFFFFu)
#define MDH_HDR_CLASS(h) (((h)->kind >> MDH_HDR_CLASS_SHIFT) & 0xFFu)

#if defined(__SANITIZE_ADDRESS__)
#define MDH_OBJ_CACHE_ON 0
#elif defined(__has_feature)
#if __has_feature(address_sanitizer)
#define MDH_OBJ_CACHE_ON 0
#endif
#endif
#ifndef MDH_OBJ_CACHE_ON
#define MDH_OBJ_CACHE_ON 1
#endif

static MdhObjHeader *__mdh_obj_cache[MDH_OBJ_CACHE_CLASSES + 1];
static uint32_t __mdh_obj_cache_len[MDH_OBJ_CACHE_CLASSES + 1];

/* The next cached block is kept where the refcount was. */
#define MDH_CACHE_NEXT(h) (*(MdhObjHeader **)(void *)(h))

static inline int __mdh_obj_cache_usable(void) {
    return MDH_OBJ_CACHE_ON && !__mdh_line_threaded;
}

/* ========== List-buffer cache (R5) ==========
 *
 * A dying list's item buffer whose capacity is a power of two up to 128 values is kept on
 * a free list per size and handed out again by __mdh_buf_alloc for exactly that many
 * bytes (a new list's default 8 items, a closure cell's 1). The buffer is a malloc block
 * at least that big (a list's capacity never exceeds its buffer), so a later realloc or
 * free of it is unchanged. Same conditions as the object cache: counting on, single
 * threaded (drained at the first spawn), off under AddressSanitizer. */
#define MDH_BUF_CACHE_CLASSES 8
#define MDH_BUF_CACHE_DEPTH 128

static void *__mdh_buf_cache[MDH_BUF_CACHE_CLASSES];
static uint32_t __mdh_buf_cache_len[MDH_BUF_CACHE_CLASSES];

/* The next cached buffer is kept in its first word. */
#define MDH_BUF_NEXT(b) (*(void **)(b))

/* The cache class of an item buffer of `bytes` (16 << class), or -1. */
static inline int __mdh_buf_class(size_t bytes) {
    if (bytes < 16 || bytes > ((size_t)16 << (MDH_BUF_CACHE_CLASSES - 1)) || (bytes & (bytes - 1))) {
        return -1;
    }
    return __builtin_ctzll((unsigned long long)bytes) - 4;
}

static void __mdh_buf_cache_drain(void) {
    for (int c = 0; c < MDH_BUF_CACHE_CLASSES; c++) {
        void *b = __mdh_buf_cache[c];
        while (b) {
            void *next = MDH_BUF_NEXT(b);
            free(b);
            b = next;
        }
        __mdh_buf_cache[c] = NULL;
        __mdh_buf_cache_len[c] = 0;
    }
}

void __mdh_list_items_free(void *items, int64_t capacity) {
    if (items && capacity > 0 && __mdh_rc_kinds != 0 && __mdh_obj_cache_usable() &&
        (uint64_t)capacity <= SIZE_MAX / sizeof(MdhValue)) {
        int c = __mdh_buf_class((size_t)capacity * sizeof(MdhValue));
        if (c >= 0 && __mdh_buf_cache_len[c] < MDH_BUF_CACHE_DEPTH) {
            MDH_BUF_NEXT(items) = __mdh_buf_cache[c];
            __mdh_buf_cache[c] = items;
            __mdh_buf_cache_len[c]++;
            return;
        }
    }
    free(items);
}

/* Called by __mdh_thread_spawn before the first thread exists. */
void __mdh_obj_cache_drain(void) {
    __mdh_buf_cache_drain();
    for (uint32_t c = 1; c <= MDH_OBJ_CACHE_CLASSES; c++) {
        MdhObjHeader *h = __mdh_obj_cache[c];
        while (h) {
            MdhObjHeader *next = MDH_CACHE_NEXT(h);
            free(h);
            h = next;
        }
        __mdh_obj_cache[c] = NULL;
        __mdh_obj_cache_len[c] = 0;
    }
}

/* Free a dead object's block, or keep it for reuse. */
static inline void __mdh_obj_block_free(MdhObjHeader *h) {
    uint32_t c = MDH_HDR_CLASS(h);
    if (c != 0 && __mdh_obj_cache_usable() && __mdh_obj_cache_len[c] < MDH_OBJ_CACHE_DEPTH) {
        MDH_CACHE_NEXT(h) = __mdh_obj_cache[c];
        __mdh_obj_cache[c] = h;
        __mdh_obj_cache_len[c]++;
        return;
    }
    free(h);
}

void *__mdh_obj_alloc(uint32_t kind, size_t size) {
    if (MDH_ALLOC_FAULT(size)) return NULL;
    /* Counting off (__mdh_rc_kinds is fixed before main allocates anything): nothing reads
     * a heap object's header, so it gets none, as before reference counting existed. */
    if (__mdh_rc_kinds == 0) {
        __mdh_alloc_note(kind);
        return malloc(size);
    }
    if (size > SIZE_MAX - sizeof(MdhObjHeader)) return NULL;
    size_t total = sizeof(MdhObjHeader) + size;
    MdhObjHeader *h;
    uint32_t class_bits = 0;
    if (total <= MDH_OBJ_CACHE_MAX && __mdh_obj_cache_usable()) {
        uint32_t c = (uint32_t)((total + 15) / 16);
        h = __mdh_obj_cache[c];
        if (h) {
            __mdh_obj_cache[c] = MDH_CACHE_NEXT(h);
            __mdh_obj_cache_len[c]--;
        } else {
            h = (MdhObjHeader *)malloc((size_t)c * 16);
        }
        class_bits = c << MDH_HDR_CLASS_SHIFT;
    } else {
        h = (MdhObjHeader *)malloc(total);
    }
    if (!h) return NULL;
    h->refcount = 1;
    h->kind = kind | class_bits;
    h->flags = MDH_OBJ_MAGIC;
    if (kind == MDH_KIND_BUFFER || kind >= MDH_KIND_COUNT) h->flags |= MDH_OBJ_SLOW;
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
        __mdh_rc_kind_name(MDH_HDR_KIND(h)),
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

void __mdh_obj_immortal(void *obj) {
    if (!obj || __mdh_rc_kinds == 0) return; /* headerless: nothing counts it */
    MdhObjHeader *h = __mdh_obj_header(obj);
    h->flags |= MDH_OBJ_IMMORTAL;
    h->refcount = MDH_REFCOUNT_IMMORTAL;
    /* Never released, so never "live" in the exit report. */
    __mdh_counter_add(&__mdh_rc_died[MDH_HDR_KIND(h)]);
}

void *__mdh_obj_realloc(void *obj, size_t size) {
    if (!obj) return __mdh_obj_alloc(MDH_KIND_STRING, size);
    if (MDH_ALLOC_FAULT(size)) return NULL;
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
    uint32_t kind = MDH_HDR_KIND(h);
    if (kind < MDH_KIND_COUNT && (__mdh_rc_kinds & (1u << kind)) && count != 1) {
        fprintf(
            stderr,
            "mdhavers rc: realloc o' a shared %s object %p (count %lld), at line %lld\n",
            __mdh_rc_kind_name(kind),
            obj,
            (long long)count,
            (long long)__mdh_rc_line()
        );
        fflush(stderr);
        abort();
    }
    size_t total = sizeof(MdhObjHeader) + size;
    uint32_t c = MDH_HDR_CLASS(h);
    if (c != 0 && total <= (size_t)c * 16) return obj; /* its class block is big enough */
    MdhObjHeader *grown = (MdhObjHeader *)realloc(h, total);
    if (!grown) return NULL;
    grown->kind = MDH_HDR_KIND(grown); /* a plain malloc block from now on */
    return grown + 1;
}

char *__mdh_obj_strdup(const char *s) {
    size_t n = strlen(s) + 1;
    char *out = (char *)__mdh_obj_alloc(MDH_KIND_STRING, n);
    if (out) memcpy(out, s, n);
    return out;
}

void *__mdh_buf_alloc(size_t size) {
    if (MDH_BUF_FAULT(size)) return NULL;
#ifdef MDH_ALLOC_STATS
    __mdh_alloc_note(MDH_KIND_BUFFER);
#endif
    if (__mdh_rc_kinds != 0 && __mdh_obj_cache_usable()) {
        int c = __mdh_buf_class(size);
        if (c >= 0 && __mdh_buf_cache[c]) {
            void *b = __mdh_buf_cache[c];
            __mdh_buf_cache[c] = MDH_BUF_NEXT(b);
            __mdh_buf_cache_len[c]--;
            return b;
        }
    }
    return malloc(size);
}

void *__mdh_buf_realloc(void *buf, size_t size) {
    if (MDH_BUF_FAULT(size)) return NULL;
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

/* The allocators compiled code calls (MDH-BUG-FLU-00193): out of memory, they raise the
 * "ower muckle" error (__mdh_raise_out_of_memory, mdh_runtime.c) instead of returning
 * NULL for the program to write through. Never for the runtimes themselves, which must
 * not be longjmp'd through. A NULL for 0 bytes is not a failure. */
static inline void *__mdh_alloc_checked_result(void *p, size_t size) {
    if (__builtin_expect(p == NULL, 0) && size != 0) __mdh_raise_out_of_memory();
    return p;
}

void *__mdh_obj_alloc_checked(uint32_t kind, size_t size) {
    return __mdh_alloc_checked_result(__mdh_obj_alloc(kind, size), size);
}

void *__mdh_obj_realloc_checked(void *obj, size_t size) {
    return __mdh_alloc_checked_result(__mdh_obj_realloc(obj, size), size);
}

void *__mdh_buf_alloc_checked(size_t size) {
    return __mdh_alloc_checked_result(__mdh_buf_alloc(size), size);
}

void *__mdh_buf_realloc_checked(void *buf, size_t size) {
    return __mdh_alloc_checked_result(__mdh_buf_realloc(buf, size), size);
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

/* See mdh_runtime.h. Cleared by __mdh_thread_spawn when it sets __mdh_line_threaded. */
int32_t __mdh_rc_fast = 0;

/* Every kind an object header can carry (all but buffers, which have none). */
#define MDH_RC_OBJECT_KINDS (((1u << MDH_KIND_COUNT) - 1u) & ~(1u << MDH_KIND_BUFFER))

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
    __mdh_rc_fast = !__mdh_line_threaded &&
                    (__mdh_rc_kinds & MDH_RC_OBJECT_KINDS) == MDH_RC_OBJECT_KINDS;
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
    uint32_t kind = MDH_HDR_KIND(h);
    if (kind >= MDH_KIND_COUNT || !(__mdh_rc_kinds & (1u << kind))) return NULL;
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

static void __mdh_rc_work_grow(MdhRcWork *w) {
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

static inline void __mdh_rc_work_push(MdhRcWork *w, MdhObjHeader *h) {
    if (__builtin_expect(w->len == w->cap, 0)) __mdh_rc_work_grow(w);
    w->items[w->len++] = h;
}

/* A string that reached zero and is freed (strings hold no counted values, see
 * __mdh_rc_children): finish it at once instead of going through the work list, by the
 * same steps as __mdh_rc_kill. 1 when done. */
static inline int __mdh_rc_kill_leaf(MdhObjHeader *h) {
    if (MDH_HDR_KIND(h) != MDH_KIND_STRING || !(__mdh_rc_free_kinds & (1u << MDH_KIND_STRING))) {
        return 0;
    }
    h->flags |= MDH_OBJ_DEAD;
    __mdh_counter_add(&__mdh_rc_died[MDH_KIND_STRING]);
    __mdh_rc_free_buffers(MDH_KIND_STRING, h + 1);
    __mdh_obj_block_free(h);
    return 1;
}

/* Drop one reference to a child; queue it when that was the last one. */
static void __mdh_rc_drop_child(MdhValue child, void *ctx) {
    if (!MDH_RC_HEAP_TAG(child.tag)) return;
    MdhObjHeader *h = __mdh_rc_counted("release", child.data);
    if (h && __mdh_rc_dec(h) && !__mdh_rc_kill_leaf(h)) __mdh_rc_work_push((MdhRcWork *)ctx, h);
}

/* `h` reached zero: release what it holds, then free it (and anything that reaches zero
 * in turn), or mark it DEAD when its kind is not freed (see __mdh_rc_free_kinds). The
 * work list keeps teardown iterative: a million-deep nested list frees in a loop. */
static void __mdh_rc_kill(MdhObjHeader *h) {
    MdhRcWork w;
    w.items = w.inline_items;
    w.len = 0;
    w.cap = sizeof(w.inline_items) / sizeof(w.inline_items[0]);
    int64_t line = -1; /* looked up for the first object kept DEAD */
    MdhObjHeader *dead = h;
    for (;;) {
        uint32_t kind = MDH_HDR_KIND(dead);
        /* DEAD before its children are released, so a miscounted path back to it aborts
         * instead of queueing it twice. */
        dead->flags |= MDH_OBJ_DEAD;
        __mdh_counter_add(&__mdh_rc_died[kind]);
        if (__mdh_rc_free_kinds & (1u << kind)) {
            __mdh_rc_children(kind, dead + 1, __mdh_rc_drop_child, &w);
            __mdh_rc_free_buffers(kind, dead + 1);
            __mdh_obj_block_free(dead);
        } else {
            if (line < 0) line = __mdh_rc_line();
            dead->refcount = line; /* the death line, for the abort message */
            __mdh_rc_children(kind, dead + 1, __mdh_rc_drop_child, &w);
        }
        if (w.len == 0) break;
        dead = w.items[--w.len];
    }
    if (w.items != w.inline_items) free(w.items);
}

void __mdh_rc_release_obj(int64_t data) {
    MdhObjHeader *h = __mdh_rc_counted("release", data);
    if (h && __mdh_rc_dec(h) && !__mdh_rc_kill_leaf(h)) __mdh_rc_kill(h);
}

void __mdh_retain(MdhValue v) {
    if (MDH_RC_HEAP_TAG(v.tag)) __mdh_rc_retain_obj(v.data);
}

void __mdh_release(MdhValue v) {
    if (MDH_RC_HEAP_TAG(v.tag)) __mdh_rc_release_obj(v.data);
}

/* ========== Shared slots (DEC round 3, R3: striped lock) ==========
 *
 * A global or a captured cell can be reassigned by one thread while another reads it.
 * A reader's load and retain must not straddle a writer's swap and release, or the
 * reader retains a freed object. So once threads exist, both go through a small table
 * of spinlocks keyed by the slot's address: a read is lock, load, retain, unlock; a
 * write is lock, swap, unlock, then release the old value outside the lock (release
 * can free a whole tree of objects). The critical sections are a few instructions and
 * never call user code, so a spinlock suits them; it yields after a short spin in case
 * the holder was preempted. Container elements are not covered: shared mutable
 * containers need a mutex (docs "Sharing between threads"). */

#if defined(_WIN32)
#include <windows.h>
#define MDH_SLOT_YIELD() SwitchToThread()
#else
#include <sched.h>
#define MDH_SLOT_YIELD() sched_yield()
#endif

#define MDH_SLOT_STRIPES 64

/* One lock per cache line, so threads using different stripes do not share a line. */
typedef struct {
    int32_t locked;
    char pad[60];
} MdhSlotLock;

static MdhSlotLock __mdh_slot_locks[MDH_SLOT_STRIPES];

static inline int32_t *__mdh_slot_lock_for(const void *slot) {
    uintptr_t a = (uintptr_t)slot >> 4; /* MdhValue slots are 16 bytes */
    a ^= a >> 6;
    a ^= a >> 12;
    return &__mdh_slot_locks[a & (MDH_SLOT_STRIPES - 1)].locked;
}

static inline void __mdh_slot_lock(int32_t *lock) {
    unsigned spins = 0;
    while (__atomic_exchange_n(lock, 1, __ATOMIC_ACQUIRE)) {
        while (__atomic_load_n(lock, __ATOMIC_RELAXED)) {
            if (++spins >= 64) {
                spins = 0;
                MDH_SLOT_YIELD();
            }
        }
    }
}

static inline void __mdh_slot_unlock(int32_t *lock) {
    __atomic_store_n(lock, 0, __ATOMIC_RELEASE);
}

/* RC-off shared values need the same indivisible pair and publication ordering,
 * without retaining/releasing headerless allocations. Always lock: these helpers
 * are only emitted for programs that can spawn, and avoid racing plain accesses
 * against locked ones. Unlock publishes the string/container's initialized bytes
 * as well as both words of the slot; the reader's acquire observes all of them. */
MdhValue __mdh_shared_load_uncounted(MdhValue *slot) {
    int32_t *lock = __mdh_slot_lock_for(slot);
    __mdh_slot_lock(lock);
    MdhValue value = *slot;
    __mdh_slot_unlock(lock);
    return value;
}

void __mdh_shared_store_uncounted(MdhValue *slot, MdhValue value) {
    int32_t *lock = __mdh_slot_lock_for(slot);
    __mdh_slot_lock(lock);
    *slot = value;
    __mdh_slot_unlock(lock);
}

MdhValue __mdh_shared_load(MdhValue *slot) {
    if (!__mdh_line_threaded) {
        MdhValue v = *slot;
        __mdh_retain(v);
        return v;
    }
    int32_t *lock = __mdh_slot_lock_for(slot);
    __mdh_slot_lock(lock);
    MdhValue v = *slot;
    __mdh_retain(v);
    __mdh_slot_unlock(lock);
    return v;
}

void __mdh_shared_store(MdhValue *slot, MdhValue value) {
    MdhValue old;
    if (!__mdh_line_threaded) {
        old = *slot;
        *slot = value;
    } else {
        int32_t *lock = __mdh_slot_lock_for(slot);
        __mdh_slot_lock(lock);
        old = *slot;
        *slot = value;
        __mdh_slot_unlock(lock);
    }
    __mdh_release(old);
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
