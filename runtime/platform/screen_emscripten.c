/* The browser's windae: a <canvas id="canvas"> on the page (Emscripten builds only).
 *
 * The framebuffer keeps its own size (the game's 320x200); CSS scales the canvas to its
 * box, nearest-neighbour and aspect kept. Input arrives through Emscripten's html5 event
 * callbacks, which queue it; each pump replays the queue into the frame's input, so a
 * key pressed and released between two frames still counts as pressed. */
#include "mdh_screen.h"
#include <emscripten.h>
#include <emscripten/html5.h>
#include <stdlib.h>
#include <string.h>

#define CANVAS "#canvas"

/* One build, one thread: Emscripten builds here have no pthreads. */
bool mdh_scr_is_main_thread(void) { return true; }

enum { EV_KEY, EV_BUTTON, EV_MOVE };
typedef struct {
    unsigned char kind, key;
    bool down;
    int dx, dy;
    int64_t x, y;
    bool has_position;
} Event;
#define QUEUE 1024
static Event queue[QUEUE];
static unsigned queue_head, queue_count;
static bool opened, grabbed, locked;
static int64_t logical_width, logical_height;
static uint8_t *rgba;

/* Pointer lock needs a user gesture, so a request outside one is refused: refusals are
 * quiet, and a grabbed but unlocked screen asks again on the next click or key, from
 * inside that event's handler (html5 callbacks run there). */
EM_JS(void, mdh_web_pointer_lock, (int on), {
    var s = Module.mdhScreen;
    if (!s) return;
    if (!on) {
        if (document.pointerLockElement) document.exitPointerLock();
        return;
    }
    if (document.pointerLockElement === s.canvas) return;
    try {
        var p = s.canvas.requestPointerLock();
        if (p && p.catch) p.catch(function () {});
    } catch (e) {}
});

static void push(Event e) {
    if (queue_count == QUEUE) return; /* a frame this late drops the newest input */
    queue[(queue_head + queue_count++) % QUEUE] = e;
}

static int key_code(const char *code) {
    if (!strncmp(code, "Key", 3) && code[3] >= 'A' && code[3] <= 'Z' && !code[4])
        return code[3] - 'A' + 'a';
    if (!strncmp(code, "Digit", 5) && code[5] >= '0' && code[5] <= '9' && !code[6])
        return code[5];
    static const struct { const char *name; int key; } named[] = {
        {"ArrowUp", MDH_KEY_UP}, {"ArrowDown", MDH_KEY_DOWN}, {"ArrowLeft", MDH_KEY_LEFT},
        {"ArrowRight", MDH_KEY_RIGHT}, {"Space", MDH_KEY_SPACE}, {"Enter", MDH_KEY_ENTER},
        {"NumpadEnter", MDH_KEY_ENTER}, {"Escape", MDH_KEY_ESCAPE},
        {"ShiftLeft", MDH_KEY_SHIFT}, {"ShiftRight", MDH_KEY_SHIFT},
        {"ControlLeft", MDH_KEY_CTRL}, {"ControlRight", MDH_KEY_CTRL},
        {"AltLeft", MDH_KEY_ALT}, {"AltRight", MDH_KEY_ALT}, {"Tab", MDH_KEY_TAB},
    };
    for (unsigned i = 0; i < sizeof(named) / sizeof(*named); ++i)
        if (!strcmp(code, named[i].name)) return named[i].key;
    return 0;
}

static bool on_key(int type, const EmscriptenKeyboardEvent *e, void *user) {
    (void)user;
    int key = key_code(e->code);
    if (!key) return false;
    Event ev = {EV_KEY, (unsigned char)key, type == EMSCRIPTEN_EVENT_KEYDOWN, 0, 0, 0, 0, false};
    push(ev);
    if (ev.down && grabbed && !locked && key != MDH_KEY_ESCAPE) mdh_web_pointer_lock(1);
    /* Keep the page from scrolling or tabbing away, but leave the browser's own
     * shortcuts (Ctrl/Cmd+key) alone. */
    bool shortcut = (e->ctrlKey || e->metaKey) && key != MDH_KEY_CTRL;
    return !shortcut;
}

