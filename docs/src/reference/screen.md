# Screen, Keyboard and Mouse

The screen built-ins give a program one window holding a **framebuffer**: a
flat `bytes` value, one byte per pixel (indexed colour) or four bytes per pixel
(BGRA). You fill the bytes yourself, then call `screen_shaw` to put them on
screen. Keyboard and mouse state is polled once per frame.

Bogle Castle (`games/bogle`) is written entirely in mdhavers on top of these
built-ins (plus the [sound built-ins](./sound.md)): the whole renderer, game
logic and menus are `.braw` code, and the shell in
`games/bogle/shell/frame.braw` is the only part that touches the window.

## A minimal complete program

This program opens a 64 by 48 window, fills its lower half with a yellow
palette entry, and shows three frames (or stops sooner if the window is closed). It needs a window
when run, so build it natively; set `MDH_SCREEN=none` to run it without one.

```mdhavers
# run: native
ken w = 64
ken h = 48
screen_open(w, h, "Hello screen")

# 256-entry RGB palette, 768 bytes. Entry 1 is a warm yellow.
ken palette = bytes(768)
bytes_set(palette, 3, 255)
bytes_set(palette, 4, 200)
bytes_set(palette, 5, 40)

ken pixels = bytes(w * h)
fer y in 0..h {
    fer x in 0..w {
        gin y > h / 2 {
            bytes_set(pixels, y * w + x, 1)
        }
    }
}

ken frames = 0
whiles nae screen_should_close() an frames < 3 {
    screen_shaw(pixels, palette)
    frames = frames + 1
}
screen_close()
```

Run it for real with:

```text
mdhavers build hello_screen.braw -o hello_screen
./hello_screen
```

## Backend support

| Backend | Status |
|---|---|
| Native (`mdhavers build`, LLVM), Apple Silicon macOS | Full support. Implemented in `runtime/mdh_screen.c` with a Cocoa window in `runtime/platform/screen_macos.c`. |
| Native, any other target | The functions exist but `runtime/platform/screen_none.c` has no window: `screen_open` hurls `Nae windae on this platform yet`, unless `MDH_SCREEN=none` is set (see below). Selected in `build.rs:101`. |
| Interpreter, built with the `graphics` (raylib) feature | Supported by `src/graphics.rs`. Same function names and the same `screen_shaw` formats; the details below note where it differs from native. |
| Interpreter, built without `graphics` (the CLI-only and minimal builds) | The names exist but every screen, key and mouse call hurls `Nae windae! Screen an' keyboard functions cannae be used in native code` (`src/graphics.rs:828`). `MDH_SCREEN=none` does not change this. |
| JavaScript (`compile`) and WebAssembly (`wasm`) | Not implemented. `src/compiler.rs` and `src/wasm_compiler.rs` define no `screen_*`, `key_*` or `mouse_*` helper. The JavaScript output compiles but stops at the call with `ReferenceError: screen_open is not defined`; the `wasm` command refuses to compile with `Dae 'screen_open' isnae kent tae the WASM backend ...`. |

All screen, key and mouse functions must be called from the main thread. The
native runtime checks this and hurls `Nae windae aff the main thread!`.

When a native program catches an error with `hae_a_bash`, the message the
handler sees is prefixed `Jings! Something went awfy wrang: `. The tables
below show the bare message.

## Offscreen runs: `MDH_SCREEN=none`

In a native program, setting the environment variable `MDH_SCREEN` to `none`
before `screen_open` keeps everything in memory and opens no window. The
functions behave the same, except:

- `screen_shaw` prints one line to standard error per frame,
  `mdh_screen frame N HASH`, where `HASH` is an FNV-1a 32-bit hash (eight hex
  digits) of the finished BGRA frame. Equal hashes mean equal frames, so
  tests can compare renderers without a display.
- No input is ever delivered: `key_down`, `key_pressed` and `mouse_down` are
  `nae`, `mouse_x`, `mouse_y`, `mouse_dx` and `mouse_dy` are `0`, and
  `screen_should_close` stays `nae` (it never becomes `aye` by itself).
- `screen_grab_mouse` checks its argument but changes nothing.

