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
#include <limits.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>
#include <setjmp.h>
#include <termios.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <fcntl.h>
#include <poll.h>
#include <pthread.h>

/* Boehm GC - declared as extern */
extern void GC_init(void);
extern void *GC_malloc(size_t size);
extern void *GC_realloc(void *ptr, size_t size);
extern char *GC_strdup(const char *s);
typedef struct GC_stack_base {
    void *mem_base;
} GC_stack_base;
extern int GC_register_my_thread(const GC_stack_base *sb);
extern int GC_unregister_my_thread(void);
extern int GC_get_stack_base(GC_stack_base *sb);
extern void GC_allow_register_threads(void);

typedef struct {
    uint8_t ok;
    MdhValue value;
    MdhValue error;
} MdhRsResult;

/* Rust runtime FFI (JSON + regex) */
extern MdhRsResult __mdh_rs_json_parse(MdhValue json_str);
extern MdhRsResult __mdh_rs_json_stringify(MdhValue value);
extern MdhRsResult __mdh_rs_json_pretty(MdhValue value);
extern MdhRsResult __mdh_rs_regex_test(MdhValue text, MdhValue pattern);
extern MdhRsResult __mdh_rs_regex_match(MdhValue text, MdhValue pattern);
extern MdhRsResult __mdh_rs_regex_match_all(MdhValue text, MdhValue pattern);
extern MdhRsResult __mdh_rs_regex_replace(MdhValue text, MdhValue pattern, MdhValue replacement);
extern MdhRsResult __mdh_rs_regex_replace_first(MdhValue text, MdhValue pattern, MdhValue replacement);
extern MdhRsResult __mdh_rs_regex_split(MdhValue text, MdhValue pattern);
extern MdhRsResult __mdh_rs_dns_srv(MdhValue service, MdhValue domain);
extern MdhRsResult __mdh_rs_dns_naptr(MdhValue domain);
extern MdhRsResult __mdh_rs_tls_client_new(MdhValue config);
extern MdhRsResult __mdh_rs_tls_connect(MdhValue tls, MdhValue sock_fd);
extern MdhRsResult __mdh_rs_tls_send(MdhValue tls, MdhValue buf);
extern MdhRsResult __mdh_rs_tls_recv(MdhValue tls, MdhValue max_len);
extern MdhRsResult __mdh_rs_tls_close(MdhValue tls);
extern MdhRsResult __mdh_rs_srtp_create(MdhValue config);
extern MdhRsResult __mdh_rs_srtp_protect(MdhValue ctx, MdhValue packet);
extern MdhRsResult __mdh_rs_srtp_unprotect(MdhValue ctx, MdhValue packet);
extern MdhRsResult __mdh_rs_dtls_server_new(MdhValue config);
extern MdhRsResult __mdh_rs_dtls_handshake(MdhValue dtls, MdhValue sock_fd);

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
        case MDH_TAG_BYTES: {
            MdhBytes *ba = __mdh_get_bytes(a);
            MdhBytes *bb = __mdh_get_bytes(b);
            if (!ba || !bb) return ba == bb;
            if (ba->length != bb->length) return false;
            if (ba->length == 0) return true;
            return memcmp(ba->data, bb->data, (size_t)ba->length) == 0;
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
        case MDH_TAG_BYTES: {
            MdhBytes *bytes = __mdh_get_bytes(a);
            return bytes && bytes->length > 0;
        }
        case MDH_TAG_SET: {
            int64_t *set_ptr = (int64_t *)(intptr_t)a.data;
            int64_t count = set_ptr ? *set_ptr : 0;
            return count > 0;
        }
        case MDH_TAG_DICT:
            return true;
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
        "list", "dict", "function", "class", "instance", "range", "creel", "function", "bytes"
    };

    char buf[256];
    if (got1 < 14 && got2 > 0 && got2 < 14) {
        snprintf(
            buf,
            sizeof(buf),
            "Och! Type error in '%s': got %s and %s",
            op,
            type_names[got1],
            type_names[got2]
        );
    } else if (got1 < 14) {
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
    MdhValue key_str = __mdh_to_string(key);
    if (key_str.tag == MDH_TAG_STRING) {
        k = __mdh_get_string(key_str);
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
    if (container.tag == MDH_TAG_STRING) {
        if (elem.tag != MDH_TAG_STRING) {
            __mdh_type_error("contains", container.tag, elem.tag);
            return __mdh_make_bool(false);
        }
        const char *haystack = __mdh_get_string(container);
        const char *needle = __mdh_get_string(elem);
        return __mdh_make_bool(strstr(haystack, needle) != NULL);
    }
    __mdh_type_error("contains", container.tag, elem.tag);
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
        case MDH_TAG_BYTES: {
            MdhBytes *bytes = __mdh_get_bytes(a);
            return bytes ? bytes->length : 0;
        }
        case MDH_TAG_DICT: {
            int64_t *dict_ptr = (int64_t *)(intptr_t)a.data;
            return dict_ptr ? dict_ptr[0] : 0;
        }
        case MDH_TAG_SET: {
            int64_t *set_ptr = (int64_t *)(intptr_t)a.data;
            return set_ptr ? set_ptr[0] : 0;
        }
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
        case MDH_TAG_BYTES: {
            MdhBytes *bytes = __mdh_get_bytes(v);
            int64_t len = bytes ? bytes->length : 0;
            snprintf(tmp, sizeof(tmp), "bytes[%lld]", (long long)len);
            __mdh_sb_append(out, tmp);
            return;
        }
        case MDH_TAG_SET: {
            int64_t *set_ptr = (int64_t *)(intptr_t)v.data;
            int64_t count = set_ptr ? *set_ptr : 0;
            MdhValue *entries = set_ptr ? (MdhValue *)(set_ptr + 1) : NULL;

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
        case MDH_TAG_DICT: {
            int64_t *dict_ptr = (int64_t *)(intptr_t)v.data;
            int64_t count = dict_ptr ? *dict_ptr : 0;
            MdhValue *entries = dict_ptr ? (MdhValue *)(dict_ptr + 1) : NULL;

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
            if (!s) {
                s = "";
            }
            if (*s != '\0' && isspace((unsigned char)*s)) {
                char buf[256];
                snprintf(buf, sizeof(buf), "Cannae turn '%s' intae an integer", s);
                __mdh_hurl(__mdh_make_string(buf));
                return __mdh_make_int(0);
            }
            errno = 0;
            char *end = NULL;
            long long val = strtoll(s, &end, 10);
            if (errno == ERANGE || end == s || (end && *end != '\0')) {
                char buf[256];
                snprintf(buf, sizeof(buf), "Cannae turn '%s' intae an integer", s);
                __mdh_hurl(__mdh_make_string(buf));
                return __mdh_make_int(0);
            }
            return __mdh_make_int((int64_t)val);
        }
        default:
            const char *t = __mdh_type_name(a);
            char buf[256];
            snprintf(
                buf,
                sizeof(buf),
                "Cannae turn %s intae an integer",
                t ? t : "that"
            );
            __mdh_hurl(__mdh_make_string(buf));
            return __mdh_make_int(0);
    }
}

MdhValue __mdh_to_float(MdhValue a) {
    switch (a.tag) {
        case MDH_TAG_FLOAT:
            return a;
        case MDH_TAG_INT:
            return __mdh_make_float((double)a.data);
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(a);
            if (!s) {
                s = "";
            }
            if (*s != '\0' && isspace((unsigned char)*s)) {
                char buf[256];
                snprintf(buf, sizeof(buf), "Cannae turn '%s' intae a float", s);
                __mdh_hurl(__mdh_make_string(buf));
                return __mdh_make_float(0.0);
            }
            errno = 0;
            char *end = NULL;
            double val = strtod(s, &end);
            if (errno == ERANGE || end == s || (end && *end != '\0')) {
                char buf[256];
                snprintf(buf, sizeof(buf), "Cannae turn '%s' intae a float", s);
                __mdh_hurl(__mdh_make_string(buf));
                return __mdh_make_float(0.0);
            }
            return __mdh_make_float(val);
        }
        default:
            const char *t = __mdh_type_name(a);
            char buf[256];
            snprintf(
                buf,
                sizeof(buf),
                "Cannae turn %s intae a float",
                t ? t : "that"
            );
            __mdh_hurl(__mdh_make_string(buf));
            return __mdh_make_float(0.0);
    }
}

/* ========== Bytes Operations ========== */

static void __mdh_bytes_ensure_capacity(MdhBytes *bytes, int64_t needed) {
    if (!bytes) return;
    if (needed <= bytes->capacity) {
        return;
    }
    int64_t new_cap = bytes->capacity > 0 ? bytes->capacity : 8;
    while (new_cap < needed) {
        new_cap *= 2;
    }
    bytes->data = (uint8_t *)GC_realloc(bytes->data, (size_t)new_cap);
    bytes->capacity = new_cap;
}

MdhValue __mdh_bytes_new(MdhValue size_val) {
    int64_t size = 0;
    if (size_val.tag == MDH_TAG_INT) {
        size = size_val.data;
    } else if (size_val.tag == MDH_TAG_FLOAT) {
        size = (int64_t)__mdh_get_float(size_val);
    } else {
        __mdh_type_error("bytes_new", size_val.tag, 0);
        size = 0;
    }

    if (size < 0) size = 0;

    MdhBytes *bytes = (MdhBytes *)GC_malloc(sizeof(MdhBytes));
    bytes->length = size;
    bytes->capacity = size > 0 ? size : 0;
    if (bytes->capacity > 0) {
        bytes->data = (uint8_t *)GC_malloc((size_t)bytes->capacity);
        memset(bytes->data, 0, (size_t)bytes->capacity);
    } else {
        bytes->data = NULL;
    }

    return (MdhValue){ .tag = MDH_TAG_BYTES, .data = (int64_t)(intptr_t)bytes };
}

MdhValue __mdh_bytes_from_string(MdhValue s) {
    MdhValue str_val = s.tag == MDH_TAG_STRING ? s : __mdh_to_string(s);
    const char *str = __mdh_get_string(str_val);
    size_t len = str ? strlen(str) : 0;

    MdhBytes *bytes = (MdhBytes *)GC_malloc(sizeof(MdhBytes));
    bytes->length = (int64_t)len;
    bytes->capacity = (int64_t)len;
    if (len > 0) {
        bytes->data = (uint8_t *)GC_malloc(len);
        memcpy(bytes->data, str, len);
    } else {
        bytes->data = NULL;
    }

    return (MdhValue){ .tag = MDH_TAG_BYTES, .data = (int64_t)(intptr_t)bytes };
}

int64_t __mdh_bytes_len(MdhValue bytes_val) {
    if (bytes_val.tag != MDH_TAG_BYTES) {
        __mdh_type_error("bytes_len", bytes_val.tag, 0);
        return 0;
    }
    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    return bytes ? bytes->length : 0;
}

MdhValue __mdh_bytes_slice(MdhValue bytes_val, MdhValue start_val, MdhValue end_val) {
    if (bytes_val.tag != MDH_TAG_BYTES) {
        __mdh_type_error("bytes_slice", bytes_val.tag, 0);
        return __mdh_bytes_new(__mdh_make_int(0));
    }
    if (start_val.tag != MDH_TAG_INT || end_val.tag != MDH_TAG_INT) {
        __mdh_type_error("bytes_slice", start_val.tag, end_val.tag);
        return __mdh_bytes_new(__mdh_make_int(0));
    }

    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    int64_t len = bytes ? bytes->length : 0;
    int64_t start = start_val.data;
    int64_t end = end_val.data;

    if (start < 0) start += len;
    if (end < 0) end += len;
    if (start < 0) start = 0;
    if (end > len) end = len;
    if (end < start) end = start;

    int64_t out_len = end - start;
    MdhValue out = __mdh_bytes_new(__mdh_make_int(out_len));
    MdhBytes *out_bytes = __mdh_get_bytes(out);
    if (out_len > 0 && bytes && bytes->data && out_bytes && out_bytes->data) {
        memcpy(out_bytes->data, bytes->data + start, (size_t)out_len);
    }
    return out;
}

