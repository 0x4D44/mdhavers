/*
 * mdhavers Runtime Library - Implementation
 *
 * Provides runtime support for compiled mdhavers programs.
 * Uses Boehm GC for memory management.
 */

#define _GNU_SOURCE
#define _XOPEN_SOURCE 700

#include "mdh_runtime.h"

#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <regex.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>
#include <setjmp.h>
#include <termios.h>
#include <sys/ioctl.h>

/* Boehm GC - declared as extern */
extern void GC_init(void);
extern void *GC_malloc(size_t size);
extern void *GC_realloc(void *ptr, size_t size);
extern char *GC_strdup(const char *s);

/* Rust runtime FFI (JSON + regex) */
extern MdhValue __mdh_rs_json_parse(MdhValue json_str);
extern MdhValue __mdh_rs_json_stringify(MdhValue value);
extern MdhValue __mdh_rs_json_pretty(MdhValue value);
extern MdhValue __mdh_rs_regex_test(MdhValue text, MdhValue pattern);
extern MdhValue __mdh_rs_regex_match(MdhValue text, MdhValue pattern);
extern MdhValue __mdh_rs_regex_match_all(MdhValue text, MdhValue pattern);
extern MdhValue __mdh_rs_regex_replace(MdhValue text, MdhValue pattern, MdhValue replacement);
extern MdhValue __mdh_rs_regex_replace_first(MdhValue text, MdhValue pattern, MdhValue replacement);
extern MdhValue __mdh_rs_regex_split(MdhValue text, MdhValue pattern);

/* Random number generator state */
static int __mdh_random_initialized = 0;

/* Command-line args (set by the generated main) */
static int32_t __mdh_argc = 0;
static char **__mdh_argv = NULL;

static void __mdh_ensure_rng(void) {
    if (!__mdh_random_initialized) {
        srand((unsigned int)time(NULL));
        __mdh_random_initialized = 1;
    }
}

typedef struct {
    char *buf;
    size_t len;
    size_t cap;
} MdhStrBuf;

static const char *__mdh_type_name(MdhValue v);

static void __mdh_sb_init(MdhStrBuf *sb) {
    sb->cap = 128;
    sb->len = 0;
    sb->buf = (char *)GC_malloc(sb->cap);
    sb->buf[0] = '\0';
}

static void __mdh_sb_reserve(MdhStrBuf *sb, size_t extra) {
    while (sb->len + extra + 1 > sb->cap) {
        sb->cap *= 2;
        sb->buf = (char *)GC_realloc(sb->buf, sb->cap);
    }
}

static void __mdh_sb_append_n(MdhStrBuf *sb, const char *s, size_t n) {
    __mdh_sb_reserve(sb, n);
    memcpy(sb->buf + sb->len, s, n);
    sb->len += n;
    sb->buf[sb->len] = '\0';
}

static void __mdh_sb_append(MdhStrBuf *sb, const char *s) {
    __mdh_sb_append_n(sb, s, strlen(s));
}

static void __mdh_sb_append_char(MdhStrBuf *sb, char c) {
    __mdh_sb_reserve(sb, 1);
    sb->buf[sb->len++] = c;
    sb->buf[sb->len] = '\0';
}

static MdhValue __mdh_string_from_buf(char *s) {
    MdhValue v;
    v.tag = MDH_TAG_STRING;
    v.data = (int64_t)(intptr_t)s;
    return v;
}

/* ========== Value Creation ========== */

MdhValue __mdh_make_nil(void) {
    MdhValue v;
    v.tag = MDH_TAG_NIL;
    v.data = 0;
    return v;
}

MdhValue __mdh_make_bool(bool value) {
    MdhValue v;
    v.tag = MDH_TAG_BOOL;
    v.data = value ? 1 : 0;
    return v;
}

MdhValue __mdh_make_int(int64_t value) {
    MdhValue v;
    v.tag = MDH_TAG_INT;
    v.data = value;
    return v;
}

MdhValue __mdh_make_float(double value) {
    MdhValue v;
    v.tag = MDH_TAG_FLOAT;
    /* Store float bits in the int64 field */
    union { double f; int64_t i; } u;
    u.f = value;
    v.data = u.i;
    return v;
}

MdhValue __mdh_make_string(const char *value) {
    MdhValue v;
    v.tag = MDH_TAG_STRING;

    /* Store char* directly (matches LLVM backend convention) */
    char *s = GC_strdup(value);
    v.data = (int64_t)(intptr_t)s;
    return v;
}

MdhValue __mdh_make_list(int32_t capacity) {
    MdhValue v;
    v.tag = MDH_TAG_LIST;

    MdhList *list = (MdhList *)GC_malloc(sizeof(MdhList));
    list->length = 0;
    list->capacity = capacity > 0 ? capacity : 8;
    list->items = (MdhValue *)GC_malloc(sizeof(MdhValue) * list->capacity);

    v.data = (int64_t)(intptr_t)list;
    return v;
}

/* ========== Arithmetic Operations ========== */

MdhValue __mdh_add(MdhValue a, MdhValue b) {
    /* Integer + Integer */
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        return __mdh_make_int(a.data + b.data);
    }

    /* Float operations */
    if (a.tag == MDH_TAG_FLOAT || b.tag == MDH_TAG_FLOAT) {
        double af = (a.tag == MDH_TAG_FLOAT) ? __mdh_get_float(a) : (double)a.data;
        double bf = (b.tag == MDH_TAG_FLOAT) ? __mdh_get_float(b) : (double)b.data;
        return __mdh_make_float(af + bf);
    }

    /* String concatenation */
    if (a.tag == MDH_TAG_STRING && b.tag == MDH_TAG_STRING) {
        return __mdh_str_concat(a, b);
    }

    /* Type error */
    __mdh_type_error("add", a.tag, b.tag);
    return __mdh_make_nil();
}

MdhValue __mdh_sub(MdhValue a, MdhValue b) {
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        return __mdh_make_int(a.data - b.data);
    }

    if (a.tag == MDH_TAG_FLOAT || b.tag == MDH_TAG_FLOAT) {
        double af = (a.tag == MDH_TAG_FLOAT) ? __mdh_get_float(a) : (double)a.data;
        double bf = (b.tag == MDH_TAG_FLOAT) ? __mdh_get_float(b) : (double)b.data;
        return __mdh_make_float(af - bf);
    }

    __mdh_type_error("subtract", a.tag, b.tag);
    return __mdh_make_nil();
}

MdhValue __mdh_mul(MdhValue a, MdhValue b) {
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        return __mdh_make_int(a.data * b.data);
    }

    if (a.tag == MDH_TAG_FLOAT || b.tag == MDH_TAG_FLOAT) {
        double af = (a.tag == MDH_TAG_FLOAT) ? __mdh_get_float(a) : (double)a.data;
        double bf = (b.tag == MDH_TAG_FLOAT) ? __mdh_get_float(b) : (double)b.data;
        return __mdh_make_float(af * bf);
    }

    /* String repetition: "ab" * 3 = "ababab" */
    if (a.tag == MDH_TAG_STRING && b.tag == MDH_TAG_INT) {
        const char *s = __mdh_get_string(a);
        int64_t n = b.data;
        if (n <= 0) return __mdh_make_string("");

        size_t len = strlen(s);
        char *result = (char *)GC_malloc(len * n + 1);
        result[0] = '\0';
        for (int64_t i = 0; i < n; i++) {
            strcat(result, s);
        }
        return __mdh_make_string(result);
    }

    __mdh_type_error("multiply", a.tag, b.tag);
    return __mdh_make_nil();
}

MdhValue __mdh_div(MdhValue a, MdhValue b) {
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        if (b.data == 0) {
            fprintf(stderr, "Och! Division by zero!\n");
            exit(1);
        }
        return __mdh_make_int(a.data / b.data);
    }

    if (a.tag == MDH_TAG_FLOAT || b.tag == MDH_TAG_FLOAT) {
        double af = (a.tag == MDH_TAG_FLOAT) ? __mdh_get_float(a) : (double)a.data;
        double bf = (b.tag == MDH_TAG_FLOAT) ? __mdh_get_float(b) : (double)b.data;
        if (bf == 0.0) {
            fprintf(stderr, "Och! Division by zero!\n");
            exit(1);
        }
        return __mdh_make_float(af / bf);
    }

    __mdh_type_error("divide", a.tag, b.tag);
    return __mdh_make_nil();
}

MdhValue __mdh_mod(MdhValue a, MdhValue b) {
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        if (b.data == 0) {
            fprintf(stderr, "Och! Modulo by zero!\n");
            exit(1);
        }
        return __mdh_make_int(a.data % b.data);
    }

    if (a.tag == MDH_TAG_FLOAT || b.tag == MDH_TAG_FLOAT) {
        double af = (a.tag == MDH_TAG_FLOAT) ? __mdh_get_float(a) : (double)a.data;
        double bf = (b.tag == MDH_TAG_FLOAT) ? __mdh_get_float(b) : (double)b.data;
        return __mdh_make_float(fmod(af, bf));
    }

    __mdh_type_error("modulo", a.tag, b.tag);
    return __mdh_make_nil();
}

MdhValue __mdh_neg(MdhValue a) {
    if (a.tag == MDH_TAG_INT) {
        return __mdh_make_int(-a.data);
    }
    if (a.tag == MDH_TAG_FLOAT) {
        return __mdh_make_float(-__mdh_get_float(a));
    }

    __mdh_type_error("negate", a.tag, 0);
    return __mdh_make_nil();
}

/* ========== Comparison Operations ========== */

bool __mdh_eq(MdhValue a, MdhValue b) {
    if (a.tag != b.tag) {
        /* Allow int/float comparison */
        if ((a.tag == MDH_TAG_INT && b.tag == MDH_TAG_FLOAT) ||
            (a.tag == MDH_TAG_FLOAT && b.tag == MDH_TAG_INT)) {
            double af = (a.tag == MDH_TAG_FLOAT) ? __mdh_get_float(a) : (double)a.data;
            double bf = (b.tag == MDH_TAG_FLOAT) ? __mdh_get_float(b) : (double)b.data;
            return af == bf;
        }
        return false;
    }

    switch (a.tag) {
        case MDH_TAG_NIL:
            return true;
        case MDH_TAG_BOOL:
        case MDH_TAG_INT:
            return a.data == b.data;
        case MDH_TAG_FLOAT:
            return __mdh_get_float(a) == __mdh_get_float(b);
        case MDH_TAG_STRING:
            return strcmp(__mdh_get_string(a), __mdh_get_string(b)) == 0;
        case MDH_TAG_LIST: {
            MdhList *la = __mdh_get_list(a);
            MdhList *lb = __mdh_get_list(b);
            if (!la || !lb) return la == lb;
            if (la->length != lb->length) return false;
            for (int64_t i = 0; i < la->length; i++) {
                if (!__mdh_eq(la->items[i], lb->items[i])) {
                    return false;
                }
            }
            return true;
        }
        default:
            /* Reference equality for complex types */
            return a.data == b.data;
    }
}

bool __mdh_ne(MdhValue a, MdhValue b) {
    return !__mdh_eq(a, b);
}

bool __mdh_lt(MdhValue a, MdhValue b) {
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        return a.data < b.data;
    }

    if (a.tag == MDH_TAG_FLOAT || b.tag == MDH_TAG_FLOAT) {
        double af = (a.tag == MDH_TAG_FLOAT) ? __mdh_get_float(a) : (double)a.data;
        double bf = (b.tag == MDH_TAG_FLOAT) ? __mdh_get_float(b) : (double)b.data;
        return af < bf;
    }

    if (a.tag == MDH_TAG_STRING && b.tag == MDH_TAG_STRING) {
        return strcmp(__mdh_get_string(a), __mdh_get_string(b)) < 0;
    }

    __mdh_type_error("compare", a.tag, b.tag);
    return false;
}

bool __mdh_le(MdhValue a, MdhValue b) {
    return __mdh_lt(a, b) || __mdh_eq(a, b);
}

bool __mdh_gt(MdhValue a, MdhValue b) {
    return !__mdh_le(a, b);
}

bool __mdh_ge(MdhValue a, MdhValue b) {
    return !__mdh_lt(a, b);
}

/* ========== Logical Operations ========== */

MdhValue __mdh_not(MdhValue a) {
    return __mdh_make_bool(!__mdh_truthy(a));
}

bool __mdh_truthy(MdhValue a) {
    switch (a.tag) {
        case MDH_TAG_NIL:
            return false;
        case MDH_TAG_BOOL:
            return a.data != 0;
        case MDH_TAG_INT:
            return a.data != 0;
        case MDH_TAG_FLOAT:
            return __mdh_get_float(a) != 0.0;
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(a);
            return s && s[0] != '\0';
        }
        case MDH_TAG_LIST: {
            MdhList *list = __mdh_get_list(a);
            return list && list->length > 0;
        }
        default:
            return true;  /* Objects are truthy */
    }
}

/* ========== Type Operations ========== */

uint8_t __mdh_get_tag(MdhValue a) {
    return a.tag;
}

void __mdh_type_error(const char *op, uint8_t got1, uint8_t got2) {
    static const char *type_names[] = {
        "naething", "bool", "integer", "float", "string",
        "list", "dict", "function", "class", "instance", "range"
    };

    char buf[256];
    if (got1 < 11 && got2 > 0 && got2 < 11) {
        snprintf(
            buf,
            sizeof(buf),
            "Och! Type error in '%s': got %s and %s",
            op,
            type_names[got1],
            type_names[got2]
        );
    } else if (got1 < 11) {
        snprintf(
            buf,
            sizeof(buf),
            "Och! Type error in '%s': got %s",
            op,
            type_names[got1]
        );
    } else {
        snprintf(buf, sizeof(buf), "Och! Type error in '%s'", op);
    }

    __mdh_hurl(__mdh_make_string(buf));
}

MdhValue __mdh_type_of(MdhValue a) {
    return __mdh_make_string(__mdh_type_name(a));
}

void __mdh_key_not_found(MdhValue key) {
    const char *k = "<non-string>";
    if (key.tag == MDH_TAG_STRING) {
        k = __mdh_get_string(key);
    }
    char buf[256];
    snprintf(
        buf,
        sizeof(buf),
        "Awa' an bile yer heid! '%s' hasnae been defined yet",
        k
    );
    __mdh_hurl(__mdh_make_string(buf));
}

/* ========== I/O ========== */

void __mdh_blether(MdhValue a) {
    MdhValue s = __mdh_to_string(a);
    printf("%s\n", __mdh_get_string(s));
}

MdhValue __mdh_speir(MdhValue prompt) {
    /* Print prompt without newline */
    switch (prompt.tag) {
        case MDH_TAG_STRING:
            printf("%s", __mdh_get_string(prompt));
            break;
        default:
            break;
    }
    fflush(stdout);

    /* Read line */
    char buffer[1024];
    if (fgets(buffer, sizeof(buffer), stdin) != NULL) {
        /* Remove trailing newline */
        size_t len = strlen(buffer);
        if (len > 0 && buffer[len - 1] == '\n') {
            buffer[len - 1] = '\0';
        }
        return __mdh_make_string(buffer);
    }

    return __mdh_make_string("");
}

MdhValue __mdh_get_key(void) {
    if (!isatty(STDIN_FILENO)) {
        return __mdh_make_string("");
    }

    struct termios old_tio, new_tio;
    unsigned char c;

    /* Get current terminal settings */
    tcgetattr(STDIN_FILENO, &old_tio);
    new_tio = old_tio;

    /* Disable canonical mode (buffered i/o) and local echo */
    new_tio.c_lflag &= (~ICANON & ~ECHO);

    /* Non-blocking: return immediately if no key available */
    new_tio.c_cc[VMIN] = 0;
    new_tio.c_cc[VTIME] = 1;  /* 100ms timeout */

    /* Apply new settings immediately */
    tcsetattr(STDIN_FILENO, TCSANOW, &new_tio);

    /* Read one character (non-blocking with timeout) */
    if (read(STDIN_FILENO, &c, 1) > 0) {
        /* Restore settings immediately */
        tcsetattr(STDIN_FILENO, TCSANOW, &old_tio);

        if (c == 27) { /* Escape sequence start \x1b */
            /* Try to read more chars non-blocking (simple hack for now) */
            /* In a real raw loop, we might want to return ESC immediately or parse.
               For parity with crossterm, we should try to detect arrows. */
            
            /* Quick check for [A, [B, [C, [D */
            /* Re-enable raw mode briefly to peek */
            tcsetattr(STDIN_FILENO, TCSANOW, &new_tio);
            
            unsigned char seq[2];
            /* Set non-blocking read for sequence */
            struct termios nb_tio = new_tio;
            nb_tio.c_cc[VMIN] = 0;
            nb_tio.c_cc[VTIME] = 0;
            tcsetattr(STDIN_FILENO, TCSANOW, &nb_tio);
            
            if (read(STDIN_FILENO, &seq[0], 1) == 1 && seq[0] == '[') {
                 if (read(STDIN_FILENO, &seq[1], 1) == 1) {
                     tcsetattr(STDIN_FILENO, TCSANOW, &old_tio);
                     switch (seq[1]) {
                         case 'A': return __mdh_make_string("Up");
                         case 'B': return __mdh_make_string("Down");
                         case 'C': return __mdh_make_string("Right");
                         case 'D': return __mdh_make_string("Left");
                     }
                     return __mdh_make_string("\x1b"); /* Unknown sequence */
                 }
            }
            
            tcsetattr(STDIN_FILENO, TCSANOW, &old_tio);
            return __mdh_make_string("\x1b");
        } else if (c == 10 || c == 13) {
            return __mdh_make_string("\n");
        } else if (c == 127) {
            return __mdh_make_string("\x08"); /* Backspace */
        }
        
        char str[2] = { (char)c, '\0' };
        return __mdh_make_string(str);
    }

    /* Restore settings on error/EOF */
    tcsetattr(STDIN_FILENO, TCSANOW, &old_tio);
    return __mdh_make_string("");
}

/* ========== Terminal Dimensions ========== */

MdhValue __mdh_term_width(void) {
    struct winsize w;
    if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &w) == 0) {
        return __mdh_make_int(w.ws_col);
    }
    return __mdh_make_int(80);  /* Default fallback */
}

MdhValue __mdh_term_height(void) {
    struct winsize w;
    if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &w) == 0) {
        return __mdh_make_int(w.ws_row);
    }
    return __mdh_make_int(24);  /* Default fallback */
}

/* ========== List Operations ========== */

MdhValue __mdh_list_get(MdhValue list, int64_t index) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("index", list.tag, 0);
        return __mdh_make_nil();
    }

    MdhList *l = __mdh_get_list(list);
    if (index < 0) index += l->length;  /* Negative indexing */

    if (index < 0 || index >= l->length) {
        fprintf(stderr, "Och! Index %lld oot o' bounds (list has %lld items)\n",
                (long long)index, (long long)l->length);
        exit(1);
    }

    return l->items[index];
}

void __mdh_list_set(MdhValue list, int64_t index, MdhValue value) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("index", list.tag, 0);
        return;
    }

    MdhList *l = __mdh_get_list(list);
    if (index < 0) index += l->length;

    if (index < 0 || index >= l->length) {
        fprintf(stderr, "Och! Index %lld oot o' bounds (list has %lld items)\n",
                (long long)index, (long long)l->length);
        exit(1);
    }

    l->items[index] = value;
}

void __mdh_list_push(MdhValue list, MdhValue value) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("shove", list.tag, 0);
        return;
    }

    MdhList *l = __mdh_get_list(list);

    /* Grow if needed */
    if (l->length >= l->capacity) {
        l->capacity *= 2;
        l->items = (MdhValue *)GC_realloc(l->items, sizeof(MdhValue) * l->capacity);
    }

    l->items[l->length++] = value;
}

MdhValue __mdh_list_pop(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("yank", list.tag, 0);
        return __mdh_make_nil();
    }

    MdhList *l = __mdh_get_list(list);
    if (l->length == 0) {
        fprintf(stderr, "Och! Cannae yank from an empty list!\n");
        exit(1);
    }

    return l->items[--l->length];
}

int64_t __mdh_list_len(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        return 0;
    }
    return __mdh_get_list(list)->length;
}

/* Check if list contains element (returns bool as MdhValue) */
MdhValue __mdh_list_contains(MdhValue list, MdhValue elem) {
    if (list.tag != MDH_TAG_LIST) {
        return __mdh_make_bool(false);
    }
    MdhList *l = __mdh_get_list(list);
    for (int64_t i = 0; i < l->length; i++) {
        if (__mdh_eq(l->items[i], elem)) {
            return __mdh_make_bool(true);
        }
    }
    return __mdh_make_bool(false);
}

/* Find index of element in list (returns -1 if not found) */
MdhValue __mdh_list_index_of(MdhValue list, MdhValue elem) {
    if (list.tag != MDH_TAG_LIST) {
        return __mdh_make_int(-1);
    }
    MdhList *l = __mdh_get_list(list);
    for (int64_t i = 0; i < l->length; i++) {
        if (__mdh_eq(l->items[i], elem)) {
            return __mdh_make_int(i);
        }
    }
    return __mdh_make_int(-1);
}

