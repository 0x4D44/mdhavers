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
void __mdh_key_not_found(MdhValue key);

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
MdhValue __mdh_range(int64_t start, int64_t end, int64_t step);

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

/* ========== Dict/Creel Operations ========== */

MdhValue __mdh_empty_dict(void);
MdhValue __mdh_empty_creel(void);
MdhValue __mdh_make_creel(MdhValue list);
MdhValue __mdh_dict_contains(MdhValue dict, MdhValue key);
MdhValue __mdh_dict_keys(MdhValue dict);
MdhValue __mdh_dict_values(MdhValue dict);
MdhValue __mdh_dict_set(MdhValue dict, MdhValue key, MdhValue value);
MdhValue __mdh_dict_get(MdhValue dict, MdhValue key);
MdhValue __mdh_dict_get_default(MdhValue dict, MdhValue key, MdhValue default_val);
MdhValue __mdh_dict_merge(MdhValue a, MdhValue b);
MdhValue __mdh_dict_remove(MdhValue dict, MdhValue key);
MdhValue __mdh_dict_invert(MdhValue dict);
MdhValue __mdh_fae_pairs(MdhValue pairs);
MdhValue __mdh_toss_in(MdhValue dict, MdhValue item);
MdhValue __mdh_heave_oot(MdhValue dict, MdhValue item);
MdhValue __mdh_creel_tae_list(MdhValue dict);
MdhValue __mdh_creels_thegither(MdhValue a, MdhValue b);
MdhValue __mdh_creels_baith(MdhValue a, MdhValue b);
MdhValue __mdh_creels_differ(MdhValue a, MdhValue b);
MdhValue __mdh_is_subset(MdhValue a, MdhValue b);
MdhValue __mdh_is_superset(MdhValue a, MdhValue b);
MdhValue __mdh_is_disjoint(MdhValue a, MdhValue b);

/* ========== File I/O ========== */

MdhValue __mdh_file_exists(MdhValue path);
MdhValue __mdh_file_size(MdhValue path);
MdhValue __mdh_file_delete(MdhValue path);
MdhValue __mdh_list_dir(MdhValue path);
MdhValue __mdh_make_dir(MdhValue path);
MdhValue __mdh_is_dir(MdhValue path);
MdhValue __mdh_slurp(MdhValue path);
MdhValue __mdh_scrieve(MdhValue path, MdhValue content);
MdhValue __mdh_scrieve_append(MdhValue path, MdhValue content);
MdhValue __mdh_lines(MdhValue path);
MdhValue __mdh_words(MdhValue str);

/* ========== Logging/Debug ========== */

MdhValue __mdh_get_log_level(void);
MdhValue __mdh_set_log_level(MdhValue level);

/* ========== Scots Builtins ========== */

MdhValue __mdh_slainte(void);
MdhValue __mdh_och(MdhValue msg);
MdhValue __mdh_help_ma_boab(MdhValue msg);
MdhValue __mdh_haver(void);
MdhValue __mdh_braw_time(void);
MdhValue __mdh_wee(MdhValue a, MdhValue b);
MdhValue __mdh_tak(MdhValue list, MdhValue n);
MdhValue __mdh_pair_up(MdhValue list1, MdhValue list2);
MdhValue __mdh_tae_binary(MdhValue n);
MdhValue __mdh_fae_binary(MdhValue str);
MdhValue __mdh_fae_hex(MdhValue str);
MdhValue __mdh_ltrim(MdhValue str);
MdhValue __mdh_rtrim(MdhValue str);
MdhValue __mdh_reverse_str(MdhValue str);
MdhValue __mdh_title_case(MdhValue str);
MdhValue __mdh_tae_hex(MdhValue num);
MdhValue __mdh_tae_octal(MdhValue num);
MdhValue __mdh_center(MdhValue str, MdhValue width);
MdhValue __mdh_repeat_say(MdhValue str, MdhValue count);
MdhValue __mdh_leftpad(MdhValue str, MdhValue width, MdhValue pad);
MdhValue __mdh_rightpad(MdhValue str, MdhValue width, MdhValue pad);
MdhValue __mdh_list_index(MdhValue list, MdhValue val);
MdhValue __mdh_count_val(MdhValue list, MdhValue val);
MdhValue __mdh_list_copy(MdhValue list);
MdhValue __mdh_list_clear(MdhValue list);
MdhValue __mdh_last_index_of(MdhValue str, MdhValue substr);
MdhValue __mdh_replace_first(MdhValue str, MdhValue old_sub, MdhValue new_sub);
MdhValue __mdh_unique(MdhValue list);
MdhValue __mdh_average(MdhValue list);
MdhValue __mdh_chynge(MdhValue str, MdhValue old_sub, MdhValue new_sub);

