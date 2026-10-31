/* OS-independent MdhValue boundary: borrowed arguments, bridge-owned pixels and input. */
#include "mdh_runtime.h"
#include "platform/mdh_screen.h"
#include "platform/platform.h"
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern void __mdh_builtin_error(const char *message);
static bool opened, offscreen;
static int64_t width, height;
static uint64_t frame;
static unsigned buffer_index;
static uint8_t *buffers[2];
static MdhScreenInput input;

static void main_thread(void) {
    if (!mdh_scr_is_main_thread())
        __mdh_builtin_error("Nae windae aff the main thread!");
}
static void pump(void) {
    if (opened && !offscreen) mdh_scr_pump(&input);
}
static void require_open(void) {
    if (!opened) __mdh_builtin_error("Nae windae! Open the screen first");
}

MdhValue __mdh_screen_open(MdhValue w, MdhValue h, MdhValue title) {
    main_thread();
    if (w.tag != MDH_TAG_INT || h.tag != MDH_TAG_INT || w.data <= 0 || h.data <= 0 ||
        w.data > INT32_MAX / 4 || h.data > INT32_MAX / 4 ||
        (uint64_t)w.data > SIZE_MAX / 4 / (uint64_t)h.data ||
        w.data > INT64_MAX / 4 / h.data)
        __mdh_builtin_error("screen_open wants positive integer dimensions");
    if (title.tag != MDH_TAG_STRING)
        __mdh_builtin_error("screen_open wants a string title");
    if (opened) __mdh_builtin_error("screen_open: a windae is already open");
    const char *mode = getenv("MDH_SCREEN");
    offscreen = mode && strcmp(mode, "none") == 0;
    size_t size = (size_t)w.data * (size_t)h.data * 4;
    buffers[0] = malloc(size);
    buffers[1] = malloc(size);
    if (!buffers[0] || !buffers[1]) {
        free(buffers[0]); free(buffers[1]);
        buffers[0] = buffers[1] = NULL;
        __mdh_raise_out_of_memory();
    }
    if (!offscreen) {
        const char *err = mdh_scr_open(w.data, h.data, (const char *)(intptr_t)title.data);
        if (err) {
            free(buffers[0]); free(buffers[1]);
            buffers[0] = buffers[1] = NULL;
            __mdh_builtin_error(err);
        }
    }
    width = w.data; height = h.data; frame = 0; buffer_index = 0;
    memset(&input, 0, sizeof(input));
    opened = true;
    return __mdh_make_nil();
}

MdhValue __mdh_screen_shaw(MdhValue pixels, MdhValue palette) {
    main_thread();
    if (!opened) __mdh_builtin_error("screen_shaw: nae windae is open");
    if (pixels.tag != MDH_TAG_BYTES) __mdh_builtin_error("screen_shaw wants bytes pixels");
    if (palette.tag != MDH_TAG_NIL &&
        (palette.tag != MDH_TAG_BYTES || ((MdhBytes *)(intptr_t)palette.data)->length != 768))
        __mdh_builtin_error("screen_shaw wants a 768-byte palette or naething");
    MdhBytes *src = (MdhBytes *)(intptr_t)pixels.data;
    int64_t count = width * height;
    bool indexed = palette.tag == MDH_TAG_BYTES;
    if (src->length != count * (indexed ? 1 : 4))
        __mdh_builtin_error("screen_shaw: pixel length doesnae match the windae");
    uint8_t *out = buffers[buffer_index];
    if (indexed) {
        const uint8_t *rgb = ((MdhBytes *)(intptr_t)palette.data)->data;
        for (int64_t i = 0; i < count; ++i) {
            unsigned p = (unsigned)src->data[i] * 3;
            out[i * 4] = rgb[p + 2];
            out[i * 4 + 1] = rgb[p + 1];
            out[i * 4 + 2] = rgb[p];
            out[i * 4 + 3] = 255;
        }
    } else {
        memcpy(out, src->data, (size_t)count * 4);
    }
    if (offscreen) {
        uint32_t hash = UINT32_C(2166136261);
        for (int64_t i = 0; i < count * 4; ++i) hash = (hash ^ out[i]) * UINT32_C(16777619);
        fprintf(stderr, "mdh_screen frame %" PRIu64 " %08" PRIx32 "\n", ++frame, hash);
    } else {
        const char *err = mdh_scr_present(out, width, height);
        if (err) __mdh_builtin_error(err);
    }
    buffer_index ^= 1;
    /* Retire the previous frame's edges, then collect this frame's new input.
     * should_close may pump too: those edges remain visible until the next shaw. */
    memset(input.pressed, 0, sizeof(input.pressed));
    /* mouse_dx/dy report whole pixels; carry the fraction so slow motion still adds up. */
    input.dx -= (double)(int64_t)input.dx;
    input.dy -= (double)(int64_t)input.dy;
    pump();
    return __mdh_make_nil();
}

