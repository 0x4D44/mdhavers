/* arm64 AppKit bridge, compiled as plain C. Cast every message to its actual ABI. */
#include "mdh_screen.h"
#include <CoreGraphics/CoreGraphics.h>
#include <objc/message.h>
#include <objc/runtime.h>
#include <dlfcn.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>

/* AppKit's NS(U)Integer and NSPoint/NSRect are these types on arm64 macOS. */
typedef unsigned long NSUInteger;
typedef long NSInteger;
extern void *objc_autoreleasePoolPush(void);
extern void objc_autoreleasePoolPop(void *pool);
static id app, window, layer, delegate;
static bool close_requested, grabbed;
static int64_t logical_width, logical_height;

static id string(const char *s) {
    return ((id (*)(id, SEL, const char *))objc_msgSend)(
        (id)objc_getClass("NSString"), sel_registerName("stringWithUTF8String:"), s);
}
static void release(id object) {
    ((void (*)(id, SEL))objc_msgSend)(object, sel_registerName("release"));
}
static void will_close(id self, SEL cmd, id notification) {
    (void)self; (void)cmd; (void)notification; close_requested = true;
}
static NSUInteger should_terminate(id self, SEL cmd, id sender) {
    (void)self; (void)cmd; (void)sender;
    close_requested = true;
    return 0; /* NSTerminateCancel: the mdhavers host closes the window and exits. */
}
bool mdh_scr_is_main_thread(void) { return pthread_main_np() != 0; }

const char *mdh_scr_open(int64_t width, int64_t height, const char *title) {
    void *pool = objc_autoreleasePoolPush();
    close_requested = false;
    logical_width = width; logical_height = height;
    app = ((id (*)(id, SEL))objc_msgSend)((id)objc_getClass("NSApplication"),
                                        sel_registerName("sharedApplication"));
    ((BOOL (*)(id, SEL, NSInteger))objc_msgSend)(app, sel_registerName("setActivationPolicy:"), 0);
    Class cls = objc_getClass("MdhScreenDelegate");
    if (!cls) {
        cls = objc_allocateClassPair(objc_getClass("NSObject"), "MdhScreenDelegate", 0);
        if (!cls || !class_addMethod(cls, sel_registerName("windowWillClose:"),
                                    (IMP)will_close, "v@:@") ||
            !class_addMethod(cls, sel_registerName("applicationShouldTerminate:"),
                             (IMP)should_terminate, "Q@:@")) {
            if (cls) objc_disposeClassPair(cls);
            objc_autoreleasePoolPop(pool);
            return "Cannae mak the windae delegate";
        }
        objc_registerClassPair(cls);
    }
    delegate = ((id (*)(id, SEL))objc_msgSend)((id)cls, sel_registerName("new"));
    ((void (*)(id, SEL, id))objc_msgSend)(app, sel_registerName("setDelegate:"), delegate);

    id menu = ((id (*)(id, SEL))objc_msgSend)((id)objc_getClass("NSMenu"), sel_registerName("new"));
    id root = ((id (*)(id, SEL))objc_msgSend)((id)objc_getClass("NSMenuItem"), sel_registerName("new"));
    id submenu = ((id (*)(id, SEL))objc_msgSend)((id)objc_getClass("NSMenu"), sel_registerName("new"));
    id quit = ((id (*)(id, SEL))objc_msgSend)((id)objc_getClass("NSMenuItem"), sel_registerName("alloc"));
    quit = ((id (*)(id, SEL, id, SEL, id))objc_msgSend)(quit,
        sel_registerName("initWithTitle:action:keyEquivalent:"), string("Quit"),
        sel_registerName("terminate:"), string("q"));
    ((void (*)(id, SEL, id))objc_msgSend)(quit, sel_registerName("setTarget:"), app);
    ((void (*)(id, SEL, id))objc_msgSend)(submenu, sel_registerName("addItem:"), quit);
    ((void (*)(id, SEL, id))objc_msgSend)(root, sel_registerName("setSubmenu:"), submenu);
    ((void (*)(id, SEL, id))objc_msgSend)(menu, sel_registerName("addItem:"), root);
    ((void (*)(id, SEL, id))objc_msgSend)(app, sel_registerName("setMainMenu:"), menu);
    release(quit); release(submenu); release(root); release(menu);

    window = ((id (*)(id, SEL))objc_msgSend)((id)objc_getClass("NSWindow"), sel_registerName("alloc"));
    window = ((id (*)(id, SEL, CGRect, NSUInteger, NSUInteger, BOOL))objc_msgSend)(window,
        sel_registerName("initWithContentRect:styleMask:backing:defer:"),
        CGRectMake(0, 0, (CGFloat)width * 4, (CGFloat)height * 4), 1 | 2 | 4, 2, NO);
    if (!window) {
        ((void (*)(id, SEL, id))objc_msgSend)(app, sel_registerName("setDelegate:"), nil);
        release(delegate); delegate = nil;
        objc_autoreleasePoolPop(pool);
        return "Cannae open the windae";
    }
    ((void (*)(id, SEL, BOOL))objc_msgSend)(window, sel_registerName("setReleasedWhenClosed:"), NO);
    ((void (*)(id, SEL, id))objc_msgSend)(window, sel_registerName("setTitle:"), string(title));
    ((void (*)(id, SEL, id))objc_msgSend)(window, sel_registerName("setDelegate:"), delegate);
    ((void (*)(id, SEL, BOOL))objc_msgSend)(window, sel_registerName("setAcceptsMouseMovedEvents:"), YES);
    id view = ((id (*)(id, SEL))objc_msgSend)(window, sel_registerName("contentView"));
    ((void (*)(id, SEL, BOOL))objc_msgSend)(view, sel_registerName("setWantsLayer:"), YES);
    layer = ((id (*)(id, SEL))objc_msgSend)(view, sel_registerName("layer"));
    /* dlsym returns the address of the exported NSString* variable. */
    id *nearest = (id *)dlsym(RTLD_DEFAULT, "kCAFilterNearest");
    ((void (*)(id, SEL, id))objc_msgSend)(layer, sel_registerName("setMagnificationFilter:"),
                                        nearest ? *nearest : string("nearest"));
    ((void (*)(id, SEL, id))objc_msgSend)(layer, sel_registerName("setMinificationFilter:"),
                                        nearest ? *nearest : string("nearest"));
    ((void (*)(id, SEL))objc_msgSend)(window, sel_registerName("center"));
    ((void (*)(id, SEL, id))objc_msgSend)(window, sel_registerName("makeKeyAndOrderFront:"), nil);
    ((void (*)(id, SEL))objc_msgSend)(app, sel_registerName("finishLaunching"));
    ((void (*)(id, SEL, BOOL))objc_msgSend)(app, sel_registerName("activateIgnoringOtherApps:"), YES);
    objc_autoreleasePoolPop(pool);
    return NULL;
}