/* ========== Testing ========== */

MdhValue __mdh_assert(MdhValue condition, MdhValue msg);
MdhValue __mdh_skip(MdhValue reason);
MdhValue __mdh_stacktrace(void);

/* ========== Additional Scots Builtins ========== */

MdhValue __mdh_muckle(MdhValue a, MdhValue b);
MdhValue __mdh_median(MdhValue list);
MdhValue __mdh_is_space(MdhValue str);
MdhValue __mdh_is_digit(MdhValue str);
MdhValue __mdh_wheesht_aw(MdhValue str);
MdhValue __mdh_bonnie(MdhValue val);
MdhValue __mdh_shuffle(MdhValue list);
MdhValue __mdh_bit_and(MdhValue a, MdhValue b);
MdhValue __mdh_bit_or(MdhValue a, MdhValue b);
MdhValue __mdh_bit_xor(MdhValue a, MdhValue b);

/* ========== Environment/System ========== */

void __mdh_set_args(int32_t argc, char **argv);
MdhValue __mdh_args(void);
MdhValue __mdh_cwd(void);
MdhValue __mdh_chdir(MdhValue path);
MdhValue __mdh_env_get(MdhValue key);
MdhValue __mdh_env_set(MdhValue key, MdhValue value);
MdhValue __mdh_env_all(void);
MdhValue __mdh_path_join(MdhValue a, MdhValue b);
MdhValue __mdh_shell(MdhValue cmd);
MdhValue __mdh_shell_status(MdhValue cmd);

/* ========== Date/Time ========== */

MdhValue __mdh_date_now(void);
MdhValue __mdh_date_format(MdhValue timestamp_secs, MdhValue format);
MdhValue __mdh_date_parse(MdhValue date_str, MdhValue format);
MdhValue __mdh_date_add(MdhValue timestamp_secs, MdhValue amount, MdhValue unit);
MdhValue __mdh_date_diff(MdhValue ts1, MdhValue ts2, MdhValue unit);
MdhValue __mdh_braw_date(MdhValue ts_or_nil);

/* ========== Regex ========== */

MdhValue __mdh_regex_test(MdhValue text, MdhValue pattern);
MdhValue __mdh_regex_match(MdhValue text, MdhValue pattern);
MdhValue __mdh_regex_match_all(MdhValue text, MdhValue pattern);
MdhValue __mdh_regex_replace(MdhValue text, MdhValue pattern, MdhValue replacement);
MdhValue __mdh_regex_replace_first(MdhValue text, MdhValue pattern, MdhValue replacement);
MdhValue __mdh_regex_split(MdhValue text, MdhValue pattern);

/* ========== JSON ========== */

MdhValue __mdh_json_parse(MdhValue json_str);
MdhValue __mdh_json_stringify(MdhValue value);
MdhValue __mdh_json_pretty(MdhValue value);

/* ========== Misc Parity Helpers ========== */