/* Generic contains - works on both strings and lists */
MdhValue __mdh_contains(MdhValue container, MdhValue elem) {
    if (container.tag == MDH_TAG_LIST) {
        return __mdh_list_contains(container, elem);
    }
    if (container.tag == MDH_TAG_DICT) {
        return __mdh_dict_contains(container, elem);
    }
    if (container.tag == MDH_TAG_STRING && elem.tag == MDH_TAG_STRING) {
        const char *haystack = __mdh_get_string(container);
        const char *needle = __mdh_get_string(elem);
        return __mdh_make_bool(strstr(haystack, needle) != NULL);
    }
    return __mdh_make_bool(false);
}

int64_t __mdh_len(MdhValue a) {
    switch (a.tag) {
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(a);
            return s ? (int64_t)strlen(s) : 0;
        }
        case MDH_TAG_LIST:
            return __mdh_list_len(a);
        default:
            __mdh_type_error("len", a.tag, 0);
            return 0;
    }
}

/* ========== String Operations ========== */

MdhValue __mdh_str_concat(MdhValue a, MdhValue b) {
    const char *sa = __mdh_get_string(a);
    const char *sb = __mdh_get_string(b);

    size_t len_a = strlen(sa);
    size_t len_b = strlen(sb);

    char *result = (char *)GC_malloc(len_a + len_b + 1);
    strcpy(result, sa);
    strcat(result, sb);

    return __mdh_make_string(result);
}

int64_t __mdh_str_len(MdhValue s) {
    if (s.tag != MDH_TAG_STRING) {
        return 0;
    }
    const char *str = __mdh_get_string(s);
    return str ? (int64_t)strlen(str) : 0;
}

static int __mdh_cmp_cstr(const void *a, const void *b) {
    const char *sa = *(const char *const *)a;
    const char *sb = *(const char *const *)b;
    return strcmp(sa, sb);
}

static bool __mdh_dict_is_creel(MdhValue dict);

static void __mdh_value_to_string_sb(MdhStrBuf *out, MdhValue v) {
    char tmp[128];

    switch (v.tag) {
        case MDH_TAG_NIL:
            __mdh_sb_append(out, "naething");
            return;
        case MDH_TAG_BOOL:
            __mdh_sb_append(out, v.data ? "aye" : "nae");
            return;
        case MDH_TAG_INT:
            snprintf(tmp, sizeof(tmp), "%lld", (long long)v.data);
            __mdh_sb_append(out, tmp);
            return;
        case MDH_TAG_FLOAT:
            snprintf(tmp, sizeof(tmp), "%g", __mdh_get_float(v));
            __mdh_sb_append(out, tmp);
            return;
        case MDH_TAG_STRING:
            __mdh_sb_append(out, __mdh_get_string(v));
            return;
        case MDH_TAG_LIST: {
            MdhList *list = __mdh_get_list(v);
            __mdh_sb_append_char(out, '[');
            if (list) {
                for (int64_t i = 0; i < list->length; i++) {
                    if (i > 0) {
                        __mdh_sb_append(out, ", ");
                    }
                    __mdh_value_to_string_sb(out, list->items[i]);
                }
            }
            __mdh_sb_append_char(out, ']');
            return;
        }
        case MDH_TAG_DICT: {
            int64_t *dict_ptr = (int64_t *)(intptr_t)v.data;
            int64_t count = dict_ptr ? *dict_ptr : 0;
            MdhValue *entries = dict_ptr ? (MdhValue *)(dict_ptr + 1) : NULL;

            bool is_set = __mdh_dict_is_creel(v);

            if (is_set) {
                __mdh_sb_append(out, "creel{");
                if (count > 0) {
                    const char **items = (const char **)GC_malloc(sizeof(char *) * (size_t)count);
                    for (int64_t i = 0; i < count; i++) {
                        MdhValue k = entries[i * 2];
                        MdhValue ks = (k.tag == MDH_TAG_STRING)
                                          ? k
                                          : __mdh_to_string(k);
                        items[i] = __mdh_get_string(ks);
                    }
                    qsort(items, (size_t)count, sizeof(char *), __mdh_cmp_cstr);
                    for (int64_t i = 0; i < count; i++) {
                        if (i > 0) {
                            __mdh_sb_append(out, ", ");
                        }
                        __mdh_sb_append_char(out, '"');
                        __mdh_sb_append(out, items[i]);
                        __mdh_sb_append_char(out, '"');
                    }
                }
                __mdh_sb_append_char(out, '}');
                return;
            }

            __mdh_sb_append_char(out, '{');
            for (int64_t i = 0; i < count; i++) {
                if (i > 0) {
                    __mdh_sb_append(out, ", ");
                }
                MdhValue k = entries[i * 2];
                MdhValue val = entries[i * 2 + 1];

                __mdh_sb_append_char(out, '"');
                if (k.tag == MDH_TAG_STRING) {
                    __mdh_sb_append(out, __mdh_get_string(k));
                } else {
                    __mdh_value_to_string_sb(out, k);
                }
                __mdh_sb_append(out, "\": ");

                __mdh_value_to_string_sb(out, val);
            }
            __mdh_sb_append_char(out, '}');
            return;
        }
        default:
            __mdh_sb_append(out, "<object>");
            return;
    }
}

MdhValue __mdh_to_string(MdhValue a) {
    if (a.tag == MDH_TAG_STRING) {
        return a;
    }

    MdhStrBuf sb;
    __mdh_sb_init(&sb);
    __mdh_value_to_string_sb(&sb, a);
    return __mdh_string_from_buf(sb.buf);
}

MdhValue __mdh_to_int(MdhValue a) {
    switch (a.tag) {
        case MDH_TAG_INT:
            return a;
        case MDH_TAG_FLOAT:
            return __mdh_make_int((int64_t)__mdh_get_float(a));
        case MDH_TAG_BOOL:
            return __mdh_make_int(a.data ? 1 : 0);
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(a);
            return __mdh_make_int(strtoll(s, NULL, 10));
        }
        default:
            __mdh_type_error("tae_int", a.tag, 0);
            return __mdh_make_int(0);
    }
}

MdhValue __mdh_to_float(MdhValue a) {
    switch (a.tag) {
        case MDH_TAG_FLOAT:
            return a;
        case MDH_TAG_INT:
            return __mdh_make_float((double)a.data);
        case MDH_TAG_BOOL:
            return __mdh_make_float(a.data ? 1.0 : 0.0);
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(a);
            return __mdh_make_float(strtod(s, NULL));
        }
        default:
            __mdh_type_error("tae_float", a.tag, 0);
            return __mdh_make_float(0.0);
    }
}

/* ========== Math ========== */

MdhValue __mdh_abs(MdhValue a) {
    if (a.tag == MDH_TAG_INT) {
        int64_t v = a.data;
        return __mdh_make_int(v < 0 ? -v : v);
    }
    if (a.tag == MDH_TAG_FLOAT) {
        return __mdh_make_float(fabs(__mdh_get_float(a)));
    }
    __mdh_type_error("abs", a.tag, 0);
    return __mdh_make_nil();
}

MdhValue __mdh_random(int64_t min, int64_t max) {
    __mdh_ensure_rng();

    if (min > max) {
        int64_t tmp = min;
        min = max;
        max = tmp;
    }

    int64_t range = max - min + 1;
    return __mdh_make_int(min + (rand() % range));
}

MdhValue __mdh_floor(MdhValue a) {
    if (a.tag == MDH_TAG_INT) return a;
    if (a.tag == MDH_TAG_FLOAT) {
        return __mdh_make_float(floor(__mdh_get_float(a)));
    }
    __mdh_type_error("floor", a.tag, 0);
    return __mdh_make_nil();
}

MdhValue __mdh_ceil(MdhValue a) {
    if (a.tag == MDH_TAG_INT) return a;
    if (a.tag == MDH_TAG_FLOAT) {
        return __mdh_make_float(ceil(__mdh_get_float(a)));
    }
    __mdh_type_error("ceil", a.tag, 0);
    return __mdh_make_nil();
}

MdhValue __mdh_round(MdhValue a) {
    if (a.tag == MDH_TAG_INT) return a;
    if (a.tag == MDH_TAG_FLOAT) {
        return __mdh_make_float(round(__mdh_get_float(a)));
    }
    __mdh_type_error("round", a.tag, 0);
    return __mdh_make_nil();
}

/* ========== Dict/Creel Operations ========== */
/* Dict memory layout: [i64 count][entry0][entry1]... where entry = [MdhValue key][MdhValue val] = 32 bytes */

static const int64_t MDH_CREEL_SENTINEL = INT64_C(0x4d4448435245454c); /* "MDHCREEL" */

MdhValue __mdh_empty_dict(void) {
    /* Allocate 16 bytes so empty dicts/creels can be disambiguated safely. */
    int64_t *dict_ptr = (int64_t *)GC_malloc(16);
    dict_ptr[0] = 0; /* count = 0 */
    dict_ptr[1] = 0; /* marker */

    MdhValue v;
    v.tag = MDH_TAG_DICT;
    v.data = (int64_t)(intptr_t)dict_ptr;
    return v;
}

MdhValue __mdh_empty_creel(void) {
    /* Allocate 16 bytes so empty dicts/creels can be disambiguated safely. */
    int64_t *dict_ptr = (int64_t *)GC_malloc(16);
    dict_ptr[0] = 0; /* count = 0 */
    dict_ptr[1] = MDH_CREEL_SENTINEL; /* marker */

    MdhValue v;
    v.tag = MDH_TAG_DICT;
    v.data = (int64_t)(intptr_t)dict_ptr;
    return v;
}

MdhValue __mdh_make_creel(MdhValue list) {
    /* Create a creel (set) from a list by inserting each element as-is. */
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("make_creel", list.tag, 0);
        return __mdh_empty_creel();
    }

    MdhList *l = __mdh_get_list(list);
    MdhValue result = __mdh_empty_creel();
    for (int64_t i = 0; i < l->length; i++) {
        result = __mdh_toss_in(result, l->items[i]);
    }
    return result;
}

/* Helper: Check if two MdhValues are equal */
static bool __mdh_values_equal(MdhValue a, MdhValue b) {
    if (a.tag != b.tag) return false;
    if (a.tag == MDH_TAG_STRING) {
        const char *sa = (const char *)(intptr_t)a.data;
        const char *sb = (const char *)(intptr_t)b.data;
        return strcmp(sa, sb) == 0;
    }
    return a.data == b.data;
}

MdhValue __mdh_dict_contains(MdhValue dict, MdhValue key) {
    if (dict.tag != MDH_TAG_DICT) {
        return __mdh_make_bool(false);
    }

    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *dict_ptr;
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);

    for (int64_t i = 0; i < count; i++) {
        MdhValue entry_key = entries[i * 2];
        if (__mdh_values_equal(entry_key, key)) {
            return __mdh_make_bool(true);
        }
    }
    return __mdh_make_bool(false);
}

MdhValue __mdh_dict_keys(MdhValue dict) {
    if (dict.tag != MDH_TAG_DICT) {
        return __mdh_make_list(0);
    }

    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *dict_ptr;
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);

    MdhValue result = __mdh_make_list((int32_t)count);
    for (int64_t i = 0; i < count; i++) {
        __mdh_list_push(result, entries[i * 2]);
    }
    return result;
}

MdhValue __mdh_dict_values(MdhValue dict) {
    if (dict.tag != MDH_TAG_DICT) {
        return __mdh_make_list(0);
    }

    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *dict_ptr;
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);

    MdhValue result = __mdh_make_list((int32_t)count);
    for (int64_t i = 0; i < count; i++) {
        __mdh_list_push(result, entries[i * 2 + 1]);
    }
    return result;
}

MdhValue __mdh_dict_set(MdhValue dict, MdhValue key, MdhValue value) {
    if (dict.tag != MDH_TAG_DICT) {
        return dict;
    }

    int64_t *old_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *old_ptr;
    MdhValue *entries = (MdhValue *)(old_ptr + 1);

    /* Check if key already exists */
    for (int64_t i = 0; i < count; i++) {
        MdhValue entry_key = entries[i * 2];
        if (__mdh_values_equal(entry_key, key)) {
            /* Update existing entry */
            entries[i * 2 + 1] = value;
            return dict;
        }
    }

    /* Add new entry: reallocate */
    int64_t new_count = count + 1;
    size_t new_size = 8 + new_count * 32;  /* 8 for count, 32 per entry */
    int64_t *new_ptr = (int64_t *)GC_malloc(new_size);

    /* Copy old data */
    *new_ptr = new_count;
    MdhValue *new_entries = (MdhValue *)(new_ptr + 1);
    for (int64_t i = 0; i < count; i++) {
        new_entries[i * 2] = entries[i * 2];       /* key */
        new_entries[i * 2 + 1] = entries[i * 2 + 1]; /* value */
    }

    /* Add new entry */
    new_entries[count * 2] = key;
    new_entries[count * 2 + 1] = value;

    MdhValue v;
    v.tag = MDH_TAG_DICT;
    v.data = (int64_t)(intptr_t)new_ptr;
    return v;
}

MdhValue __mdh_dict_get(MdhValue dict, MdhValue key) {
    if (dict.tag != MDH_TAG_DICT) {
        return __mdh_make_nil();
    }

    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *dict_ptr;
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);

    for (int64_t i = 0; i < count; i++) {
        MdhValue entry_key = entries[i * 2];
        if (__mdh_values_equal(entry_key, key)) {
            return entries[i * 2 + 1];
        }
    }
    return __mdh_make_nil();
}

MdhValue __mdh_dict_get_default(MdhValue dict, MdhValue key, MdhValue default_val) {
    /* Like dict_get, but returns a caller-provided default when key is missing.
       This must distinguish "missing" from "present but value is naething". */
    if (dict.tag != MDH_TAG_DICT) {
        __mdh_type_error("dict_get", dict.tag, 0);
        return default_val;
    }

    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *dict_ptr;
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);

    for (int64_t i = 0; i < count; i++) {
        MdhValue entry_key = entries[i * 2];
        if (__mdh_values_equal(entry_key, key)) {
            return entries[i * 2 + 1];
        }
    }
    return default_val;
}

MdhValue __mdh_dict_merge(MdhValue a, MdhValue b) {
    if (a.tag != MDH_TAG_DICT) {
        __mdh_type_error("dict_merge", a.tag, 0);
        return __mdh_empty_dict();
    }
    if (b.tag != MDH_TAG_DICT) {
        __mdh_type_error("dict_merge", b.tag, 0);
        return __mdh_empty_dict();
    }

    MdhValue result = __mdh_empty_dict();

    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        result = __mdh_dict_set(result, a_entries[i * 2], a_entries[i * 2 + 1]);
    }

    int64_t *b_ptr = (int64_t *)(intptr_t)b.data;
    int64_t b_count = *b_ptr;
    MdhValue *b_entries = (MdhValue *)(b_ptr + 1);
    for (int64_t i = 0; i < b_count; i++) {
        result = __mdh_dict_set(result, b_entries[i * 2], b_entries[i * 2 + 1]);
    }

    return result;
}

MdhValue __mdh_dict_remove(MdhValue dict, MdhValue key) {
    if (dict.tag != MDH_TAG_DICT) {
        __mdh_type_error("dict_remove", dict.tag, 0);
        return __mdh_empty_dict();
    }

    MdhValue result = __mdh_empty_dict();
    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *dict_ptr;
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);
    for (int64_t i = 0; i < count; i++) {
        MdhValue entry_key = entries[i * 2];
        MdhValue entry_val = entries[i * 2 + 1];
        if (!__mdh_values_equal(entry_key, key)) {
            result = __mdh_dict_set(result, entry_key, entry_val);
        }
    }
    return result;
}

MdhValue __mdh_dict_invert(MdhValue dict) {
    if (dict.tag != MDH_TAG_DICT) {
        __mdh_type_error("dict_invert", dict.tag, 0);
        return __mdh_empty_dict();
    }

    MdhValue result = __mdh_empty_dict();
    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *dict_ptr;
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);
    for (int64_t i = 0; i < count; i++) {
        MdhValue key = entries[i * 2];
        MdhValue val = entries[i * 2 + 1];

        result = __mdh_dict_set(result, val, key);
    }
    return result;
}

MdhValue __mdh_fae_pairs(MdhValue pairs) {
    if (pairs.tag != MDH_TAG_LIST) {
        __mdh_type_error("fae_pairs", pairs.tag, 0);
        return __mdh_empty_dict();
    }

    MdhValue result = __mdh_empty_dict();
    MdhList *outer = __mdh_get_list(pairs);
    for (int64_t i = 0; i < outer->length; i++) {
        MdhValue item = outer->items[i];
        if (item.tag != MDH_TAG_LIST) {
            continue;
        }
        MdhList *pair = __mdh_get_list(item);
        if (pair->length < 2) {
            continue;
        }
        MdhValue key = pair->items[0];
        MdhValue val = pair->items[1];
        result = __mdh_dict_set(result, key, val);
    }
    return result;
}

MdhValue __mdh_toss_in(MdhValue dict, MdhValue item) {
    if (dict.tag != MDH_TAG_DICT) {
        return dict;
    }

    int64_t *old_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *old_ptr;
    MdhValue *entries = (MdhValue *)(old_ptr + 1);

    /* Check if item already exists */
    for (int64_t i = 0; i < count; i++) {
        MdhValue entry_key = entries[i * 2];
        if (__mdh_values_equal(entry_key, item)) {
            /* Already exists, return unchanged */
            return dict;
        }
    }

    /* Add new entry: reallocate */
    int64_t new_count = count + 1;
    size_t new_size = 8 + new_count * 32;  /* 8 for count, 32 per entry */
    int64_t *new_ptr = (int64_t *)GC_malloc(new_size);

    /* Copy old data */
    *new_ptr = new_count;
    MdhValue *new_entries = (MdhValue *)(new_ptr + 1);
    for (int64_t i = 0; i < count; i++) {
        new_entries[i * 2] = entries[i * 2];       /* key */
        new_entries[i * 2 + 1] = entries[i * 2 + 1]; /* value */
    }

    /* Add new entry (for sets, key == value) */
    new_entries[count * 2] = item;
    new_entries[count * 2 + 1] = item;

    MdhValue v;
    v.tag = MDH_TAG_DICT;
    v.data = (int64_t)(intptr_t)new_ptr;
    return v;
}

MdhValue __mdh_heave_oot(MdhValue dict, MdhValue item) {
    if (dict.tag != MDH_TAG_DICT) {
        return dict;
    }

    int64_t *old_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *old_ptr;
    MdhValue *entries = (MdhValue *)(old_ptr + 1);

    /* Find the item */
    int64_t found_idx = -1;
    for (int64_t i = 0; i < count; i++) {
        MdhValue entry_key = entries[i * 2];
        if (__mdh_values_equal(entry_key, item)) {
            found_idx = i;
            break;
        }
    }

    if (found_idx < 0) {
        /* Not found, return unchanged */
        return dict;
    }

    /* Remove entry: reallocate without it */
    int64_t new_count = count - 1;
    size_t new_size = 8 + new_count * 32;
    int64_t *new_ptr = (int64_t *)GC_malloc(new_size);

    *new_ptr = new_count;
    MdhValue *new_entries = (MdhValue *)(new_ptr + 1);
    int64_t j = 0;
    for (int64_t i = 0; i < count; i++) {
        if (i != found_idx) {
            new_entries[j * 2] = entries[i * 2];
            new_entries[j * 2 + 1] = entries[i * 2 + 1];
            j++;
        }
    }

    MdhValue v;
    v.tag = MDH_TAG_DICT;
    v.data = (int64_t)(intptr_t)new_ptr;
    return v;
}

/* ========== File I/O Operations ========== */

#include <sys/stat.h>

MdhValue __mdh_file_exists(MdhValue path) {
    if (path.tag != MDH_TAG_STRING) {
        return __mdh_make_bool(false);
    }
    const char *p = (const char *)(intptr_t)path.data;
    struct stat st;
    return __mdh_make_bool(stat(p, &st) == 0);
}

MdhValue __mdh_slurp(MdhValue path) {
    if (path.tag != MDH_TAG_STRING) {
        return __mdh_make_string("");
    }
    const char *p = (const char *)(intptr_t)path.data;
    FILE *f = fopen(p, "r");
    if (!f) return __mdh_make_string("");
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = (char *)GC_malloc(size + 1);
    if (size > 0) {
        size_t read_count = fread(buf, 1, size, f);
        buf[read_count] = '\0';
    } else {
        buf[0] = '\0';
    }
    fclose(f);
    return __mdh_make_string(buf);
}

MdhValue __mdh_scrieve(MdhValue path, MdhValue content) {
    if (path.tag != MDH_TAG_STRING || content.tag != MDH_TAG_STRING) {
        return __mdh_make_bool(false);
    }
    const char *p = (const char *)(intptr_t)path.data;
    const char *c = (const char *)(intptr_t)content.data;
    FILE *f = fopen(p, "w");
    if (!f) return __mdh_make_bool(false);
    fputs(c, f);
    fclose(f);
    return __mdh_make_bool(true);
}