static void free_image(void *info, const void *data, size_t size) {
    (void)info; (void)size; free((void *)data);
}
const char *mdh_scr_present(const uint8_t *bgra, int64_t width, int64_t height) {
    /* Core Animation may hold old images beyond a second frame. Give each provider
     * an immutable bridge-owned snapshot, freed only by its release callback. The
     * two conversion buffers can then be reused/closed without a renderer race. */
    size_t size = (size_t)width * (size_t)height * 4;
    uint8_t *snapshot = malloc(size);
    if (!snapshot) return "Cannae allocate the windae frame";
    memcpy(snapshot, bgra, size);
    CGDataProviderRef provider = CGDataProviderCreateWithData(NULL, snapshot, size, free_image);
    if (!provider) { free(snapshot); return "Cannae mak the frame provider"; }
    CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    CGImageRef image = space ? CGImageCreate((size_t)width, (size_t)height, 8, 32,
        (size_t)width * 4, space, kCGImageAlphaNoneSkipFirst | kCGBitmapByteOrder32Little,
        provider, NULL, false, kCGRenderingIntentDefault) : NULL;
    CGDataProviderRelease(provider);
    if (space) CGColorSpaceRelease(space);
    if (!image) return "Cannae mak the windae image";
    void *pool = objc_autoreleasePoolPush();
    id transaction = (id)objc_getClass("CATransaction");
    ((void (*)(id, SEL))objc_msgSend)(transaction, sel_registerName("begin"));
    ((void (*)(id, SEL, BOOL))objc_msgSend)(transaction, sel_registerName("setDisableActions:"), YES);
    CGFloat scale = ((CGFloat (*)(id, SEL))objc_msgSend)(window, sel_registerName("backingScaleFactor"));
    ((void (*)(id, SEL, CGFloat))objc_msgSend)(layer, sel_registerName("setContentsScale:"), scale);
    ((void (*)(id, SEL, id))objc_msgSend)(layer, sel_registerName("setContents:"), (id)image);
    ((void (*)(id, SEL))objc_msgSend)(transaction, sel_registerName("commit"));
    CGImageRelease(image);
    objc_autoreleasePoolPop(pool);
    return NULL;
}