`MDH_SCREEN=none` also turns the [sound built-ins](./sound.md) into a null
device (see there).

## The framebuffer

`screen_open(w, h, title)` fixes the framebuffer size at `w` by `h`. Every
`screen_shaw` call must pass exactly that many pixels.

| Format | Palette argument | Bytes per pixel | Bytes required | Layout |
|---|---|---|---|---|
| Indexed | 768-byte `bytes` | 1 | `w * h` | Each byte indexes a 256-entry palette of RGB triples (red, green, blue, 3 bytes each). |
| BGRA | `naething` | 4 | `w * h * 4` | Blue, green, red, alpha for each pixel. Alpha is ignored; every pixel is shown opaque. |

On macOS (Apple Silicon) the native window is created four times the
framebuffer size (`w * 4` by `h * 4` points) and the frame is magnified with
nearest-neighbour filtering, so pixels stay sharp. The interpreter's raylib
build opens a window of exactly `w` by `h`.

Native keeps two internal frame buffers and swaps them on every
`screen_shaw`; you never see this, but your own `bytes` is copied, not kept,
so you can reuse and overwrite it straight away.

## Functions

### `screen_open(width, height, title)`

Opens the window and allocates the framebuffer.

| Argument | Type | Meaning |
|---|---|---|
| `width` | integer | Framebuffer width in pixels, greater than 0. |
| `height` | integer | Framebuffer height in pixels, greater than 0. |
| `title` | string | Window title. |

Returns `naething`. Only one window can be open at a time; call
`screen_close` first to open another.

Native errors:

| Message | When |
|---|---|
| `screen_open wants positive integer dimensions` | A dimension is not an integer, is zero or negative, is more than `INT32_MAX / 4`, or the buffer size would overflow. |
| `screen_open wants a string title` | `title` is not a string. |
| `screen_open: a windae is already open` | Called while a window is open. |
| (platform message) | The platform layer could not open a window; its message is passed through. |

Interpreter (raylib): `width` and `height` must be integers
(`width must be an integer`, `height must be an integer`), `title` a string
(`title must be a string`), and a second call hurls `Window already open`.

### `screen_shaw(pixels, palette)`

Presents one frame. ("Shaw" is Scots for show.)

| Argument | Type | Meaning |
|---|---|---|
| `pixels` | bytes | The frame, `w * h` bytes with a palette or `w * h * 4` without. |
| `palette` | bytes of length 768, or `naething` | Indexed colours, or `naething` for BGRA pixels. |

Returns `naething`. Besides drawing the frame, `screen_shaw` is the input
frame boundary: it clears the "pressed this frame" edges, keeps any fractional
mouse movement, and polls the keyboard and mouse again.

Native errors:

| Message | When |
|---|---|
| `screen_shaw: nae windae is open` | No window is open. |
| `screen_shaw wants bytes pixels` | `pixels` is not `bytes`. |
| `screen_shaw wants a 768-byte palette or naething` | `palette` is neither `naething` nor 768 bytes of `bytes`. |
| `screen_shaw: pixel length doesnae match the windae` | `pixels` has the wrong length for the format. |

Interpreter (raylib) uses different wording: `screen_shaw() pixels must be
bytes`, `screen_shaw() palette must be bytes or naething`, `screen_shaw()
palette must contain 768 RGB bytes`, `screen_shaw() needs exactly N pixel
bytes, got M`, and `Window not open`. It sizes the framebuffer from the
current window size.

### `screen_should_close()`

Returns `aye` once the user has asked to close the window (the close button),
otherwise `nae`. The interpreter's raylib build also returns `aye` when Escape is pressed (raylib's default exit key). In native code it also polls the input devices, so the
result is fresh even if you never call `screen_shaw`. Takes no arguments.
Native does not require an open window for this call (it returns `nae`);
the interpreter hurls `Window not open` without one.

### `screen_close()`

Closes the window and frees the framebuffers. Safe to call when nothing is
open (it does nothing). Returns `naething`.

### `screen_width()` and `screen_height()`

Return the framebuffer width and height as integers.

Native error: `Nae windae! Open the screen first` when no window is open.
Interpreter error: `Window not open`. The interpreter reports the live window
size rather than the size you asked for.

