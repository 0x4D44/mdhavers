/*
 * Browser build (Emscripten) only, and built with wasm32's standard C ABI, not the
 * multivalue one the rest of the C runtime uses (see wasm_abi.c).
 *
 * clang and LLVM check a 64-bit multiply for overflow with a 128-bit one: a call to
 * __multi3, which (in compiler-rt, and in Rust's copy) returns through memory. Code built
 * with the multivalue ABI expects two results back instead, so it must not make that
 * call itself: it comes here, through scalars and a pointer, which both ABIs pass alike.
 */

#ifdef __EMSCRIPTEN__

#include <stdbool.h>
#include <stdint.h>

bool __mdh_wasm_smul_overflow_p(int64_t a, int64_t b, int64_t *out) {
    if (a == (int32_t)a && b == (int32_t)b) {
        *out = a * b; /* cannot overflow */
        return false;
    }
    return __builtin_mul_overflow(a, b, out);
}

bool __mdh_wasm_umul_overflow_p(uint64_t a, uint64_t b, uint64_t *out) {
    if (a >> 32 == 0 && b >> 32 == 0) {
        *out = a * b; /* cannot overflow */
        return false;
    }
    return __builtin_mul_overflow(a, b, out);
}

#endif /* __EMSCRIPTEN__ */