/* Hardware key codes are layout-independent, matching the raylib movement keys. */
static int key_code(unsigned short code) {
    static const unsigned char letters[128] = {
        [0]='a', [1]='s', [2]='d', [3]='f', [4]='h', [5]='g', [6]='z', [7]='x',
        [8]='c', [9]='v', [11]='b', [12]='q', [13]='w', [14]='e', [15]='r',
        [16]='y', [17]='t', [18]='1', [19]='2', [20]='3', [21]='4', [22]='6',
        [23]='5', [25]='9', [26]='7', [28]='8', [29]='0', [31]='o', [32]='u',
        [34]='i', [35]='p', [37]='l', [38]='j', [40]='k', [45]='n', [46]='m',
        [36]=MDH_KEY_ENTER, [48]=MDH_KEY_TAB, [49]=MDH_KEY_SPACE, [53]=MDH_KEY_ESCAPE,
        [123]=MDH_KEY_LEFT, [124]=MDH_KEY_RIGHT, [125]=MDH_KEY_DOWN, [126]=MDH_KEY_UP
    };
    return code < 128 ? letters[code] : 0;
}
static void key_state(MdhScreenInput *in, int key, bool down) {
    if (!key) return;
    if (down && !in->down[key]) in->pressed[key] = true;
    in->down[key] = down;
}
void mdh_scr_pump(MdhScreenInput *in) {
    void *pool = objc_autoreleasePoolPush();
    id past = ((id (*)(id, SEL))objc_msgSend)((id)objc_getClass("NSDate"), sel_registerName("distantPast"));
    id *mode_ptr = (id *)dlsym(RTLD_DEFAULT, "NSDefaultRunLoopMode");
    id mode = mode_ptr ? *mode_ptr : string("kCFRunLoopDefaultMode");
    id event;
    while ((event = ((id (*)(id, SEL, NSUInteger, id, id, BOOL))objc_msgSend)(app,
        sel_registerName("nextEventMatchingMask:untilDate:inMode:dequeue:"), ~0UL, past, mode, YES))) {
        NSUInteger type = ((NSUInteger (*)(id, SEL))objc_msgSend)(event, sel_registerName("type"));
        NSUInteger flags = ((NSUInteger (*)(id, SEL))objc_msgSend)(event, sel_registerName("modifierFlags"));
        key_state(in, MDH_KEY_SHIFT, (flags & (1UL << 17)) != 0);
        key_state(in, MDH_KEY_CTRL, (flags & (1UL << 18)) != 0);
        key_state(in, MDH_KEY_ALT, (flags & (1UL << 19)) != 0);
        if (type == 10 || type == 11) { /* keyDown, keyUp: consume, avoiding the system beep */
            unsigned short code = ((unsigned short (*)(id, SEL))objc_msgSend)(event, sel_registerName("keyCode"));
            key_state(in, key_code(code), type == 10);
            if (type == 10 && code == 12 && (flags & (1UL << 20))) close_requested = true;
        } else {
            if ((type >= 1 && type <= 7) || type == 25 || type == 26 || type == 27) {
                CGPoint point = ((CGPoint (*)(id, SEL))objc_msgSend)(event, sel_registerName("locationInWindow"));
                id view = ((id (*)(id, SEL))objc_msgSend)(window, sel_registerName("contentView"));
                CGRect rect = ((CGRect (*)(id, SEL))objc_msgSend)(view, sel_registerName("frame"));
                if (rect.size.width > 0 && rect.size.height > 0) {
                    double x = point.x * (double)logical_width / rect.size.width;
                    double y = (rect.size.height - point.y) * (double)logical_height / rect.size.height;
                    in->x = x < 0 ? 0 : x >= logical_width ? logical_width - 1 : (int64_t)x;
                    in->y = y < 0 ? 0 : y >= logical_height ? logical_height - 1 : (int64_t)y;
                }
                if (type == 5 || type == 6 || type == 7 || type == 27) {
                    in->dx += ((CGFloat (*)(id, SEL))objc_msgSend)(event, sel_registerName("deltaX"));
                    in->dy += ((CGFloat (*)(id, SEL))objc_msgSend)(event, sel_registerName("deltaY"));
                }
                if (type == 1 || type == 2) in->buttons[0] = type == 1;
                if (type == 3 || type == 4) in->buttons[1] = type == 3;
                if (type == 25 || type == 26) {
                    NSInteger button = ((NSInteger (*)(id, SEL))objc_msgSend)(event, sel_registerName("buttonNumber"));
                    if (button == 2) in->buttons[2] = type == 25;
                }
            }
            ((void (*)(id, SEL, id))objc_msgSend)(app, sel_registerName("sendEvent:"), event);
        }
    }
    ((void (*)(id, SEL))objc_msgSend)(app, sel_registerName("updateWindows"));
    if (!((BOOL (*)(id, SEL))objc_msgSend)(window, sel_registerName("isKeyWindow"))) {
        memset(in->down, 0, sizeof(in->down));
        memset(in->buttons, 0, sizeof(in->buttons));
    }
    in->should_close = close_requested;
    objc_autoreleasePoolPop(pool);
}
void mdh_scr_grab(bool on) {
    if (on == grabbed) return;
    void *pool = objc_autoreleasePoolPush();
    CGAssociateMouseAndMouseCursorPosition(!on);
    ((void (*)(id, SEL))objc_msgSend)((id)objc_getClass("NSCursor"), sel_registerName(on ? "hide" : "unhide"));
    grabbed = on;
    objc_autoreleasePoolPop(pool);
}
void mdh_scr_close(void) {
    if (!window) return;
    mdh_scr_grab(false);
    void *pool = objc_autoreleasePoolPush();
    ((void (*)(id, SEL, id))objc_msgSend)(layer, sel_registerName("setContents:"), nil);
    ((void (*)(id, SEL, id))objc_msgSend)(window, sel_registerName("setDelegate:"), nil);
    ((void (*)(id, SEL))objc_msgSend)(window, sel_registerName("close"));
    ((void (*)(id, SEL, id))objc_msgSend)(app, sel_registerName("setDelegate:"), nil);
    release(window); release(delegate);
    window = layer = delegate = nil;
    objc_autoreleasePoolPop(pool);
}