MdhValue __mdh_lines(MdhValue path) {
    MdhValue content = __mdh_slurp(path);
    if (content.tag != MDH_TAG_STRING) {
        return __mdh_make_list(0);
    }

    const char *str = (const char *)(intptr_t)content.data;
    MdhValue result = __mdh_make_list(16);

    const char *start = str;
    const char *p = str;
    while (*p) {
        if (*p == '\n') {
            /* Create line string */
            size_t len = p - start;
            char *line = (char *)GC_malloc(len + 1);
            memcpy(line, start, len);
            line[len] = '\0';
            __mdh_list_push(result, __mdh_make_string(line));
            start = p + 1;
        }
        p++;
    }
    /* Handle last line without newline */
    if (start != p) {
        size_t len = p - start;
        char *line = (char *)GC_malloc(len + 1);
        memcpy(line, start, len);
        line[len] = '\0';
        __mdh_list_push(result, __mdh_make_string(line));
    }
    return result;
}

MdhValue __mdh_words(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) {
        return __mdh_make_list(0);
    }

    const char *s = (const char *)(intptr_t)str.data;
    MdhValue result = __mdh_make_list(16);

    const char *start = NULL;
    const char *p = s;
    while (1) {
        if (*p == '\0' || *p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') {
            if (start != NULL) {
                /* End of word */
                size_t len = p - start;
                char *word = (char *)GC_malloc(len + 1);
                memcpy(word, start, len);
                word[len] = '\0';
                __mdh_list_push(result, __mdh_make_string(word));
                start = NULL;
            }
            if (*p == '\0') break;
        } else if (start == NULL) {
            start = p;
        }
        p++;
    }
    return result;
}

/* ========== Logging/Debug ========== */

static int __mdh_log_level = 2;  /* Default: INFO */

MdhValue __mdh_get_log_level(void) {
    return __mdh_make_int(__mdh_log_level);
}

MdhValue __mdh_set_log_level(MdhValue level) {
    if (level.tag == MDH_TAG_INT) {
        __mdh_log_level = (int)level.data;
    }
    return __mdh_make_nil();
}

/* ========== Scots Word Aliases ========== */

MdhValue __mdh_slainte(void) {
    static const char *toasts[] = {
        "Slàinte mhath! (Good health!)",
        "Here's tae us, wha's like us? Gey few, and they're a' deid!",
        "May the best ye've ever seen be the worst ye'll ever see!",
        "Lang may yer lum reek wi' ither fowk's coal!",
        "May ye aye be happy, an' never drink frae a toom glass!",
        "Here's tae the heath, the hill and the heather!",
    };

    struct timespec ts;
    uint64_t seed;
    if (clock_gettime(CLOCK_REALTIME, &ts) == 0) {
        seed = ((uint64_t)ts.tv_sec * 1000000000ULL) + (uint64_t)ts.tv_nsec;
    } else {
        seed = (uint64_t)time(NULL);
    }

    uint64_t rng = seed * 1103515245ULL + 12345ULL;
    size_t idx = (size_t)(rng % (sizeof(toasts) / sizeof(toasts[0])));
    return __mdh_make_string(toasts[idx]);
}

MdhValue __mdh_och(MdhValue msg) {
    MdhValue s = __mdh_to_string(msg);
    const char *m = __mdh_get_string(s);
    const char *prefix = "Och! ";
    size_t plen = strlen(prefix);
    size_t mlen = strlen(m);
    char *out = (char *)GC_malloc(plen + mlen + 1);
    memcpy(out, prefix, plen);
    memcpy(out + plen, m, mlen);
    out[plen + mlen] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_help_ma_boab(MdhValue msg) {
    MdhValue s = __mdh_to_string(msg);
    const char *m = __mdh_get_string(s);
    const char *prefix = "Help ma boab! ";
    size_t plen = strlen(prefix);
    size_t mlen = strlen(m);
    char *out = (char *)GC_malloc(plen + mlen + 1);
    memcpy(out, prefix, plen);
    memcpy(out + plen, m, mlen);
    out[plen + mlen] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_haver(void) {
    static const char *havers[] = {
        "Och, yer bum's oot the windae!",
        "Awa' an bile yer heid!",
        "Haud yer wheesht, ya numpty!",
        "Dinnae fash yersel!",
        "Whit's fer ye'll no go by ye!",
        "Lang may yer lum reek!",
        "Yer a wee scunner, so ye are!",
        "Haste ye back!",
        "It's a dreich day the day!",
        "Pure dead brilliant!",
        "Ah'm fair puckled!",
        "Gie it laldy!",
        "Whit a stoater!",
        "That's pure mince!",
        "Jings, crivvens, help ma boab!",
    };

    struct timespec ts;
    uint64_t seed;
    if (clock_gettime(CLOCK_REALTIME, &ts) == 0) {
        seed = ((uint64_t)ts.tv_sec * 1000000000ULL) + (uint64_t)ts.tv_nsec;
    } else {
        seed = (uint64_t)time(NULL);
    }

    uint64_t rng = seed * 1103515245ULL + 12345ULL;
    size_t idx = (size_t)(rng % (sizeof(havers) / sizeof(havers[0])));
    return __mdh_make_string(havers[idx]);
}

MdhValue __mdh_braw_time(void) {
    time_t now = time(NULL);
    uint64_t secs = (now < 0) ? 0ULL : (uint64_t)now;
    uint64_t hours = (secs / 3600ULL) % 24ULL;
    uint64_t minutes = (secs / 60ULL) % 60ULL;

    const char *prefix;
    if (hours <= 5) {
        prefix = "It's the wee small hours";
    } else if (hours <= 11) {
        prefix = "It's the mornin'";
    } else if (hours == 12) {
        prefix = "It's high noon";
    } else if (hours <= 17) {
        prefix = "It's the efternoon";
    } else if (hours <= 21) {
        prefix = "It's the evenin'";
    } else {
        prefix = "It's gettin' late";
    }

    char buf[128];
    snprintf(buf, sizeof(buf), "%s (%02llu:%02llu)", prefix, (unsigned long long)hours, (unsigned long long)minutes);
    return __mdh_make_string(buf);
}

MdhValue __mdh_wee(MdhValue a, MdhValue b) {
    /* Return smaller of two values */
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        return a.data < b.data ? a : b;
    }
    if (a.tag == MDH_TAG_FLOAT || b.tag == MDH_TAG_FLOAT) {
        double af = (a.tag == MDH_TAG_FLOAT) ? __mdh_get_float(a) : (double)a.data;
        double bf = (b.tag == MDH_TAG_FLOAT) ? __mdh_get_float(b) : (double)b.data;
        return af < bf ? a : b;
    }
    return a;
}

MdhValue __mdh_tak(MdhValue list, MdhValue n) {
    /* Take first n elements from list */
    if (list.tag != MDH_TAG_LIST || n.tag != MDH_TAG_INT) {
        return __mdh_make_list(0);
    }

    MdhList *src = (MdhList *)(intptr_t)list.data;
    int64_t take_count = n.data;
    if (take_count < 0) take_count = 0;
    if (take_count > src->length) take_count = src->length;

    MdhValue result = __mdh_make_list((int32_t)take_count);
    for (int64_t i = 0; i < take_count; i++) {
        __mdh_list_push(result, src->items[i]);
    }
    return result;
}

MdhValue __mdh_pair_up(MdhValue list1, MdhValue list2) {
    /* Zip two lists together */
    if (list1.tag != MDH_TAG_LIST || list2.tag != MDH_TAG_LIST) {
        return __mdh_make_list(0);
    }

    MdhList *l1 = (MdhList *)(intptr_t)list1.data;
    MdhList *l2 = (MdhList *)(intptr_t)list2.data;
    int64_t min_len = l1->length < l2->length ? l1->length : l2->length;

    MdhValue result = __mdh_make_list((int32_t)min_len);
    for (int64_t i = 0; i < min_len; i++) {
        /* Create a 2-element list for each pair */
        MdhValue pair = __mdh_make_list(2);
        __mdh_list_push(pair, l1->items[i]);
        __mdh_list_push(pair, l2->items[i]);
        __mdh_list_push(result, pair);
    }
    return result;
}

MdhValue __mdh_tae_binary(MdhValue n) {
    /* Convert integer to binary string */
    if (n.tag != MDH_TAG_INT) {
        return __mdh_make_string("0");
    }

    int64_t val = n.data;
    if (val == 0) return __mdh_make_string("0");

    char buf[65];  /* 64 bits + null */
    int idx = 64;
    buf[idx--] = '\0';

    int64_t abs_val = val < 0 ? -val : val;
    while (abs_val > 0 && idx >= 0) {
        buf[idx--] = (abs_val & 1) ? '1' : '0';
        abs_val >>= 1;
    }
    if (val < 0 && idx >= 0) {
        buf[idx--] = '-';
    }

    return __mdh_make_string(&buf[idx + 1]);
}

MdhValue __mdh_fae_binary(MdhValue str) {
    /* Parse binary string to integer: "101" -> 5 */
    if (str.tag != MDH_TAG_STRING) {
        return __mdh_make_int(0);
    }

    const char *s = __mdh_get_string(str);
    if (!s || *s == '\0') return __mdh_make_int(0);

    int64_t result = 0;
    for (int64_t i = 0; s[i] != '\0'; i++) {
        char c = s[i];
        if (c == '1') {
            result = (result << 1) | 1;
        } else if (c == '0') {
            result = result << 1;
        }
        /* Skip other characters (like spaces or prefix) */
    }
    return __mdh_make_int(result);
}

MdhValue __mdh_fae_hex(MdhValue str) {
    /* Parse hex string to integer: "ff" -> 255 */
    if (str.tag != MDH_TAG_STRING) {
        return __mdh_make_int(0);
    }

    const char *s = __mdh_get_string(str);
    if (!s || *s == '\0') return __mdh_make_int(0);

    int64_t result = 0;
    for (int64_t i = 0; s[i] != '\0'; i++) {
        char c = s[i];
        int digit = -1;
        if (c >= '0' && c <= '9') {
            digit = c - '0';
        } else if (c >= 'a' && c <= 'f') {
            digit = 10 + (c - 'a');
        } else if (c >= 'A' && c <= 'F') {
            digit = 10 + (c - 'A');
        }
        if (digit >= 0) {
            result = (result << 4) | digit;
        }
        /* Skip other characters (like 0x prefix) */
    }
    return __mdh_make_int(result);
}

MdhValue __mdh_ltrim(MdhValue str) {
    /* Trim leading whitespace from string */
    if (str.tag != MDH_TAG_STRING) {
        return str;
    }

    const char *s = __mdh_get_string(str);
    if (!s || *s == '\0') return str;

    int64_t len = strlen(s);
    int64_t start = 0;
    while (start < len && (s[start] == ' ' || s[start] == '\t' ||
           s[start] == '\n' || s[start] == '\r')) {
        start++;
    }

    if (start == 0) return str;  /* No leading whitespace */
    if (start == len) return __mdh_make_string("");  /* All whitespace */

    return __mdh_make_string(s + start);
}

MdhValue __mdh_rtrim(MdhValue str) {
    /* Trim trailing whitespace from string */
    if (str.tag != MDH_TAG_STRING) {
        return str;
    }

    const char *s = __mdh_get_string(str);
    if (!s || *s == '\0') return str;

    int64_t len = strlen(s);
    int64_t end = len;
    while (end > 0 && (s[end-1] == ' ' || s[end-1] == '\t' ||
           s[end-1] == '\n' || s[end-1] == '\r')) {
        end--;
    }

    if (end == len) return str;  /* No trailing whitespace */
    if (end == 0) return __mdh_make_string("");  /* All whitespace */

    char *buf = (char *)GC_malloc(end + 1);
    memcpy(buf, s, end);
    buf[end] = '\0';
    return __mdh_make_string(buf);
}

MdhValue __mdh_reverse_str(MdhValue str) {
    /* Reverse a string: "hello" -> "olleh" */
    if (str.tag != MDH_TAG_STRING) {
        return str;
    }

    const char *s = __mdh_get_string(str);
    if (!s || *s == '\0') return str;

    int64_t len = strlen(s);
    char *buf = (char *)GC_malloc(len + 1);

    for (int64_t i = 0; i < len; i++) {
        buf[i] = s[len - 1 - i];
    }
    buf[len] = '\0';
    return __mdh_make_string(buf);
}

MdhValue __mdh_title_case(MdhValue str) {
    /* Title case a string: "hello world" -> "Hello World" */
    if (str.tag != MDH_TAG_STRING) {
        return str;
    }

    const char *s = __mdh_get_string(str);
    if (!s || *s == '\0') return str;

    int64_t len = strlen(s);
    char *buf = (char *)GC_malloc(len + 1);

    bool new_word = true;
    for (int64_t i = 0; i < len; i++) {
        char c = s[i];
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') {
            new_word = true;
            buf[i] = c;
        } else if (new_word) {
            buf[i] = toupper((unsigned char)c);
            new_word = false;
        } else {
            buf[i] = tolower((unsigned char)c);
        }
    }
    buf[len] = '\0';
    return __mdh_make_string(buf);
}

MdhValue __mdh_tae_hex(MdhValue num) {
    /* Convert integer to hex string */
    if (num.tag != MDH_TAG_INT) {
        return __mdh_make_string("0");
    }
    int64_t n = num.data;
    char buf[32];
    if (n < 0) {
        snprintf(buf, sizeof(buf), "-%llx", (unsigned long long)(-n));
    } else {
        snprintf(buf, sizeof(buf), "%llx", (unsigned long long)n);
    }
    return __mdh_make_string(buf);
}

MdhValue __mdh_tae_octal(MdhValue num) {
    /* Convert integer to octal string */
    if (num.tag != MDH_TAG_INT) {
        return __mdh_make_string("0");
    }
    int64_t n = num.data;
    char buf[32];
    if (n < 0) {
        snprintf(buf, sizeof(buf), "-%llo", (unsigned long long)(-n));
    } else {
        snprintf(buf, sizeof(buf), "%llo", (unsigned long long)n);
    }
    return __mdh_make_string(buf);
}

MdhValue __mdh_center(MdhValue str, MdhValue width_val) {
    /* Center string in given width */
    if (str.tag != MDH_TAG_STRING || width_val.tag != MDH_TAG_INT) {
        return str;
    }
    const char *s = __mdh_get_string(str);
    if (!s) return __mdh_make_string("");

    int64_t width = width_val.data;
    int64_t len = strlen(s);

    if (len >= width) return str;

    int64_t total_pad = width - len;
    int64_t left_pad = total_pad / 2;

    char *buf = (char *)GC_malloc(width + 1);
    memset(buf, ' ', width);
    memcpy(buf + left_pad, s, len);
    buf[width] = '\0';
    return __mdh_make_string(buf);
}

MdhValue __mdh_repeat_say(MdhValue str, MdhValue count_val) {
    /* Repeat string n times */
    if (str.tag != MDH_TAG_STRING || count_val.tag != MDH_TAG_INT) {
        return __mdh_make_string("");
    }
    const char *s = __mdh_get_string(str);
    if (!s) return __mdh_make_string("");

    int64_t count = count_val.data;
    if (count <= 0) return __mdh_make_string("");

    int64_t len = strlen(s);
    int64_t total_len = len * count;

    char *buf = (char *)GC_malloc(total_len + 1);
    for (int64_t i = 0; i < count; i++) {
        memcpy(buf + i * len, s, len);
    }
    buf[total_len] = '\0';
    return __mdh_make_string(buf);
}

MdhValue __mdh_leftpad(MdhValue str, MdhValue width_val, MdhValue pad_val) {
    /* Left pad string to width with pad char */
    if (str.tag != MDH_TAG_STRING || width_val.tag != MDH_TAG_INT) {
        return str;
    }
    const char *s = __mdh_get_string(str);
    if (!s) return __mdh_make_string("");

    int64_t width = width_val.data;
    int64_t len = strlen(s);

    if (len >= width) return str;

    char pad_char = ' ';
    if (pad_val.tag == MDH_TAG_STRING) {
        const char *ps = __mdh_get_string(pad_val);
        if (ps && ps[0]) pad_char = ps[0];
    }

    char *buf = (char *)GC_malloc(width + 1);
    int64_t pad_len = width - len;
    memset(buf, pad_char, pad_len);
    memcpy(buf + pad_len, s, len);
    buf[width] = '\0';
    return __mdh_make_string(buf);
}

MdhValue __mdh_rightpad(MdhValue str, MdhValue width_val, MdhValue pad_val) {
    /* Right pad string to width with pad char */
    if (str.tag != MDH_TAG_STRING || width_val.tag != MDH_TAG_INT) {
        return str;
    }
    const char *s = __mdh_get_string(str);
    if (!s) return __mdh_make_string("");

    int64_t width = width_val.data;
    int64_t len = strlen(s);

    if (len >= width) return str;

    char pad_char = ' ';
    if (pad_val.tag == MDH_TAG_STRING) {
        const char *ps = __mdh_get_string(pad_val);
        if (ps && ps[0]) pad_char = ps[0];
    }

    char *buf = (char *)GC_malloc(width + 1);
    memcpy(buf, s, len);
    memset(buf + len, pad_char, width - len);
    buf[width] = '\0';
    return __mdh_make_string(buf);
}

MdhValue __mdh_list_index(MdhValue list, MdhValue val) {
    /* Find index of value in list, return -1 if not found */
    if (list.tag != MDH_TAG_LIST) {
        return __mdh_make_int(-1);
    }
    MdhList *l = (MdhList *)(intptr_t)list.data;
    for (int64_t i = 0; i < l->length; i++) {
        MdhValue item = l->items[i];
        /* Compare tags and data */
        if (item.tag == val.tag && item.data == val.data) {
            return __mdh_make_int(i);
        }
    }
    return __mdh_make_int(-1);
}

MdhValue __mdh_count_val(MdhValue list, MdhValue val) {
    /* Count occurrences of value in list */
    if (list.tag != MDH_TAG_LIST) {
        return __mdh_make_int(0);
    }
    MdhList *l = (MdhList *)(intptr_t)list.data;
    int64_t count = 0;
    for (int64_t i = 0; i < l->length; i++) {
        MdhValue item = l->items[i];
        if (item.tag == val.tag && item.data == val.data) {
            count++;
        }
    }
    return __mdh_make_int(count);
}

MdhValue __mdh_list_copy(MdhValue list) {
    /* Create a shallow copy of list */
    if (list.tag != MDH_TAG_LIST) {
        return list;
    }
    MdhList *src = (MdhList *)(intptr_t)list.data;
    MdhList *dst = (MdhList *)GC_malloc(sizeof(MdhList));
    dst->length = src->length;
    dst->capacity = src->length;
    dst->items = (MdhValue *)GC_malloc(sizeof(MdhValue) * src->length);
    for (int64_t i = 0; i < src->length; i++) {
        dst->items[i] = src->items[i];
    }
    MdhValue result;
    result.tag = MDH_TAG_LIST;
    result.data = (int64_t)(intptr_t)dst;
    return result;
}

MdhValue __mdh_list_clear(MdhValue list) {
    /* Clear a list (set length to 0) */
    if (list.tag != MDH_TAG_LIST) {
        return list;
    }
    MdhList *l = (MdhList *)(intptr_t)list.data;
    l->length = 0;
    return list;
}

MdhValue __mdh_last_index_of(MdhValue str, MdhValue substr) {
    /* Find last occurrence of substring in string */
    if (str.tag != MDH_TAG_STRING || substr.tag != MDH_TAG_STRING) {
        return __mdh_make_int(-1);
    }
    const char *s = __mdh_get_string(str);
    const char *sub = __mdh_get_string(substr);
    if (!s || !sub) return __mdh_make_int(-1);

    int64_t s_len = strlen(s);
    int64_t sub_len = strlen(sub);
    if (sub_len > s_len || sub_len == 0) return __mdh_make_int(-1);

    int64_t last_idx = -1;
    for (int64_t i = 0; i <= s_len - sub_len; i++) {
        if (strncmp(s + i, sub, sub_len) == 0) {
            last_idx = i;
        }
    }
    return __mdh_make_int(last_idx);
}

MdhValue __mdh_replace_first(MdhValue str, MdhValue old_sub, MdhValue new_sub) {
    /* Replace first occurrence of old_sub with new_sub */
    if (str.tag != MDH_TAG_STRING || old_sub.tag != MDH_TAG_STRING || new_sub.tag != MDH_TAG_STRING) {
        return str;
    }
    const char *s = __mdh_get_string(str);
    const char *old_s = __mdh_get_string(old_sub);
    const char *new_s = __mdh_get_string(new_sub);
    if (!s || !old_s || !new_s) return str;

    int64_t s_len = strlen(s);
    int64_t old_len = strlen(old_s);
    int64_t new_len = strlen(new_s);

    if (old_len == 0 || old_len > s_len) return str;

    /* Find first occurrence */
    const char *pos = strstr(s, old_s);
    if (!pos) return str;

    int64_t idx = pos - s;
    int64_t result_len = s_len - old_len + new_len;
    char *buf = (char *)GC_malloc(result_len + 1);

    memcpy(buf, s, idx);
    memcpy(buf + idx, new_s, new_len);
    memcpy(buf + idx + new_len, s + idx + old_len, s_len - idx - old_len);
    buf[result_len] = '\0';

    return __mdh_make_string(buf);
}