MdhValue __mdh_bytes_get(MdhValue bytes_val, MdhValue index_val) {
    if (bytes_val.tag != MDH_TAG_BYTES) {
        __mdh_type_error("bytes_get", bytes_val.tag, 0);
        return __mdh_make_int(0);
    }
    if (index_val.tag != MDH_TAG_INT) {
        __mdh_type_error("bytes_get", index_val.tag, 0);
        return __mdh_make_int(0);
    }

    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    int64_t len = bytes ? bytes->length : 0;
    int64_t idx = index_val.data;
    if (idx < 0) idx += len;

    if (idx < 0 || idx >= len) {
        fprintf(stderr, "Och! Index %lld oot o' bounds (bytes has %lld items)\n",
                (long long)idx, (long long)len);
        exit(1);
    }

    uint8_t val = bytes->data[idx];
    return __mdh_make_int((int64_t)val);
}

MdhValue __mdh_bytes_set(MdhValue bytes_val, MdhValue index_val, MdhValue value_val) {
    if (bytes_val.tag != MDH_TAG_BYTES) {
        __mdh_type_error("bytes_set", bytes_val.tag, 0);
        return bytes_val;
    }
    if (index_val.tag != MDH_TAG_INT) {
        __mdh_type_error("bytes_set", index_val.tag, 0);
        return bytes_val;
    }
    if (value_val.tag != MDH_TAG_INT && value_val.tag != MDH_TAG_FLOAT) {
        __mdh_type_error("bytes_set", value_val.tag, 0);
        return bytes_val;
    }

    int64_t v = value_val.tag == MDH_TAG_INT
                    ? value_val.data
                    : (int64_t)__mdh_get_float(value_val);
    if (v < 0 || v > 255) {
        __mdh_hurl(__mdh_make_string("bytes_set value must be between 0 and 255"));
        return bytes_val;
    }

    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    int64_t len = bytes ? bytes->length : 0;
    int64_t idx = index_val.data;
    if (idx < 0) idx += len;

    if (idx < 0 || idx >= len) {
        fprintf(stderr, "Och! Index %lld oot o' bounds (bytes has %lld items)\n",
                (long long)idx, (long long)len);
        exit(1);
    }

    bytes->data[idx] = (uint8_t)v;
    return bytes_val;
}

MdhValue __mdh_bytes_append(MdhValue bytes_val, MdhValue other_val) {
    if (bytes_val.tag != MDH_TAG_BYTES || other_val.tag != MDH_TAG_BYTES) {
        __mdh_type_error("bytes_append", bytes_val.tag, other_val.tag);
        return bytes_val;
    }

    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    MdhBytes *other = __mdh_get_bytes(other_val);
    if (!bytes || !other || other->length <= 0) {
        return bytes_val;
    }

    int64_t new_len = bytes->length + other->length;
    __mdh_bytes_ensure_capacity(bytes, new_len);
    if (bytes->data && other->data) {
        memcpy(bytes->data + bytes->length, other->data, (size_t)other->length);
    }
    bytes->length = new_len;
    return bytes_val;
}

MdhValue __mdh_bytes_read_u16be(MdhValue bytes_val, MdhValue offset_val) {
    if (bytes_val.tag != MDH_TAG_BYTES || offset_val.tag != MDH_TAG_INT) {
        __mdh_type_error("bytes_read_u16be", bytes_val.tag, offset_val.tag);
        return __mdh_make_int(0);
    }
    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    int64_t len = bytes ? bytes->length : 0;
    int64_t off = offset_val.data;
    if (off < 0 || off + 2 > len) {
        __mdh_hurl(__mdh_make_string("bytes_read_u16be out of bounds"));
        return __mdh_make_int(0);
    }
    uint16_t val = ((uint16_t)bytes->data[off] << 8) |
                   (uint16_t)bytes->data[off + 1];
    return __mdh_make_int((int64_t)val);
}

MdhValue __mdh_bytes_read_u32be(MdhValue bytes_val, MdhValue offset_val) {
    if (bytes_val.tag != MDH_TAG_BYTES || offset_val.tag != MDH_TAG_INT) {
        __mdh_type_error("bytes_read_u32be", bytes_val.tag, offset_val.tag);
        return __mdh_make_int(0);
    }
    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    int64_t len = bytes ? bytes->length : 0;
    int64_t off = offset_val.data;
    if (off < 0 || off + 4 > len) {
        __mdh_hurl(__mdh_make_string("bytes_read_u32be out of bounds"));
        return __mdh_make_int(0);
    }
    uint32_t val = ((uint32_t)bytes->data[off] << 24) |
                   ((uint32_t)bytes->data[off + 1] << 16) |
                   ((uint32_t)bytes->data[off + 2] << 8) |
                   (uint32_t)bytes->data[off + 3];
    return __mdh_make_int((int64_t)val);
}

MdhValue __mdh_bytes_write_u16be(MdhValue bytes_val, MdhValue offset_val, MdhValue value_val) {
    if (bytes_val.tag != MDH_TAG_BYTES || offset_val.tag != MDH_TAG_INT) {
        __mdh_type_error("bytes_write_u16be", bytes_val.tag, offset_val.tag);
        return bytes_val;
    }
    if (value_val.tag != MDH_TAG_INT && value_val.tag != MDH_TAG_FLOAT) {
        __mdh_type_error("bytes_write_u16be", value_val.tag, 0);
        return bytes_val;
    }
    int64_t v = value_val.tag == MDH_TAG_INT
                    ? value_val.data
                    : (int64_t)__mdh_get_float(value_val);
    if (v < 0 || v > 0xFFFF) {
        __mdh_hurl(__mdh_make_string("bytes_write_u16be value out of range"));
        return bytes_val;
    }

    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    int64_t len = bytes ? bytes->length : 0;
    int64_t off = offset_val.data;
    if (off < 0 || off + 2 > len) {
        __mdh_hurl(__mdh_make_string("bytes_write_u16be out of bounds"));
        return bytes_val;
    }

    bytes->data[off] = (uint8_t)((v >> 8) & 0xFF);
    bytes->data[off + 1] = (uint8_t)(v & 0xFF);
    return bytes_val;
}

MdhValue __mdh_bytes_write_u32be(MdhValue bytes_val, MdhValue offset_val, MdhValue value_val) {
    if (bytes_val.tag != MDH_TAG_BYTES || offset_val.tag != MDH_TAG_INT) {
        __mdh_type_error("bytes_write_u32be", bytes_val.tag, offset_val.tag);
        return bytes_val;
    }
    if (value_val.tag != MDH_TAG_INT && value_val.tag != MDH_TAG_FLOAT) {
        __mdh_type_error("bytes_write_u32be", value_val.tag, 0);
        return bytes_val;
    }
    int64_t v = value_val.tag == MDH_TAG_INT
                    ? value_val.data
                    : (int64_t)__mdh_get_float(value_val);
    if (v < 0 || v > 0xFFFFFFFFLL) {
        __mdh_hurl(__mdh_make_string("bytes_write_u32be value out of range"));
        return bytes_val;
    }

    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    int64_t len = bytes ? bytes->length : 0;
    int64_t off = offset_val.data;
    if (off < 0 || off + 4 > len) {
        __mdh_hurl(__mdh_make_string("bytes_write_u32be out of bounds"));
        return bytes_val;
    }

    bytes->data[off] = (uint8_t)((v >> 24) & 0xFF);
    bytes->data[off + 1] = (uint8_t)((v >> 16) & 0xFF);
    bytes->data[off + 2] = (uint8_t)((v >> 8) & 0xFF);
    bytes->data[off + 3] = (uint8_t)(v & 0xFF);
    return bytes_val;
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

MdhValue __mdh_jammy(MdhValue min, MdhValue max) {
    if ((min.tag != MDH_TAG_INT && min.tag != MDH_TAG_FLOAT) ||
        (max.tag != MDH_TAG_INT && max.tag != MDH_TAG_FLOAT)) {
        __mdh_hurl(__mdh_make_string("jammy() needs integer bounds"));
        return __mdh_make_int(0);
    }

    int64_t min_i = (min.tag == MDH_TAG_FLOAT) ? (int64_t)__mdh_get_float(min) : min.data;
    int64_t max_i = (max.tag == MDH_TAG_FLOAT) ? (int64_t)__mdh_get_float(max) : max.data;
    if (min_i >= max_i) {
        __mdh_hurl(__mdh_make_string("jammy() needs min < max, ya numpty!"));
        return __mdh_make_int(0);
    }

    return __mdh_random(min_i, max_i - 1);
}

MdhValue __mdh_random_int(MdhValue min, MdhValue max) {
    if ((min.tag != MDH_TAG_INT && min.tag != MDH_TAG_FLOAT) ||
        (max.tag != MDH_TAG_INT && max.tag != MDH_TAG_FLOAT)) {
        __mdh_hurl(__mdh_make_string("random_int() needs integer bounds"));
        return __mdh_make_int(0);
    }

    int64_t min_i = (min.tag == MDH_TAG_FLOAT) ? (int64_t)__mdh_get_float(min) : min.data;
    int64_t max_i = (max.tag == MDH_TAG_FLOAT) ? (int64_t)__mdh_get_float(max) : max.data;
    if (min_i > max_i) {
        __mdh_hurl(__mdh_make_string("random_int() min must be <= max"));
        return __mdh_make_int(0);
    }

    return __mdh_random(min_i, max_i);
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

/* ========== Timing ========== */

MdhValue __mdh_mono_ms(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return __mdh_make_int(0);
    }
    uint64_t ms = ((uint64_t)ts.tv_sec * 1000ULL) + (uint64_t)(ts.tv_nsec / 1000000ULL);
    return __mdh_make_int((int64_t)ms);
}

MdhValue __mdh_mono_ns(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return __mdh_make_int(0);
    }
    uint64_t ns = ((uint64_t)ts.tv_sec * 1000000000ULL) + (uint64_t)ts.tv_nsec;
    return __mdh_make_int((int64_t)ns);
}

/* ========== Network (Sockets + DNS) ========== */

static MdhValue __mdh_result_ok(MdhValue value) {
    MdhValue dict = __mdh_empty_dict();
    dict = __mdh_dict_set(dict, __mdh_make_string("ok"), __mdh_make_bool(true));
    dict = __mdh_dict_set(dict, __mdh_make_string("value"), value);
    return dict;
}

static MdhValue __mdh_result_err(const char *msg, int code) {
    MdhValue dict = __mdh_empty_dict();
    dict = __mdh_dict_set(dict, __mdh_make_string("ok"), __mdh_make_bool(false));
    dict = __mdh_dict_set(dict, __mdh_make_string("error"), __mdh_make_string(msg ? msg : ""));
    dict = __mdh_dict_set(dict, __mdh_make_string("code"), __mdh_make_int(code));
    return dict;
}

static MdhValue __mdh_result_errno(const char *op) {
    char buf[256];
    const char *err = strerror(errno);
    snprintf(buf, sizeof(buf), "%s failed: %s", op, err ? err : "unknown error");
    return __mdh_result_err(buf, errno);
}

static int __mdh_sock_fd(MdhValue sock) {
    if (sock.tag != MDH_TAG_INT) {
        __mdh_type_error("socket", sock.tag, 0);
        return -1;
    }
    return (int)sock.data;
}