/* The canvas's CSS box holds the framebuffer scaled to fit and centred. */
static void to_logical(const EmscriptenMouseEvent *e, Event *ev) {
    double w = 0, h = 0;
    if (emscripten_get_element_css_size(CANVAS, &w, &h) != EMSCRIPTEN_RESULT_SUCCESS || w <= 0 ||
        h <= 0)
        return;
    double sx = w / (double)logical_width, sy = h / (double)logical_height;
    double s = sx < sy ? sx : sy;
    double x = ((double)e->targetX - (w - (double)logical_width * s) / 2) / s;
    double y = ((double)e->targetY - (h - (double)logical_height * s) / 2) / s;
    ev->x = x < 0 ? 0 : x >= logical_width ? logical_width - 1 : (int64_t)x;
    ev->y = y < 0 ? 0 : y >= logical_height ? logical_height - 1 : (int64_t)y;
    ev->has_position = true;
}

static bool on_mouse(int type, const EmscriptenMouseEvent *e, void *user) {
    (void)user;
    Event ev = {EV_MOVE, 0, false, e->movementX, e->movementY, 0, 0, false};
    if (!locked) to_logical(e, &ev);
    if (type == EMSCRIPTEN_EVENT_MOUSEDOWN || type == EMSCRIPTEN_EVENT_MOUSEUP) {
        /* Web buttons: 0 left, 1 middle, 2 right; ours: 0 left, 1 right, 2 middle. */
        int button = e->button == 0 ? 0 : e->button == 2 ? 1 : e->button == 1 ? 2 : -1;
        if (button < 0) return false;
        ev.kind = EV_BUTTON;
        ev.key = (unsigned char)button;
        ev.down = type == EMSCRIPTEN_EVENT_MOUSEDOWN;
        ev.dx = ev.dy = 0;
        if (ev.down && grabbed && !locked) mdh_web_pointer_lock(1);
    }
    push(ev);
    return type != EMSCRIPTEN_EVENT_MOUSEMOVE;
}

static bool on_lock(int type, const EmscriptenPointerlockChangeEvent *e, void *user) {
    (void)type; (void)user;
    locked = e->isActive;
    return false;
}

static bool on_context_menu(int type, const EmscriptenMouseEvent *e, void *user) {
    (void)type; (void)e; (void)user;
    return true;
}

EM_JS(int, mdh_web_canvas_open, (int w, int h, const char *title), {
    if (typeof document === 'undefined') return 1;
    var canvas = document.getElementById('canvas');
    if (!canvas) return 2;
    canvas.width = w;
    canvas.height = h;
    canvas.style.imageRendering = 'pixelated';
    canvas.style.objectFit = 'contain';
    canvas.style.cursor = 'none';
    document.title = UTF8ToString(title);
    var ctx = canvas.getContext('2d', {alpha: false});
    if (!ctx) return 3;
    Module.mdhScreen = {canvas: canvas, ctx: ctx, image: ctx.createImageData(w, h)};
    return 0;
});

EM_JS(void, mdh_web_canvas_present, (const uint8_t *pixels, int w, int h), {
    var s = Module.mdhScreen;
    if (!s) return;
    s.image.data.set(HEAPU8.subarray(pixels, pixels + w * h * 4));
    s.ctx.putImageData(s.image, 0, 0);
    Module.mdhPresented = true; /* the next snooze waits for the display (wasm_sleep) */
});

EM_JS(int, mdh_web_focused, (void), {
    if (typeof document === 'undefined') return 0;
    return document.visibilityState === 'visible' && document.hasFocus() ? 1 : 0;
});

EM_JS(void, mdh_web_canvas_close, (void), {
    Module.mdhScreen = null;
});

static const char *const OPEN_ERRORS[] = {
    NULL,
    "Nae windae: there's nae web page here (run it in a browser)",
    "Nae windae: the page has nae <canvas id=\"canvas\">",
    "Nae windae: the canvas wouldnae gie a 2D context",
};

