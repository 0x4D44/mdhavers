/*
 * Browser build (Emscripten) only: the bridge between the two wasm32 C ABIs.
 *
 * Compiled code passes and returns MdhValue ({u8 tag, i64 data}) as plain values. On
 * native targets the C ABI does the same, but wasm32's standard C ABI passes such a
 * struct by pointer. So the browser build compiles the C runtime with the multivalue ABI
 * (`-mmultivalue -Xclang -target-abi -Xclang experimental-mv`), which matches what the
 * LLVM backend emits. Rust has no such ABI: on Emscripten the Rust runtime keeps the
 * standard one and names each entry that carries an MdhValue `<name>__rsabi` (see
 * `export_name`/`link_name` in runtime/mdh_runtime_rs). This file, built with the
 * multivalue ABI like the rest of the C runtime, joins the two in both directions: Rust
 * calling C runtime entries, and C (compiled code and the C runtime) calling Rust ones,
 * including the scalar value constructors that runtime/mdh_runtime_core defines. The
 * standard ABI's form of `T f(MdhValue a)` is `void f(T *out, MdhValue *a)` when T is a
 * struct, and `T f(MdhValue *a)` when T is a scalar; that form reads the same in both.
 */

#ifdef __EMSCRIPTEN__

#include <stdbool.h>
#include <stdint.h>

#include "../mdh_runtime.h"

/* ---- 64-bit multiplies that check for overflow ----
 *
 * Compiled code's llvm.{s,u}mul.with.overflow.i64 are pointed here (`redirect_mul_overflow`
 * in src/llvm/wasm.rs), and the C runtime calls these itself: done in place, the check
 * would call __multi3 with the wrong ABI (see wasm_mul.c, which does it instead). */

typedef struct {
    int64_t value;
    bool overflow;
} MdhMulOverflow;

bool __mdh_wasm_smul_overflow_p(int64_t a, int64_t b, int64_t *out);
bool __mdh_wasm_umul_overflow_p(uint64_t a, uint64_t b, uint64_t *out);

MdhMulOverflow __mdh_wasm_smul_overflow(int64_t a, int64_t b) {
    MdhMulOverflow r;
    r.overflow = __mdh_wasm_smul_overflow_p(a, b, &r.value);
    return r;
}

MdhMulOverflow __mdh_wasm_umul_overflow(uint64_t a, uint64_t b) {
    MdhMulOverflow r;
    uint64_t value;
    r.overflow = __mdh_wasm_umul_overflow_p(a, b, &value);
    r.value = (int64_t)value;
    return r;
}

/* ---- Rust calls the C runtime ---- */

MdhValue __mdh_stack_overflow_message(void);
MdhValue __mdh_out_of_memory_message(void);
int32_t __mdh_list_push_owned_try(MdhValue list, MdhValue value);
int32_t __mdh_dict_set_owned_try(MdhValue dict, MdhValue key, MdhValue value);
MdhValue __mdh_dict_get_default(MdhValue dict, MdhValue key, MdhValue default_val);
int32_t __mdh_to_string_checked(MdhValue value, MdhValue *out);
int32_t __mdh_kv_densify(MdhValue v);

void __mdh_stack_overflow_message__rsabi(MdhValue *out) { *out = __mdh_stack_overflow_message(); }
void __mdh_out_of_memory_message__rsabi(MdhValue *out) { *out = __mdh_out_of_memory_message(); }

int32_t __mdh_list_push_owned_try__rsabi(const MdhValue *list, const MdhValue *value) {
    return __mdh_list_push_owned_try(*list, *value);
}

int32_t __mdh_dict_set_owned_try__rsabi(const MdhValue *dict, const MdhValue *key,
                                         const MdhValue *value) {
    return __mdh_dict_set_owned_try(*dict, *key, *value);
}

void __mdh_dict_get_default__rsabi(MdhValue *out, const MdhValue *dict, const MdhValue *key,
                                   const MdhValue *default_val) {
    *out = __mdh_dict_get_default(*dict, *key, *default_val);
}

int32_t __mdh_to_string_checked__rsabi(const MdhValue *value, MdhValue *out) {
    return __mdh_to_string_checked(*value, out);
}