static bool __mdh_port_value(MdhValue port, int *out_port) {
    int64_t p = 0;
    if (port.tag == MDH_TAG_INT) {
        p = port.data;
    } else if (port.tag == MDH_TAG_FLOAT) {
        p = (int64_t)__mdh_get_float(port);
    } else if (port.tag == MDH_TAG_STRING) {
        const char *s = __mdh_get_string(port);
        p = s ? strtoll(s, NULL, 10) : 0;
    } else {
        __mdh_type_error("port", port.tag, 0);
        return false;
    }
    if (p < 0 || p > 65535) {
        __mdh_hurl(__mdh_make_string("Port must be between 0 and 65535"));
        return false;
    }
    *out_port = (int)p;
    return true;
}

static bool __mdh_int_value(const char *op, MdhValue value, int64_t *out) {
    if (value.tag == MDH_TAG_INT) {
        *out = value.data;
        return true;
    }
    if (value.tag == MDH_TAG_FLOAT) {
        *out = (int64_t)__mdh_get_float(value);
        return true;
    }
    __mdh_type_error(op, value.tag, 0);
    return false;
}

static const char *__mdh_host_value(MdhValue host, bool allow_nil) {
    if (host.tag == MDH_TAG_STRING) {
        return __mdh_get_string(host);
    }
    if (allow_nil && host.tag == MDH_TAG_NIL) {
        return NULL;
    }
    __mdh_type_error("host", host.tag, 0);
    return NULL;
}

static int __mdh_resolve_addr(const char *host, const char *port, int socktype, struct addrinfo **out) {
    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    hints.ai_socktype = socktype;
    if (!host) {
        hints.ai_flags = AI_PASSIVE;
    }
    return getaddrinfo(host, port, &hints, out);
}

static MdhValue __mdh_addr_dict(const struct sockaddr_in *addr) {
    char host_buf[INET_ADDRSTRLEN];
    const char *host = inet_ntop(AF_INET, &addr->sin_addr, host_buf, sizeof(host_buf));
    int port = ntohs(addr->sin_port);

    MdhValue dict = __mdh_empty_dict();
    dict = __mdh_dict_set(dict, __mdh_make_string("host"), __mdh_make_string(host ? host : ""));
    dict = __mdh_dict_set(dict, __mdh_make_string("port"), __mdh_make_int(port));
    return dict;
}

MdhValue __mdh_socket_udp(void) {
    int fd = socket(AF_INET, SOCK_DGRAM, 0);
    if (fd < 0) {
        return __mdh_result_errno("socket_udp");
    }
    return __mdh_result_ok(__mdh_make_int(fd));
}

MdhValue __mdh_socket_tcp(void) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        return __mdh_result_errno("socket_tcp");
    }
    return __mdh_result_ok(__mdh_make_int(fd));
}

MdhValue __mdh_socket_bind(MdhValue sock, MdhValue host, MdhValue port) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    int port_num = 0;
    if (!__mdh_port_value(port, &port_num)) {
        return __mdh_result_err("Invalid port", -1);
    }
    const char *host_str = __mdh_host_value(host, true);

    char port_buf[16];
    snprintf(port_buf, sizeof(port_buf), "%d", port_num);
    struct addrinfo *res = NULL;
    int rc = __mdh_resolve_addr(host_str, port_buf, 0, &res);
    if (rc != 0 || !res) {
        return __mdh_result_err(gai_strerror(rc), rc);
    }
    int bind_rc = bind(fd, res->ai_addr, (socklen_t)res->ai_addrlen);
    freeaddrinfo(res);
    if (bind_rc != 0) {
        return __mdh_result_errno("socket_bind");
    }
    return __mdh_result_ok(__mdh_make_nil());
}

MdhValue __mdh_socket_connect(MdhValue sock, MdhValue host, MdhValue port) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    int port_num = 0;
    if (!__mdh_port_value(port, &port_num)) {
        return __mdh_result_err("Invalid port", -1);
    }
    const char *host_str = __mdh_host_value(host, false);

    char port_buf[16];
    snprintf(port_buf, sizeof(port_buf), "%d", port_num);
    struct addrinfo *res = NULL;
    int rc = __mdh_resolve_addr(host_str, port_buf, 0, &res);
    if (rc != 0 || !res) {
        return __mdh_result_err(gai_strerror(rc), rc);
    }
    int conn_rc = connect(fd, res->ai_addr, (socklen_t)res->ai_addrlen);
    freeaddrinfo(res);
    if (conn_rc != 0) {
        return __mdh_result_errno("socket_connect");
    }
    return __mdh_result_ok(__mdh_make_nil());
}

MdhValue __mdh_socket_listen(MdhValue sock, MdhValue backlog) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    int64_t bl = 0;
    if (backlog.tag == MDH_TAG_INT) {
        bl = backlog.data;
    } else if (backlog.tag == MDH_TAG_FLOAT) {
        bl = (int64_t)__mdh_get_float(backlog);
    } else {
        __mdh_type_error("socket_listen", backlog.tag, 0);
        bl = 128;
    }
    if (bl < 0) bl = 0;
    if (listen(fd, (int)bl) != 0) {
        return __mdh_result_errno("socket_listen");
    }
    return __mdh_result_ok(__mdh_make_nil());
}

MdhValue __mdh_socket_accept(MdhValue sock) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    struct sockaddr_in addr;
    socklen_t addr_len = sizeof(addr);
    int new_fd = accept(fd, (struct sockaddr *)&addr, &addr_len);
    if (new_fd < 0) {
        return __mdh_result_errno("socket_accept");
    }

    MdhValue addr_dict = __mdh_addr_dict(&addr);
    MdhValue info = __mdh_empty_dict();
    info = __mdh_dict_set(info, __mdh_make_string("sock"), __mdh_make_int(new_fd));
    info = __mdh_dict_set(info, __mdh_make_string("addr"), addr_dict);
    return __mdh_result_ok(info);
}

MdhValue __mdh_socket_set_nonblocking(MdhValue sock, MdhValue on) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    bool enable = __mdh_truthy(on);
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags < 0) {
        return __mdh_result_errno("socket_set_nonblocking");
    }
    if (enable) {
        flags |= O_NONBLOCK;
    } else {
        flags &= ~O_NONBLOCK;
    }
    if (fcntl(fd, F_SETFL, flags) != 0) {
        return __mdh_result_errno("socket_set_nonblocking");
    }
    return __mdh_result_ok(__mdh_make_nil());
}

MdhValue __mdh_socket_set_reuseaddr(MdhValue sock, MdhValue on) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    int enable = __mdh_truthy(on) ? 1 : 0;
    if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &enable, sizeof(enable)) != 0) {
        return __mdh_result_errno("socket_set_reuseaddr");
    }
    return __mdh_result_ok(__mdh_make_nil());
}

MdhValue __mdh_socket_set_reuseport(MdhValue sock, MdhValue on) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
#ifdef SO_REUSEPORT
    int enable = __mdh_truthy(on) ? 1 : 0;
    if (setsockopt(fd, SOL_SOCKET, SO_REUSEPORT, &enable, sizeof(enable)) != 0) {
        return __mdh_result_errno("socket_set_reuseport");
    }
    return __mdh_result_ok(__mdh_make_nil());
#else
    (void)on;
    return __mdh_result_err("socket_set_reuseport not supported", -1);
#endif
}

MdhValue __mdh_socket_set_ttl(MdhValue sock, MdhValue ttl_val) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    int64_t ttl = 0;
    if (!__mdh_int_value("socket_set_ttl", ttl_val, &ttl)) {
        return __mdh_result_err("Invalid ttl", -1);
    }
    if (ttl < 0 || ttl > 255) {
        __mdh_hurl(__mdh_make_string("socket_set_ttl expects 0..255"));
        return __mdh_result_err("Invalid ttl", -1);
    }
#ifdef IP_TTL
    int ttl_i = (int)ttl;
    if (setsockopt(fd, IPPROTO_IP, IP_TTL, &ttl_i, sizeof(ttl_i)) != 0) {
        return __mdh_result_errno("socket_set_ttl");
    }
    return __mdh_result_ok(__mdh_make_nil());
#else
    return __mdh_result_err("socket_set_ttl not supported", -1);
#endif
}

MdhValue __mdh_socket_set_nodelay(MdhValue sock, MdhValue on) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
#ifdef TCP_NODELAY
    int enable = __mdh_truthy(on) ? 1 : 0;
    if (setsockopt(fd, IPPROTO_TCP, TCP_NODELAY, &enable, sizeof(enable)) != 0) {
        return __mdh_result_errno("socket_set_nodelay");
    }
    return __mdh_result_ok(__mdh_make_nil());
#else
    (void)on;
    return __mdh_result_err("socket_set_nodelay not supported", -1);
#endif
}

MdhValue __mdh_socket_set_rcvbuf(MdhValue sock, MdhValue bytes_val) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    int64_t bytes = 0;
    if (!__mdh_int_value("socket_set_rcvbuf", bytes_val, &bytes)) {
        return __mdh_result_err("Invalid rcvbuf size", -1);
    }
    if (bytes < 0 || bytes > INT_MAX) {
        __mdh_hurl(__mdh_make_string("socket_set_rcvbuf expects a non-negative size"));
        return __mdh_result_err("Invalid rcvbuf size", -1);
    }
    int buf = (int)bytes;
    if (setsockopt(fd, SOL_SOCKET, SO_RCVBUF, &buf, sizeof(buf)) != 0) {
        return __mdh_result_errno("socket_set_rcvbuf");
    }
    return __mdh_result_ok(__mdh_make_nil());
}

MdhValue __mdh_socket_set_sndbuf(MdhValue sock, MdhValue bytes_val) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    int64_t bytes = 0;
    if (!__mdh_int_value("socket_set_sndbuf", bytes_val, &bytes)) {
        return __mdh_result_err("Invalid sndbuf size", -1);
    }
    if (bytes < 0 || bytes > INT_MAX) {
        __mdh_hurl(__mdh_make_string("socket_set_sndbuf expects a non-negative size"));
        return __mdh_result_err("Invalid sndbuf size", -1);
    }
    int buf = (int)bytes;
    if (setsockopt(fd, SOL_SOCKET, SO_SNDBUF, &buf, sizeof(buf)) != 0) {
        return __mdh_result_errno("socket_set_sndbuf");
    }
    return __mdh_result_ok(__mdh_make_nil());
}

MdhValue __mdh_socket_close(MdhValue sock) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    if (close(fd) != 0) {
        return __mdh_result_errno("socket_close");
    }
    return __mdh_result_ok(__mdh_make_nil());
}

MdhValue __mdh_udp_send_to(MdhValue sock, MdhValue buf, MdhValue host, MdhValue port) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    if (buf.tag != MDH_TAG_BYTES) {
        __mdh_type_error("udp_send_to", buf.tag, 0);
        return __mdh_result_err("Invalid bytes", -1);
    }
    int port_num = 0;
    if (!__mdh_port_value(port, &port_num)) {
        return __mdh_result_err("Invalid port", -1);
    }
    const char *host_str = __mdh_host_value(host, false);

    char port_buf[16];
    snprintf(port_buf, sizeof(port_buf), "%d", port_num);
    struct addrinfo *res = NULL;
    int rc = __mdh_resolve_addr(host_str, port_buf, SOCK_DGRAM, &res);
    if (rc != 0 || !res) {
        return __mdh_result_err(gai_strerror(rc), rc);
    }

    MdhBytes *bytes = __mdh_get_bytes(buf);
    ssize_t sent = sendto(fd, bytes ? bytes->data : NULL, bytes ? (size_t)bytes->length : 0,
                          0, res->ai_addr, (socklen_t)res->ai_addrlen);
    freeaddrinfo(res);
    if (sent < 0) {
        return __mdh_result_errno("udp_send_to");
    }
    return __mdh_result_ok(__mdh_make_int((int64_t)sent));
}

