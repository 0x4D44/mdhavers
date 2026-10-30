/*
 * Browser build (Emscripten) only: `snooze`. Compiled code's nanosleep comes here
 * (`redirect_sleep` in src/llvm/wasm.rs). Built twice:
 *
 * - plain: sleeps as nanosleep does (node, and programs without a screen);
 * - with MDH_WEB_YIELD, for programs that open a screen, linked with JSPI (JS Promise
 *   Integration): the program's stack is suspended and the browser runs (draws, delivers input, plays
 *   sound) until the sleep is over. After a frame was shown the sleep lasts until the
 *   display's next frame (requestAnimationFrame), so a game loop runs at its rate.
 */

#ifdef __EMSCRIPTEN__

#include <time.h>

#ifdef MDH_WEB_YIELD
#include <emscripten.h>

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
int __mdh_wasm_nanosleep(const struct timespec *req, struct timespec *rem) {
    return nanosleep(req, rem);
}
#endif

#endif /* __EMSCRIPTEN__ */
