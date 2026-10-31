/*
 * Browser build (Emscripten) only: `snooze`. Compiled code's nanosleep comes here
 * (`redirect_sleep` in src/llvm/wasm.rs). Built twice:
 *
 * - plain (the default): sleeps as nanosleep does where a thread may block (node, a
 *   worker). A web page's main thread may not: there snooze raises an error, and a
 *   program draws its frames with screen_loop instead.
 * - with MDH_WEB_YIELD, linked when the program is built with JSPI (JS Promise
 *   Integration, `--emcc-arg=-sJSPI`): the program's stack is suspended and the
 *   browser runs (draws, delivers input, plays sound) until the sleep is over. After a
 *   frame was shown the sleep lasts until the display's next frame
 *   (requestAnimationFrame), so a game loop runs at its rate.
 */

#ifdef __EMSCRIPTEN__

#include <emscripten.h>
#include <time.h>

#ifdef MDH_WEB_YIELD
EM_ASYNC_JS(void, mdh_web_yield, (double ms), {
    if (Module.mdhPresented && typeof requestAnimationFrame === 'function') {
        Module.mdhPresented = false;
        await new Promise(function (resume) { requestAnimationFrame(function () { resume(); }); });
    } else {
        await new Promise(function (resume) { setTimeout(resume, ms); });
    }
});

int __mdh_wasm_nanosleep(const struct timespec *req, struct timespec *rem) {
    mdh_web_yield((double)req->tv_sec * 1000.0 + (double)req->tv_nsec / 1e6);
    if (rem) {
        rem->tv_sec = 0;
        rem->tv_nsec = 0;
    }
    return 0;
}
#else
extern void __mdh_builtin_error(const char *message);

EM_JS(int, mdh_web_page, (void), {
    return typeof ENVIRONMENT_IS_WEB !== 'undefined' && ENVIRONMENT_IS_WEB ? 1 : 0;
});

int __mdh_wasm_nanosleep(const struct timespec *req, struct timespec *rem) {
    if (mdh_web_page())
        __mdh_builtin_error("snooze cannae haud up a web page: let screen_loop rin yer "
                            "frames, or build wi' --emcc-arg=-sJSPI");
    return nanosleep(req, rem);
}
#endif

#endif /* __EMSCRIPTEN__ */