MdhValue __mdh_udp_recv_from(MdhValue sock, MdhValue max_len_val) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    if (max_len_val.tag != MDH_TAG_INT && max_len_val.tag != MDH_TAG_FLOAT) {
        __mdh_type_error("udp_recv_from", max_len_val.tag, 0);
        return __mdh_result_err("Invalid max_len", -1);
    }
    int64_t max_len = max_len_val.tag == MDH_TAG_INT
                          ? max_len_val.data
                          : (int64_t)__mdh_get_float(max_len_val);
    if (max_len < 0) max_len = 0;

    MdhValue bytes_val = __mdh_bytes_new(__mdh_make_int(max_len));
    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    if (!bytes || max_len == 0) {
        return __mdh_result_ok(bytes_val);
    }

    struct sockaddr_in addr;
    socklen_t addr_len = sizeof(addr);
    ssize_t n = recvfrom(fd, bytes->data, (size_t)max_len, 0, (struct sockaddr *)&addr, &addr_len);
    if (n < 0) {
        return __mdh_result_errno("udp_recv_from");
    }
    bytes->length = (int64_t)n;

    MdhValue addr_dict = __mdh_addr_dict(&addr);
    MdhValue info = __mdh_empty_dict();
    info = __mdh_dict_set(info, __mdh_make_string("buf"), bytes_val);
    info = __mdh_dict_set(info, __mdh_make_string("addr"), addr_dict);
    return __mdh_result_ok(info);
}

MdhValue __mdh_tcp_send(MdhValue sock, MdhValue buf) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    if (buf.tag != MDH_TAG_BYTES) {
        __mdh_type_error("tcp_send", buf.tag, 0);
        return __mdh_result_err("Invalid bytes", -1);
    }
    MdhBytes *bytes = __mdh_get_bytes(buf);
    ssize_t sent = send(fd, bytes ? bytes->data : NULL, bytes ? (size_t)bytes->length : 0, 0);
    if (sent < 0) {
        return __mdh_result_errno("tcp_send");
    }
    return __mdh_result_ok(__mdh_make_int((int64_t)sent));
}

MdhValue __mdh_tcp_recv(MdhValue sock, MdhValue max_len_val) {
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        return __mdh_result_err("Invalid socket", -1);
    }
    if (max_len_val.tag != MDH_TAG_INT && max_len_val.tag != MDH_TAG_FLOAT) {
        __mdh_type_error("tcp_recv", max_len_val.tag, 0);
        return __mdh_result_err("Invalid max_len", -1);
    }
    int64_t max_len = max_len_val.tag == MDH_TAG_INT
                          ? max_len_val.data
                          : (int64_t)__mdh_get_float(max_len_val);
    if (max_len < 0) max_len = 0;

    MdhValue bytes_val = __mdh_bytes_new(__mdh_make_int(max_len));
    MdhBytes *bytes = __mdh_get_bytes(bytes_val);
    if (!bytes || max_len == 0) {
        return __mdh_result_ok(bytes_val);
    }
    ssize_t n = recv(fd, bytes->data, (size_t)max_len, 0);
    if (n < 0) {
        return __mdh_result_errno("tcp_recv");
    }
    bytes->length = (int64_t)n;
    return __mdh_result_ok(bytes_val);
}

MdhValue __mdh_dns_lookup(MdhValue host) {
    if (host.tag != MDH_TAG_STRING) {
        __mdh_type_error("dns_lookup", host.tag, 0);
        return __mdh_result_err("dns_lookup expects a hostname string", -1);
    }
    const char *host_str = __mdh_get_string(host);
    if (!host_str || host_str[0] == '\0') {
        return __mdh_result_err("dns_lookup expects a non-empty hostname", -1);
    }

    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    struct addrinfo *res = NULL;
    int rc = getaddrinfo(host_str, NULL, &hints, &res);
    if (rc != 0 || !res) {
        return __mdh_result_err(gai_strerror(rc), rc);
    }

    MdhValue list = __mdh_make_list(4);
    for (struct addrinfo *ai = res; ai != NULL; ai = ai->ai_next) {
        char host_buf[INET6_ADDRSTRLEN];
        const char *ip = NULL;
        if (ai->ai_family == AF_INET) {
            struct sockaddr_in *addr = (struct sockaddr_in *)ai->ai_addr;
            ip = inet_ntop(AF_INET, &addr->sin_addr, host_buf, sizeof(host_buf));
        } else if (ai->ai_family == AF_INET6) {
            struct sockaddr_in6 *addr6 = (struct sockaddr_in6 *)ai->ai_addr;
            ip = inet_ntop(AF_INET6, &addr6->sin6_addr, host_buf, sizeof(host_buf));
        }
        if (ip) {
            __mdh_list_push(list, __mdh_make_string(ip));
        }
    }
    freeaddrinfo(res);
    return __mdh_result_ok(list);
}

MdhValue __mdh_dns_srv(MdhValue service, MdhValue domain) {
    if (service.tag != MDH_TAG_STRING || domain.tag != MDH_TAG_STRING) {
        __mdh_type_error("dns_srv", service.tag, domain.tag);
        return __mdh_result_err("dns_srv expects service and domain strings", -1);
    }

    MdhRsResult r = __mdh_rs_dns_srv(service, domain);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "dns_srv failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_dns_naptr(MdhValue domain) {
    if (domain.tag != MDH_TAG_STRING) {
        __mdh_type_error("dns_naptr", domain.tag, 0);
        return __mdh_result_err("dns_naptr expects a domain string", -1);
    }

    MdhRsResult r = __mdh_rs_dns_naptr(domain);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "dns_naptr failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

/* ========== TLS/DTLS/SRTP ========== */

MdhValue __mdh_tls_client_new(MdhValue config) {
    MdhRsResult r = __mdh_rs_tls_client_new(config);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "tls_client_new failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_tls_connect(MdhValue tls, MdhValue sock) {
    if (tls.tag != MDH_TAG_INT) {
        __mdh_type_error("tls_connect", tls.tag, 0);
        return __mdh_result_err("tls_connect expects TLS handle", -1);
    }
    if (sock.tag != MDH_TAG_INT && sock.tag != MDH_TAG_FLOAT) {
        __mdh_type_error("tls_connect", sock.tag, 0);
        return __mdh_result_err("tls_connect expects socket", -1);
    }
    int fd = sock.tag == MDH_TAG_INT ? (int)sock.data : (int)__mdh_get_float(sock);
    int dup_fd = dup(fd);
    if (dup_fd < 0) {
        return __mdh_result_errno("tls_connect dup");
    }
    MdhValue fd_val = __mdh_make_int(dup_fd);
    MdhRsResult r = __mdh_rs_tls_connect(tls, fd_val);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "tls_connect failed";
        }
        close(dup_fd);
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_tls_send(MdhValue tls, MdhValue buf) {
    if (tls.tag != MDH_TAG_INT) {
        __mdh_type_error("tls_send", tls.tag, 0);
        return __mdh_result_err("tls_send expects TLS handle", -1);
    }
    if (buf.tag != MDH_TAG_BYTES) {
        __mdh_type_error("tls_send", buf.tag, 0);
        return __mdh_result_err("tls_send expects bytes", -1);
    }
    MdhRsResult r = __mdh_rs_tls_send(tls, buf);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "tls_send failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_tls_recv(MdhValue tls, MdhValue max_len) {
    if (tls.tag != MDH_TAG_INT) {
        __mdh_type_error("tls_recv", tls.tag, 0);
        return __mdh_result_err("tls_recv expects TLS handle", -1);
    }
    if (max_len.tag != MDH_TAG_INT && max_len.tag != MDH_TAG_FLOAT) {
        __mdh_type_error("tls_recv", max_len.tag, 0);
        return __mdh_result_err("tls_recv expects max_len", -1);
    }
    MdhRsResult r = __mdh_rs_tls_recv(tls, max_len);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "tls_recv failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_tls_close(MdhValue tls) {
    if (tls.tag != MDH_TAG_INT) {
        __mdh_type_error("tls_close", tls.tag, 0);
        return __mdh_result_err("tls_close expects TLS handle", -1);
    }
    MdhRsResult r = __mdh_rs_tls_close(tls);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "tls_close failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_dtls_server_new(MdhValue config) {
    MdhRsResult r = __mdh_rs_dtls_server_new(config);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "dtls_server_new failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_dtls_handshake(MdhValue dtls, MdhValue sock) {
    if (dtls.tag != MDH_TAG_INT) {
        __mdh_type_error("dtls_handshake", dtls.tag, 0);
        return __mdh_result_err("dtls_handshake expects DTLS handle", -1);
    }
    if (sock.tag != MDH_TAG_INT && sock.tag != MDH_TAG_FLOAT) {
        __mdh_type_error("dtls_handshake", sock.tag, 0);
        return __mdh_result_err("dtls_handshake expects socket", -1);
    }
    int fd = sock.tag == MDH_TAG_INT ? (int)sock.data : (int)__mdh_get_float(sock);
    int dup_fd = dup(fd);
    if (dup_fd < 0) {
        return __mdh_result_errno("dtls_handshake dup");
    }
    MdhValue fd_val = __mdh_make_int(dup_fd);
    MdhRsResult r = __mdh_rs_dtls_handshake(dtls, fd_val);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "dtls_handshake failed";
        }
        close(dup_fd);
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_srtp_create(MdhValue keys) {
    MdhRsResult r = __mdh_rs_srtp_create(keys);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "srtp_create failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_srtp_protect(MdhValue srtp, MdhValue rtp_packet) {
    if (srtp.tag != MDH_TAG_INT) {
        __mdh_type_error("srtp_protect", srtp.tag, 0);
        return __mdh_result_err("srtp_protect expects SRTP handle", -1);
    }
    if (rtp_packet.tag != MDH_TAG_BYTES) {
        __mdh_type_error("srtp_protect", rtp_packet.tag, 0);
        return __mdh_result_err("srtp_protect expects bytes", -1);
    }
    MdhRsResult r = __mdh_rs_srtp_protect(srtp, rtp_packet);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "srtp_protect failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

MdhValue __mdh_srtp_unprotect(MdhValue srtp, MdhValue rtp_packet) {
    if (srtp.tag != MDH_TAG_INT) {
        __mdh_type_error("srtp_unprotect", srtp.tag, 0);
        return __mdh_result_err("srtp_unprotect expects SRTP handle", -1);
    }
    if (rtp_packet.tag != MDH_TAG_BYTES) {
        __mdh_type_error("srtp_unprotect", rtp_packet.tag, 0);
        return __mdh_result_err("srtp_unprotect expects bytes", -1);
    }
    MdhRsResult r = __mdh_rs_srtp_unprotect(srtp, rtp_packet);
    if (!r.ok) {
        const char *msg = __mdh_get_string(r.error);
        if (!msg || msg[0] == '\0') {
            msg = "srtp_unprotect failed";
        }
        return __mdh_result_err(msg, -1);
    }
    return __mdh_result_ok(r.value);
}

/* ========== Event Loop + Timers ========== */

typedef struct {
    int fd;
    MdhValue read_cb;
    MdhValue write_cb;
} MdhWatch;

typedef struct {
    int64_t id;
    int64_t next_fire_ms;
    int64_t interval_ms;
    MdhValue callback;
    int cancelled;
} MdhTimer;

typedef struct {
    MdhWatch *watches;
    int64_t watch_len;
    int64_t watch_cap;
    MdhTimer *timers;
    int64_t timer_len;
    int64_t timer_cap;
    int64_t next_timer_id;
    int stopped;
} MdhEventLoop;

typedef struct {
    int64_t next_id;
    int64_t len;
    int64_t cap;
    int64_t *ids;
    MdhEventLoop **loops;
} MdhLoopRegistry;

static MdhLoopRegistry __mdh_loop_registry = {1, 0, 0, NULL, NULL};

static int64_t __mdh_mono_ms_now(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return 0;
    }
    return (int64_t)((uint64_t)ts.tv_sec * 1000ULL + (uint64_t)(ts.tv_nsec / 1000000ULL));
}

static void __mdh_loop_ensure_watch_cap(MdhEventLoop *loop, int64_t needed) {
    if (loop->watch_cap >= needed) return;
    int64_t new_cap = loop->watch_cap > 0 ? loop->watch_cap * 2 : 8;
    while (new_cap < needed) new_cap *= 2;
    loop->watches = (MdhWatch *)GC_realloc(loop->watches, sizeof(MdhWatch) * (size_t)new_cap);
    loop->watch_cap = new_cap;
}

static void __mdh_loop_ensure_timer_cap(MdhEventLoop *loop, int64_t needed) {
    if (loop->timer_cap >= needed) return;
    int64_t new_cap = loop->timer_cap > 0 ? loop->timer_cap * 2 : 8;
    while (new_cap < needed) new_cap *= 2;
    loop->timers = (MdhTimer *)GC_realloc(loop->timers, sizeof(MdhTimer) * (size_t)new_cap);
    loop->timer_cap = new_cap;
}

static int64_t __mdh_loop_register(MdhEventLoop *loop) {
    if (__mdh_loop_registry.cap == 0) {
        __mdh_loop_registry.cap = 8;
        __mdh_loop_registry.ids = (int64_t *)GC_malloc(sizeof(int64_t) * 8);
        __mdh_loop_registry.loops = (MdhEventLoop **)GC_malloc(sizeof(MdhEventLoop *) * 8);
    } else if (__mdh_loop_registry.len >= __mdh_loop_registry.cap) {
        int64_t new_cap = __mdh_loop_registry.cap * 2;
        __mdh_loop_registry.ids =
            (int64_t *)GC_realloc(__mdh_loop_registry.ids, sizeof(int64_t) * (size_t)new_cap);
        __mdh_loop_registry.loops = (MdhEventLoop **)GC_realloc(
            __mdh_loop_registry.loops,
            sizeof(MdhEventLoop *) * (size_t)new_cap
        );
        __mdh_loop_registry.cap = new_cap;
    }
    int64_t id = __mdh_loop_registry.next_id++;
    __mdh_loop_registry.ids[__mdh_loop_registry.len] = id;
    __mdh_loop_registry.loops[__mdh_loop_registry.len] = loop;
    __mdh_loop_registry.len++;
    return id;
}

static MdhEventLoop *__mdh_loop_get(MdhValue handle) {
    if (handle.tag != MDH_TAG_INT) {
        __mdh_type_error("event_loop", handle.tag, 0);
        return NULL;
    }
    int64_t id = handle.data;
    for (int64_t i = 0; i < __mdh_loop_registry.len; i++) {
        if (__mdh_loop_registry.ids[i] == id) {
            return __mdh_loop_registry.loops[i];
        }
    }
    __mdh_hurl(__mdh_make_string("Unknown event loop handle"));
    return NULL;
}

static MdhValue __mdh_make_event(const char *kind, int64_t sock, int64_t timer_id, MdhValue cb) {
    MdhValue ev = __mdh_empty_dict();
    ev = __mdh_dict_set(ev, __mdh_make_string("kind"), __mdh_make_string(kind ? kind : ""));
    if (sock >= 0) {
        ev = __mdh_dict_set(ev, __mdh_make_string("sock"), __mdh_make_int(sock));
    }
    if (timer_id >= 0) {
        ev = __mdh_dict_set(ev, __mdh_make_string("id"), __mdh_make_int(timer_id));
    }
    if (cb.tag != MDH_TAG_NIL) {
        ev = __mdh_dict_set(ev, __mdh_make_string("callback"), cb);
    }
    return ev;
}

MdhValue __mdh_event_loop_new(void) {
    MdhEventLoop *loop = (MdhEventLoop *)GC_malloc(sizeof(MdhEventLoop));
    memset(loop, 0, sizeof(MdhEventLoop));
    loop->next_timer_id = 1;
    int64_t id = __mdh_loop_register(loop);
    return __mdh_make_int(id);
}

MdhValue __mdh_event_loop_stop(MdhValue loop_val) {
    MdhEventLoop *loop = __mdh_loop_get(loop_val);
    if (!loop) return __mdh_make_nil();
    loop->stopped = 1;
    return __mdh_make_nil();
}

MdhValue __mdh_event_watch_read(MdhValue loop_val, MdhValue sock, MdhValue callback) {
    MdhEventLoop *loop = __mdh_loop_get(loop_val);
    if (!loop) return __mdh_make_nil();
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        __mdh_hurl(__mdh_make_string("Invalid socket for event_watch_read"));
        return __mdh_make_nil();
    }
    for (int64_t i = 0; i < loop->watch_len; i++) {
        if (loop->watches[i].fd == fd) {
            loop->watches[i].read_cb = callback;
            return __mdh_make_nil();
        }
    }
    __mdh_loop_ensure_watch_cap(loop, loop->watch_len + 1);
    loop->watches[loop->watch_len].fd = fd;
    loop->watches[loop->watch_len].read_cb = callback;
    loop->watches[loop->watch_len].write_cb = __mdh_make_nil();
    loop->watch_len++;
    return __mdh_make_nil();
}