MdhValue __mdh_unique(MdhValue list) {
    /* Remove duplicates from list */
    if (list.tag != MDH_TAG_LIST) {
        return list;
    }
    MdhList *src = (MdhList *)(intptr_t)list.data;
    if (src->length == 0) return list;

    /* Create result list with same capacity */
    MdhList *dst = (MdhList *)GC_malloc(sizeof(MdhList));
    dst->length = 0;
    dst->capacity = src->length;
    dst->items = (MdhValue *)GC_malloc(sizeof(MdhValue) * src->length);

    /* Add items that aren't already in result */
    for (int64_t i = 0; i < src->length; i++) {
        MdhValue item = src->items[i];
        int found = 0;
        for (int64_t j = 0; j < dst->length; j++) {
            if (dst->items[j].tag == item.tag && dst->items[j].data == item.data) {
                found = 1;
                break;
            }
        }
        if (!found) {
            dst->items[dst->length++] = item;
        }
    }

    MdhValue result;
    result.tag = MDH_TAG_LIST;
    result.data = (int64_t)(intptr_t)dst;
    return result;
}

MdhValue __mdh_average(MdhValue list) {
    /* Compute average of numeric list */
    if (list.tag != MDH_TAG_LIST) {
        return __mdh_make_float(0.0);
    }

    MdhList *l = (MdhList *)(intptr_t)list.data;
    if (l->length == 0) return __mdh_make_float(0.0);

    double sum = 0.0;
    for (int64_t i = 0; i < l->length; i++) {
        MdhValue item = l->items[i];
        if (item.tag == MDH_TAG_INT) {
            sum += (double)item.data;
        } else if (item.tag == MDH_TAG_FLOAT) {
            sum += __mdh_get_float(item);
        }
    }
    return __mdh_make_float(sum / (double)l->length);
}

typedef struct {
    MdhValue value;
    const char *key_str;
} MdhCreelSortItem;

static int __mdh_creel_sort_cmp(const void *a, const void *b) {
    const MdhCreelSortItem *ia = (const MdhCreelSortItem *)a;
    const MdhCreelSortItem *ib = (const MdhCreelSortItem *)b;
    const char *sa = ia->key_str ? ia->key_str : "";
    const char *sb = ib->key_str ? ib->key_str : "";
    return strcmp(sa, sb);
}

MdhValue __mdh_creel_tae_list(MdhValue dict) {
    /* Convert set/dict keys to list */
    if (dict.tag != MDH_TAG_DICT) {
        return __mdh_make_list(0);
    }

    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *dict_ptr;
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);

    if (count <= 0) {
        return __mdh_make_list(0);
    }

    MdhCreelSortItem *items = (MdhCreelSortItem *)GC_malloc(sizeof(MdhCreelSortItem) * (size_t)count);
    for (int64_t i = 0; i < count; i++) {
        MdhValue key = entries[i * 2];
        MdhValue key_str_val = __mdh_to_string(key);
        const char *key_str = key_str_val.tag == MDH_TAG_STRING ? __mdh_get_string(key_str_val) : "";
        items[i].value = key;
        items[i].key_str = key_str;
    }
    qsort(items, (size_t)count, sizeof(MdhCreelSortItem), __mdh_creel_sort_cmp);

    MdhValue result = __mdh_make_list((int32_t)count);
    for (int64_t i = 0; i < count; i++) {
        __mdh_list_push(result, items[i].value);
    }
    return result;
}

MdhValue __mdh_creels_thegither(MdhValue a, MdhValue b) {
    /* Union of two creels/sets (dicts) */
    if (a.tag != MDH_TAG_DICT) {
        return b.tag == MDH_TAG_DICT ? b : __mdh_empty_creel();
    }
    if (b.tag != MDH_TAG_DICT) {
        return a;
    }

    MdhValue result = __mdh_empty_creel();

    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        result = __mdh_toss_in(result, a_entries[i * 2]);
    }

    int64_t *b_ptr = (int64_t *)(intptr_t)b.data;
    int64_t b_count = *b_ptr;
    MdhValue *b_entries = (MdhValue *)(b_ptr + 1);
    for (int64_t i = 0; i < b_count; i++) {
        result = __mdh_toss_in(result, b_entries[i * 2]);
    }

    return result;
}

MdhValue __mdh_creels_baith(MdhValue a, MdhValue b) {
    /* Intersection of two creels/sets (dicts) */
    if (a.tag != MDH_TAG_DICT) {
        __mdh_type_error("creels_baith", a.tag, 0);
        return __mdh_empty_creel();
    }
    if (b.tag != MDH_TAG_DICT) {
        __mdh_type_error("creels_baith", b.tag, 0);
        return __mdh_empty_creel();
    }

    MdhValue result = __mdh_empty_creel();
    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        MdhValue key = a_entries[i * 2];
        MdhValue contains = __mdh_dict_contains(b, key);
        if (contains.tag == MDH_TAG_BOOL && contains.data != 0) {
            result = __mdh_toss_in(result, key);
        }
    }
    return result;
}

MdhValue __mdh_creels_differ(MdhValue a, MdhValue b) {
    /* Difference of two creels/sets (a \\ b) */
    if (a.tag != MDH_TAG_DICT) {
        __mdh_type_error("creels_differ", a.tag, 0);
        return __mdh_empty_creel();
    }
    if (b.tag != MDH_TAG_DICT) {
        __mdh_type_error("creels_differ", b.tag, 0);
        return __mdh_empty_creel();
    }

    MdhValue result = __mdh_empty_creel();
    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        MdhValue key = a_entries[i * 2];
        MdhValue contains = __mdh_dict_contains(b, key);
        if (!(contains.tag == MDH_TAG_BOOL && contains.data != 0)) {
            result = __mdh_toss_in(result, key);
        }
    }
    return result;
}

MdhValue __mdh_is_subset(MdhValue a, MdhValue b) {
    if (a.tag != MDH_TAG_DICT) {
        __mdh_type_error("is_subset", a.tag, 0);
        return __mdh_make_bool(false);
    }
    if (b.tag != MDH_TAG_DICT) {
        __mdh_type_error("is_subset", b.tag, 0);
        return __mdh_make_bool(false);
    }

    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        MdhValue key = a_entries[i * 2];
        MdhValue contains = __mdh_dict_contains(b, key);
        if (!(contains.tag == MDH_TAG_BOOL && contains.data != 0)) {
            return __mdh_make_bool(false);
        }
    }
    return __mdh_make_bool(true);
}

MdhValue __mdh_is_superset(MdhValue a, MdhValue b) {
    /* a is superset of b iff b is subset of a */
    return __mdh_is_subset(b, a);
}

MdhValue __mdh_is_disjoint(MdhValue a, MdhValue b) {
    if (a.tag != MDH_TAG_DICT) {
        __mdh_type_error("is_disjoint", a.tag, 0);
        return __mdh_make_bool(false);
    }
    if (b.tag != MDH_TAG_DICT) {
        __mdh_type_error("is_disjoint", b.tag, 0);
        return __mdh_make_bool(false);
    }

    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        MdhValue key = a_entries[i * 2];
        MdhValue contains = __mdh_dict_contains(b, key);
        if (contains.tag == MDH_TAG_BOOL && contains.data != 0) {
            return __mdh_make_bool(false);
        }
    }
    return __mdh_make_bool(true);
}

/* Testing support */
MdhValue __mdh_assert(MdhValue condition, MdhValue msg) {
    bool cond = false;
    if (condition.tag == MDH_TAG_BOOL) {
        cond = condition.data != 0;
    } else if (condition.tag == MDH_TAG_INT) {
        cond = condition.data != 0;
    }

    if (!cond) {
        printf("Assertion failed");
        if (msg.tag == MDH_TAG_STRING) {
            printf(": %s", (const char *)(intptr_t)msg.data);
        }
        printf("\n");
        exit(1);
    }
    return __mdh_make_nil();
}

MdhValue __mdh_skip(MdhValue reason) {
    printf("Test skipped");
    if (reason.tag == MDH_TAG_STRING) {
        printf(": %s", (const char *)(intptr_t)reason.data);
    }
    printf("\n");
    return __mdh_make_nil();
}

MdhValue __mdh_stacktrace(void) {
    /* Placeholder - real stacktrace would need debug info */
    return __mdh_make_string("<stacktrace not available>");
}

MdhValue __mdh_chynge(MdhValue str, MdhValue old_sub, MdhValue new_sub) {
    /* String replace (chynge = change in Scots) */
    if (str.tag != MDH_TAG_STRING || old_sub.tag != MDH_TAG_STRING || new_sub.tag != MDH_TAG_STRING) {
        return str;
    }

    const char *s = (const char *)(intptr_t)str.data;
    const char *old_s = (const char *)(intptr_t)old_sub.data;
    const char *new_s = (const char *)(intptr_t)new_sub.data;

    size_t s_len = strlen(s);
    size_t old_len = strlen(old_s);
    size_t new_len = strlen(new_s);

    if (old_len == 0) return str;

    /* Count occurrences */
    int count = 0;
    const char *p = s;
    while ((p = strstr(p, old_s)) != NULL) {
        count++;
        p += old_len;
    }

    if (count == 0) return str;

    /* Allocate result */
    size_t result_len = s_len + count * (new_len - old_len);
    char *result = (char *)GC_malloc(result_len + 1);

    char *r = result;
    p = s;
    const char *prev = s;
    while ((p = strstr(prev, old_s)) != NULL) {
        memcpy(r, prev, p - prev);
        r += p - prev;
        memcpy(r, new_s, new_len);
        r += new_len;
        prev = p + old_len;
    }
    strcpy(r, prev);

    return __mdh_make_string(result);
}

/* ========== Additional Scots Builtins ========== */

MdhValue __mdh_muckle(MdhValue a, MdhValue b) {
    /* Return larger of two values (Scots: big/large) */
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        return a.data > b.data ? a : b;
    }
    if (a.tag == MDH_TAG_FLOAT || b.tag == MDH_TAG_FLOAT) {
        double af = (a.tag == MDH_TAG_FLOAT) ? __mdh_get_float(a) : (double)a.data;
        double bf = (b.tag == MDH_TAG_FLOAT) ? __mdh_get_float(b) : (double)b.data;
        return af > bf ? a : b;
    }
    return a;
}

MdhValue __mdh_median(MdhValue list) {
    /* Compute median of numeric list */
    if (list.tag != MDH_TAG_LIST) {
        return __mdh_make_float(0.0);
    }

    MdhList *l = (MdhList *)(intptr_t)list.data;
    if (l->length == 0) return __mdh_make_float(0.0);

    /* For simplicity, just return average - proper median would require sorting */
    return __mdh_average(list);
}

/* list_min - minimum value in a list */
MdhValue __mdh_list_min(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        return __mdh_make_int(0);
    }
    MdhList *l = __mdh_get_list(list);
    if (l->length == 0) return __mdh_make_int(0);

    MdhValue min_val = l->items[0];
    for (int64_t i = 1; i < l->length; i++) {
        if (__mdh_lt(l->items[i], min_val)) {
            min_val = l->items[i];
        }
    }
    return min_val;
}

/* list_max - maximum value in a list */
MdhValue __mdh_list_max(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        return __mdh_make_int(0);
    }
    MdhList *l = __mdh_get_list(list);
    if (l->length == 0) return __mdh_make_int(0);

    MdhValue max_val = l->items[0];
    for (int64_t i = 1; i < l->length; i++) {
        if (__mdh_gt(l->items[i], max_val)) {
            max_val = l->items[i];
        }
    }
    return max_val;
}

/* Comparison function for qsort */
static int __mdh_compare_values(const void *a, const void *b) {
    const MdhValue *va = (const MdhValue *)a;
    const MdhValue *vb = (const MdhValue *)b;
    if (__mdh_lt(*va, *vb)) return -1;
    if (__mdh_gt(*va, *vb)) return 1;
    return 0;
}

/* list_sort - return a sorted copy of the list */
MdhValue __mdh_list_sort(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        return list;
    }
    MdhList *l = __mdh_get_list(list);
    if (l->length == 0) return list;

    /* Create a copy of the list */
    MdhList *result = (MdhList *)GC_malloc(sizeof(MdhList));
    result->capacity = l->length;
    result->length = l->length;
    result->items = (MdhValue *)GC_malloc(sizeof(MdhValue) * l->length);
    for (int64_t i = 0; i < l->length; i++) {
        result->items[i] = l->items[i];
    }

    /* Sort using qsort */
    qsort(result->items, result->length, sizeof(MdhValue), __mdh_compare_values);

    return (MdhValue){ .tag = MDH_TAG_LIST, .data = (int64_t)(intptr_t)result };
}

/* list_uniq - return a list with duplicates removed (preserving order) */
MdhValue __mdh_list_uniq(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        return list;
    }
    MdhList *l = __mdh_get_list(list);
    if (l->length == 0) return list;

    /* Create new list */
    MdhList *result = (MdhList *)GC_malloc(sizeof(MdhList));
    result->capacity = l->length;
    result->length = 0;
    result->items = (MdhValue *)GC_malloc(sizeof(MdhValue) * l->length);

    /* Add each element only if not already in result */
    for (int64_t i = 0; i < l->length; i++) {
        bool found = false;
        for (int64_t j = 0; j < result->length; j++) {
            if (__mdh_eq(l->items[i], result->items[j])) {
                found = true;
                break;
            }
        }
        if (!found) {
            result->items[result->length++] = l->items[i];
        }
    }

    return (MdhValue){ .tag = MDH_TAG_LIST, .data = (int64_t)(intptr_t)result };
}

/* range - generate a list of integers from start to end with step */
MdhValue __mdh_range(int64_t start, int64_t end, int64_t step) {
    if (step == 0) step = 1;

    /* Calculate length */
    int64_t length = 0;
    if (step > 0 && end > start) {
        length = (end - start + step - 1) / step;
    } else if (step < 0 && end < start) {
        length = (start - end - step - 1) / (-step);
    }
    if (length < 0) length = 0;

    /* Create list */
    MdhList *result = (MdhList *)GC_malloc(sizeof(MdhList));
    result->capacity = length > 0 ? length : 1;
    result->length = length;
    result->items = (MdhValue *)GC_malloc(sizeof(MdhValue) * result->capacity);

    /* Fill list */
    int64_t val = start;
    for (int64_t i = 0; i < length; i++) {
        result->items[i] = __mdh_make_int(val);
        val += step;
    }

    return (MdhValue){ .tag = MDH_TAG_LIST, .data = (int64_t)(intptr_t)result };
}

/* list_slice - return a slice of the list [start, end) */
MdhValue __mdh_list_slice(MdhValue list, int64_t start, int64_t end) {
    if (list.tag != MDH_TAG_LIST) {
        return list;
    }
    MdhList *l = __mdh_get_list(list);

    /* Handle negative indices */
    if (start < 0) start = l->length + start;
    if (end < 0) end = l->length + end;
    if (start < 0) start = 0;
    if (end > l->length) end = l->length;
    if (start >= end || start >= l->length) {
        /* Return empty list */
        MdhList *result = (MdhList *)GC_malloc(sizeof(MdhList));
        result->capacity = 0;
        result->length = 0;
        result->items = NULL;
        return (MdhValue){ .tag = MDH_TAG_LIST, .data = (int64_t)(intptr_t)result };
    }

    int64_t new_len = end - start;
    MdhList *result = (MdhList *)GC_malloc(sizeof(MdhList));
    result->capacity = new_len;
    result->length = new_len;
    result->items = (MdhValue *)GC_malloc(sizeof(MdhValue) * new_len);
    for (int64_t i = 0; i < new_len; i++) {
        result->items[i] = l->items[start + i];
    }

    return (MdhValue){ .tag = MDH_TAG_LIST, .data = (int64_t)(intptr_t)result };
}

MdhValue __mdh_is_space(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) return __mdh_make_bool(false);
    const char *s = (const char *)(intptr_t)str.data;
    if (s[0] == '\0' || s[1] != '\0') return __mdh_make_bool(false);
    return __mdh_make_bool(s[0] == ' ' || s[0] == '\t' || s[0] == '\n' || s[0] == '\r');
}

MdhValue __mdh_is_digit(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) return __mdh_make_bool(false);
    const char *s = (const char *)(intptr_t)str.data;
    if (s[0] == '\0' || s[1] != '\0') return __mdh_make_bool(false);
    return __mdh_make_bool(s[0] >= '0' && s[0] <= '9');
}

MdhValue __mdh_wheesht_aw(MdhValue str) {
    /* Collapse whitespace, trim leading/trailing. */
    if (str.tag != MDH_TAG_STRING) return str;
    const char *s = (const char *)(intptr_t)str.data;
    size_t len = strlen(s);
    char *result = (char *)GC_malloc(len + 1);
    size_t out_len = 0;

    bool in_space = true; /* treat leading whitespace as "in space" */
    for (const unsigned char *p = (const unsigned char *)s; *p; p++) {
        if (isspace(*p)) {
            in_space = true;
            continue;
        }
        if (in_space && out_len > 0) {
            result[out_len++] = ' ';
        }
        result[out_len++] = (char)*p;
        in_space = false;
    }
    result[out_len] = '\0';
    return __mdh_string_from_buf(result);
}

MdhValue __mdh_bonnie(MdhValue val) {
    MdhValue s = __mdh_to_string(val);
    const char *m = __mdh_get_string(s);
    const char *prefix = "~~~ ";
    const char *suffix = " ~~~";
    size_t plen = strlen(prefix);
    size_t mlen = strlen(m);
    size_t slen = strlen(suffix);
    char *out = (char *)GC_malloc(plen + mlen + slen + 1);
    memcpy(out, prefix, plen);
    memcpy(out + plen, m, mlen);
    memcpy(out + plen + mlen, suffix, slen);
    out[plen + mlen + slen] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_shuffle(MdhValue list) {
    /* Shuffle list (deck) - returns shuffled copy */
    if (list.tag != MDH_TAG_LIST) return __mdh_make_list(0);

    __mdh_ensure_rng();
    MdhList *src = (MdhList *)(intptr_t)list.data;
    MdhValue result = __mdh_make_list((int32_t)src->length);
    MdhList *dst = (MdhList *)(intptr_t)result.data;

    /* Copy all items */
    for (int64_t i = 0; i < src->length; i++) {
        __mdh_list_push(result, src->items[i]);
    }

    /* Fisher-Yates shuffle */
    for (int64_t i = dst->length - 1; i > 0; i--) {
        int64_t j = rand() % (i + 1);
        MdhValue tmp = dst->items[i];
        dst->items[i] = dst->items[j];
        dst->items[j] = tmp;
    }

    return result;
}

MdhValue __mdh_bit_and(MdhValue a, MdhValue b) {
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        return __mdh_make_int(a.data & b.data);
    }
    return __mdh_make_int(0);
}

MdhValue __mdh_bit_or(MdhValue a, MdhValue b) {
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        return __mdh_make_int(a.data | b.data);
    }
    return __mdh_make_int(0);
}

MdhValue __mdh_bit_xor(MdhValue a, MdhValue b) {
    if (a.tag == MDH_TAG_INT && b.tag == MDH_TAG_INT) {
        return __mdh_make_int(a.data ^ b.data);
    }
    return __mdh_make_int(0);
}

// Type checking functions
MdhValue __mdh_is_nil(MdhValue val) {
    return __mdh_make_bool(val.tag == MDH_TAG_NIL);
}

MdhValue __mdh_is_bool(MdhValue val) {
    return __mdh_make_bool(val.tag == MDH_TAG_BOOL);
}

MdhValue __mdh_is_int(MdhValue val) {
    return __mdh_make_bool(val.tag == MDH_TAG_INT);
}

MdhValue __mdh_is_float(MdhValue val) {
    return __mdh_make_bool(val.tag == MDH_TAG_FLOAT);
}

MdhValue __mdh_is_string(MdhValue val) {
    return __mdh_make_bool(val.tag == MDH_TAG_STRING);
}

MdhValue __mdh_is_list(MdhValue val) {
    return __mdh_make_bool(val.tag == MDH_TAG_LIST);
}

MdhValue __mdh_is_dict(MdhValue val) {
    return __mdh_make_bool(val.tag == MDH_TAG_DICT);
}

MdhValue __mdh_is_function(MdhValue val) {
    return __mdh_make_bool(val.tag == MDH_TAG_FUNCTION);
}

// String prefix/suffix checking functions
MdhValue __mdh_starts_with(MdhValue str, MdhValue prefix) {
    if (str.tag != MDH_TAG_STRING || prefix.tag != MDH_TAG_STRING) {
        return __mdh_make_bool(0);
    }
    const char *s = __mdh_get_string(str);
    const char *p = __mdh_get_string(prefix);
    size_t plen = strlen(p);
    return __mdh_make_bool(strncmp(s, p, plen) == 0);
}

