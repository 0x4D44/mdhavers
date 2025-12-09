/*
 * mdhavers Runtime Library - Header
 *
 * Provides runtime support for compiled mdhavers programs.
 * All mdhavers values are represented as MdhValue structs.
 */

#ifndef MDH_RUNTIME_H
#define MDH_RUNTIME_H

#include <stdint.h>
#include <stdbool.h>

/* Value type tags - must match src/llvm/types.rs */
typedef enum {
    MDH_TAG_NIL = 0,
    MDH_TAG_BOOL = 1,
    MDH_TAG_INT = 2,
    MDH_TAG_FLOAT = 3,
    MDH_TAG_STRING = 4,
    MDH_TAG_LIST = 5,
    MDH_TAG_DICT = 6,
    MDH_TAG_FUNCTION = 7,
    MDH_TAG_CLASS = 8,
    MDH_TAG_INSTANCE = 9,
    MDH_TAG_RANGE = 10,
} MdhTag;

/*
 * The main value type for mdhavers.
 * Uses a tagged union where:
 * - tag identifies the type
 * - data holds either an immediate value (int, float, bool)
 *   or a pointer (string, list, etc.)
 */
typedef struct {
    uint8_t tag;
    int64_t data;  /* Can be cast to pointer or numeric types */
} MdhValue;

/* Forward declarations for complex types */
typedef struct MdhList MdhList;
typedef struct MdhDict MdhDict;
typedef struct MdhString MdhString;

/* List structure */
struct MdhList {
    MdhValue *items;
    int64_t length;
    int64_t capacity;
};

/* String structure (GC-managed) */
struct MdhString {
    char *data;
    int64_t length;
};

/* ========== Value Creation ========== */

MdhValue __mdh_make_nil(void);
MdhValue __mdh_make_bool(bool value);
MdhValue __mdh_make_int(int64_t value);
MdhValue __mdh_make_float(double value);
MdhValue __mdh_make_string(const char *value);
MdhValue __mdh_make_list(int32_t capacity);

/* ========== Arithmetic Operations ========== */

MdhValue __mdh_add(MdhValue a, MdhValue b);
MdhValue __mdh_sub(MdhValue a, MdhValue b);
MdhValue __mdh_mul(MdhValue a, MdhValue b);
MdhValue __mdh_div(MdhValue a, MdhValue b);
MdhValue __mdh_mod(MdhValue a, MdhValue b);
MdhValue __mdh_neg(MdhValue a);

/* ========== Comparison Operations ========== */

bool __mdh_eq(MdhValue a, MdhValue b);
bool __mdh_ne(MdhValue a, MdhValue b);
bool __mdh_lt(MdhValue a, MdhValue b);
bool __mdh_le(MdhValue a, MdhValue b);
bool __mdh_gt(MdhValue a, MdhValue b);
bool __mdh_ge(MdhValue a, MdhValue b);

/* ========== Logical Operations ========== */

MdhValue __mdh_not(MdhValue a);
bool __mdh_truthy(MdhValue a);

/* ========== Type Operations ========== */

uint8_t __mdh_get_tag(MdhValue a);
void __mdh_type_error(const char *op, uint8_t got1, uint8_t got2);
MdhValue __mdh_type_of(MdhValue a);

/* ========== I/O ========== */

void __mdh_blether(MdhValue a);
MdhValue __mdh_speir(MdhValue prompt);
MdhValue __mdh_get_key(void);

/* ========== List Operations ========== */

MdhValue __mdh_list_get(MdhValue list, int64_t index);
void __mdh_list_set(MdhValue list, int64_t index, MdhValue value);
void __mdh_list_push(MdhValue list, MdhValue value);
MdhValue __mdh_list_pop(MdhValue list);
int64_t __mdh_list_len(MdhValue list);
int64_t __mdh_len(MdhValue a);

/* ========== String Operations ========== */

MdhValue __mdh_str_concat(MdhValue a, MdhValue b);
int64_t __mdh_str_len(MdhValue s);
MdhValue __mdh_to_string(MdhValue a);
MdhValue __mdh_to_int(MdhValue a);
MdhValue __mdh_to_float(MdhValue a);

/* ========== Math ========== */

MdhValue __mdh_abs(MdhValue a);
MdhValue __mdh_random(int64_t min, int64_t max);
MdhValue __mdh_floor(MdhValue a);
MdhValue __mdh_ceil(MdhValue a);
MdhValue __mdh_round(MdhValue a);

/* ========== Helpers ========== */

/* Get string pointer from MdhValue (assumes tag is STRING) */
static inline const char *__mdh_get_string(MdhValue v) {
    /* data field contains char* directly (matches LLVM convention) */
    return (const char *)(intptr_t)v.data;
}

/* Get list pointer from MdhValue (assumes tag is LIST) */
static inline MdhList *__mdh_get_list(MdhValue v) {
    return (MdhList *)(intptr_t)v.data;
}

/* Get integer value from MdhValue (assumes tag is INT) */
static inline int64_t __mdh_get_int(MdhValue v) {
    return v.data;
}

/* Get float value from MdhValue (assumes tag is FLOAT) */
static inline double __mdh_get_float(MdhValue v) {
    union { int64_t i; double f; } u;
    u.i = v.data;
    return u.f;
}

/* Get bool value from MdhValue (assumes tag is BOOL) */
static inline bool __mdh_get_bool(MdhValue v) {
    return v.data != 0;
}

#endif /* MDH_RUNTIME_H */