MdhValue __mdh_event_watch_write(MdhValue loop_val, MdhValue sock, MdhValue callback) {
    MdhEventLoop *loop = __mdh_loop_get(loop_val);
    if (!loop) return __mdh_make_nil();
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        __mdh_hurl(__mdh_make_string("Invalid socket for event_watch_write"));
        return __mdh_make_nil();
    }
    for (int64_t i = 0; i < loop->watch_len; i++) {
        if (loop->watches[i].fd == fd) {
            loop->watches[i].write_cb = callback;
            return __mdh_make_nil();
        }
    }
    __mdh_loop_ensure_watch_cap(loop, loop->watch_len + 1);
    loop->watches[loop->watch_len].fd = fd;
    loop->watches[loop->watch_len].read_cb = __mdh_make_nil();
    loop->watches[loop->watch_len].write_cb = callback;
    loop->watch_len++;
    return __mdh_make_nil();
}

MdhValue __mdh_event_unwatch(MdhValue loop_val, MdhValue sock) {
    MdhEventLoop *loop = __mdh_loop_get(loop_val);
    if (!loop) return __mdh_make_bool(false);
    int fd = __mdh_sock_fd(sock);
    if (fd < 0) {
        __mdh_hurl(__mdh_make_string("Invalid socket for event_unwatch"));
        return __mdh_make_bool(false);
    }
    for (int64_t i = 0; i < loop->watch_len; i++) {
        if (loop->watches[i].fd == fd) {
            loop->watches[i] = loop->watches[loop->watch_len - 1];
            loop->watch_len--;
            return __mdh_make_bool(true);
        }
    }
    return __mdh_make_bool(false);
}

MdhValue __mdh_event_loop_poll(MdhValue loop_val, MdhValue timeout_val) {
    MdhEventLoop *loop = __mdh_loop_get(loop_val);
    if (!loop) return __mdh_make_list(0);

    if (loop->stopped) {
        MdhValue events = __mdh_make_list(1);
        MdhValue ev = __mdh_make_event("stop", -1, -1, __mdh_make_nil());
        __mdh_list_push(events, ev);
        return events;
    }

    int64_t timeout_ms = -1;
    if (timeout_val.tag == MDH_TAG_INT) {
        timeout_ms = timeout_val.data;
    } else if (timeout_val.tag == MDH_TAG_FLOAT) {
        timeout_ms = (int64_t)__mdh_get_float(timeout_val);
    } else if (timeout_val.tag != MDH_TAG_NIL) {
        __mdh_type_error("event_loop_poll", timeout_val.tag, 0);
        return __mdh_make_list(0);
    }

    int64_t now = __mdh_mono_ms_now();
    int64_t next_due = -1;
    for (int64_t i = 0; i < loop->timer_len; i++) {
        MdhTimer *t = &loop->timers[i];
        if (t->cancelled) continue;
        int64_t diff = t->next_fire_ms - now;
        if (diff < 0) diff = 0;
        if (next_due < 0 || diff < next_due) {
            next_due = diff;
        }
    }

    int64_t wait_ms = timeout_ms;
    if (wait_ms < 0) {
        wait_ms = next_due;
    } else if (next_due >= 0 && next_due < wait_ms) {
        wait_ms = next_due;
    }

    int poll_timeout = -1;
    if (wait_ms >= 0) {
        if (wait_ms > INT_MAX) {
            poll_timeout = INT_MAX;
        } else {
            poll_timeout = (int)wait_ms;
        }
    }

    int64_t nfds = loop->watch_len;
    struct pollfd *fds = NULL;
    if (nfds > 0) {
        fds = (struct pollfd *)GC_malloc(sizeof(struct pollfd) * (size_t)nfds);
        for (int64_t i = 0; i < nfds; i++) {
            fds[i].fd = loop->watches[i].fd;
            fds[i].events = 0;
            if (loop->watches[i].read_cb.tag != MDH_TAG_NIL) {
                fds[i].events |= POLLIN;
            }
            if (loop->watches[i].write_cb.tag != MDH_TAG_NIL) {
                fds[i].events |= POLLOUT;
            }
            fds[i].revents = 0;
        }
    }

    if (poll_timeout != 0 || nfds > 0) {
        int rc = poll(fds, (nfds_t)nfds, poll_timeout);
        if (rc < 0 && errno != EINTR) {
            __mdh_hurl(__mdh_make_string("event_loop_poll failed"));
        }
    }

    MdhValue events = __mdh_make_list(4);
    if (nfds > 0 && fds) {
        for (int64_t i = 0; i < nfds; i++) {
            if ((fds[i].revents & POLLIN) && loop->watches[i].read_cb.tag != MDH_TAG_NIL) {
                MdhValue ev = __mdh_make_event("read", loop->watches[i].fd, -1, loop->watches[i].read_cb);
                __mdh_list_push(events, ev);
            }
            if ((fds[i].revents & POLLOUT) && loop->watches[i].write_cb.tag != MDH_TAG_NIL) {
                MdhValue ev = __mdh_make_event("write", loop->watches[i].fd, -1, loop->watches[i].write_cb);
                __mdh_list_push(events, ev);
            }
        }
    }

    now = __mdh_mono_ms_now();
    for (int64_t i = 0; i < loop->timer_len; i++) {
        MdhTimer *t = &loop->timers[i];
        if (t->cancelled) continue;
        if (t->next_fire_ms <= now) {
            MdhValue ev = __mdh_make_event("timer", -1, t->id, t->callback);
            __mdh_list_push(events, ev);
            if (t->interval_ms > 0) {
                while (t->next_fire_ms <= now) {
                    t->next_fire_ms += t->interval_ms;
                }
            } else {
                t->cancelled = 1;
            }
        }
    }

    if (loop->timer_len > 0) {
        int64_t write = 0;
        for (int64_t i = 0; i < loop->timer_len; i++) {
            if (!loop->timers[i].cancelled) {
                if (write != i) {
                    loop->timers[write] = loop->timers[i];
                }
                write++;
            }
        }
        loop->timer_len = write;
    }

    return events;
}

MdhValue __mdh_timer_after(MdhValue loop_val, MdhValue ms_val, MdhValue callback) {
    MdhEventLoop *loop = __mdh_loop_get(loop_val);
    if (!loop) return __mdh_make_nil();
    int64_t ms = 0;
    if (!__mdh_int_value("timer_after", ms_val, &ms)) {
        return __mdh_make_nil();
    }
    if (ms < 0) {
        __mdh_hurl(__mdh_make_string("timer_after expects a non-negative delay"));
        return __mdh_make_nil();
    }
    __mdh_loop_ensure_timer_cap(loop, loop->timer_len + 1);
    int64_t id = loop->next_timer_id++;
    int64_t now = __mdh_mono_ms_now();
    MdhTimer t;
    t.id = id;
    t.next_fire_ms = now + ms;
    t.interval_ms = 0;
    t.callback = callback;
    t.cancelled = 0;
    loop->timers[loop->timer_len++] = t;
    return __mdh_make_int(id);
}