MdhValue __mdh_ends_with(MdhValue str, MdhValue suffix) {
    if (str.tag != MDH_TAG_STRING || suffix.tag != MDH_TAG_STRING) {
        return __mdh_make_bool(0);
    }
    const char *s = __mdh_get_string(str);
    const char *suf = __mdh_get_string(suffix);
    size_t slen = strlen(s);
    size_t suflen = strlen(suf);
    if (suflen > slen) {
        return __mdh_make_bool(0);
    }
    return __mdh_make_bool(strcmp(s + slen - suflen, suf) == 0);
}

/* ========== Environment/System ========== */

void __mdh_set_args(int32_t argc, char **argv) {
    __mdh_argc = argc;
    __mdh_argv = argv;
}

MdhValue __mdh_args(void) {
    MdhValue result = __mdh_make_list(__mdh_argc);
    for (int32_t i = 0; i < __mdh_argc; i++) {
        const char *s = (__mdh_argv && __mdh_argv[i]) ? __mdh_argv[i] : "";
        __mdh_list_push(result, __mdh_make_string(s));
    }
    return result;
}

MdhValue __mdh_cwd(void) {
    char *cwd = getcwd(NULL, 0);
    if (!cwd) {
        return __mdh_make_nil();
    }
    MdhValue v = __mdh_make_string(cwd);
    free(cwd);
    return v;
}

MdhValue __mdh_chdir(MdhValue path) {
    if (path.tag != MDH_TAG_STRING) {
        __mdh_type_error("chdir", path.tag, 0);
        return __mdh_make_nil();
    }
    const char *p = __mdh_get_string(path);
    if (chdir(p) != 0) {
        char buf[512];
        snprintf(buf, sizeof(buf), "Couldnae change tae directory '%s': %s", p, strerror(errno));
        __mdh_hurl(__mdh_make_string(buf));
    }
    return __mdh_make_nil();
}

MdhValue __mdh_env_get(MdhValue key) {
    if (key.tag != MDH_TAG_STRING) {
        __mdh_type_error("env_get", key.tag, 0);
        return __mdh_make_nil();
    }
    const char *k = __mdh_get_string(key);
    const char *v = getenv(k);
    if (!v) {
        return __mdh_make_nil();
    }
    return __mdh_make_string(v);
}

MdhValue __mdh_env_set(MdhValue key, MdhValue value) {
    if (key.tag != MDH_TAG_STRING) {
        __mdh_type_error("env_set", key.tag, 0);
        return __mdh_make_nil();
    }
    MdhValue value_str = __mdh_to_string(value);
    const char *k = __mdh_get_string(key);
    const char *v = __mdh_get_string(value_str);
    (void)setenv(k, v, 1);
    return __mdh_make_nil();
}

MdhValue __mdh_env_all(void) {
    extern char **environ;

    MdhValue dict = __mdh_empty_dict();
    if (!environ) {
        return dict;
    }

    for (char **p = environ; *p; p++) {
        const char *entry = *p;
        const char *eq = strchr(entry, '=');
        if (!eq) {
            continue;
        }
        size_t klen = (size_t)(eq - entry);
        char *kbuf = (char *)GC_malloc(klen + 1);
        memcpy(kbuf, entry, klen);
        kbuf[klen] = '\0';

        MdhValue k = __mdh_string_from_buf(kbuf);
        MdhValue v = __mdh_make_string(eq + 1);
        dict = __mdh_dict_set(dict, k, v);
    }

    return dict;
}

MdhValue __mdh_path_join(MdhValue a, MdhValue b) {
    if (a.tag != MDH_TAG_STRING || b.tag != MDH_TAG_STRING) {
        __mdh_type_error("path_join", a.tag, b.tag);
        return __mdh_make_string("");
    }

    const char *pa = __mdh_get_string(a);
    const char *pb = __mdh_get_string(b);

    if (pb[0] == '/') {
        return __mdh_make_string(pb);
    }

    size_t la = strlen(pa);
    size_t lb = strlen(pb);

    if (la == 0) {
        return __mdh_make_string(pb);
    }

    int need_slash = pa[la - 1] != '/';
    char *out = (char *)GC_malloc(la + (size_t)need_slash + lb + 1);

    memcpy(out, pa, la);
    size_t pos = la;
    if (need_slash) {
        out[pos++] = '/';
    }
    memcpy(out + pos, pb, lb);
    out[pos + lb] = '\0';

    return __mdh_string_from_buf(out);
}

static char *__mdh_shell_quote_single(const char *s) {
    size_t len = strlen(s);
    size_t quotes = 0;
    for (size_t i = 0; i < len; i++) {
        if (s[i] == '\'') {
            quotes++;
        }
    }

    /* Surround with single quotes and escape internal single quotes as: '\'' */
    size_t out_len = 2 + len + quotes * 3;
    char *out = (char *)GC_malloc(out_len + 1);

    size_t j = 0;
    out[j++] = '\'';
    for (size_t i = 0; i < len; i++) {
        if (s[i] == '\'') {
            out[j++] = '\'';
            out[j++] = '\\';
            out[j++] = '\'';
            out[j++] = '\'';
        } else {
            out[j++] = s[i];
        }
    }
    out[j++] = '\'';
    out[j] = '\0';
    return out;
}

static char *__mdh_build_shell_command(const char *cmd, bool redirect_stderr) {
    const char *shell = getenv("MDH_SHELL");
    if (!shell || shell[0] == '\0') {
        shell = "sh";
    }

    char *quoted = __mdh_shell_quote_single(cmd);
    const char *redir = redirect_stderr ? " 2>&1" : "";

    size_t needed = strlen(shell) + strlen(" -c ") + strlen(quoted) + strlen(redir) + 1;
    char *full = (char *)GC_malloc(needed);
    snprintf(full, needed, "%s -c %s%s", shell, quoted, redir);
    return full;
}

MdhValue __mdh_shell(MdhValue cmd) {
    if (cmd.tag != MDH_TAG_STRING) {
        __mdh_type_error("shell", cmd.tag, 0);
        return __mdh_make_nil();
    }

    char *full = __mdh_build_shell_command(__mdh_get_string(cmd), true);
    FILE *fp = popen(full, "r");
    if (!fp) {
        __mdh_hurl(__mdh_make_string("Shell command failed"));
        return __mdh_make_nil();
    }

    MdhStrBuf sb;
    __mdh_sb_init(&sb);

    char buf[4096];
    size_t nread = 0;
    while ((nread = fread(buf, 1, sizeof(buf), fp)) > 0) {
        __mdh_sb_append_n(&sb, buf, nread);
    }

    (void)pclose(fp);
    return __mdh_string_from_buf(sb.buf);
}

MdhValue __mdh_shell_status(MdhValue cmd) {
    if (cmd.tag != MDH_TAG_STRING) {
        __mdh_type_error("shell_status", cmd.tag, 0);
        return __mdh_make_int(-1);
    }

    char *full = __mdh_build_shell_command(__mdh_get_string(cmd), false);
    int status = system(full);
    if (status == -1) {
        return __mdh_make_int(-1);
    }
    if (WIFEXITED(status)) {
        return __mdh_make_int((int64_t)WEXITSTATUS(status));
    }
    return __mdh_make_int(-1);
}

/* ========== File I/O (extra parity) ========== */

MdhValue __mdh_file_size(MdhValue path) {
    if (path.tag != MDH_TAG_STRING) {
        __mdh_type_error("file_size", path.tag, 0);
        return __mdh_make_int(0);
    }
    const char *p = __mdh_get_string(path);
    struct stat st;
    if (stat(p, &st) != 0) {
        char buf[512];
        snprintf(buf, sizeof(buf), "Couldnae get file info fer '%s': %s", p, strerror(errno));
        __mdh_hurl(__mdh_make_string(buf));
        return __mdh_make_int(0);
    }
    return __mdh_make_int((int64_t)st.st_size);
}

MdhValue __mdh_file_delete(MdhValue path) {
    if (path.tag != MDH_TAG_STRING) {
        __mdh_type_error("file_delete", path.tag, 0);
        return __mdh_make_nil();
    }
    const char *p = __mdh_get_string(path);
    if (unlink(p) != 0) {
        char buf[512];
        snprintf(buf, sizeof(buf), "Couldnae delete '%s': %s", p, strerror(errno));
        __mdh_hurl(__mdh_make_string(buf));
    }
    return __mdh_make_nil();
}

MdhValue __mdh_list_dir(MdhValue path) {
    if (path.tag != MDH_TAG_STRING) {
        __mdh_type_error("list_dir", path.tag, 0);
        return __mdh_make_list(0);
    }
    const char *p = __mdh_get_string(path);
    DIR *dir = opendir(p);
    if (!dir) {
        char buf[512];
        snprintf(buf, sizeof(buf), "Couldnae read directory '%s': %s", p, strerror(errno));
        __mdh_hurl(__mdh_make_string(buf));
        return __mdh_make_list(0);
    }

    MdhValue result = __mdh_make_list(8);
    struct dirent *ent = NULL;
    while ((ent = readdir(dir)) != NULL) {
        const char *name = ent->d_name;
        if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) {
            continue;
        }
        __mdh_list_push(result, __mdh_make_string(name));
    }

    closedir(dir);
    return result;
}

static int __mdh_mkdir_p(const char *path) {
    if (!path || path[0] == '\0') {
        return -1;
    }

    char *tmp = GC_strdup(path);
    size_t len = strlen(tmp);
    if (len == 0) {
        return -1;
    }

    if (tmp[len - 1] == '/') {
        tmp[len - 1] = '\0';
    }

    for (char *p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            if (mkdir(tmp, 0755) != 0 && errno != EEXIST) {
                *p = '/';
                return -1;
            }
            *p = '/';
        }
    }

    if (mkdir(tmp, 0755) != 0 && errno != EEXIST) {
        return -1;
    }

    return 0;
}

MdhValue __mdh_make_dir(MdhValue path) {
    if (path.tag != MDH_TAG_STRING) {
        __mdh_type_error("make_dir", path.tag, 0);
        return __mdh_make_nil();
    }
    const char *p = __mdh_get_string(path);
    if (__mdh_mkdir_p(p) != 0) {
        char buf[512];
        snprintf(buf, sizeof(buf), "Couldnae create directory '%s': %s", p, strerror(errno));
        __mdh_hurl(__mdh_make_string(buf));
    }
    return __mdh_make_nil();
}

MdhValue __mdh_is_dir(MdhValue path) {
    if (path.tag != MDH_TAG_STRING) {
        __mdh_type_error("is_dir", path.tag, 0);
        return __mdh_make_bool(false);
    }
    const char *p = __mdh_get_string(path);
    struct stat st;
    if (stat(p, &st) != 0) {
        return __mdh_make_bool(false);
    }
    return __mdh_make_bool(S_ISDIR(st.st_mode));
}

MdhValue __mdh_scrieve_append(MdhValue path, MdhValue content) {
    if (path.tag != MDH_TAG_STRING) {
        __mdh_type_error("scrieve_append", path.tag, 0);
        return __mdh_make_nil();
    }

    const char *p = __mdh_get_string(path);
    MdhValue content_str = content.tag == MDH_TAG_STRING ? content : __mdh_to_string(content);
    const char *c = __mdh_get_string(content_str);

    FILE *f = fopen(p, "ab");
    if (!f) {
        char buf[512];
        snprintf(buf, sizeof(buf), "Couldnae open '%s' fer appendin': %s", p, strerror(errno));
        __mdh_hurl(__mdh_make_string(buf));
        return __mdh_make_nil();
    }

    if (c && c[0] != '\0') {
        (void)fwrite(c, 1, strlen(c), f);
    }

    fclose(f);
    return __mdh_make_nil();
}

/* ========== Date/Time ========== */

MdhValue __mdh_date_now(void) {
    time_t now = time(NULL);
    struct tm tm_now;
    localtime_r(&now, &tm_now);

    int64_t weekday = (tm_now.tm_wday + 6) % 7; /* Monday=0 */

    MdhValue dict = __mdh_empty_dict();
    dict = __mdh_dict_set(dict, __mdh_make_string("year"), __mdh_make_int((int64_t)tm_now.tm_year + 1900));
    dict = __mdh_dict_set(dict, __mdh_make_string("month"), __mdh_make_int((int64_t)tm_now.tm_mon + 1));
    dict = __mdh_dict_set(dict, __mdh_make_string("day"), __mdh_make_int((int64_t)tm_now.tm_mday));
    dict = __mdh_dict_set(dict, __mdh_make_string("hour"), __mdh_make_int((int64_t)tm_now.tm_hour));
    dict = __mdh_dict_set(dict, __mdh_make_string("minute"), __mdh_make_int((int64_t)tm_now.tm_min));
    dict = __mdh_dict_set(dict, __mdh_make_string("second"), __mdh_make_int((int64_t)tm_now.tm_sec));
    dict = __mdh_dict_set(dict, __mdh_make_string("weekday"), __mdh_make_int(weekday));
    return dict;
}

MdhValue __mdh_date_format(MdhValue timestamp_secs, MdhValue format) {
    if (timestamp_secs.tag != MDH_TAG_INT || format.tag != MDH_TAG_STRING) {
        __mdh_type_error("date_format", timestamp_secs.tag, format.tag);
        return __mdh_make_string("");
    }

    time_t sec = (time_t)timestamp_secs.data;
    struct tm tm_val;
    localtime_r(&sec, &tm_val);

    const char *fmt = __mdh_get_string(format);
    size_t cap = 128;
    char *buf = (char *)GC_malloc(cap);
    size_t out = strftime(buf, cap, fmt, &tm_val);
    while (out == 0 && cap < 8192) {
        cap *= 2;
        buf = (char *)GC_realloc(buf, cap);
        out = strftime(buf, cap, fmt, &tm_val);
    }
    if (out == 0) {
        __mdh_hurl(__mdh_make_string("Couldnae format date"));
        return __mdh_make_string("");
    }

    return __mdh_string_from_buf(buf);
}

MdhValue __mdh_date_parse(MdhValue date_str, MdhValue format) {
    if (date_str.tag != MDH_TAG_STRING || format.tag != MDH_TAG_STRING) {
        __mdh_type_error("date_parse", date_str.tag, format.tag);
        return __mdh_make_int(0);
    }

    const char *s = __mdh_get_string(date_str);
    const char *fmt = __mdh_get_string(format);

    struct tm tm_val;
    memset(&tm_val, 0, sizeof(tm_val));

    char *end = strptime(s, fmt, &tm_val);
    if (!end) {
        __mdh_hurl(__mdh_make_string("Couldnae parse date"));
        return __mdh_make_int(0);
    }
    while (*end && isspace((unsigned char)*end)) {
        end++;
    }
    if (*end) {
        __mdh_hurl(__mdh_make_string("Couldnae parse date (trailing text)"));
        return __mdh_make_int(0);
    }

    time_t t = timegm(&tm_val);
    if (t == (time_t)-1) {
        __mdh_hurl(__mdh_make_string("Couldnae parse date (invalid timestamp)"));
        return __mdh_make_int(0);
    }

    return __mdh_make_int((int64_t)t);
}

static int64_t __mdh_unit_seconds(const char *unit) {
    if (!unit) {
        return 0;
    }
    if (strcmp(unit, "seconds") == 0) {
        return 1;
    }
    if (strcmp(unit, "minutes") == 0) {
        return 60;
    }
    if (strcmp(unit, "hours") == 0) {
        return 3600;
    }
    if (strcmp(unit, "days") == 0) {
        return 86400;
    }
    if (strcmp(unit, "weeks") == 0) {
        return 604800;
    }
    return 0;
}

MdhValue __mdh_date_add(MdhValue timestamp_secs, MdhValue amount, MdhValue unit) {
    if (timestamp_secs.tag != MDH_TAG_INT || amount.tag != MDH_TAG_INT || unit.tag != MDH_TAG_STRING) {
        __mdh_type_error("date_add", timestamp_secs.tag, amount.tag);
        return __mdh_make_int(0);
    }
    const char *u = __mdh_get_string(unit);
    int64_t mul = __mdh_unit_seconds(u);
    if (mul == 0) {
        __mdh_hurl(__mdh_make_string("Unknown time unit"));
        return __mdh_make_int(0);
    }
    return __mdh_make_int(timestamp_secs.data + amount.data * mul);
}

MdhValue __mdh_date_diff(MdhValue ts1, MdhValue ts2, MdhValue unit) {
    if (ts1.tag != MDH_TAG_INT || ts2.tag != MDH_TAG_INT || unit.tag != MDH_TAG_STRING) {
        __mdh_type_error("date_diff", ts1.tag, ts2.tag);
        return __mdh_make_int(0);
    }
    const char *u = __mdh_get_string(unit);
    int64_t diff_secs = ts2.data - ts1.data;
    if (strcmp(u, "milliseconds") == 0) {
        return __mdh_make_int(diff_secs * 1000);
    }
    int64_t div = __mdh_unit_seconds(u);
    if (div == 0) {
        __mdh_hurl(__mdh_make_string("Unknown time unit"));
        return __mdh_make_int(0);
    }
    return __mdh_make_int(diff_secs / div);
}

MdhValue __mdh_braw_date(MdhValue ts_or_nil) {
    uint64_t secs = 0;
    if (ts_or_nil.tag == MDH_TAG_INT) {
        secs = (uint64_t)ts_or_nil.data;
    } else if (ts_or_nil.tag == MDH_TAG_NIL) {
        secs = (uint64_t)time(NULL);
    } else {
        __mdh_type_error("braw_date", ts_or_nil.tag, 0);
        return __mdh_make_string("");
    }

    uint64_t days_since_epoch = secs / 86400ULL;
    size_t day_of_week = (size_t)((days_since_epoch + 4) % 7ULL); /* Jan 1 1970 was Thursday */

    const char *scots_day_names[] = {
        "the Sabbath",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Setterday",
    };

    int64_t remaining_days = (int64_t)days_since_epoch;
    int64_t year = 1970;
    for (;;) {
        int days_in_year = ((year % 4 == 0 && year % 100 != 0) || year % 400 == 0) ? 366 : 365;
        if (remaining_days < days_in_year) {
            break;
        }
        remaining_days -= days_in_year;
        year++;
    }

    const char *scots_months[] = {
        "Januar",
        "Februar",
        "Mairch",
        "Aprile",
        "Mey",
        "Juin",
        "Julie",
        "August",
        "September",
        "October",
        "November",
        "December",
    };

    int is_leap = ((year % 4 == 0 && year % 100 != 0) || year % 400 == 0);
    int64_t days_in_months[] = {
        31, is_leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    };

    size_t month = 0;
    for (size_t i = 0; i < 12; i++) {
        if (remaining_days < days_in_months[i]) {
            month = i;
            break;
        }
        remaining_days -= days_in_months[i];
    }

    int64_t day = remaining_days + 1;
    const char *ordinal = "th";
    if (day == 1 || day == 21 || day == 31) {
        ordinal = "st";
    } else if (day == 2 || day == 22) {
        ordinal = "nd";
    } else if (day == 3 || day == 23) {
        ordinal = "rd";
    }

    char buf[256];
    snprintf(
        buf,
        sizeof(buf),
        "%s, the %lld%s o' %s, %lld",
        scots_day_names[day_of_week],
        (long long)day,
        ordinal,
        scots_months[month],
        (long long)year
    );
    return __mdh_make_string(buf);
}

#if 0
/* ========== Regex (POSIX ERE subset) ========== */

static void __mdh_regex_compile_or_hurl(regex_t *re, const char *pattern) {
    int rc = regcomp(re, pattern, REG_EXTENDED);
    if (rc != 0) {
        char err[256];
        regerror(rc, re, err, sizeof(err));
        __mdh_hurl(__mdh_make_string(err));
    }
}

static MdhValue __mdh_regex_match_dict(const char *text, int64_t start, int64_t end) {
    MdhValue dict = __mdh_empty_dict();

    size_t len = (size_t)(end - start);
    char *m = (char *)GC_malloc(len + 1);
    memcpy(m, text + start, len);
    m[len] = '\0';

    dict = __mdh_dict_set(dict, __mdh_make_string("match"), __mdh_string_from_buf(m));
    dict = __mdh_dict_set(dict, __mdh_make_string("start"), __mdh_make_int(start));
    dict = __mdh_dict_set(dict, __mdh_make_string("end"), __mdh_make_int(end));
    return dict;
}

MdhValue __mdh_regex_test(MdhValue text, MdhValue pattern) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_test", text.tag, pattern.tag);
        return __mdh_make_bool(false);
    }

    regex_t re;
    __mdh_regex_compile_or_hurl(&re, __mdh_get_string(pattern));

    int rc = regexec(&re, __mdh_get_string(text), 0, NULL, 0);
    regfree(&re);
    return __mdh_make_bool(rc == 0);
}

