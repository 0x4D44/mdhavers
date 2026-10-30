/* Internal, synchronous framebuffer bridge. No MdhValue or user buffer survives a call. */
#ifndef MDH_SCREEN_H
#define MDH_SCREEN_H
#include <stdbool.h>
#include <stdint.h>

/* Letters and digits use ASCII; the remaining names have stable table indices. */
enum {
    MDH_KEY_UP = 128, MDH_KEY_DOWN, MDH_KEY_LEFT, MDH_KEY_RIGHT,
    MDH_KEY_SPACE, MDH_KEY_ENTER, MDH_KEY_ESCAPE, MDH_KEY_SHIFT,
    MDH_KEY_CTRL, MDH_KEY_ALT, MDH_KEY_TAB, MDH_KEY_COUNT
};
typedef struct {
    bool down[MDH_KEY_COUNT], pressed[MDH_KEY_COUNT];
    bool buttons[3], should_close;
    bool focused; /* The window has keyboard focus; set by every pump. */
    int64_t x, y;
    double dx, dy; /* Subpixel motion carries over the frame boundary. */
} MdhScreenInput;

bool mdh_scr_is_main_thread(void);
/* A non-NULL error is static text. Platform functions never hurl through a pool. */
const char *mdh_scr_open(int64_t width, int64_t height, const char *title);
const char *mdh_scr_present(const uint8_t *bgra, int64_t width, int64_t height);
void mdh_scr_pump(MdhScreenInput *input);
void mdh_scr_close(void);
void mdh_scr_grab(bool on);
#endif