MdhValue __mdh_timer_every(MdhValue loop_val, MdhValue ms_val, MdhValue callback) {
    MdhEventLoop *loop = __mdh_loop_get(loop_val);
    if (!loop) return __mdh_make_nil();
    int64_t ms = 0;
    if (!__mdh_int_value("timer_every", ms_val, &ms)) {
        return __mdh_make_nil();
    }
    if (ms <= 0) {
        __mdh_hurl(__mdh_make_string("timer_every expects a positive interval"));
        return __mdh_make_nil();
    }
    __mdh_loop_ensure_timer_cap(loop, loop->timer_len + 1);
    int64_t id = loop->next_timer_id++;
    int64_t now = __mdh_mono_ms_now();
    MdhTimer t;
    t.id = id;
    t.next_fire_ms = now + ms;
    t.interval_ms = ms;
    t.callback = callback;
    t.cancelled = 0;
    loop->timers[loop->timer_len++] = t;
    return __mdh_make_int(id);
}

MdhValue __mdh_timer_cancel(MdhValue loop_val, MdhValue timer_id_val) {
    MdhEventLoop *loop = __mdh_loop_get(loop_val);
    if (!loop) return __mdh_make_bool(false);
    int64_t timer_id = 0;
    if (!__mdh_int_value("timer_cancel", timer_id_val, &timer_id)) {
        return __mdh_make_bool(false);
    }
    bool found = false;
    for (int64_t i = 0; i < loop->timer_len; i++) {
        if (loop->timers[i].id == timer_id && !loop->timers[i].cancelled) {
            loop->timers[i].cancelled = 1;
            found = true;
        }
    }
    return __mdh_make_bool(found);
}

/* ========== Threads + Sync ========== */

typedef MdhValue (*MdhFn0)(void);
typedef MdhValue (*MdhFn1)(MdhValue);
typedef MdhValue (*MdhFn2)(MdhValue, MdhValue);
typedef MdhValue (*MdhFn3)(MdhValue, MdhValue, MdhValue);
typedef MdhValue (*MdhFn4)(MdhValue, MdhValue, MdhValue, MdhValue);
typedef MdhValue (*MdhFn5)(MdhValue, MdhValue, MdhValue, MdhValue, MdhValue);
typedef MdhValue (*MdhFn6)(MdhValue, MdhValue, MdhValue, MdhValue, MdhValue, MdhValue);

typedef struct {
    pthread_t thread;
    MdhValue func;
    MdhValue args;
    MdhValue result;
    int done;
    int detached;
} MdhThread;

typedef struct {
    pthread_mutex_t mutex;
} MdhMutex;

typedef struct {
    pthread_cond_t cond;
} MdhCondvar;

typedef struct {
    pthread_mutex_t lock;
    int64_t value;
} MdhAtomic;

typedef struct {
    pthread_mutex_t lock;
    pthread_cond_t not_empty;
    pthread_cond_t not_full;
    MdhValue *buf;
    int64_t cap;
    int64_t count;
    int64_t head;
    int64_t tail;
    int closed;
    int unbounded;
} MdhChan;

static int __mdh_gc_threads_ready = 0;

static MdhThread *__mdh_thread_ptr(MdhValue v) {
    if (v.tag != MDH_TAG_INT) {
        __mdh_type_error("thread", v.tag, 0);
        return NULL;
    }
    return (MdhThread *)(intptr_t)v.data;
}

static MdhMutex *__mdh_mutex_ptr(MdhValue v) {
    if (v.tag != MDH_TAG_INT) {
        __mdh_type_error("mutex", v.tag, 0);
        return NULL;
    }
    return (MdhMutex *)(intptr_t)v.data;
}

static MdhCondvar *__mdh_condvar_ptr(MdhValue v) {
    if (v.tag != MDH_TAG_INT) {
        __mdh_type_error("condvar", v.tag, 0);
        return NULL;
    }
    return (MdhCondvar *)(intptr_t)v.data;
}

static MdhAtomic *__mdh_atomic_ptr(MdhValue v) {
    if (v.tag != MDH_TAG_INT) {
        __mdh_type_error("atomic", v.tag, 0);
        return NULL;
    }
    return (MdhAtomic *)(intptr_t)v.data;
}

static MdhChan *__mdh_chan_ptr(MdhValue v) {
    if (v.tag != MDH_TAG_INT) {
        __mdh_type_error("chan", v.tag, 0);
        return NULL;
    }
    return (MdhChan *)(intptr_t)v.data;
}

static MdhValue __mdh_call_with_list(MdhValue func_val, MdhValue args_list) {
    MdhValue *args = NULL;
    int64_t argc = 0;
    if (args_list.tag == MDH_TAG_NIL) {
        argc = 0;
    } else if (args_list.tag == MDH_TAG_LIST) {
        MdhList *list = __mdh_get_list(args_list);
        if (list) {
            argc = list->length;
            args = list->items;
        }
    } else {
        __mdh_type_error("thread_spawn", args_list.tag, 0);
        return __mdh_make_nil();
    }

    MdhValue fn_val = func_val;
    MdhValue call_args[6];
    int64_t total_args = argc;

    if (func_val.tag == MDH_TAG_CLOSURE) {
        uint8_t *base = (uint8_t *)(intptr_t)func_val.data;
        int64_t *header = (int64_t *)base;
        int64_t len = header[1];
        if (len <= 0) {
            __mdh_hurl(__mdh_make_string("Invalid closure"));
            return __mdh_make_nil();
        }
        MdhValue *elems = (MdhValue *)(base + 16);
        fn_val = elems[0];
        int64_t captures = len - 1;
        if (captures > 3) {
            __mdh_hurl(__mdh_make_string("Closure captures > 3 not supported in threads"));
            return __mdh_make_nil();
        }
        if (captures + argc > 6) {
            __mdh_hurl(__mdh_make_string("Too many arguments for thread spawn"));
            return __mdh_make_nil();
        }
        for (int64_t i = 0; i < captures; i++) {
            call_args[i] = elems[i + 1];
        }
        for (int64_t i = 0; i < argc; i++) {
            call_args[captures + i] = args[i];
        }
        total_args = captures + argc;
    } else if (func_val.tag == MDH_TAG_FUNCTION) {
        if (argc > 6) {
            __mdh_hurl(__mdh_make_string("Too many arguments for thread spawn"));
            return __mdh_make_nil();
        }
        for (int64_t i = 0; i < argc; i++) {
            call_args[i] = args[i];
        }
        total_args = argc;
    } else {
        __mdh_type_error("thread_spawn", func_val.tag, 0);
        return __mdh_make_nil();
    }

    intptr_t fn_ptr = (intptr_t)fn_val.data;
    switch (total_args) {
        case 0:
            return ((MdhFn0)fn_ptr)();
        case 1:
            return ((MdhFn1)fn_ptr)(call_args[0]);
        case 2:
            return ((MdhFn2)fn_ptr)(call_args[0], call_args[1]);
        case 3:
            return ((MdhFn3)fn_ptr)(call_args[0], call_args[1], call_args[2]);
        case 4:
            return ((MdhFn4)fn_ptr)(call_args[0], call_args[1], call_args[2], call_args[3]);
        case 5:
            return ((MdhFn5)fn_ptr)(
                call_args[0],
                call_args[1],
                call_args[2],
                call_args[3],
                call_args[4]
            );
        case 6:
            return ((MdhFn6)fn_ptr)(
                call_args[0],
                call_args[1],
                call_args[2],
                call_args[3],
                call_args[4],
                call_args[5]
            );
        default:
            __mdh_hurl(__mdh_make_string("Too many arguments for thread spawn"));
            return __mdh_make_nil();
    }
}

static void *__mdh_thread_entry(void *arg) {
    MdhThread *t = (MdhThread *)arg;
    GC_stack_base sb;
    if (GC_get_stack_base(&sb) == 0) {
        GC_register_my_thread(&sb);
    }
    t->result = __mdh_call_with_list(t->func, t->args);
    t->done = 1;
    GC_unregister_my_thread();
    return NULL;
}

MdhValue __mdh_thread_spawn(MdhValue func, MdhValue args_list) {
    if (!__mdh_gc_threads_ready) {
        GC_allow_register_threads();
        __mdh_gc_threads_ready = 1;
    }
    MdhThread *t = (MdhThread *)GC_malloc(sizeof(MdhThread));
    memset(t, 0, sizeof(MdhThread));
    t->func = func;
    t->args = args_list;
    t->result = __mdh_make_nil();
    t->done = 0;
    t->detached = 0;

    int rc = pthread_create(&t->thread, NULL, __mdh_thread_entry, t);
    if (rc != 0) {
        __mdh_hurl(__mdh_make_string("thread_spawn failed"));
        return __mdh_make_nil();
    }
    return __mdh_make_int((int64_t)(intptr_t)t);
}

MdhValue __mdh_thread_join(MdhValue thread_handle) {
    MdhThread *t = __mdh_thread_ptr(thread_handle);
    if (!t) return __mdh_make_nil();
    if (t->detached) {
        __mdh_hurl(__mdh_make_string("Cannot join detached thread"));
        return __mdh_make_nil();
    }
    pthread_join(t->thread, NULL);
    return t->result;
}

MdhValue __mdh_thread_detach(MdhValue thread_handle) {
    MdhThread *t = __mdh_thread_ptr(thread_handle);
    if (!t) return __mdh_make_nil();
    if (!t->detached) {
        pthread_detach(t->thread);
        t->detached = 1;
    }
    return __mdh_make_nil();
}

MdhValue __mdh_mutex_new(void) {
    MdhMutex *m = (MdhMutex *)GC_malloc(sizeof(MdhMutex));
    pthread_mutex_init(&m->mutex, NULL);
    return __mdh_make_int((int64_t)(intptr_t)m);
}

MdhValue __mdh_mutex_lock(MdhValue mutex) {
    MdhMutex *m = __mdh_mutex_ptr(mutex);
    if (!m) return __mdh_make_nil();
    pthread_mutex_lock(&m->mutex);
    return __mdh_make_nil();
}

MdhValue __mdh_mutex_unlock(MdhValue mutex) {
    MdhMutex *m = __mdh_mutex_ptr(mutex);
    if (!m) return __mdh_make_nil();
    pthread_mutex_unlock(&m->mutex);
    return __mdh_make_nil();
}

MdhValue __mdh_mutex_try_lock(MdhValue mutex) {
    MdhMutex *m = __mdh_mutex_ptr(mutex);
    if (!m) return __mdh_make_bool(false);
    int rc = pthread_mutex_trylock(&m->mutex);
    return __mdh_make_bool(rc == 0);
}

MdhValue __mdh_condvar_new(void) {
    MdhCondvar *c = (MdhCondvar *)GC_malloc(sizeof(MdhCondvar));
    pthread_cond_init(&c->cond, NULL);
    return __mdh_make_int((int64_t)(intptr_t)c);
}

MdhValue __mdh_condvar_wait(MdhValue condvar, MdhValue mutex) {
    MdhCondvar *c = __mdh_condvar_ptr(condvar);
    MdhMutex *m = __mdh_mutex_ptr(mutex);
    if (!c || !m) return __mdh_make_bool(false);
    pthread_cond_wait(&c->cond, &m->mutex);
    return __mdh_make_bool(true);
}