MdhValue __mdh_is_a(MdhValue value, MdhValue type_name);
MdhValue __mdh_numpty_check(MdhValue value);
MdhValue __mdh_indices_o(MdhValue container, MdhValue needle);
MdhValue __mdh_grup(MdhValue list, MdhValue size);
MdhValue __mdh_chunks(MdhValue list, MdhValue size);
MdhValue __mdh_window(MdhValue str, MdhValue size);
MdhValue __mdh_interleave(MdhValue list_a, MdhValue list_b);
MdhValue __mdh_pair_adjacent(MdhValue list);
MdhValue __mdh_skelp(MdhValue str, MdhValue size);
MdhValue __mdh_strip_left(MdhValue str, MdhValue chars);
MdhValue __mdh_strip_right(MdhValue str, MdhValue chars);
MdhValue __mdh_swapcase(MdhValue str);
MdhValue __mdh_sporran_fill(MdhValue str, MdhValue width, MdhValue fill_char);
MdhValue __mdh_scottify(MdhValue str);
MdhValue __mdh_mutter(MdhValue str);
MdhValue __mdh_blooter(MdhValue str);
MdhValue __mdh_stooshie(MdhValue str);
MdhValue __mdh_dreich(MdhValue str);
MdhValue __mdh_geggie(MdhValue str);
MdhValue __mdh_jings(MdhValue msg);
MdhValue __mdh_crivvens(MdhValue msg);
MdhValue __mdh_braw(MdhValue val);
MdhValue __mdh_crabbit(MdhValue val);
MdhValue __mdh_gallus(MdhValue val);
MdhValue __mdh_drookit(MdhValue list);
MdhValue __mdh_clarty(MdhValue val);
MdhValue __mdh_glaikit(MdhValue val);
MdhValue __mdh_is_wee(MdhValue val);
MdhValue __mdh_is_muckle(MdhValue val);
MdhValue __mdh_is_blank(MdhValue str);
MdhValue __mdh_haverin(MdhValue val);
MdhValue __mdh_banter(MdhValue a, MdhValue b);
MdhValue __mdh_capitalize(MdhValue str);
MdhValue __mdh_scunner(MdhValue val);
MdhValue __mdh_scunner_check(MdhValue val, MdhValue expected_type);
MdhValue __mdh_clype(MdhValue val);
MdhValue __mdh_stoater(MdhValue list);
MdhValue __mdh_dicht(MdhValue list, MdhValue index);
MdhValue __mdh_redd_up(MdhValue list);
MdhValue __mdh_split_by(MdhValue list, MdhValue pred);
MdhValue __mdh_grup_runs(MdhValue list);
MdhValue __mdh_range_o(MdhValue list);
MdhValue __mdh_tattie_scone(MdhValue str, MdhValue n);
MdhValue __mdh_haggis_hunt(MdhValue haystack, MdhValue needle);
MdhValue __mdh_blether_format(MdhValue template, MdhValue dict);
MdhValue __mdh_bampot_mode(MdhValue list);

/* ========== Exceptions (Try/Catch/Hurl) ========== */

int64_t __mdh_jmp_buf_size(void);
void __mdh_try_push(void *env);
void __mdh_try_pop(void);
void __mdh_hurl(MdhValue msg);
MdhValue __mdh_get_last_error(void);

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

/* Type checking functions */
MdhValue __mdh_is_nil(MdhValue val);
MdhValue __mdh_is_bool(MdhValue val);
MdhValue __mdh_is_int(MdhValue val);
MdhValue __mdh_is_float(MdhValue val);
MdhValue __mdh_is_string(MdhValue val);
MdhValue __mdh_is_list(MdhValue val);
MdhValue __mdh_is_dict(MdhValue val);
MdhValue __mdh_is_function(MdhValue val);

/* String prefix/suffix checking */
MdhValue __mdh_starts_with(MdhValue str, MdhValue prefix);
MdhValue __mdh_ends_with(MdhValue str, MdhValue suffix);

#endif /* MDH_RUNTIME_H */
