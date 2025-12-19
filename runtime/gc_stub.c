/**
 * gc_stub.c - Minimal GC stub for standalone executables
 *
 * These functions provide trivial wrappers around standard memory
 * allocation functions, allowing mdhavers programs to compile
 * without linking to a full garbage collector.
 */

#include <stdlib.h>
#include <string.h>

void GC_init(void) {
    // No-op - nothing to initialize
}

typedef struct GC_stack_base {
    void *mem_base;
} GC_stack_base;

int GC_register_my_thread(const GC_stack_base *sb) {
    (void)sb;
    return 0;
}

int GC_unregister_my_thread(void) {
    return 0;
}

int GC_get_stack_base(GC_stack_base *sb) {
    if (sb) {
        sb->mem_base = NULL;
    }
    return 0;
}

void GC_allow_register_threads(void) {
    // No-op for stub
}

void* GC_malloc(size_t size) {
    return malloc(size);
}

void* GC_realloc(void* ptr, size_t size) {
    return realloc(ptr, size);
}

char* GC_strdup(const char* s) {
    return strdup(s);
}