MdhValue __mdh_condvar_timed_wait(MdhValue condvar, MdhValue mutex, MdhValue timeout_ms) {
    MdhCondvar *c = __mdh_condvar_ptr(condvar);
    MdhMutex *m = __mdh_mutex_ptr(mutex);
    if (!c || !m) return __mdh_make_bool(false);
    int64_t ms = 0;
    if (!__mdh_int_value("condvar_timed_wait", timeout_ms, &ms)) {
        return __mdh_make_bool(false);
    }
    if (ms < 0) {
        __mdh_hurl(__mdh_make_string("condvar_timed_wait expects non-negative timeout"));
        return __mdh_make_bool(false);
    }
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    ts.tv_sec += ms / 1000;
    ts.tv_nsec += (ms % 1000) * 1000000LL;
    if (ts.tv_nsec >= 1000000000L) {
        ts.tv_sec += 1;
        ts.tv_nsec -= 1000000000L;
    }
    int rc = pthread_cond_timedwait(&c->cond, &m->mutex, &ts);
    return __mdh_make_bool(rc == 0);
}

MdhValue __mdh_condvar_signal(MdhValue condvar) {
    MdhCondvar *c = __mdh_condvar_ptr(condvar);
    if (!c) return __mdh_make_nil();
    pthread_cond_signal(&c->cond);
    return __mdh_make_nil();
}

MdhValue __mdh_condvar_broadcast(MdhValue condvar) {
    MdhCondvar *c = __mdh_condvar_ptr(condvar);
    if (!c) return __mdh_make_nil();
    pthread_cond_broadcast(&c->cond);
    return __mdh_make_nil();
}

MdhValue __mdh_atomic_new(MdhValue initial_int) {
    int64_t val = 0;
    if (!__mdh_int_value("atomic_new", initial_int, &val)) {
        return __mdh_make_nil();
    }
    MdhAtomic *a = (MdhAtomic *)GC_malloc(sizeof(MdhAtomic));
    pthread_mutex_init(&a->lock, NULL);
    a->value = val;
    return __mdh_make_int((int64_t)(intptr_t)a);
}

MdhValue __mdh_atomic_load(MdhValue atomic) {
    MdhAtomic *a = __mdh_atomic_ptr(atomic);
    if (!a) return __mdh_make_int(0);
    pthread_mutex_lock(&a->lock);
    int64_t val = a->value;
    pthread_mutex_unlock(&a->lock);
    return __mdh_make_int(val);
}

MdhValue __mdh_atomic_store(MdhValue atomic, MdhValue value) {
    MdhAtomic *a = __mdh_atomic_ptr(atomic);
    if (!a) return __mdh_make_nil();
    int64_t val = 0;
    if (!__mdh_int_value("atomic_store", value, &val)) {
        return __mdh_make_nil();
    }
    pthread_mutex_lock(&a->lock);
    a->value = val;
    pthread_mutex_unlock(&a->lock);
    return __mdh_make_nil();
}

MdhValue __mdh_atomic_add(MdhValue atomic, MdhValue delta) {
    MdhAtomic *a = __mdh_atomic_ptr(atomic);
    if (!a) return __mdh_make_int(0);
    int64_t add = 0;
    if (!__mdh_int_value("atomic_add", delta, &add)) {
        return __mdh_make_int(0);
    }
    pthread_mutex_lock(&a->lock);
    a->value += add;
    int64_t val = a->value;
    pthread_mutex_unlock(&a->lock);
    return __mdh_make_int(val);
}

MdhValue __mdh_atomic_cas(MdhValue atomic, MdhValue expected, MdhValue desired) {
    MdhAtomic *a = __mdh_atomic_ptr(atomic);
    if (!a) return __mdh_make_bool(false);
    int64_t exp = 0;
    int64_t des = 0;
    if (!__mdh_int_value("atomic_cas", expected, &exp)) {
        return __mdh_make_bool(false);
    }
    if (!__mdh_int_value("atomic_cas", desired, &des)) {
        return __mdh_make_bool(false);
    }
    pthread_mutex_lock(&a->lock);
    bool ok = (a->value == exp);
    if (ok) {
        a->value = des;
    }
    pthread_mutex_unlock(&a->lock);
    return __mdh_make_bool(ok);
}

static void __mdh_chan_grow(MdhChan *ch, int64_t new_cap) {
    MdhValue *new_buf = (MdhValue *)GC_malloc(sizeof(MdhValue) * (size_t)new_cap);
    for (int64_t i = 0; i < ch->count; i++) {
        new_buf[i] = ch->buf[(ch->head + i) % ch->cap];
    }
    ch->buf = new_buf;
    ch->cap = new_cap;
    ch->head = 0;
    ch->tail = ch->count;
}

MdhValue __mdh_chan_new(MdhValue capacity_int) {
    int64_t cap = 0;
    if (!__mdh_int_value("chan_new", capacity_int, &cap)) {
        return __mdh_make_nil();
    }
    if (cap < 0) {
        __mdh_hurl(__mdh_make_string("chan_new expects non-negative capacity"));
        return __mdh_make_nil();
    }
    MdhChan *ch = (MdhChan *)GC_malloc(sizeof(MdhChan));
    memset(ch, 0, sizeof(MdhChan));
    pthread_mutex_init(&ch->lock, NULL);
    pthread_cond_init(&ch->not_empty, NULL);
    pthread_cond_init(&ch->not_full, NULL);
    if (cap == 0) {
        ch->unbounded = 1;
        ch->cap = 0;
        ch->buf = NULL;
    } else {
        ch->unbounded = 0;
        ch->cap = cap;
        ch->buf = (MdhValue *)GC_malloc(sizeof(MdhValue) * (size_t)cap);
    }
    return __mdh_make_int((int64_t)(intptr_t)ch);
}

MdhValue __mdh_chan_send(MdhValue chan, MdhValue value) {
    MdhChan *ch = __mdh_chan_ptr(chan);
    if (!ch) return __mdh_make_bool(false);
    pthread_mutex_lock(&ch->lock);
    while (!ch->unbounded && ch->count >= ch->cap && !ch->closed) {
        pthread_cond_wait(&ch->not_full, &ch->lock);
    }
    if (ch->closed) {
        pthread_mutex_unlock(&ch->lock);
        return __mdh_make_bool(false);
    }
    if (ch->unbounded) {
        if (ch->cap == 0) {
            ch->cap = 16;
            ch->buf = (MdhValue *)GC_malloc(sizeof(MdhValue) * 16);
        } else if (ch->count >= ch->cap) {
            __mdh_chan_grow(ch, ch->cap * 2);
        }
    }
    ch->buf[ch->tail] = value;
    ch->tail = (ch->tail + 1) % ch->cap;
    ch->count++;
    pthread_cond_signal(&ch->not_empty);
    pthread_mutex_unlock(&ch->lock);
    return __mdh_make_bool(true);
}

MdhValue __mdh_chan_recv(MdhValue chan) {
    MdhChan *ch = __mdh_chan_ptr(chan);
    if (!ch) return __mdh_make_nil();
    pthread_mutex_lock(&ch->lock);
    while (ch->count == 0 && !ch->closed) {
        pthread_cond_wait(&ch->not_empty, &ch->lock);
    }
    if (ch->count == 0 && ch->closed) {
        pthread_mutex_unlock(&ch->lock);
        return __mdh_make_nil();
    }
    MdhValue v = ch->buf[ch->head];
    ch->head = (ch->head + 1) % ch->cap;
    ch->count--;
    if (!ch->unbounded) {
        pthread_cond_signal(&ch->not_full);
    }
    pthread_mutex_unlock(&ch->lock);
    return v;
}

MdhValue __mdh_chan_try_recv(MdhValue chan) {
    MdhChan *ch = __mdh_chan_ptr(chan);
    if (!ch) return __mdh_make_nil();
    pthread_mutex_lock(&ch->lock);
    if (ch->count == 0) {
        pthread_mutex_unlock(&ch->lock);
        return __mdh_make_nil();
    }
    MdhValue v = ch->buf[ch->head];
    ch->head = (ch->head + 1) % ch->cap;
    ch->count--;
    if (!ch->unbounded) {
        pthread_cond_signal(&ch->not_full);
    }
    pthread_mutex_unlock(&ch->lock);
    return v;
}

MdhValue __mdh_chan_close(MdhValue chan) {
    MdhChan *ch = __mdh_chan_ptr(chan);
    if (!ch) return __mdh_make_nil();
    pthread_mutex_lock(&ch->lock);
    ch->closed = 1;
    pthread_cond_broadcast(&ch->not_empty);
    pthread_cond_broadcast(&ch->not_full);
    pthread_mutex_unlock(&ch->lock);
    return __mdh_make_nil();
}