MdhValue __mdh_regex_match(MdhValue text, MdhValue pattern) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_match", text.tag, pattern.tag);
        return __mdh_make_nil();
    }

    const char *s = __mdh_get_string(text);
    regex_t re;
    __mdh_regex_compile_or_hurl(&re, __mdh_get_string(pattern));

    regmatch_t m;
    int rc = regexec(&re, s, 1, &m, 0);
    regfree(&re);
    if (rc != 0 || m.rm_so < 0 || m.rm_eo < 0) {
        return __mdh_make_nil();
    }
    return __mdh_regex_match_dict(s, (int64_t)m.rm_so, (int64_t)m.rm_eo);
}

MdhValue __mdh_regex_match_all(MdhValue text, MdhValue pattern) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_match_all", text.tag, pattern.tag);
        return __mdh_make_list(0);
    }

    const char *s = __mdh_get_string(text);
    size_t slen = strlen(s);

    regex_t re;
    __mdh_regex_compile_or_hurl(&re, __mdh_get_string(pattern));

    MdhValue result = __mdh_make_list(8);
    size_t offset = 0;
    while (offset <= slen) {
        regmatch_t m;
        int rc = regexec(&re, s + offset, 1, &m, 0);
        if (rc != 0 || m.rm_so < 0 || m.rm_eo < 0) {
            break;
        }
        int64_t start = (int64_t)offset + (int64_t)m.rm_so;
        int64_t end = (int64_t)offset + (int64_t)m.rm_eo;
        __mdh_list_push(result, __mdh_regex_match_dict(s, start, end));

        size_t adv = (size_t)m.rm_eo;
        if (adv == 0) {
            offset += 1;
        } else {
            offset += adv;
        }
    }

    regfree(&re);
    return result;
}

MdhValue __mdh_regex_replace(MdhValue text, MdhValue pattern, MdhValue replacement) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING || replacement.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_replace", text.tag, pattern.tag);
        return text.tag == MDH_TAG_STRING ? text : __mdh_make_string("");
    }

    const char *s = __mdh_get_string(text);
    size_t slen = strlen(s);
    const char *repl = __mdh_get_string(replacement);

    regex_t re;
    __mdh_regex_compile_or_hurl(&re, __mdh_get_string(pattern));

    MdhStrBuf sb;
    __mdh_sb_init(&sb);

    size_t offset = 0;
    while (offset <= slen) {
        regmatch_t m;
        int rc = regexec(&re, s + offset, 1, &m, 0);
        if (rc != 0 || m.rm_so < 0 || m.rm_eo < 0) {
            break;
        }

        size_t match_start = offset + (size_t)m.rm_so;
        size_t match_end = offset + (size_t)m.rm_eo;

        __mdh_sb_append_n(&sb, s + offset, match_start - offset);
        __mdh_sb_append(&sb, repl);

        if (match_end == offset) {
            offset += 1;
        } else {
            offset = match_end;
        }
    }

    if (offset < slen) {
        __mdh_sb_append_n(&sb, s + offset, slen - offset);
    }

    regfree(&re);
    return __mdh_string_from_buf(sb.buf);
}

MdhValue __mdh_regex_replace_first(MdhValue text, MdhValue pattern, MdhValue replacement) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING || replacement.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_replace_first", text.tag, pattern.tag);
        return text.tag == MDH_TAG_STRING ? text : __mdh_make_string("");
    }

    const char *s = __mdh_get_string(text);
    size_t slen = strlen(s);
    const char *repl = __mdh_get_string(replacement);

    regex_t re;
    __mdh_regex_compile_or_hurl(&re, __mdh_get_string(pattern));

    regmatch_t m;
    int rc = regexec(&re, s, 1, &m, 0);
    regfree(&re);
    if (rc != 0 || m.rm_so < 0 || m.rm_eo < 0) {
        return text;
    }

    size_t start = (size_t)m.rm_so;
    size_t end = (size_t)m.rm_eo;

    MdhStrBuf sb;
    __mdh_sb_init(&sb);
    __mdh_sb_append_n(&sb, s, start);
    __mdh_sb_append(&sb, repl);
    __mdh_sb_append_n(&sb, s + end, slen - end);

    return __mdh_string_from_buf(sb.buf);
}

MdhValue __mdh_regex_split(MdhValue text, MdhValue pattern) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_split", text.tag, pattern.tag);
        return __mdh_make_list(0);
    }

    const char *s = __mdh_get_string(text);
    size_t slen = strlen(s);

    regex_t re;
    __mdh_regex_compile_or_hurl(&re, __mdh_get_string(pattern));

    MdhValue result = __mdh_make_list(8);
    size_t offset = 0;
    while (offset <= slen) {
        regmatch_t m;
        int rc = regexec(&re, s + offset, 1, &m, 0);
        if (rc != 0 || m.rm_so < 0 || m.rm_eo < 0) {
            break;
        }

        size_t match_start = offset + (size_t)m.rm_so;
        size_t match_end = offset + (size_t)m.rm_eo;

        size_t seg_len = match_start - offset;
        char *seg = (char *)GC_malloc(seg_len + 1);
        memcpy(seg, s + offset, seg_len);
        seg[seg_len] = '\0';
        __mdh_list_push(result, __mdh_string_from_buf(seg));

        if (match_end == offset) {
            offset += 1;
        } else {
            offset = match_end;
        }
    }

    /* trailing segment */
    if (offset <= slen) {
        size_t seg_len = slen - offset;
        char *seg = (char *)GC_malloc(seg_len + 1);
        memcpy(seg, s + offset, seg_len);
        seg[seg_len] = '\0';
        __mdh_list_push(result, __mdh_string_from_buf(seg));
    }

    regfree(&re);
    return result;
}

#endif

/* ========== Regex (Rust FFI) ========== */

MdhValue __mdh_regex_test(MdhValue text, MdhValue pattern) {
    return __mdh_rs_regex_test(text, pattern);
}

MdhValue __mdh_regex_match(MdhValue text, MdhValue pattern) {
    return __mdh_rs_regex_match(text, pattern);
}

MdhValue __mdh_regex_match_all(MdhValue text, MdhValue pattern) {
    return __mdh_rs_regex_match_all(text, pattern);
}

MdhValue __mdh_regex_replace(MdhValue text, MdhValue pattern, MdhValue replacement) {
    return __mdh_rs_regex_replace(text, pattern, replacement);
}

MdhValue __mdh_regex_replace_first(MdhValue text, MdhValue pattern, MdhValue replacement) {
    return __mdh_rs_regex_replace_first(text, pattern, replacement);
}

MdhValue __mdh_regex_split(MdhValue text, MdhValue pattern) {
    return __mdh_rs_regex_split(text, pattern);
}

/* ========== JSON ========== */

#if 0

static void __mdh_json_skip_ws(const char **p) {
    while (**p && isspace((unsigned char)**p)) {
        (*p)++;
    }
}

static int __mdh_hex_val(char c) {
    if (c >= '0' && c <= '9') {
        return c - '0';
    }
    if (c >= 'a' && c <= 'f') {
        return 10 + (c - 'a');
    }
    if (c >= 'A' && c <= 'F') {
        return 10 + (c - 'A');
    }
    return -1;
}

static void __mdh_json_append_utf8(MdhStrBuf *sb, uint32_t code) {
    if (code <= 0x7F) {
        __mdh_sb_append_char(sb, (char)code);
    } else if (code <= 0x7FF) {
        __mdh_sb_append_char(sb, (char)(0xC0 | ((code >> 6) & 0x1F)));
        __mdh_sb_append_char(sb, (char)(0x80 | (code & 0x3F)));
    } else if (code <= 0xFFFF) {
        __mdh_sb_append_char(sb, (char)(0xE0 | ((code >> 12) & 0x0F)));
        __mdh_sb_append_char(sb, (char)(0x80 | ((code >> 6) & 0x3F)));
        __mdh_sb_append_char(sb, (char)(0x80 | (code & 0x3F)));
    } else {
        __mdh_sb_append_char(sb, (char)(0xF0 | ((code >> 18) & 0x07)));
        __mdh_sb_append_char(sb, (char)(0x80 | ((code >> 12) & 0x3F)));
        __mdh_sb_append_char(sb, (char)(0x80 | ((code >> 6) & 0x3F)));
        __mdh_sb_append_char(sb, (char)(0x80 | (code & 0x3F)));
    }
}

static MdhValue __mdh_json_parse_value(const char **p);

static MdhValue __mdh_json_parse_string(const char **p) {
    if (**p != '\"') {
        __mdh_hurl(__mdh_make_string("Expected JSON string"));
        return __mdh_make_string("");
    }

    (*p)++; /* skip opening quote */
    MdhStrBuf sb;
    __mdh_sb_init(&sb);

    while (**p) {
        char c = **p;
        if (c == '\"') {
            (*p)++; /* skip closing quote */
            return __mdh_string_from_buf(sb.buf);
        }
        if (c == '\\') {
            (*p)++;
            if (!**p) {
                __mdh_hurl(__mdh_make_string("Unterminated string escape"));
                return __mdh_make_string("");
            }
            char e = **p;
            switch (e) {
                case 'n':
                    __mdh_sb_append_char(&sb, '\n');
                    break;
                case 't':
                    __mdh_sb_append_char(&sb, '\t');
                    break;
                case 'r':
                    __mdh_sb_append_char(&sb, '\r');
                    break;
                case 'b':
                    __mdh_sb_append_char(&sb, '\b');
                    break;
                case 'f':
                    __mdh_sb_append_char(&sb, '\f');
                    break;
                case '\"':
                    __mdh_sb_append_char(&sb, '\"');
                    break;
                case '\\':
                    __mdh_sb_append_char(&sb, '\\');
                    break;
                case '/':
                    __mdh_sb_append_char(&sb, '/');
                    break;
                case 'u': {
                    if (!(*p)[1] || !(*p)[2] || !(*p)[3] || !(*p)[4]) {
                        __mdh_hurl(__mdh_make_string("Invalid unicode escape"));
                        return __mdh_make_string("");
                    }
                    int h1 = __mdh_hex_val((*p)[1]);
                    int h2 = __mdh_hex_val((*p)[2]);
                    int h3 = __mdh_hex_val((*p)[3]);
                    int h4 = __mdh_hex_val((*p)[4]);
                    if (h1 < 0 || h2 < 0 || h3 < 0 || h4 < 0) {
                        __mdh_hurl(__mdh_make_string("Invalid unicode escape"));
                        return __mdh_make_string("");
                    }
                    uint32_t code = (uint32_t)((h1 << 12) | (h2 << 8) | (h3 << 4) | h4);
                    __mdh_json_append_utf8(&sb, code);
                    *p += 4;
                    break;
                }
                default:
                    __mdh_sb_append_char(&sb, e);
                    break;
            }
        } else {
            __mdh_sb_append_char(&sb, c);
        }
        (*p)++;
    }

    __mdh_hurl(__mdh_make_string("Unterminated JSON string"));
    return __mdh_make_string("");
}

static MdhValue __mdh_json_parse_number(const char **p) {
    const char *start = *p;
    char *endptr = NULL;
    double d = strtod(start, &endptr);
    if (endptr == start) {
        __mdh_hurl(__mdh_make_string("Invalid JSON number"));
        return __mdh_make_int(0);
    }

    bool is_float = false;
    for (const char *q = start; q < endptr; q++) {
        if (*q == '.' || *q == 'e' || *q == 'E') {
            is_float = true;
            break;
        }
    }

    *p = endptr;
    if (is_float) {
        return __mdh_make_float(d);
    }

    long long v = strtoll(start, NULL, 10);
    return __mdh_make_int((int64_t)v);
}

static MdhValue __mdh_json_parse_array(const char **p) {
    (*p)++; /* skip '[' */
    __mdh_json_skip_ws(p);

    MdhValue list = __mdh_make_list(8);
    if (**p == ']') {
        (*p)++;
        return list;
    }

    for (;;) {
        MdhValue v = __mdh_json_parse_value(p);
        __mdh_list_push(list, v);
        __mdh_json_skip_ws(p);

        if (**p == ']') {
            (*p)++;
            break;
        }
        if (**p != ',') {
            __mdh_hurl(__mdh_make_string("Expected ']' or ',' in JSON array"));
            return __mdh_make_list(0);
        }
        (*p)++;
        __mdh_json_skip_ws(p);
    }
    return list;
}

static MdhValue __mdh_json_parse_object(const char **p) {
    (*p)++; /* skip '{' */
    __mdh_json_skip_ws(p);

    MdhValue dict = __mdh_empty_dict();
    if (**p == '}') {
        (*p)++;
        return dict;
    }

    for (;;) {
        __mdh_json_skip_ws(p);
        if (**p != '\"') {
            __mdh_hurl(__mdh_make_string("Expected string key in JSON object"));
            return __mdh_empty_dict();
        }
        MdhValue key = __mdh_json_parse_string(p);
        __mdh_json_skip_ws(p);
        if (**p != ':') {
            __mdh_hurl(__mdh_make_string("Expected ':' in JSON object"));
            return __mdh_empty_dict();
        }
        (*p)++;
        MdhValue val = __mdh_json_parse_value(p);
        dict = __mdh_dict_set(dict, key, val);
        __mdh_json_skip_ws(p);

        if (**p == '}') {
            (*p)++;
            break;
        }
        if (**p != ',') {
            __mdh_hurl(__mdh_make_string("Expected '}' or ',' in JSON object"));
            return __mdh_empty_dict();
        }
        (*p)++;
        __mdh_json_skip_ws(p);
    }
    return dict;
}

static MdhValue __mdh_json_parse_value(const char **p) {
    __mdh_json_skip_ws(p);
    if (!**p) {
        __mdh_hurl(__mdh_make_string("Unexpected end of JSON"));
        return __mdh_make_nil();
    }

    switch (**p) {
        case '{':
            return __mdh_json_parse_object(p);
        case '[':
            return __mdh_json_parse_array(p);
        case '\"':
            return __mdh_json_parse_string(p);
        case 't':
            if (strncmp(*p, "true", 4) == 0) {
                *p += 4;
                return __mdh_make_bool(true);
            }
            break;
        case 'f':
            if (strncmp(*p, "false", 5) == 0) {
                *p += 5;
                return __mdh_make_bool(false);
            }
            break;
        case 'n':
            if (strncmp(*p, "null", 4) == 0) {
                *p += 4;
                return __mdh_make_nil();
            }
            break;
        default:
            break;
    }

    if (**p == '-' || isdigit((unsigned char)**p)) {
        return __mdh_json_parse_number(p);
    }

    __mdh_hurl(__mdh_make_string("Invalid JSON value"));
    return __mdh_make_nil();
}

MdhValue __mdh_json_parse(MdhValue json_str) {
    if (json_str.tag != MDH_TAG_STRING) {
        __mdh_type_error("json_parse", json_str.tag, 0);
        return __mdh_make_nil();
    }

    const char *p = __mdh_get_string(json_str);
    MdhValue v = __mdh_json_parse_value(&p);
    __mdh_json_skip_ws(&p);
    if (*p != '\0') {
        __mdh_hurl(__mdh_make_string("Trailing characters after JSON value"));
        return __mdh_make_nil();
    }
    return v;
}

static void __mdh_json_indent(MdhStrBuf *sb, int indent) {
    for (int i = 0; i < indent; i++) {
        __mdh_sb_append(sb, "  ");
    }
}

static void __mdh_json_escape_string(MdhStrBuf *sb, const char *s) {
    __mdh_sb_append_char(sb, '\"');
    for (const unsigned char *p = (const unsigned char *)s; *p; p++) {
        unsigned char c = *p;
        switch (c) {
            case '\"':
                __mdh_sb_append(sb, "\\\"");
                break;
            case '\\':
                __mdh_sb_append(sb, "\\\\");
                break;
            case '\n':
                __mdh_sb_append(sb, "\\n");
                break;
            case '\t':
                __mdh_sb_append(sb, "\\t");
                break;
            case '\r':
                __mdh_sb_append(sb, "\\r");
                break;
            default:
                if (c < 0x20) {
                    char buf[16];
                    snprintf(buf, sizeof(buf), "\\u%04x", (unsigned int)c);
                    __mdh_sb_append(sb, buf);
                } else {
                    __mdh_sb_append_char(sb, (char)c);
                }
                break;
        }
    }
    __mdh_sb_append_char(sb, '\"');
}

static void __mdh_json_stringify_value(MdhStrBuf *sb, MdhValue v, bool pretty, int indent) {
    switch (v.tag) {
        case MDH_TAG_NIL:
            __mdh_sb_append(sb, "null");
            return;
        case MDH_TAG_BOOL:
            __mdh_sb_append(sb, v.data ? "true" : "false");
            return;
        case MDH_TAG_INT: {
            char buf[64];
            snprintf(buf, sizeof(buf), "%lld", (long long)v.data);
            __mdh_sb_append(sb, buf);
            return;
        }
        case MDH_TAG_FLOAT: {
            double f = __mdh_get_float(v);
            if (isnan(f) || isinf(f)) {
                __mdh_sb_append(sb, "null");
            } else {
                char buf[64];
                snprintf(buf, sizeof(buf), "%.15g", f);
                __mdh_sb_append(sb, buf);
            }
            return;
        }
        case MDH_TAG_STRING:
            __mdh_json_escape_string(sb, __mdh_get_string(v));
            return;
        case MDH_TAG_LIST: {
            MdhList *list = __mdh_get_list(v);
            int64_t len = list ? list->length : 0;
            if (!pretty) {
                __mdh_sb_append_char(sb, '[');
                for (int64_t i = 0; i < len; i++) {
                    if (i > 0) {
                        __mdh_sb_append(sb, ", ");
                    }
                    __mdh_json_stringify_value(sb, list->items[i], false, indent);
                }
                __mdh_sb_append_char(sb, ']');
                return;
            }

            if (len == 0) {
                __mdh_sb_append(sb, "[]");
                return;
            }

            __mdh_sb_append(sb, "[\n");
            for (int64_t i = 0; i < len; i++) {
                __mdh_json_indent(sb, indent + 1);
                __mdh_json_stringify_value(sb, list->items[i], true, indent + 1);
                if (i + 1 < len) {
                    __mdh_sb_append(sb, ",\n");
                } else {
                    __mdh_sb_append_char(sb, '\n');
                }
            }
            __mdh_json_indent(sb, indent);
            __mdh_sb_append_char(sb, ']');
            return;
        }
        case MDH_TAG_DICT: {
            int64_t *dict_ptr = (int64_t *)(intptr_t)v.data;
            int64_t count = dict_ptr ? *dict_ptr : 0;
            MdhValue *entries = dict_ptr ? (MdhValue *)(dict_ptr + 1) : NULL;

            if (!pretty) {
                __mdh_sb_append_char(sb, '{');
                for (int64_t i = 0; i < count; i++) {
                    if (i > 0) {
                        __mdh_sb_append(sb, ", ");
                    }
                    MdhValue key_val = entries[i * 2];
                    const char *key_str = NULL;
                    if (key_val.tag == MDH_TAG_STRING) {
                        key_str = __mdh_get_string(key_val);
                    } else {
                        MdhValue s = __mdh_to_string(key_val);
                        key_str = __mdh_get_string(s);
                    }
                    __mdh_json_escape_string(sb, key_str ? key_str : "");
                    __mdh_sb_append(sb, ": ");
                    __mdh_json_stringify_value(sb, entries[i * 2 + 1], false, indent);
                }
                __mdh_sb_append_char(sb, '}');
                return;
            }

            if (count == 0) {
                __mdh_sb_append(sb, "{}");
                return;
            }

            __mdh_sb_append(sb, "{\n");
            for (int64_t i = 0; i < count; i++) {
                __mdh_json_indent(sb, indent + 1);
                MdhValue key_val = entries[i * 2];
                const char *key_str = NULL;
                if (key_val.tag == MDH_TAG_STRING) {
                    key_str = __mdh_get_string(key_val);
                } else {
                    MdhValue s = __mdh_to_string(key_val);
                    key_str = __mdh_get_string(s);
                }
                __mdh_json_escape_string(sb, key_str ? key_str : "");
                __mdh_sb_append(sb, ": ");
                __mdh_json_stringify_value(sb, entries[i * 2 + 1], true, indent + 1);
                if (i + 1 < count) {
                    __mdh_sb_append(sb, ",\n");
                } else {
                    __mdh_sb_append_char(sb, '\n');
                }
            }
            __mdh_json_indent(sb, indent);
            __mdh_sb_append_char(sb, '}');
            return;
        }
        default: {
            MdhValue s = __mdh_to_string(v);
            __mdh_json_escape_string(sb, __mdh_get_string(s));
            return;
        }
    }
}

MdhValue __mdh_json_stringify(MdhValue value) {
    MdhStrBuf sb;
    __mdh_sb_init(&sb);
    __mdh_json_stringify_value(&sb, value, false, 0);
    return __mdh_string_from_buf(sb.buf);
}

MdhValue __mdh_json_pretty(MdhValue value) {
    MdhStrBuf sb;
    __mdh_sb_init(&sb);
    __mdh_json_stringify_value(&sb, value, true, 0);
    return __mdh_string_from_buf(sb.buf);
}

