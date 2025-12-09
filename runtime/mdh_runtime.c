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

    MdhString *s = (MdhString *)GC_malloc(sizeof(MdhString));
    s->length = strlen(value);
    s->data = GC_strdup(value);

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

    /* Apply new settings immediately */
    tcsetattr(STDIN_FILENO, TCSANOW, &new_tio);

    /* Read one character */
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