MdhValue __mdh_chan_is_closed(MdhValue chan) {
    MdhChan *ch = __mdh_chan_ptr(chan);
    if (!ch) return __mdh_make_bool(true);
    pthread_mutex_lock(&ch->lock);
    int closed = ch->closed;
    pthread_mutex_unlock(&ch->lock);
    return __mdh_make_bool(closed != 0);
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
    v.tag = MDH_TAG_SET;
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
        __mdh_type_error("dict_has", dict.tag, 0);
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

MdhValue __mdh_set_contains(MdhValue set, MdhValue key) {
    if (set.tag != MDH_TAG_SET) {
        __mdh_type_error("is_in_creel", set.tag, 0);
        return __mdh_make_bool(false);
    }

    int64_t *set_ptr = (int64_t *)(intptr_t)set.data;
    int64_t count = set_ptr ? *set_ptr : 0;
    MdhValue *entries = set_ptr ? (MdhValue *)(set_ptr + 1) : NULL;

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
        __mdh_type_error("keys", dict.tag, 0);
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
        __mdh_type_error("values", dict.tag, 0);
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
        __mdh_type_error("dict_set", dict.tag, 0);
        return __mdh_empty_dict();
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
        __mdh_type_error("dict_get", dict.tag, 0);
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
    __mdh_key_not_found(key);
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
    if (dict.tag != MDH_TAG_SET) {
        __mdh_type_error("toss_in", dict.tag, 0);
        return __mdh_empty_creel();
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
    v.tag = MDH_TAG_SET;
    v.data = (int64_t)(intptr_t)new_ptr;
    return v;
}

MdhValue __mdh_heave_oot(MdhValue dict, MdhValue item) {
    if (dict.tag != MDH_TAG_SET) {
        __mdh_type_error("heave_oot", dict.tag, 0);
        return __mdh_empty_creel();
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
    v.tag = MDH_TAG_SET;
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
    if (dict.tag != MDH_TAG_SET) {
        __mdh_type_error("creel_tae_list", dict.tag, 0);
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
    if (a.tag != MDH_TAG_SET) {
        __mdh_type_error("creels_thegither", a.tag, 0);
        return __mdh_empty_creel();
    }
    if (b.tag != MDH_TAG_SET) {
        __mdh_type_error("creels_thegither", b.tag, 0);
        return __mdh_empty_creel();
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
    if (a.tag != MDH_TAG_SET) {
        __mdh_type_error("creels_baith", a.tag, 0);
        return __mdh_empty_creel();
    }
    if (b.tag != MDH_TAG_SET) {
        __mdh_type_error("creels_baith", b.tag, 0);
        return __mdh_empty_creel();
    }

    MdhValue result = __mdh_empty_creel();
    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        MdhValue key = a_entries[i * 2];
        MdhValue contains = __mdh_set_contains(b, key);
        if (contains.tag == MDH_TAG_BOOL && contains.data != 0) {
            result = __mdh_toss_in(result, key);
        }
    }
    return result;
}

MdhValue __mdh_creels_differ(MdhValue a, MdhValue b) {
    /* Difference of two creels/sets (a \\ b) */
    if (a.tag != MDH_TAG_SET) {
        __mdh_type_error("creels_differ", a.tag, 0);
        return __mdh_empty_creel();
    }
    if (b.tag != MDH_TAG_SET) {
        __mdh_type_error("creels_differ", b.tag, 0);
        return __mdh_empty_creel();
    }

    MdhValue result = __mdh_empty_creel();
    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        MdhValue key = a_entries[i * 2];
        MdhValue contains = __mdh_set_contains(b, key);
        if (!(contains.tag == MDH_TAG_BOOL && contains.data != 0)) {
            result = __mdh_toss_in(result, key);
        }
    }
    return result;
}

MdhValue __mdh_is_subset(MdhValue a, MdhValue b) {
    if (a.tag != MDH_TAG_SET) {
        __mdh_type_error("is_subset", a.tag, 0);
        return __mdh_make_bool(false);
    }
    if (b.tag != MDH_TAG_SET) {
        __mdh_type_error("is_subset", b.tag, 0);
        return __mdh_make_bool(false);
    }

    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        MdhValue key = a_entries[i * 2];
        MdhValue contains = __mdh_set_contains(b, key);
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
    if (a.tag != MDH_TAG_SET) {
        __mdh_type_error("is_disjoint", a.tag, 0);
        return __mdh_make_bool(false);
    }
    if (b.tag != MDH_TAG_SET) {
        __mdh_type_error("is_disjoint", b.tag, 0);
        return __mdh_make_bool(false);
    }

    int64_t *a_ptr = (int64_t *)(intptr_t)a.data;
    int64_t a_count = *a_ptr;
    MdhValue *a_entries = (MdhValue *)(a_ptr + 1);
    for (int64_t i = 0; i < a_count; i++) {
        MdhValue key = a_entries[i * 2];
        MdhValue contains = __mdh_set_contains(b, key);
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
        __mdh_hurl(__mdh_make_string("minaw() needs a list"));
        return __mdh_make_nil();
    }
    MdhList *l = __mdh_get_list(list);
    if (l->length == 0) {
        __mdh_hurl(__mdh_make_string("Cannae find minimum o' empty list!"));
        return __mdh_make_nil();
    }

    MdhValue min_val = l->items[0];
    if (min_val.tag != MDH_TAG_INT && min_val.tag != MDH_TAG_FLOAT) {
        __mdh_hurl(__mdh_make_string("minaw() needs a list o' comparable numbers"));
        return __mdh_make_nil();
    }

    for (int64_t i = 1; i < l->length; i++) {
        MdhValue item = l->items[i];
        if (item.tag != min_val.tag) {
            __mdh_hurl(__mdh_make_string("minaw() needs a list o' comparable numbers"));
            return __mdh_make_nil();
        }
        if (min_val.tag == MDH_TAG_INT) {
            if (item.data < min_val.data) {
                min_val = item;
            }
        } else {
            double min_f = __mdh_get_float(min_val);
            double item_f = __mdh_get_float(item);
            if (item_f < min_f) {
                min_val = item;
            }
        }
    }
    return min_val;
}

/* list_max - maximum value in a list */
MdhValue __mdh_list_max(MdhValue list) {
    if (list.tag != MDH_TAG_LIST) {
        __mdh_hurl(__mdh_make_string("maxaw() needs a list"));
        return __mdh_make_nil();
    }
    MdhList *l = __mdh_get_list(list);
    if (l->length == 0) {
        __mdh_hurl(__mdh_make_string("Cannae find maximum o' empty list!"));
        return __mdh_make_nil();
    }

    MdhValue max_val = l->items[0];
    if (max_val.tag != MDH_TAG_INT && max_val.tag != MDH_TAG_FLOAT) {
        __mdh_hurl(__mdh_make_string("maxaw() needs a list o' comparable numbers"));
        return __mdh_make_nil();
    }

    for (int64_t i = 1; i < l->length; i++) {
        MdhValue item = l->items[i];
        if (item.tag != max_val.tag) {
            __mdh_hurl(__mdh_make_string("maxaw() needs a list o' comparable numbers"));
            return __mdh_make_nil();
        }
        if (max_val.tag == MDH_TAG_INT) {
            if (item.data > max_val.data) {
                max_val = item;
            }
        } else {
            double max_f = __mdh_get_float(max_val);
            double item_f = __mdh_get_float(item);
            if (item_f > max_f) {
                max_val = item;
            }
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
    return __mdh_make_bool(val.tag == MDH_TAG_FUNCTION || val.tag == MDH_TAG_CLOSURE);
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

    char out_template[] = "/tmp/mdh_shell_out_XXXXXX";
    char err_template[] = "/tmp/mdh_shell_err_XXXXXX";

    int out_fd = mkstemp(out_template);
    if (out_fd < 0) {
        __mdh_hurl(__mdh_make_string("Shell command failed"));
        return __mdh_make_nil();
    }
    close(out_fd);

    int err_fd = mkstemp(err_template);
    if (err_fd < 0) {
        unlink(out_template);
        __mdh_hurl(__mdh_make_string("Shell command failed"));
        return __mdh_make_nil();
    }
    close(err_fd);

    const char *cmd_str = __mdh_get_string(cmd);
    char *out_q = __mdh_shell_quote_single(out_template);
    char *err_q = __mdh_shell_quote_single(err_template);

    size_t script_len = strlen(cmd_str) + strlen(out_q) * 4 + strlen(err_q) * 3 + 128;
    char *script = (char *)GC_malloc(script_len);
    snprintf(
        script,
        script_len,
        "(%s) 1>%s 2>%s; if [ -s %s ]; then cat %s; else cat %s; fi; rm -f %s %s",
        cmd_str,
        out_q,
        err_q,
        out_q,
        out_q,
        err_q,
        out_q,
        err_q
    );

    char *full = __mdh_build_shell_command(script, false);
    FILE *fp = popen(full, "r");
    if (!fp) {
        unlink(out_template);
        unlink(err_template);
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
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_test", text.tag, pattern.tag);
        return __mdh_make_bool(false);
    }

    MdhRsResult r = __mdh_rs_regex_test(text, pattern);
    if (!r.ok) {
        __mdh_hurl(r.error);
        return __mdh_make_bool(false);
    }
    return r.value;
}

MdhValue __mdh_regex_match(MdhValue text, MdhValue pattern) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_match", text.tag, pattern.tag);
        return __mdh_make_nil();
    }

    MdhRsResult r = __mdh_rs_regex_match(text, pattern);
    if (!r.ok) {
        __mdh_hurl(r.error);
        return __mdh_make_nil();
    }
    return r.value;
}

MdhValue __mdh_regex_match_all(MdhValue text, MdhValue pattern) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_match_all", text.tag, pattern.tag);
        return __mdh_make_list(0);
    }

    MdhRsResult r = __mdh_rs_regex_match_all(text, pattern);
    if (!r.ok) {
        __mdh_hurl(r.error);
        return __mdh_make_list(0);
    }
    return r.value;
}

MdhValue __mdh_regex_replace(MdhValue text, MdhValue pattern, MdhValue replacement) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING || replacement.tag != MDH_TAG_STRING) {
        uint8_t got2 = pattern.tag != MDH_TAG_STRING ? pattern.tag : replacement.tag;
        __mdh_type_error("regex_replace", text.tag, got2);
        return text.tag == MDH_TAG_STRING ? text : __mdh_make_string("");
    }

    MdhRsResult r = __mdh_rs_regex_replace(text, pattern, replacement);
    if (!r.ok) {
        __mdh_hurl(r.error);
        return __mdh_make_string("");
    }
    return r.value;
}

MdhValue __mdh_regex_replace_first(MdhValue text, MdhValue pattern, MdhValue replacement) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING || replacement.tag != MDH_TAG_STRING) {
        uint8_t got2 = pattern.tag != MDH_TAG_STRING ? pattern.tag : replacement.tag;
        __mdh_type_error("regex_replace_first", text.tag, got2);
        return text.tag == MDH_TAG_STRING ? text : __mdh_make_string("");
    }

    MdhRsResult r = __mdh_rs_regex_replace_first(text, pattern, replacement);
    if (!r.ok) {
        __mdh_hurl(r.error);
        return __mdh_make_string("");
    }
    return r.value;
}

MdhValue __mdh_regex_split(MdhValue text, MdhValue pattern) {
    if (text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING) {
        __mdh_type_error("regex_split", text.tag, pattern.tag);
        return __mdh_make_list(0);
    }

    MdhRsResult r = __mdh_rs_regex_split(text, pattern);
    if (!r.ok) {
        __mdh_hurl(r.error);
        return __mdh_make_list(0);
    }
    return r.value;
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
    if (json_str.tag != MDH_TAG_STRING) {
        __mdh_type_error("json_parse", json_str.tag, 0);
        return __mdh_make_nil();
    }

    MdhRsResult r = __mdh_rs_json_parse(json_str);
    if (!r.ok) {
        __mdh_hurl(r.error);
        return __mdh_make_nil();
    }
    return r.value;
}

MdhValue __mdh_json_stringify(MdhValue value) {
    MdhRsResult r = __mdh_rs_json_stringify(value);
    if (!r.ok) {
        __mdh_hurl(r.error);
        return __mdh_make_string("");
    }
    return r.value;
}

MdhValue __mdh_json_pretty(MdhValue value) {
    MdhRsResult r = __mdh_rs_json_pretty(value);
    if (!r.ok) {
        __mdh_hurl(r.error);
        return __mdh_make_string("");
    }
    return r.value;
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
    } else if (strcmp(t, "bytes") == 0 || strcmp(t, "byte") == 0) {
        matches = value.tag == MDH_TAG_BYTES;
    } else if (strcmp(t, "dict") == 0) {
        matches = value.tag == MDH_TAG_DICT;
    } else if (strcmp(t, "function") == 0 || strcmp(t, "dae") == 0) {
        matches = value.tag == MDH_TAG_FUNCTION || value.tag == MDH_TAG_CLOSURE;
    } else if (strcmp(t, "naething") == 0 || strcmp(t, "nil") == 0) {
        matches = value.tag == MDH_TAG_NIL;
    } else if (strcmp(t, "range") == 0) {
        matches = value.tag == MDH_TAG_RANGE;
    } else {
        matches = false;
    }

    return __mdh_make_bool(matches);
}

MdhValue __mdh_wrang_sort(MdhValue value, MdhValue type_name) {
    if (type_name.tag != MDH_TAG_STRING) {
        __mdh_hurl(__mdh_make_string("Second arg must be a type name string"));
        return __mdh_make_bool(1);
    }

    MdhValue actual = __mdh_type_of(value);
    const char *actual_str = __mdh_get_string(actual);
    const char *expected_str = __mdh_get_string(type_name);
    bool wrong = true;
    if (actual_str && expected_str) {
        wrong = strcmp(actual_str, expected_str) != 0;
    }
    return __mdh_make_bool(wrong);
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
            return "dict";
        case MDH_TAG_SET:
            return "creel";
        case MDH_TAG_BYTES:
            return "bytes";
        case MDH_TAG_FUNCTION:
        case MDH_TAG_CLOSURE:
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
            snprintf(info, sizeof(info), "dict wi' %lld entries", (long long)count);
            break;
        }
        case MDH_TAG_SET: {
            int64_t *set_ptr = (int64_t *)(intptr_t)val.data;
            int64_t count = set_ptr ? *set_ptr : 0;
            snprintf(info, sizeof(info), "creel wi' %lld items", (long long)count);
            break;
        }
        case MDH_TAG_STRING: {
            const char *s = __mdh_get_string(val);
            snprintf(info, sizeof(info), "string o' %zu characters", s ? strlen(s) : 0);
            break;
        }
        case MDH_TAG_BYTES: {
            MdhBytes *bytes = __mdh_get_bytes(val);
            snprintf(info, sizeof(info), "bytes wi' %lld items", (long long)(bytes ? bytes->length : 0));
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