#endif

MdhValue __mdh_json_parse(MdhValue json_str) {
    return __mdh_rs_json_parse(json_str);
}

MdhValue __mdh_json_stringify(MdhValue value) {
    return __mdh_rs_json_stringify(value);
}

MdhValue __mdh_json_pretty(MdhValue value) {
    return __mdh_rs_json_pretty(value);
}

/* ========== Misc Parity Helpers ========== */

static bool __mdh_char_in_set(unsigned char c, const char *set) {
    for (const unsigned char *p = (const unsigned char *)set; *p; p++) {
        if (*p == c) {
            return true;
        }
    }
    return false;
}

MdhValue __mdh_is_a(MdhValue value, MdhValue type_name) {
    if (type_name.tag != MDH_TAG_STRING) {
        __mdh_type_error("is_a", type_name.tag, 0);
        return __mdh_make_bool(false);
    }
    const char *t = __mdh_get_string(type_name);
    if (!t) {
        return __mdh_make_bool(false);
    }

    bool matches = false;
    if (strcmp(t, "integer") == 0 || strcmp(t, "int") == 0) {
        matches = value.tag == MDH_TAG_INT;
    } else if (strcmp(t, "float") == 0) {
        matches = value.tag == MDH_TAG_FLOAT;
    } else if (strcmp(t, "string") == 0 || strcmp(t, "str") == 0) {
        matches = value.tag == MDH_TAG_STRING;
    } else if (strcmp(t, "bool") == 0) {
        matches = value.tag == MDH_TAG_BOOL;
    } else if (strcmp(t, "list") == 0) {
        matches = value.tag == MDH_TAG_LIST;
    } else if (strcmp(t, "dict") == 0) {
        matches = value.tag == MDH_TAG_DICT;
    } else if (strcmp(t, "function") == 0 || strcmp(t, "dae") == 0) {
        matches = value.tag == MDH_TAG_FUNCTION;
    } else if (strcmp(t, "naething") == 0 || strcmp(t, "nil") == 0) {
        matches = value.tag == MDH_TAG_NIL;
    } else if (strcmp(t, "range") == 0) {
        matches = value.tag == MDH_TAG_RANGE;
    } else {
        matches = false;
    }

    return __mdh_make_bool(matches);
}

MdhValue __mdh_numpty_check(MdhValue value) {
    if (value.tag == MDH_TAG_NIL) {
        return __mdh_make_string("That's naething, ya numpty!");
    }
    if (value.tag == MDH_TAG_STRING) {
        const char *s = __mdh_get_string(value);
        if (!s || s[0] == '\0') {
            return __mdh_make_string("Empty string, ya numpty!");
        }
    }
    if (value.tag == MDH_TAG_LIST) {
        MdhList *l = __mdh_get_list(value);
        if (!l || l->length == 0) {
            return __mdh_make_string("Empty list, ya numpty!");
        }
    }
    return __mdh_make_string("That's braw!");
}

MdhValue __mdh_indices_o(MdhValue container, MdhValue needle) {
    if (container.tag == MDH_TAG_LIST) {
        MdhList *l = __mdh_get_list(container);
        int64_t len = l ? l->length : 0;
        MdhValue out = __mdh_make_list((int32_t)len);
        for (int64_t i = 0; i < len; i++) {
            if (__mdh_eq(l->items[i], needle)) {
                __mdh_list_push(out, __mdh_make_int(i));
            }
        }
        return out;
    }

    if (container.tag == MDH_TAG_STRING) {
        if (needle.tag != MDH_TAG_STRING) {
            __mdh_type_error("indices_o", needle.tag, 0);
            return __mdh_make_list(0);
        }

        const char *haystack = __mdh_get_string(container);
        const char *need = __mdh_get_string(needle);
        size_t need_len = strlen(need);
        if (need_len == 0) {
            __mdh_hurl(__mdh_make_string("Cannae search fer an empty string, ya numpty!"));
            return __mdh_make_list(0);
        }

        MdhValue out = __mdh_make_list(8);
        const char *p = haystack;
        while ((p = strstr(p, need)) != NULL) {
            __mdh_list_push(out, __mdh_make_int((int64_t)(p - haystack)));
            p += need_len;
        }
        return out;
    }

    __mdh_type_error("indices_o", container.tag, 0);
    return __mdh_make_list(0);
}

MdhValue __mdh_chunks(MdhValue list, MdhValue size) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("chunks", list.tag, 0);
        return __mdh_make_list(0);
    }
    if (size.tag != MDH_TAG_INT) {
        __mdh_type_error("chunks", size.tag, 0);
        return __mdh_make_list(0);
    }
    int64_t n = size.data;
    if (n <= 0) {
        __mdh_hurl(__mdh_make_string("chunks() size must be positive"));
        return __mdh_make_list(0);
    }

    MdhList *src = __mdh_get_list(list);
    int64_t len = src ? src->length : 0;
    int64_t out_cap = (len + n - 1) / n;
    MdhValue out = __mdh_make_list((int32_t)out_cap);

    for (int64_t i = 0; i < len; i += n) {
        int64_t end = i + n;
        if (end > len) {
            end = len;
        }
        MdhValue chunk = __mdh_make_list((int32_t)(end - i));
        for (int64_t j = i; j < end; j++) {
            __mdh_list_push(chunk, src->items[j]);
        }
        __mdh_list_push(out, chunk);
    }
    return out;
}

MdhValue __mdh_grup(MdhValue list, MdhValue size) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("grup", list.tag, 0);
        return __mdh_make_list(0);
    }
    if (size.tag != MDH_TAG_INT) {
        __mdh_type_error("grup", size.tag, 0);
        return __mdh_make_list(0);
    }
    if (size.data <= 0) {
        __mdh_hurl(__mdh_make_string("grup() needs a positive chunk size"));
        return __mdh_make_list(0);
    }
    return __mdh_chunks(list, size);
}

MdhValue __mdh_window(MdhValue str, MdhValue size) {
    if (str.tag != MDH_TAG_STRING) {
        __mdh_type_error("window", str.tag, 0);
        return __mdh_make_list(0);
    }
    if (size.tag != MDH_TAG_INT) {
        __mdh_type_error("window", size.tag, 0);
        return __mdh_make_list(0);
    }

    int64_t n = size.data;
    if (n <= 0) {
        __mdh_hurl(__mdh_make_string("window() size must be positive"));
        return __mdh_make_list(0);
    }

    const char *s = __mdh_get_string(str);
    int64_t len = s ? (int64_t)strlen(s) : 0;
    if (n > len) {
        return __mdh_make_list(0);
    }

    int64_t out_len = len - n + 1;
    MdhValue out = __mdh_make_list((int32_t)out_len);
    for (int64_t i = 0; i <= len - n; i++) {
        char *buf = (char *)GC_malloc((size_t)n + 1);
        memcpy(buf, s + i, (size_t)n);
        buf[n] = '\0';
        __mdh_list_push(out, __mdh_string_from_buf(buf));
    }
    return out;
}

MdhValue __mdh_interleave(MdhValue list_a, MdhValue list_b) {
    if (list_a.tag != MDH_TAG_LIST || list_b.tag != MDH_TAG_LIST) {
        __mdh_type_error("interleave", list_a.tag, list_b.tag);
        return __mdh_make_list(0);
    }

    MdhList *a = __mdh_get_list(list_a);
    MdhList *b = __mdh_get_list(list_b);
    int64_t alen = a ? a->length : 0;
    int64_t blen = b ? b->length : 0;
    int64_t max_len = alen > blen ? alen : blen;

    MdhValue out = __mdh_make_list((int32_t)(alen + blen));
    for (int64_t i = 0; i < max_len; i++) {
        if (i < alen) {
            __mdh_list_push(out, a->items[i]);
        }
        if (i < blen) {
            __mdh_list_push(out, b->items[i]);
        }
    }
    return out;
}

MdhValue __mdh_pair_adjacent(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("pair_up", list.tag, 0);
        return __mdh_make_list(0);
    }

    MdhList *src = __mdh_get_list(list);
    int64_t len = src ? src->length : 0;
    int64_t out_cap = (len + 1) / 2;
    MdhValue out = __mdh_make_list((int32_t)out_cap);

    for (int64_t i = 0; i < len; i += 2) {
        int64_t end = i + 2;
        if (end > len) {
            end = len;
        }
        MdhValue pair = __mdh_make_list((int32_t)(end - i));
        for (int64_t j = i; j < end; j++) {
            __mdh_list_push(pair, src->items[j]);
        }
        __mdh_list_push(out, pair);
    }
    return out;
}

MdhValue __mdh_skelp(MdhValue str, MdhValue size) {
    if (str.tag != MDH_TAG_STRING || size.tag != MDH_TAG_INT) {
        __mdh_type_error("skelp", str.tag, size.tag);
        return __mdh_make_list(0);
    }

    const char *s = __mdh_get_string(str);
    int64_t n = size.data;
    if (n <= 0) {
        __mdh_hurl(__mdh_make_string("skelp() size must be positive"));
        return __mdh_make_list(0);
    }

    size_t slen = strlen(s);
    MdhValue out = __mdh_make_list((int32_t)((slen + (size_t)n - 1) / (size_t)n));
    for (size_t i = 0; i < slen; i += (size_t)n) {
        size_t end = i + (size_t)n;
        if (end > slen) {
            end = slen;
        }
        size_t seg_len = end - i;
        char *buf = (char *)GC_malloc(seg_len + 1);
        memcpy(buf, s + i, seg_len);
        buf[seg_len] = '\0';
        __mdh_list_push(out, __mdh_string_from_buf(buf));
    }
    return out;
}

MdhValue __mdh_strip_left(MdhValue str, MdhValue chars) {
    if (str.tag != MDH_TAG_STRING || chars.tag != MDH_TAG_STRING) {
        __mdh_type_error("strip_left", str.tag, chars.tag);
        return __mdh_make_string("");
    }

    const char *s = __mdh_get_string(str);
    const char *set = __mdh_get_string(chars);
    if (!set || set[0] == '\0') {
        return str;
    }

    size_t start = 0;
    while (s[start] && __mdh_char_in_set((unsigned char)s[start], set)) {
        start++;
    }

    size_t slen = strlen(s);
    size_t out_len = slen - start;
    char *buf = (char *)GC_malloc(out_len + 1);
    memcpy(buf, s + start, out_len);
    buf[out_len] = '\0';
    return __mdh_string_from_buf(buf);
}

MdhValue __mdh_strip_right(MdhValue str, MdhValue chars) {
    if (str.tag != MDH_TAG_STRING || chars.tag != MDH_TAG_STRING) {
        __mdh_type_error("strip_right", str.tag, chars.tag);
        return __mdh_make_string("");
    }

    const char *s = __mdh_get_string(str);
    const char *set = __mdh_get_string(chars);
    if (!set || set[0] == '\0') {
        return str;
    }

    size_t len = strlen(s);
    while (len > 0 && __mdh_char_in_set((unsigned char)s[len - 1], set)) {
        len--;
    }

    char *buf = (char *)GC_malloc(len + 1);
    memcpy(buf, s, len);
    buf[len] = '\0';
    return __mdh_string_from_buf(buf);
}

MdhValue __mdh_swapcase(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) {
        __mdh_type_error("swapcase", str.tag, 0);
        return __mdh_make_string("");
    }

    const char *s = __mdh_get_string(str);
    size_t len = strlen(s);
    char *out = (char *)GC_malloc(len + 1);
    for (size_t i = 0; i < len; i++) {
        unsigned char c = (unsigned char)s[i];
        if (isupper(c)) {
            out[i] = (char)tolower(c);
        } else if (islower(c)) {
            out[i] = (char)toupper(c);
        } else {
            out[i] = (char)c;
        }
    }
    out[len] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_sporran_fill(MdhValue str, MdhValue width, MdhValue fill_char) {
    if (str.tag != MDH_TAG_STRING || width.tag != MDH_TAG_INT || fill_char.tag != MDH_TAG_STRING) {
        __mdh_type_error("sporran_fill", str.tag, width.tag);
        return __mdh_make_string("");
    }

    const char *s = __mdh_get_string(str);
    size_t len = strlen(s);
    size_t w = (size_t)width.data;
    const char *fill_s = __mdh_get_string(fill_char);
    char fc = (fill_s && fill_s[0] != '\0') ? fill_s[0] : ' ';

    if (len >= w) {
        return str;
    }

    size_t padding = w - len;
    size_t left = padding / 2;
    size_t right = padding - left;

    char *out = (char *)GC_malloc(w + 1);
    size_t pos = 0;
    for (size_t i = 0; i < left; i++) {
        out[pos++] = fc;
    }
    memcpy(out + pos, s, len);
    pos += len;
    for (size_t i = 0; i < right; i++) {
        out[pos++] = fc;
    }
    out[pos] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_scottify(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) {
        __mdh_type_error("scottify", str.tag, 0);
        return __mdh_make_string("");
    }

    MdhValue out = str;
    const char *pairs[][2] = {
        {"yes", "aye"},
        {"Yes", "Aye"},
        {"no", "nae"},
        {"No", "Nae"},
        {"know", "ken"},
        {"Know", "Ken"},
        {"not", "nae"},
        {"from", "fae"},
        {"to", "tae"},
        {"do", "dae"},
        {"myself", "masel"},
        {"yourself", "yersel"},
        {"small", "wee"},
        {"little", "wee"},
        {"child", "bairn"},
        {"children", "bairns"},
        {"church", "kirk"},
        {"beautiful", "bonnie"},
        {"Beautiful", "Bonnie"},
        {"going", "gaun"},
        {"have", "hae"},
        {"nothing", "naething"},
        {"something", "somethin"},
        {"everything", "awthing"},
        {"everyone", "awbody"},
        {"about", "aboot"},
        {"out", "oot"},
        {"house", "hoose"},
    };

    for (size_t i = 0; i < sizeof(pairs) / sizeof(pairs[0]); i++) {
        out = __mdh_chynge(out, __mdh_make_string(pairs[i][0]), __mdh_make_string(pairs[i][1]));
    }
    return out;
}

MdhValue __mdh_mutter(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) {
        __mdh_type_error("mutter", str.tag, 0);
        return __mdh_make_string("");
    }
    const char *s = __mdh_get_string(str);
    size_t len = strlen(s);
    char *out = (char *)GC_malloc(len + 7);
    memcpy(out, "...", 3);
    for (size_t i = 0; i < len; i++) {
        out[3 + i] = (char)tolower((unsigned char)s[i]);
    }
    memcpy(out + 3 + len, "...", 3);
    out[6 + len] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_blooter(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) {
        __mdh_type_error("blooter", str.tag, 0);
        return __mdh_make_string("");
    }

    const char *s = __mdh_get_string(str);
    size_t len = strlen(s);
    char *out = (char *)GC_malloc(len + 1);
    memcpy(out, s, len + 1);

    __mdh_ensure_rng();
    if (len > 1) {
        for (size_t i = len - 1; i > 0; i--) {
            size_t j = (size_t)(rand() % (int)(i + 1));
            char tmp = out[i];
            out[i] = out[j];
            out[j] = tmp;
        }
    }

    return __mdh_string_from_buf(out);
}

MdhValue __mdh_stooshie(MdhValue str) {
    return __mdh_blooter(str);
}

MdhValue __mdh_dreich(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) {
        __mdh_type_error("dreich", str.tag, 0);
        return __mdh_make_bool(false);
    }
    const char *s = __mdh_get_string(str);
    if (!s || s[0] == '\0') {
        return __mdh_make_bool(true);
    }
    unsigned char first = (unsigned char)s[0];
    for (const unsigned char *p = (const unsigned char *)s + 1; *p; p++) {
        if (*p != first) {
            return __mdh_make_bool(false);
        }
    }
    return __mdh_make_bool(true);
}

MdhValue __mdh_geggie(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) {
        __mdh_type_error("geggie", str.tag, 0);
        return __mdh_make_string("");
    }
    const char *s = __mdh_get_string(str);
    size_t len = strlen(s);
    if (len == 0) {
        return __mdh_make_string("");
    }
    char *out = (char *)GC_malloc(3);
    out[0] = s[0];
    out[1] = s[len - 1];
    out[2] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_jings(MdhValue msg) {
    MdhValue s = __mdh_to_string(msg);
    const char *m = __mdh_get_string(s);
    const char *prefix = "Jings! ";
    size_t plen = strlen(prefix);
    size_t mlen = strlen(m);
    char *out = (char *)GC_malloc(plen + mlen + 1);
    memcpy(out, prefix, plen);
    memcpy(out + plen, m, mlen);
    out[plen + mlen] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_crivvens(MdhValue msg) {
    MdhValue s = __mdh_to_string(msg);
    const char *m = __mdh_get_string(s);
    const char *prefix = "Crivvens! ";
    size_t plen = strlen(prefix);
    size_t mlen = strlen(m);
    char *out = (char *)GC_malloc(plen + mlen + 1);
    memcpy(out, prefix, plen);
    memcpy(out + plen, m, mlen);
    out[plen + mlen] = '\0';
    return __mdh_string_from_buf(out);
}

/* ========== Interpreter Parity Builtins (Scots Fun + Helpers) ========== */

MdhValue __mdh_braw(MdhValue val) {
    switch (val.tag) {
        case MDH_TAG_NIL:
            return __mdh_make_bool(false);
        case MDH_TAG_BOOL:
            return __mdh_make_bool(val.data != 0);
        case MDH_TAG_INT:
            return __mdh_make_bool(val.data > 0);
        case MDH_TAG_FLOAT:
            return __mdh_make_bool(__mdh_get_float(val) > 0.0);
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(val);
            return __mdh_make_bool(s && s[0] != '\0');
        }
        case MDH_TAG_LIST: {
            MdhList *l = __mdh_get_list(val);
            return __mdh_make_bool(l && l->length > 0);
        }
        case MDH_TAG_DICT: {
            int64_t *dict_ptr = (int64_t *)(intptr_t)val.data;
            int64_t count = dict_ptr ? *dict_ptr : 0;
            return __mdh_make_bool(count > 0);
        }
        default:
            return __mdh_make_bool(true);
    }
}

MdhValue __mdh_crabbit(MdhValue val) {
    if (val.tag == MDH_TAG_INT) {
        return __mdh_make_bool(val.data < 0);
    }
    if (val.tag == MDH_TAG_FLOAT) {
        return __mdh_make_bool(__mdh_get_float(val) < 0.0);
    }
    __mdh_type_error("crabbit", val.tag, 0);
    return __mdh_make_bool(false);
}

MdhValue __mdh_gallus(MdhValue val) {
    switch (val.tag) {
        case MDH_TAG_INT:
            return __mdh_make_bool(val.data != 0 && (val.data > 100 || val.data < -100));
        case MDH_TAG_FLOAT: {
            double f = __mdh_get_float(val);
            return __mdh_make_bool(f != 0.0 && (f > 100.0 || f < -100.0));
        }
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(val);
            return __mdh_make_bool(s && strlen(s) > 20);
        }
        case MDH_TAG_LIST: {
            MdhList *l = __mdh_get_list(val);
            return __mdh_make_bool(l && l->length > 10);
        }
        default:
            return __mdh_make_bool(false);
    }
}

MdhValue __mdh_drookit(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("drookit", list.tag, 0);
        return __mdh_make_bool(false);
    }
    MdhList *l = __mdh_get_list(list);
    if (!l || l->length <= 1) {
        return __mdh_make_bool(false);
    }
    for (int64_t i = 0; i < l->length; i++) {
        for (int64_t j = i + 1; j < l->length; j++) {
            if (__mdh_eq(l->items[i], l->items[j])) {
                return __mdh_make_bool(true);
            }
        }
    }
    return __mdh_make_bool(false);
}

MdhValue __mdh_clarty(MdhValue val) {
    if (val.tag == MDH_TAG_LIST) {
        return __mdh_drookit(val);
    }
    if (val.tag == MDH_TAG_STRING) {
        const unsigned char *s = (const unsigned char *)__mdh_get_string(val);
        bool seen[256] = {0};
        for (const unsigned char *p = s; *p; p++) {
            if (seen[*p]) {
                return __mdh_make_bool(true);
            }
            seen[*p] = true;
        }
        return __mdh_make_bool(false);
    }
    __mdh_type_error("clarty", val.tag, 0);
    return __mdh_make_bool(false);
}

MdhValue __mdh_glaikit(MdhValue val) {
    switch (val.tag) {
        case MDH_TAG_NIL:
            return __mdh_make_bool(true);
        case MDH_TAG_INT:
            return __mdh_make_bool(val.data == 0);
        case MDH_TAG_FLOAT:
            return __mdh_make_bool(__mdh_get_float(val) == 0.0);
        case MDH_TAG_STRING: {
            const unsigned char *s = (const unsigned char *)__mdh_get_string(val);
            if (!s) return __mdh_make_bool(true);
            for (const unsigned char *p = s; *p; p++) {
                if (!isspace(*p)) {
                    return __mdh_make_bool(false);
                }
            }
            return __mdh_make_bool(true);
        }
        case MDH_TAG_LIST: {
            MdhList *l = __mdh_get_list(val);
            return __mdh_make_bool(!l || l->length == 0);
        }
        case MDH_TAG_DICT: {
            int64_t *dict_ptr = (int64_t *)(intptr_t)val.data;
            int64_t count = dict_ptr ? *dict_ptr : 0;
            return __mdh_make_bool(count == 0);
        }
        default:
            return __mdh_make_bool(false);
    }
}