int32_t __mdh_kv_densify__rsabi(const MdhValue *value) { return __mdh_kv_densify(*value); }
void __mdh_release__rsabi(const MdhValue *value) { __mdh_release(*value); }
void __mdh_hurl__rsabi(const MdhValue *value) { __mdh_hurl(*value); }

/* ---- Compiled code and the C runtime call the Rust core's scalar values ----
 *
 * runtime/mdh_runtime_core defines these and, on Emscripten, exports them as
 * `<name>__rsabi` with the standard ABI. Rust callers use them as Rust items directly. */

void __mdh_make_nil__rsabi(MdhValue *out);
void __mdh_make_bool__rsabi(MdhValue *out, bool value);
void __mdh_make_int__rsabi(MdhValue *out, int64_t value);
void __mdh_make_float__rsabi(MdhValue *out, double value);
uint8_t __mdh_get_tag__rsabi(const MdhValue *value);

MdhValue __mdh_make_nil(void) {
    MdhValue r;
    __mdh_make_nil__rsabi(&r);
    return r;
}

MdhValue __mdh_make_bool(bool value) {
    MdhValue r;
    __mdh_make_bool__rsabi(&r, value);
    return r;
}

MdhValue __mdh_make_int(int64_t value) {
    MdhValue r;
    __mdh_make_int__rsabi(&r, value);
    return r;
}

MdhValue __mdh_make_float(double value) {
    MdhValue r;
    __mdh_make_float__rsabi(&r, value);
    return r;
}

uint8_t __mdh_get_tag(MdhValue value) { return __mdh_get_tag__rsabi(&value); }

/* ---- The C runtime calls Rust (JSON, regex, and the network stubs) ---- */

#define RS_ENTRY1(name)                                                                  \
    void name##__rsabi(MdhRsResult *out, const MdhValue *a);                             \
    MdhRsResult name(MdhValue a) {                                                       \
        MdhRsResult r;                                                                   \
        name##__rsabi(&r, &a);                                                           \
        return r;                                                                        \
    }
#define RS_ENTRY2(name)                                                                  \
    void name##__rsabi(MdhRsResult *out, const MdhValue *a, const MdhValue *b);          \
    MdhRsResult name(MdhValue a, MdhValue b) {                                           \
        MdhRsResult r;                                                                   \
        name##__rsabi(&r, &a, &b);                                                       \
        return r;                                                                        \
    }
#define RS_ENTRY3(name)                                                                  \
    void name##__rsabi(MdhRsResult *out, const MdhValue *a, const MdhValue *b,           \
                       const MdhValue *c);                                               \
    MdhRsResult name(MdhValue a, MdhValue b, MdhValue c) {                               \
        MdhRsResult r;                                                                   \
        name##__rsabi(&r, &a, &b, &c);                                                   \
        return r;                                                                        \
    }

RS_ENTRY1(__mdh_rs_json_parse)
RS_ENTRY1(__mdh_rs_json_stringify)
RS_ENTRY1(__mdh_rs_json_pretty)
RS_ENTRY2(__mdh_rs_regex_test)
RS_ENTRY2(__mdh_rs_regex_match)
RS_ENTRY2(__mdh_rs_regex_match_all)
RS_ENTRY3(__mdh_rs_regex_replace)
RS_ENTRY3(__mdh_rs_regex_replace_first)
RS_ENTRY2(__mdh_rs_regex_split)
RS_ENTRY2(__mdh_rs_dns_srv)
RS_ENTRY1(__mdh_rs_dns_naptr)
RS_ENTRY1(__mdh_rs_tls_client_new)
RS_ENTRY2(__mdh_rs_tls_connect)
RS_ENTRY2(__mdh_rs_tls_send)
RS_ENTRY2(__mdh_rs_tls_recv)
RS_ENTRY1(__mdh_rs_tls_close)
RS_ENTRY1(__mdh_rs_srtp_create)
RS_ENTRY2(__mdh_rs_srtp_protect)
RS_ENTRY2(__mdh_rs_srtp_unprotect)
RS_ENTRY1(__mdh_rs_dtls_server_new)
RS_ENTRY2(__mdh_rs_dtls_handshake)

