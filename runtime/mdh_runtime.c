/*
 * mdhavers Runtime Library - Implementation
 *
 * Provides runtime support for compiled mdhavers programs.
 * Uses Boehm GC for memory management.
 */

#include "mdh_runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include <unistd.h>
#include <termios.h>
#include <sys/ioctl.h>

/* Boehm GC - declared as extern */
extern void GC_init(void);
extern void *GC_malloc(size_t size);
extern void *GC_realloc(void *ptr, size_t size);
extern char *GC_strdup(const char *s);

/* Random number generator state */
static int __mdh_random_initialized = 0;

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
            MdhString *s = (MdhString *)(intptr_t)a.data;
            return s && s->length > 0;
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

    fprintf(stderr, "Och! Type error in '%s': ", op);
    if (got1 < 11) {
        fprintf(stderr, "got %s", type_names[got1]);
    }
    if (got2 > 0 && got2 < 11) {
        fprintf(stderr, " and %s", type_names[got2]);
    }
    fprintf(stderr, "\n");
    exit(1);
}

MdhValue __mdh_type_of(MdhValue a) {
    static const char *type_names[] = {
        "naething", "bool", "integer", "float", "string",
        "list", "dict", "function", "class", "instance", "range"
    };

    if (a.tag < 11) {
        return __mdh_make_string(type_names[a.tag]);
    }
    return __mdh_make_string("unknown");
}

/* ========== I/O ========== */

void __mdh_blether(MdhValue a) {
    switch (a.tag) {
        case MDH_TAG_NIL:
            printf("naething\n");
            break;
        case MDH_TAG_BOOL:
            printf("%s\n", a.data ? "aye" : "nae");
            break;
        case MDH_TAG_INT:
            printf("%lld\n", (long long)a.data);
            break;
        case MDH_TAG_FLOAT:
            printf("%g\n", __mdh_get_float(a));
            break;
        case MDH_TAG_STRING:
            printf("%s\n", __mdh_get_string(a));
            break;
        case MDH_TAG_LIST: {
            MdhList *list = __mdh_get_list(a);
            printf("[");
            for (int64_t i = 0; i < list->length; i++) {
                if (i > 0) printf(", ");
                /* Print element without newline */
                MdhValue elem = list->items[i];
                switch (elem.tag) {
                    case MDH_TAG_NIL: printf("naething"); break;
                    case MDH_TAG_BOOL: printf("%s", elem.data ? "aye" : "nae"); break;
                    case MDH_TAG_INT: printf("%lld", (long long)elem.data); break;
                    case MDH_TAG_FLOAT: printf("%g", __mdh_get_float(elem)); break;
                    case MDH_TAG_STRING: printf("\"%s\"", __mdh_get_string(elem)); break;
                    default: printf("<object>"); break;
                }
            }
            printf("]\n");
            break;
        }
        case MDH_TAG_DICT: {
            int64_t *dict_ptr = (int64_t *)(intptr_t)a.data;
            int64_t count = *dict_ptr;
            MdhValue *entries = (MdhValue *)(dict_ptr + 1);
            printf("{");
            for (int64_t i = 0; i < count; i++) {
                if (i > 0) printf(", ");
                MdhValue key = entries[i * 2];
                MdhValue value = entries[i * 2 + 1];
                /* Print key */
                switch (key.tag) {
                    case MDH_TAG_NIL: printf("naething"); break;
                    case MDH_TAG_BOOL: printf("%s", key.data ? "aye" : "nae"); break;
                    case MDH_TAG_INT: printf("%lld", (long long)key.data); break;
                    case MDH_TAG_FLOAT: printf("%g", __mdh_get_float(key)); break;
                    case MDH_TAG_STRING: printf("\"%s\"", __mdh_get_string(key)); break;
                    default: printf("<object>"); break;
                }
                printf(": ");
                /* Print value */
                switch (value.tag) {
                    case MDH_TAG_NIL: printf("naething"); break;
                    case MDH_TAG_BOOL: printf("%s", value.data ? "aye" : "nae"); break;
                    case MDH_TAG_INT: printf("%lld", (long long)value.data); break;
                    case MDH_TAG_FLOAT: printf("%g", __mdh_get_float(value)); break;
                    case MDH_TAG_STRING: printf("\"%s\"", __mdh_get_string(value)); break;
                    default: printf("<object>"); break;
                }
            }
            printf("}\n");
            break;
        }
        default:
            printf("<object>\n");
            break;
    }
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
            MdhString *s = (MdhString *)(intptr_t)a.data;
            return s ? s->length : 0;
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
    MdhString *str = (MdhString *)(intptr_t)s.data;
    return str ? str->length : 0;
}

MdhValue __mdh_to_string(MdhValue a) {
    char buffer[128];

    switch (a.tag) {
        case MDH_TAG_NIL:
            return __mdh_make_string("naething");
        case MDH_TAG_BOOL:
            return __mdh_make_string(a.data ? "aye" : "nae");
        case MDH_TAG_INT:
            snprintf(buffer, sizeof(buffer), "%lld", (long long)a.data);
            return __mdh_make_string(buffer);
        case MDH_TAG_FLOAT:
            snprintf(buffer, sizeof(buffer), "%g", __mdh_get_float(a));
            return __mdh_make_string(buffer);
        case MDH_TAG_STRING:
            return a;  /* Already a string */
        default:
            return __mdh_make_string("<object>");
    }
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
    if (!__mdh_random_initialized) {
        srand((unsigned int)time(NULL));
        __mdh_random_initialized = 1;
    }

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

MdhValue __mdh_empty_creel(void) {
    /* Allocate 8 bytes for count (which is 0) */
    int64_t *dict_ptr = (int64_t *)GC_malloc(8);
    *dict_ptr = 0;  /* count = 0 */

    MdhValue v;
    v.tag = MDH_TAG_DICT;
    v.data = (int64_t)(intptr_t)dict_ptr;
    return v;
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
    /* Scots greeting - just returns nil (like a void greeting) */
    return __mdh_make_nil();
}

MdhValue __mdh_och(MdhValue msg) {
    /* Scots warning print - like println but for warnings */
    __mdh_blether(msg);
    return __mdh_make_nil();
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

MdhValue __mdh_creel_tae_list(MdhValue dict) {
    /* Convert set/dict keys to list */
    if (dict.tag != MDH_TAG_DICT) {
        return __mdh_make_list(0);
    }

    int64_t *dict_ptr = (int64_t *)(intptr_t)dict.data;
    int64_t count = *dict_ptr;
    MdhValue *entries = (MdhValue *)(dict_ptr + 1);

    MdhValue result = __mdh_make_list((int32_t)count);
    for (int64_t i = 0; i < count; i++) {
        __mdh_list_push(result, entries[i * 2]);  /* key */
    }
    return result;
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
    /* Remove all whitespace from string */
    if (str.tag != MDH_TAG_STRING) return str;
    const char *s = (const char *)(intptr_t)str.data;
    size_t len = strlen(s);
    char *result = (char *)GC_malloc(len + 1);
    char *r = result;
    for (const char *p = s; *p; p++) {
        if (*p != ' ' && *p != '\t' && *p != '\n' && *p != '\r') {
            *r++ = *p;
        }
    }
    *r = '\0';
    return __mdh_make_string(result);
}

MdhValue __mdh_bonnie(MdhValue val) {
    /* Pretty print - just convert to string for now */
    return __mdh_to_string(val);
}

MdhValue __mdh_shuffle(MdhValue list) {
    /* Shuffle list (deck) - returns shuffled copy */
    if (list.tag != MDH_TAG_LIST) return __mdh_make_list(0);

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
