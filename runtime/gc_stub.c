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

void* GC_malloc(size_t size) {
    return malloc(size);
}

void* GC_realloc(void* ptr, size_t size) {
    return realloc(ptr, size);
}

char* GC_strdup(const char* s) {
    return strdup(s);
}