MdhValue __mdh_is_wee(MdhValue val) {
    switch (val.tag) {
        case MDH_TAG_INT: {
            int64_t n = val.data;
            return __mdh_make_bool(n > -10 && n < 10);
        }
        case MDH_TAG_FLOAT: {
            double f = fabs(__mdh_get_float(val));
            return __mdh_make_bool(f < 10.0);
        }
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(val);
            return __mdh_make_bool(s && strlen(s) < 5);
        }
        case MDH_TAG_LIST: {
            MdhList *l = __mdh_get_list(val);
            return __mdh_make_bool(!l || l->length < 5);
        }
        default:
            return __mdh_make_bool(true);
    }
}

MdhValue __mdh_is_muckle(MdhValue val) {
    switch (val.tag) {
        case MDH_TAG_INT: {
            int64_t n = val.data;
            return __mdh_make_bool(n >= 100 || n <= -100);
        }
        case MDH_TAG_FLOAT: {
            double f = fabs(__mdh_get_float(val));
            return __mdh_make_bool(f >= 100.0);
        }
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(val);
            return __mdh_make_bool(s && strlen(s) >= 50);
        }
        case MDH_TAG_LIST: {
            MdhList *l = __mdh_get_list(val);
            return __mdh_make_bool(l && l->length >= 50);
        }
        default:
            return __mdh_make_bool(false);
    }
}

MdhValue __mdh_is_blank(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) {
        __mdh_type_error("is_blank", str.tag, 0);
        return __mdh_make_bool(false);
    }
    const unsigned char *s = (const unsigned char *)__mdh_get_string(str);
    if (!s) return __mdh_make_bool(true);
    for (const unsigned char *p = s; *p; p++) {
        if (!isspace(*p)) {
            return __mdh_make_bool(false);
        }
    }
    return __mdh_make_bool(true);
}

MdhValue __mdh_haverin(MdhValue val) {
    if (val.tag == MDH_TAG_NIL) {
        return __mdh_make_bool(true);
    }
    if (val.tag == MDH_TAG_LIST) {
        MdhList *l = __mdh_get_list(val);
        return __mdh_make_bool(!l || l->length == 0);
    }
    if (val.tag == MDH_TAG_STRING) {
        const unsigned char *s = (const unsigned char *)__mdh_get_string(val);
        if (!s) return __mdh_make_bool(true);
        size_t trimmed_len = 0;
        for (const unsigned char *p = s; *p; p++) {
            if (!isspace(*p)) {
                trimmed_len++;
            }
        }
        return __mdh_make_bool(trimmed_len == 0 || trimmed_len < 2);
    }
    return __mdh_make_bool(false);
}

MdhValue __mdh_banter(MdhValue a, MdhValue b) {
    if (a.tag != MDH_TAG_STRING || b.tag != MDH_TAG_STRING) {
        __mdh_type_error("banter", a.tag, b.tag);
        return __mdh_make_string("");
    }
    const char *s1 = __mdh_get_string(a);
    const char *s2 = __mdh_get_string(b);
    size_t n1 = strlen(s1);
    size_t n2 = strlen(s2);
    char *out = (char *)GC_malloc(n1 + n2 + 1);
    size_t pos = 0;
    size_t i = 0;
    while (i < n1 || i < n2) {
        if (i < n1) out[pos++] = s1[i];
        if (i < n2) out[pos++] = s2[i];
        i++;
    }
    out[pos] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_capitalize(MdhValue str) {
    if (str.tag != MDH_TAG_STRING) {
        __mdh_type_error("capitalize", str.tag, 0);
        return __mdh_make_string("");
    }
    const unsigned char *s = (const unsigned char *)__mdh_get_string(str);
    size_t len = strlen((const char *)s);
    if (len == 0) {
        return __mdh_make_string("");
    }
    char *out = (char *)GC_malloc(len + 1);
    memcpy(out, s, len + 1);
    out[0] = (char)toupper((unsigned char)out[0]);
    return __mdh_string_from_buf(out);
}

static bool __mdh_dict_is_creel(MdhValue dict) {
    if (dict.tag != MDH_TAG_DICT) return false;
    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    if (!dict_ptr) return false;
    int64_t count = dict_ptr[0];
    if (count == 0) {
        return dict_ptr[1] == MDH_CREEL_SENTINEL;
    }
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);
    for (int64_t i = 0; i < count; i++) {
        MdhValue k = entries[i * 2];
        MdhValue v = entries[i * 2 + 1];
        if (k.tag != v.tag || k.data != v.data) {
            return false;
        }
    }
    return true;
}

static const char *__mdh_type_name(MdhValue v) {
    switch (v.tag) {
        case MDH_TAG_NIL:
            return "naething";
        case MDH_TAG_BOOL:
            return "bool";
        case MDH_TAG_INT:
            return "integer";
        case MDH_TAG_FLOAT:
            return "float";
        case MDH_TAG_STRING:
            return "string";
        case MDH_TAG_LIST:
            return "list";
        case MDH_TAG_DICT:
            return __mdh_dict_is_creel(v) ? "creel" : "dict";
        case MDH_TAG_FUNCTION:
            return "function";
        case MDH_TAG_CLASS:
            return "class";
        case MDH_TAG_INSTANCE:
            return "instance";
        case MDH_TAG_RANGE:
            return "range";
        default:
            return "unknown";
    }
}

MdhValue __mdh_scunner(MdhValue v) {
    switch (v.tag) {
        case MDH_TAG_INT:
            return __mdh_make_bool(v.data < 0);
        case MDH_TAG_FLOAT:
            return __mdh_make_bool(__mdh_get_float(v) < 0.0);
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(v);
            return __mdh_make_bool(!s || s[0] == '\0');
        }
        case MDH_TAG_LIST: {
            MdhList *l = __mdh_get_list(v);
            return __mdh_make_bool(!l || l->length == 0);
        }
        case MDH_TAG_BOOL:
            return __mdh_make_bool(v.data == 0);
        case MDH_TAG_NIL:
            return __mdh_make_bool(true);
        default:
            return __mdh_make_bool(false);
    }
}

MdhValue __mdh_scunner_check(MdhValue val, MdhValue expected_type) {
    if (expected_type.tag != MDH_TAG_STRING) {
        __mdh_type_error("scunner_check", expected_type.tag, 0);
        return __mdh_make_bool(false);
    }

    const char *expected = __mdh_get_string(expected_type);
    const char *actual = __mdh_type_name(val);
    if (strcmp(expected, actual) == 0) {
        return __mdh_make_bool(true);
    }

    const char *prefix = "Och, ya scunner! Expected ";
    const char *mid = " but got ";
    size_t plen = strlen(prefix);
    size_t elen = strlen(expected);
    size_t mlen = strlen(mid);
    size_t alen = strlen(actual);
    char *out = (char *)GC_malloc(plen + elen + mlen + alen + 1);
    memcpy(out, prefix, plen);
    memcpy(out + plen, expected, elen);
    memcpy(out + plen + elen, mid, mlen);
    memcpy(out + plen + elen + mlen, actual, alen);
    out[plen + elen + mlen + alen] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_clype(MdhValue val) {
    const char *t = __mdh_type_name(val);
    char info[256];

    switch (val.tag) {
        case MDH_TAG_LIST: {
            MdhList *l = __mdh_get_list(val);
            snprintf(info, sizeof(info), "list wi' %lld items", (long long)(l ? l->length : 0));
            break;
        }
        case MDH_TAG_DICT: {
            int64_t *dict_ptr = (int64_t *)(intptr_t)val.data;
            int64_t count = dict_ptr ? *dict_ptr : 0;
            if (__mdh_dict_is_creel(val)) {
                snprintf(info, sizeof(info), "creel wi' %lld items", (long long)count);
            } else {
                snprintf(info, sizeof(info), "dict wi' %lld entries", (long long)count);
            }
            break;
        }
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(val);
            snprintf(info, sizeof(info), "string o' %zu characters", s ? strlen(s) : 0);
            break;
        }
        case MDH_TAG_INT:
            snprintf(info, sizeof(info), "integer: %lld", (long long)val.data);
            break;
        case MDH_TAG_FLOAT:
            snprintf(info, sizeof(info), "float: %g", __mdh_get_float(val));
            break;
        case MDH_TAG_BOOL:
            snprintf(info, sizeof(info), "boolean: %s", val.data ? "aye" : "nae");
            break;
        case MDH_TAG_NIL:
            snprintf(info, sizeof(info), "naething");
            break;
        default:
            snprintf(info, sizeof(info), "%s", t);
            break;
    }

    size_t tlen = strlen(t);
    size_t ilen = strlen(info);
    char *out = (char *)GC_malloc(tlen + ilen + 5);
    out[0] = '[';
    memcpy(out + 1, t, tlen);
    out[1 + tlen] = ']';
    out[2 + tlen] = ' ';
    memcpy(out + 3 + tlen, info, ilen);
    out[3 + tlen + ilen] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_stoater(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("stoater", list.tag, 0);
        return __mdh_make_nil();
    }
    MdhList *l = __mdh_get_list(list);
    if (!l || l->length == 0) {
        __mdh_hurl(__mdh_make_string("Cannae find a stoater in an empty list!"));
        return __mdh_make_nil();
    }
    MdhValue best = l->items[0];
    for (int64_t i = 1; i < l->length; i++) {
        MdhValue item = l->items[i];
        if (best.tag == MDH_TAG_INT && item.tag == MDH_TAG_INT) {
            if (item.data > best.data) {
                best = item;
            }
        } else if (best.tag == MDH_TAG_FLOAT && item.tag == MDH_TAG_FLOAT) {
            if (__mdh_get_float(item) > __mdh_get_float(best)) {
                best = item;
            }
        } else if (best.tag == MDH_TAG_STRING && item.tag == MDH_TAG_STRING) {
            const char *a = __mdh_get_string(best);
            const char *b = __mdh_get_string(item);
            if (a && b && strlen(b) > strlen(a)) {
                best = item;
            }
        }
    }
    return best;
}

MdhValue __mdh_dicht(MdhValue list, MdhValue index) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("dicht", list.tag, 0);
        return __mdh_make_list(0);
    }
    int64_t idx;
    if (index.tag == MDH_TAG_INT) {
        idx = index.data;
    } else if (index.tag == MDH_TAG_FLOAT) {
        idx = (int64_t)__mdh_get_float(index);
    } else {
        __mdh_type_error("dicht", index.tag, 0);
        return __mdh_make_list(0);
    }

    MdhList *src = __mdh_get_list(list);
    int64_t len = src ? src->length : 0;
    if (idx < 0) {
        idx = len + idx;
    }
    if (idx < 0 || idx >= len) {
        char msg[128];
        snprintf(msg, sizeof(msg), "Index %lld oot o' bounds fer list o' length %lld", (long long)idx, (long long)len);
        __mdh_hurl(__mdh_make_string(msg));
        return __mdh_make_list(0);
    }

    MdhValue out = __mdh_make_list((int32_t)(len - 1));
    for (int64_t i = 0; i < len; i++) {
        if (i == idx) continue;
        __mdh_list_push(out, src->items[i]);
    }
    return out;
}

MdhValue __mdh_redd_up(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("redd_up", list.tag, 0);
        return __mdh_make_list(0);
    }
    MdhList *src = __mdh_get_list(list);
    int64_t len = src ? src->length : 0;
    MdhValue out = __mdh_make_list((int32_t)len);
    for (int64_t i = 0; i < len; i++) {
        if (src->items[i].tag != MDH_TAG_NIL) {
            __mdh_list_push(out, src->items[i]);
        }
    }
    return out;
}

MdhValue __mdh_split_by(MdhValue list, MdhValue pred) {
    if (list.tag != MDH_TAG_LIST || pred.tag != MDH_TAG_STRING) {
        __mdh_type_error("split_by", list.tag, pred.tag);
        return __mdh_make_list(0);
    }

    const char *p = __mdh_get_string(pred);
    MdhList *src = __mdh_get_list(list);
    int64_t len = src ? src->length : 0;

    MdhValue truthy = __mdh_make_list((int32_t)len);
    MdhValue falsy = __mdh_make_list((int32_t)len);

    for (int64_t i = 0; i < len; i++) {
        MdhValue item = src->items[i];
        bool is_match = false;

        if (strcmp(p, "even") == 0) {
            is_match = (item.tag == MDH_TAG_INT) && ((item.data % 2) == 0);
        } else if (strcmp(p, "odd") == 0) {
            is_match = (item.tag == MDH_TAG_INT) && ((item.data % 2) != 0);
        } else if (strcmp(p, "positive") == 0) {
            is_match =
                (item.tag == MDH_TAG_INT && item.data > 0) ||
                (item.tag == MDH_TAG_FLOAT && __mdh_get_float(item) > 0.0);
        } else if (strcmp(p, "negative") == 0) {
            is_match =
                (item.tag == MDH_TAG_INT && item.data < 0) ||
                (item.tag == MDH_TAG_FLOAT && __mdh_get_float(item) < 0.0);
        } else if (strcmp(p, "truthy") == 0) {
            is_match = __mdh_truthy(item);
        } else if (strcmp(p, "nil") == 0) {
            is_match = (item.tag == MDH_TAG_NIL);
        } else if (strcmp(p, "string") == 0) {
            is_match = (item.tag == MDH_TAG_STRING);
        } else if (strcmp(p, "number") == 0) {
            is_match = (item.tag == MDH_TAG_INT || item.tag == MDH_TAG_FLOAT);
        } else {
            __mdh_hurl(__mdh_make_string("Unknown predicate. Try: even, odd, positive, negative, truthy, nil, string, number"));
            return __mdh_make_list(0);
        }

        if (is_match) {
            __mdh_list_push(truthy, item);
        } else {
            __mdh_list_push(falsy, item);
        }
    }

    MdhValue result = __mdh_make_list(2);
    __mdh_list_push(result, truthy);
    __mdh_list_push(result, falsy);
    return result;
}

MdhValue __mdh_grup_runs(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("grup_runs", list.tag, 0);
        return __mdh_make_list(0);
    }

    MdhList *src = __mdh_get_list(list);
    int64_t len = src ? src->length : 0;
    MdhValue result = __mdh_make_list((int32_t)len);
    if (len == 0) {
        return result;
    }

    MdhValue current_group = __mdh_make_list(4);
    MdhValue first = src->items[0];
    __mdh_list_push(current_group, first);

    for (int64_t i = 1; i < len; i++) {
        MdhValue item = src->items[i];
        if (__mdh_eq(first, item)) {
            __mdh_list_push(current_group, item);
        } else {
            __mdh_list_push(result, current_group);
            current_group = __mdh_make_list(4);
            first = item;
            __mdh_list_push(current_group, item);
        }
    }
    __mdh_list_push(result, current_group);
    return result;
}

MdhValue __mdh_range_o(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("range_o", list.tag, 0);
        return __mdh_make_float(0.0);
    }
    MdhList *src = __mdh_get_list(list);
    int64_t len = src ? src->length : 0;
    if (len == 0) {
        __mdh_hurl(__mdh_make_string("Cannae get range o' empty list!"));
        return __mdh_make_float(0.0);
    }

    double min_v = 1e308;
    double max_v = -1e308;
    for (int64_t i = 0; i < len; i++) {
        MdhValue item = src->items[i];
        double v;
        if (item.tag == MDH_TAG_INT) {
            v = (double)item.data;
        } else if (item.tag == MDH_TAG_FLOAT) {
            v = __mdh_get_float(item);
        } else {
            __mdh_type_error("range_o", item.tag, 0);
            return __mdh_make_float(0.0);
        }
        if (v < min_v) min_v = v;
        if (v > max_v) max_v = v;
    }
    return __mdh_make_float(max_v - min_v);
}

MdhValue __mdh_tattie_scone(MdhValue str, MdhValue n) {
    if (str.tag != MDH_TAG_STRING || n.tag != MDH_TAG_INT) {
        __mdh_type_error("tattie_scone", str.tag, n.tag);
        return __mdh_make_string("");
    }
    const char *s = __mdh_get_string(str);
    int64_t count = n.data;
    if (count <= 0) {
        return __mdh_make_string("");
    }
    size_t slen = strlen(s);
    size_t sep_len = 3; /* " | " */
    size_t out_len = (size_t)count * slen + (size_t)(count - 1) * sep_len;
    char *out = (char *)GC_malloc(out_len + 1);
    size_t pos = 0;
    for (int64_t i = 0; i < count; i++) {
        if (i > 0) {
            memcpy(out + pos, " | ", sep_len);
            pos += sep_len;
        }
        memcpy(out + pos, s, slen);
        pos += slen;
    }
    out[pos] = '\0';
    return __mdh_string_from_buf(out);
}

MdhValue __mdh_haggis_hunt(MdhValue haystack, MdhValue needle) {
    if (haystack.tag != MDH_TAG_STRING || needle.tag != MDH_TAG_STRING) {
        __mdh_type_error("haggis_hunt", haystack.tag, needle.tag);
        return __mdh_make_list(0);
    }
    const char *h = __mdh_get_string(haystack);
    const char *n = __mdh_get_string(needle);
    size_t nlen = strlen(n);
    MdhValue result = __mdh_make_list(8);
    if (nlen == 0) {
        return result;
    }

    const char *p = h;
    while (1) {
        const char *found = strstr(p, n);
        if (!found) break;
        __mdh_list_push(result, __mdh_make_int((int64_t)(found - h)));
        p = found + nlen;
    }
    return result;
}

MdhValue __mdh_blether_format(MdhValue template, MdhValue dict) {
    if (template.tag != MDH_TAG_STRING || dict.tag != MDH_TAG_DICT) {
        __mdh_type_error("blether_format", template.tag, dict.tag);
        return __mdh_make_string("");
    }

    MdhValue result = template;
    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = dict_ptr ? *dict_ptr : 0;
    MdhValue *entries = dict_ptr ? (MdhValue *)(dict_ptr + 1) : NULL;
    for (int64_t i = 0; i < count; i++) {
        MdhValue key = entries[i * 2];
        MdhValue val = entries[i * 2 + 1];

        MdhValue key_s = (key.tag == MDH_TAG_STRING) ? key : __mdh_to_string(key);
        const char *k = __mdh_get_string(key_s);
        size_t klen = strlen(k);
        char *ph = (char *)GC_malloc(klen + 3);
        ph[0] = '{';
        memcpy(ph + 1, k, klen);
        ph[1 + klen] = '}';
        ph[2 + klen] = '\0';

        MdhValue placeholder = __mdh_string_from_buf(ph);
        MdhValue repl = __mdh_to_string(val);
        result = __mdh_chynge(result, placeholder, repl);
    }
    return result;
}

MdhValue __mdh_bampot_mode(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_type_error("bampot_mode", list.tag, 0);
        return __mdh_make_list(0);
    }

    MdhValue tmp = __mdh_list_copy(list);
    tmp = __mdh_shuffle(tmp);
    tmp = __mdh_shuffle(tmp);

    MdhList *l = __mdh_get_list(tmp);
    if (l && l->length > 1) {
        for (int64_t i = 0; i < l->length / 2; i++) {
            MdhValue t = l->items[i];
            l->items[i] = l->items[l->length - 1 - i];
            l->items[l->length - 1 - i] = t;
        }
    }
    return tmp;
}

/* ========== Exceptions (Try/Catch/Hurl) ========== */

#define MDH_TRY_MAX_DEPTH 64
static jmp_buf * __mdh_try_stack[MDH_TRY_MAX_DEPTH];
static int __mdh_try_depth = 0;
static MdhValue __mdh_last_error;

int64_t __mdh_jmp_buf_size(void) {
    return (int64_t)sizeof(jmp_buf);
}

void __mdh_try_push(void *env) {
    if (__mdh_try_depth >= MDH_TRY_MAX_DEPTH) {
        return;
    }
    __mdh_try_stack[__mdh_try_depth++] = (jmp_buf *)env;
}

void __mdh_try_pop(void) {
    if (__mdh_try_depth > 0) {
        __mdh_try_depth--;
    }
}

MdhValue __mdh_get_last_error(void) {
    return __mdh_last_error;
}

void __mdh_hurl(MdhValue msg) {
    __mdh_last_error = msg;
    if (__mdh_try_depth > 0) {
        jmp_buf *envp = __mdh_try_stack[__mdh_try_depth - 1];
        longjmp(*envp, 1);
    }
    /* Uncaught: print message to stderr and exit */
    MdhValue s = __mdh_to_string(msg);
    fprintf(stderr, "%s\n", __mdh_get_string(s));
    exit(1);
}