### `screen_grab_mouse(on)`

`on` must be `aye` (capture: hide the cursor and report relative motion) or
`nae` (release it). Returns `naething`. Bogle grabs the mouse while playing
and releases it in menus.

Errors: native `screen_grab_mouse wants aye or nae`, then
`Nae windae! Open the screen first`; interpreter `screen_grab_mouse() expects
aye or nae` and `Window not open`.

## Keyboard

Key state is sampled when the screen polls: after each `screen_shaw` and on
each `screen_should_close`.

### `key_down(name)`

`aye` while the key is held. `name` is a string.

### `key_pressed(name)`

`aye` if the key went down since the previous `screen_shaw`: one `aye` per
physical press, however long it is held.

Accepted names (native, case-insensitive):

- one letter `a` to `z`, or one digit `0` to `9`
- `up`, `down`, `left`, `right`, `space`, `enter`, `escape` (also `esc`),
  `shift`, `ctrl`, `alt`, `tab`

The interpreter (raylib) accepts the same names except `shift`, `ctrl`,
`alt` and `tab`, which it does not know.

Errors: native `key_down wants a string key` / `key_pressed wants a string
key`, `Dinnae ken that key name` for a name not listed, then `Nae windae! Open
the screen first`. Interpreter: `key name must be a string`,
`Unknown key: NAME`, `Window not open`.

```mdhavers
# run: native
screen_open(8, 8, "keys")
ken px = bytes(64)
ken pal = bytes(768)
screen_shaw(px, pal)
gin key_down("w") or key_down("up") {
    blether "walking forrit"
}
gin key_pressed("space") {
    blether "bang"
}
screen_close()
```

## Mouse

The pointer position is measured in framebuffer pixels from the top left.

| Function | Returns |
|---|---|
| `mouse_x()` | Pointer x, integer. |
| `mouse_y()` | Pointer y, integer. |
| `mouse_dx()` | Horizontal movement since the previous `screen_shaw`, whole pixels. |
| `mouse_dy()` | Vertical movement since the previous `screen_shaw`, whole pixels. |
| `mouse_down(button)` | `aye` while the button is held. `button` is `0`, `1` or `2`. |

Native `mouse_dx` and `mouse_dy` truncate toward zero and carry the fraction
to the next frame, so slow steady motion still adds up.

Errors: native `mouse_down wants a button frae 0 tae 2` for any other value,
and `Nae windae! Open the screen first` without a window. In the interpreter,
`button must be an integer`, `Invalid mouse button (use 0=left, 1=right,
2=middle)` and `Window not open`. The button numbering is `0` left, `1` right, `2` middle in both
(`runtime/platform/screen_macos.c:200-204` for native).

The interpreter (raylib) also has `mouse_pressed(button)`, which is `aye` for
one frame when the button goes down. It is not part of the native bridge, so
leave it out of programs that must also build natively.

## Interpreter-only drawing functions

A raylib interpreter build also has an immediate-mode drawing API
(`screen_begin`, `screen_end`, `screen_clear`, `screen_fps` and the `draw_*`
functions). They are described in [Graphics (raylib)](./builtins/graphics.md)
and are not available natively. Use the framebuffer and `screen_shaw` for
code that has to run in both places.

## The Bogle Castle loop

`games/bogle/shell/native.braw` is the whole native program:

```text
fetch "frame"
bogle_init(320, 200)
ken last = mono_ms()
whiles nae screen_should_close() {
    ken now = mono_ms()
    gin nae bogle_frame(now - last) {
        brak
    }
    last = now
    snooze(1)
}
bogle_close()
```

`bogle_init` in `games/bogle/shell/frame.braw` calls
`screen_open(320, 200, ...)` then `screen_grab_mouse(nae)`. Each frame,
`bogle_frame` polls `key_down` for W, A, S, D, the arrows, space, E, 1 to 4,
Enter and Escape (plus `mouse_down(0)` and `mouse_dx()`), steps the game,
renders into a `bytes(320 * 200)` indexed framebuffer and calls
`screen_shaw(bogle_fb, bogle_palette)`. `bogle_close` calls `screen_close()`.
