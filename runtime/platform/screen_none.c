#include "mdh_screen.h"
#if defined(_WIN32)
#include <windows.h>
static DWORD main_thread_id;
static void capture_main_thread(void) { main_thread_id = GetCurrentThreadId(); }
#if defined(_MSC_VER)
#pragma section(".CRT$XCU", read)
__declspec(allocate(".CRT$XCU")) static void (*main_thread_init)(void) = capture_main_thread;
#else
__attribute__((constructor)) static void init_thread(void) { capture_main_thread(); }
#endif
bool mdh_scr_is_main_thread(void) { return GetCurrentThreadId() == main_thread_id; }
#else
#include <pthread.h>
static pthread_t main_thread_id;
__attribute__((constructor)) static void capture_main_thread(void) { main_thread_id = pthread_self(); }
bool mdh_scr_is_main_thread(void) { return pthread_equal(pthread_self(), main_thread_id) != 0; }
#endif
const char *mdh_scr_open(int64_t w, int64_t h, const char *title) {
    (void)w; (void)h; (void)title;
    return "Nae windae on this platform yet";
}
const char *mdh_scr_present(const uint8_t *bgra, int64_t w, int64_t h) {
    (void)bgra; (void)w; (void)h;
    return "Nae windae on this platform yet";
}
void mdh_scr_pump(MdhScreenInput *input) { (void)input; }
void mdh_scr_close(void) {}
void mdh_scr_grab(bool on) { (void)on; }