const char *mdh_scr_open(int64_t w, int64_t h, const char *title) {
    int err = mdh_web_canvas_open((int)w, (int)h, title);
    if (err) return OPEN_ERRORS[err];
    rgba = malloc((size_t)w * (size_t)h * 4);
    if (!rgba) return "Nae windae: oot o' memory for the framebuffer";
    logical_width = w;
    logical_height = h;
    queue_head = queue_count = 0;
    grabbed = locked = false;
    const char *win = EMSCRIPTEN_EVENT_TARGET_WINDOW;
    emscripten_set_keydown_callback(win, NULL, true, on_key);
    emscripten_set_keyup_callback(win, NULL, true, on_key);
    emscripten_set_mousedown_callback(CANVAS, NULL, true, on_mouse);
    emscripten_set_mouseup_callback(win, NULL, true, on_mouse);
    emscripten_set_mousemove_callback(win, NULL, true, on_mouse);
    emscripten_set_pointerlockchange_callback(EMSCRIPTEN_EVENT_TARGET_DOCUMENT, NULL, true, on_lock);
    emscripten_set_contextmenu_callback(CANVAS, NULL, true, on_context_menu);
    opened = true;
    return NULL;
}

const char *mdh_scr_present(const uint8_t *bgra, int64_t w, int64_t h) {
    if (!opened) return "Nae windae is open";
    size_t count = (size_t)w * (size_t)h;
    for (size_t i = 0; i < count; ++i) {
        rgba[i * 4] = bgra[i * 4 + 2];
        rgba[i * 4 + 1] = bgra[i * 4 + 1];
        rgba[i * 4 + 2] = bgra[i * 4];
        rgba[i * 4 + 3] = 255;
    }
    mdh_web_canvas_present(rgba, (int)w, (int)h);
    return NULL;
}

static void key_state(MdhScreenInput *in, int key, bool down) {
    if (down && !in->down[key]) in->pressed[key] = true;
    in->down[key] = down;
}

void mdh_scr_pump(MdhScreenInput *in) {
    for (; queue_count; --queue_count, queue_head = (queue_head + 1) % QUEUE) {
        const Event *e = &queue[queue_head];
        if (e->kind == EV_KEY) key_state(in, e->key, e->down);
        if (e->kind == EV_BUTTON) in->buttons[e->key] = e->down;
        in->dx += e->dx;
        in->dy += e->dy;
        if (e->has_position) {
            in->x = e->x;
            in->y = e->y;
        }
    }
    in->focused = mdh_web_focused() != 0;
    if (!in->focused) {
        /* Keys let go while the page was away never arrive: forget what was held. */
        memset(in->down, 0, sizeof(in->down));
        memset(in->buttons, 0, sizeof(in->buttons));
    }
    in->should_close = false; /* a web page is closed by its tab, not by the game */
}

void mdh_scr_grab(bool on) {
    if (on == grabbed) return;
    grabbed = on;
    mdh_web_pointer_lock(on); /* refused outside a gesture: asked again on the next one */
}

void mdh_scr_close(void) {
    if (!opened) return;
    mdh_scr_grab(false);
    const char *win = EMSCRIPTEN_EVENT_TARGET_WINDOW;
    emscripten_set_keydown_callback(win, NULL, true, NULL);
    emscripten_set_keyup_callback(win, NULL, true, NULL);
    emscripten_set_mousedown_callback(CANVAS, NULL, true, NULL);
    emscripten_set_mouseup_callback(win, NULL, true, NULL);
    emscripten_set_mousemove_callback(win, NULL, true, NULL);
    emscripten_set_pointerlockchange_callback(EMSCRIPTEN_EVENT_TARGET_DOCUMENT, NULL, true, NULL);
    emscripten_set_contextmenu_callback(CANVAS, NULL, true, NULL);
    mdh_web_canvas_close();
    free(rgba);
    rgba = NULL;
    opened = false;
}
