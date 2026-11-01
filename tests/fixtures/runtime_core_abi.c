/* C caller for the production Rust core. No replacement runtime symbols or stubs.
 * Keep this driver usable with a target C compiler for cross-architecture checks. */
#include "mdh_runtime.h"
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _MSC_VER
#define MDH_ALIGNOF(type) __alignof(type)
#else
#define MDH_ALIGNOF(type) _Alignof(type)
#endif

static void check_value(MdhValue actual, uint8_t tag, uint64_t bits) {
    if (actual.tag != tag || (uint64_t)actual.data != bits) {
        fprintf(stderr, "value mismatch: tag=%u bits=%016" PRIx64
                ", expected tag=%u bits=%016" PRIx64 "\n",
                (unsigned)actual.tag, (uint64_t)actual.data, (unsigned)tag, bits);
        exit(1);
    }
    /* The returned aggregate is passed back into Rust using the C calling ABI. */
    if (__mdh_get_tag(actual) != tag) {
        fprintf(stderr, "aggregate argument/tag mismatch\n");
        exit(1);
    }
}

static void check_float_bits(uint64_t bits) {
    double value;
    /* memcpy is independent of aliasing and byte order. Do not perform floating
     * arithmetic: it may quiet signalling NaNs or discard their payloads. */
    memcpy(&value, &bits, sizeof(value));
    check_value(__mdh_make_float(value), MDH_TAG_FLOAT, bits);
}

int main(void) {
    if (sizeof(double) != sizeof(uint64_t)) {
        fprintf(stderr, "the value ABI requires a 64-bit double\n");
        return 1;
    }

    /* Compare these observations with the Rust types, not with a duplicated C
     * definition or with raw struct bytes (padding is unspecified). */
    printf("value %zu %zu %zu %zu\n", sizeof(MdhValue),
           (size_t)MDH_ALIGNOF(MdhValue), offsetof(MdhValue, tag), offsetof(MdhValue, data));
    printf("result %zu %zu %zu %zu %zu\n", sizeof(MdhRsResult),
           (size_t)MDH_ALIGNOF(MdhRsResult), offsetof(MdhRsResult, ok),
           offsetof(MdhRsResult, value), offsetof(MdhRsResult, error));
    printf("tags %u %u %u %u %u %u %u %u %u %u %u %u %u %u %u\n",
           (unsigned)MDH_TAG_NIL, (unsigned)MDH_TAG_BOOL, (unsigned)MDH_TAG_INT,
           (unsigned)MDH_TAG_FLOAT, (unsigned)MDH_TAG_STRING, (unsigned)MDH_TAG_LIST,
           (unsigned)MDH_TAG_DICT, (unsigned)MDH_TAG_FUNCTION, (unsigned)MDH_TAG_CLASS,
           (unsigned)MDH_TAG_INSTANCE, (unsigned)MDH_TAG_RANGE, (unsigned)MDH_TAG_SET,
           (unsigned)MDH_TAG_CLOSURE, (unsigned)MDH_TAG_BYTES, (unsigned)MDH_TAG_NATIVE);

    check_value(__mdh_make_nil(), MDH_TAG_NIL, 0);
    check_value(__mdh_make_bool(false), MDH_TAG_BOOL, 0);
    check_value(__mdh_make_bool(true), MDH_TAG_BOOL, 1);

    const int64_t integers[] = {
        INT64_MIN, INT64_MIN + 1, -INT64_C(9007199254740993), -1, 0, 1,
        INT64_C(9007199254740993), INT64_MAX - 1, INT64_MAX
    };
    for (size_t i = 0; i < sizeof(integers) / sizeof(integers[0]); i++) {
        check_value(__mdh_make_int(integers[i]), MDH_TAG_INT, (uint64_t)integers[i]);
    }

    const uint64_t floats[] = {
        UINT64_C(0x0000000000000000), UINT64_C(0x8000000000000000), /* +/- zero */
        UINT64_C(0x0000000000000001), UINT64_C(0x8000000000000001), /* tiny subnormals */
        UINT64_C(0x000fffffffffffff), UINT64_C(0x0010000000000000), /* normal boundary */
        UINT64_C(0x3ff0000000000000), UINT64_C(0xc004000000000000), /* 1, -2.5 */
        UINT64_C(0x7fefffffffffffff), UINT64_C(0xffefffffffffffff), /* finite extremes */
        UINT64_C(0x7ff0000000000000), UINT64_C(0xfff0000000000000), /* infinities */
        UINT64_C(0x7ff8000000000000), UINT64_C(0xfff8000000000000), /* quiet NaNs */
        UINT64_C(0x7ff8123456789abc), UINT64_C(0xfff8fedcba987654), /* NaN payloads */
        UINT64_C(0x7ff0000000000001), UINT64_C(0xfff0000000000001)  /* signalling NaNs */
    };
    for (size_t i = 0; i < sizeof(floats) / sizeof(floats[0]); i++) {
        check_float_bits(floats[i]);
    }

    /* Deterministic full-width input coverage catches accidental numeric conversion,
     * truncation and byte-swapping in either the integer or floating-point path. */
    uint64_t bits = UINT64_C(0x6d64686176657273);
    for (int i = 0; i < 4096; i++) {
        bits ^= bits << 13;
        bits ^= bits >> 7;
        bits ^= bits << 17;
        int64_t integer;
        memcpy(&integer, &bits, sizeof(integer));
        check_value(__mdh_make_int(integer), MDH_TAG_INT, bits);
        check_float_bits(bits);
    }

    /* Inspecting any tag, including a heap tag, must not dereference, retain or
     * release the payload. These deliberately invalid addresses are never objects. */
    for (unsigned tag = 0; tag <= UINT8_MAX; tag++) {
        MdhValue value = { (uint8_t)tag, INT64_MIN + (int64_t)tag };
        if (__mdh_get_tag(value) != tag) {
            fprintf(stderr, "tag %u did not round-trip\n", tag);
            return 1;
        }
    }

    puts("operations ok");
    return 0;
}
