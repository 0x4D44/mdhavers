/**
 * gc_stub.c - the single allocator for compiled mdhavers programs
 *
 * Compiled code and the runtimes allocate only through these functions (see
 * "Allocation" in mdh_runtime.h). Memory management is reference counting only
 * (wrk_docs DEC, round 2, F6 Q7): there is no garbage collector behind this file.
 * Nothing is freed yet; reference counting adds its object header and the frees here.
 *
 * Build with -DMDH_ALLOC_STATS to count allocations per kind (__mdh_alloc_count).
 */

#include <stdlib.h>
#include <string.h>

#include "mdh_runtime.h"

_Static_assert(sizeof(MdhObjHeader) == 16, "the object header keeps payloads 16-byte aligned");

#ifdef MDH_ALLOC_STATS
static int64_t __mdh_alloc_counts[MDH_KIND_COUNT];

static void __mdh_alloc_note(uint32_t kind) {
    if (kind >= MDH_KIND_COUNT) kind = MDH_KIND_BUFFER;
    __atomic_fetch_add(&__mdh_alloc_counts[kind], 1, __ATOMIC_RELAXED);
}

int64_t __mdh_alloc_count(uint32_t kind) {
    if (kind >= MDH_KIND_COUNT) return -1;
    return __atomic_load_n(&__mdh_alloc_counts[kind], __ATOMIC_RELAXED);
}
#else
#define __mdh_alloc_note(kind) ((void)(kind))

int64_t __mdh_alloc_count(uint32_t kind) {
    (void)kind;
    return -1;
}
#endif

void *__mdh_obj_alloc(uint32_t kind, size_t size) {
    __mdh_alloc_note(kind);
    return malloc(size);
}

void *__mdh_obj_realloc(void *obj, size_t size) {
    if (!obj) __mdh_alloc_note(MDH_KIND_STRING);
    return realloc(obj, size);
}

char *__mdh_obj_strdup(const char *s) {
    size_t n = strlen(s) + 1;
    char *out = (char *)__mdh_obj_alloc(MDH_KIND_STRING, n);
    if (out) memcpy(out, s, n);
    return out;
}

void *__mdh_buf_alloc(size_t size) {
    __mdh_alloc_note(MDH_KIND_BUFFER);
    return malloc(size);
}

void *__mdh_buf_realloc(void *buf, size_t size) {
    if (!buf) __mdh_alloc_note(MDH_KIND_BUFFER);
    return realloc(buf, size);
}

char *__mdh_buf_strdup(const char *s) {
    size_t n = strlen(s) + 1;
    char *out = (char *)__mdh_buf_alloc(n);
    if (out) memcpy(out, s, n);
    return out;
}