/* ---- Compiled code calls Rust (the audio builtins; stubs in the browser for now) ---- */

#define AUDIO_ENTRY0(name)                                                               \
    void name##__rsabi(MdhValue *out);                                                   \
    MdhValue name(void) {                                                                \
        MdhValue r;                                                                      \
        name##__rsabi(&r);                                                               \
        return r;                                                                        \
    }
#define AUDIO_ENTRY1(name)                                                               \
    void name##__rsabi(MdhValue *out, const MdhValue *a);                                \
    MdhValue name(MdhValue a) {                                                          \
        MdhValue r;                                                                      \
        name##__rsabi(&r, &a);                                                           \
        return r;                                                                        \
    }
#define AUDIO_ENTRY2(name)                                                               \
    void name##__rsabi(MdhValue *out, const MdhValue *a, const MdhValue *b);             \
    MdhValue name(MdhValue a, MdhValue b) {                                              \
        MdhValue r;                                                                      \
        name##__rsabi(&r, &a, &b);                                                       \
        return r;                                                                        \
    }

AUDIO_ENTRY0(__mdh_soond_stairt)
AUDIO_ENTRY0(__mdh_soond_steek)
AUDIO_ENTRY1(__mdh_soond_wheesht)
AUDIO_ENTRY1(__mdh_soond_luid)
AUDIO_ENTRY0(__mdh_soond_hou_luid)
AUDIO_ENTRY0(__mdh_soond_haud_gang)
AUDIO_ENTRY1(__mdh_soond_lade)
AUDIO_ENTRY1(__mdh_soond_spiel)
AUDIO_ENTRY1(__mdh_soond_haud)
AUDIO_ENTRY1(__mdh_soond_gae_on)
AUDIO_ENTRY1(__mdh_soond_stap)
AUDIO_ENTRY1(__mdh_soond_unlade)
AUDIO_ENTRY1(__mdh_soond_is_spielin)
AUDIO_ENTRY2(__mdh_soond_pit_luid)
AUDIO_ENTRY2(__mdh_soond_pit_pan)
AUDIO_ENTRY2(__mdh_soond_pit_tune)
AUDIO_ENTRY2(__mdh_soond_pit_rin_roond)
AUDIO_ENTRY1(__mdh_soond_ready)
AUDIO_ENTRY1(__mdh_muisic_lade)
AUDIO_ENTRY1(__mdh_muisic_spiel)
AUDIO_ENTRY1(__mdh_muisic_haud)
AUDIO_ENTRY1(__mdh_muisic_gae_on)
AUDIO_ENTRY1(__mdh_muisic_stap)
AUDIO_ENTRY1(__mdh_muisic_unlade)
AUDIO_ENTRY1(__mdh_muisic_is_spielin)
AUDIO_ENTRY2(__mdh_muisic_loup)
AUDIO_ENTRY1(__mdh_muisic_hou_lang)
AUDIO_ENTRY1(__mdh_muisic_whaur)
AUDIO_ENTRY2(__mdh_muisic_pit_luid)
AUDIO_ENTRY2(__mdh_muisic_pit_pan)
AUDIO_ENTRY2(__mdh_muisic_pit_tune)
AUDIO_ENTRY2(__mdh_muisic_pit_rin_roond)
AUDIO_ENTRY2(__mdh_midi_lade)
AUDIO_ENTRY1(__mdh_midi_spiel)
AUDIO_ENTRY1(__mdh_midi_haud)
AUDIO_ENTRY1(__mdh_midi_gae_on)
AUDIO_ENTRY1(__mdh_midi_stap)
AUDIO_ENTRY1(__mdh_midi_unlade)
AUDIO_ENTRY1(__mdh_midi_is_spielin)
AUDIO_ENTRY2(__mdh_midi_loup)
AUDIO_ENTRY1(__mdh_midi_hou_lang)
AUDIO_ENTRY1(__mdh_midi_whaur)
AUDIO_ENTRY2(__mdh_midi_pit_luid)
AUDIO_ENTRY2(__mdh_midi_pit_pan)
AUDIO_ENTRY2(__mdh_midi_pit_rin_roond)

#endif /* __EMSCRIPTEN__ */