MdhValue __mdh_screen_should_close(void) {
    main_thread(); pump(); return __mdh_make_bool(input.should_close);
}
MdhValue __mdh_screen_close(void) {
    main_thread();
    if (opened && !offscreen) mdh_scr_close();
    free(buffers[0]); free(buffers[1]); buffers[0] = buffers[1] = NULL;
    opened = false; width = height = 0;
    memset(&input, 0, sizeof(input));
    return __mdh_make_nil();
}
MdhValue __mdh_screen_width(void) { main_thread(); require_open(); return __mdh_make_int(width); }
MdhValue __mdh_screen_height(void) { main_thread(); require_open(); return __mdh_make_int(height); }

static int key_index(MdhValue name, const char *error) {
    if (name.tag != MDH_TAG_STRING) __mdh_builtin_error(error);
    const char *s = (const char *)(intptr_t)name.data;
    if (s[0] && !s[1]) {
        unsigned char c = (unsigned char)s[0];
        if (c >= 'A' && c <= 'Z') c += 'a' - 'A';
        if ((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9')) return c;
    }
    static const char *names[] = {"up", "down", "left", "right", "space", "enter",
                                  "escape", "shift", "ctrl", "alt", "tab"};
    for (unsigned i = 0; i < sizeof(names) / sizeof(*names); ++i)
        if (!mdh_strcasecmp(s, names[i])) return MDH_KEY_UP + (int)i;
    if (!mdh_strcasecmp(s, "esc")) return MDH_KEY_ESCAPE;
    __mdh_builtin_error("Dinnae ken that key name");
    return 0;
}
MdhValue __mdh_key_down(MdhValue name) {
    main_thread(); int k = key_index(name, "key_down wants a string key"); require_open();
    return __mdh_make_bool(input.down[k]);
}
MdhValue __mdh_key_pressed(MdhValue name) {
    main_thread(); int k = key_index(name, "key_pressed wants a string key"); require_open();
    return __mdh_make_bool(input.pressed[k]);
}
MdhValue __mdh_mouse_x(void) { main_thread(); require_open(); return __mdh_make_int(input.x); }
MdhValue __mdh_mouse_y(void) { main_thread(); require_open(); return __mdh_make_int(input.y); }
MdhValue __mdh_mouse_dx(void) { main_thread(); require_open(); return __mdh_make_int((int64_t)input.dx); }
MdhValue __mdh_mouse_dy(void) { main_thread(); require_open(); return __mdh_make_int((int64_t)input.dy); }
MdhValue __mdh_mouse_down(MdhValue button) {
    main_thread();
    if (button.tag != MDH_TAG_INT || button.data < 0 || button.data > 2)
        __mdh_builtin_error("mouse_down wants a button frae 0 tae 2");
    require_open(); return __mdh_make_bool(input.buttons[button.data]);
}
MdhValue __mdh_screen_grab_mouse(MdhValue on) {
    main_thread();
    if (on.tag != MDH_TAG_BOOL) __mdh_builtin_error("screen_grab_mouse wants aye or nae");
    require_open(); if (!offscreen) mdh_scr_grab(on.data != 0);
    return __mdh_make_nil();
}
/* MDH_SCREEN=none has no window to lose focus, so it always reports aye. */
MdhValue __mdh_screen_focused(void) {
    main_thread(); require_open(); pump();
    return __mdh_make_bool(offscreen || input.focused);
}

/* screen_loop(frame_fn): the host drives the frames. `call` is compiled code's late
 * dispatcher for one argument (`__mdh_late_call_1`): it calls any function value with
 * the language's own arity, default-argument and closure rules, borrowing both
 * arguments and returning +1. The frame function gets the milliseconds since the
 * previous frame (mono_ms, so 0 on the first) and the loop ends when it gives `nae`,
 * when the window should close, or when it closes the window itself. */
typedef MdhValue (*MdhLoopCall)(MdhValue fn, MdhValue arg);

static bool loop_callable(MdhValue fn) {
    return fn.tag == MDH_TAG_FUNCTION || fn.tag == MDH_TAG_CLOSURE;
}
/* One frame; false when the loop should stop. */
static bool loop_frame(MdhValue fn, MdhLoopCall call, int64_t *last) {
    int64_t now = __mdh_mono_ms().data;
    MdhValue result = call(fn, __mdh_make_int(now - *last));
    *last = now;
    bool stop = result.tag == MDH_TAG_BOOL && result.data == 0;
    __mdh_release(result);
    return !stop && opened;
}

#ifndef __EMSCRIPTEN__
/* Natively the shells' own loop, timing included: a frame, then a 1 ms snooze (the
 * frame's screen_shaw paces it to the display). */
MdhValue __mdh_screen_loop(MdhValue fn, MdhLoopCall call) {
    main_thread();
    if (!loop_callable(fn)) __mdh_builtin_error("screen_loop wants a function");
    require_open();
    int64_t last = __mdh_mono_ms().data;
    for (;;) {
        pump();
        if (input.should_close || !loop_frame(fn, call, &last)) break;
        mdh_sleep_ms(1);
    }
    return __mdh_make_nil();
}
#else
#include <emscripten.h>
#include <setjmp.h>
/* In a web page nothing may block: the browser calls each frame from
 * requestAnimationFrame (emscripten_set_main_loop), and the call itself never returns
 * (the rest of main does not run; Emscripten unwinds it). When the loop ends the
 * program exits, with 0, or with 1 after an uncaught error. Handlers around the
 * screen_loop call lived in main's unwound frames, so each frame has its own: an
 * error a frame does not catch ends the program as an uncaught one does. */
static MdhValue web_fn;
static MdhLoopCall web_call;
static int64_t web_last;

/* An import, not EM_ASM: EM_ASM cannot sit in a function that uses setjmp. */
EM_JS(void, mdh_web_hush, (void), {
    if (Module.mdhSound) {
        try { Module.mdhSound.ctx.close(); } catch (e) {}
        Module.mdhSound = null;
    }
});

static void web_finish(int status) {
    emscripten_cancel_main_loop();
    __mdh_release(web_fn);
    web_fn = __mdh_make_nil();
    if (opened) __mdh_screen_close();
    mdh_web_hush(); /* the rest of main (a sound_close, say) never runs */
    emscripten_force_exit(status);
}

static void web_frame(void) {
    jmp_buf env;
    __mdh_try_push(&env);
    if (setjmp(env) != 0) {
        __mdh_try_pop();
        MdhValue text = __mdh_to_string(__mdh_get_last_error());
        fprintf(stderr, "%s\n", __mdh_get_string(text));
        __mdh_release(text);
        web_finish(1);
        return;
    }
    pump();
    bool more = !input.should_close && loop_frame(web_fn, web_call, &web_last);
    __mdh_try_pop();
    if (!more) web_finish(0);
}

MdhValue __mdh_screen_loop(MdhValue fn, MdhLoopCall call) {
    main_thread();
    if (!loop_callable(fn)) __mdh_builtin_error("screen_loop wants a function");
    require_open();
    __mdh_retain(fn); /* kept past this call: the frames outlive main */
    web_fn = fn;
    web_call = call;
    web_last = __mdh_mono_ms().data;
    emscripten_set_main_loop(web_frame, 0, true);
    return __mdh_make_nil(); /* not reached: set_main_loop unwinds main */
}
#endif
