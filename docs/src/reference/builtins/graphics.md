# 2D drawing built-ins

`draw_pixel`, `draw_line`, `draw_rect`, `draw_circle`, `draw_text`, `rgb` and `rgba` draw shapes and text in the window that `screen_open` creates. The window, frame, keyboard, mouse and PCM sound calls (`screen_*`, `key_*`, `mouse_*`, `sound_*`) are covered in [screen and sound](./../screen.md).

## Availability

These seven names exist only in a build with the `graphics` feature (raylib). In a build without it they are not defined at all, so calling one fails with a "hasnae been defined yet" error. The default `full` build includes `graphics`.

Native code compiled with the LLVM backend has no window. A call to `draw_pixel`, `draw_line`, `draw_rect`, `draw_circle` or `draw_text` there stops with `Nae windae! Screen an' keyboard functions cannae be used in native code`. `rgb` and `rgba` are not known to the LLVM backend at all: the build fails with `Undefined variable: rgb`. The JavaScript output has none of the seven; a call fails in Node with `ReferenceError: draw_pixel is not defined`. The WAT/WASM compiler passes built-in calls by name to its host (`__mdh_builtinN`); whether a host provides these is untested.

## Window rule

Every drawing call needs an open window. Without one it fails with `Window not open`. Open one first with `screen_open(width, height, title)`; a second `screen_open` fails with `Window already open`.

## Frames

The code does not need `screen_begin` or `screen_end` around the drawing calls. In the interpreter both do nothing (`screen_begin` and `screen_end` return `naething`). Each `draw_*` call opens its own raylib drawing scope and draws straight away, so there is no batching between calls. The usual loop is:

1. `screen_clear(colour)` at the top of each frame
2. your `draw_*` calls
3. loop while `screen_should_close()` is `nae`

You may call `screen_begin()` and `screen_end()` around the drawing for clarity, but they change nothing.

## Drawing functions

All coordinates, sizes and the radius must be numbers. A float is truncated towards zero (`1.9` becomes `1`); anything else is rejected, for example `x must be an integer`. Colours are described below. Each call returns `naething`.

| Function | Arguments | Error messages for bad arguments |
|---|---|---|
| `draw_pixel(x, y, colour)` | one pixel | `x must be an integer`, `y must be an integer` |
| `draw_line(x1, y1, x2, y2, colour)` | a line from point 1 to point 2 | `x1 must be an integer` and so on |
| `draw_rect(x, y, width, height, colour)` | a filled rectangle, `x` and `y` is the top-left corner | `width must be an integer`, `height must be an integer` |
| `draw_circle(x, y, radius, colour)` | a filled circle centred on `x`, `y` | `radius must be an integer` |
| `draw_text(text, x, y, size, colour)` | text with its top-left at `x`, `y`, `size` is the font height in pixels | `text must be a string`, `size must be an integer` |

## Colours

A colour argument is either a name (a string) or a list of numbers.

### Colour lists

A list needs at least three numbers: red, green, blue (floats are truncated). A fourth is alpha, and defaults to 255 (opaque). A shorter list fails with `Color list must have at least 3 elements (r, g, b)`. Any other kind of value fails with `Color must be a string name or [r, g, b] list`.

Values in a hand-made list are cast to a byte without clamping, so keep them in 0 to 255. Use `rgb` and `rgba` to get clamped values.

| Function | Returns |
|---|---|
| `rgb(r, g, b)` | the list `[r, g, b, 255]`, each of `r`, `g`, `b` clamped to 0 to 255 |
| `rgba(r, g, b, a)` | the list `[r, g, b, a]`, each value clamped to 0 to 255 |

Both need numbers; a float is truncated towards zero before clamping. Anything else fails with `r must be an integer`, `g must be an integer`, `b must be an integer` or `a must be an integer`.

### Colour names

Names are not case sensitive. An unknown name fails with `Unknown color: <name>`.

| Colour | Scots name | English name |
|---|---|---|
| red | `reid` | `red` |
| green | `gress` | `green` |
| blue | `blae` | `blue` |
| white | `whit` | `white` |
| black | `bleck` | `black` |
| yellow | `yella` | `yellow` |
| orange | | `orange` |
| pink | | `pink` |
| purple | `purpie` | `purple` |
| gray | | `gray`, `grey` |
| dark gray | `derkgrey` | `darkgray` |
| light gray | `lichtgrey` | `lightgray` |
| brown | `broon` | `brown` |
| gold | `gowd` | `gold` |
| sky blue | `skyblae` | `skyblue` |

## Example

Needs a window/audio device. It passes `mdhavers check`, but running it opens a window and needs a `graphics` build.

```mdhavers
# run: check
screen_open(400, 300, "Drawing")

whiles nae screen_should_close() {
    screen_clear("bleck")
    draw_rect(20, 20, 100, 60, rgb(255, 180, 60))
    draw_circle(200, 150, 40, "skyblae")
    draw_line(0, 0, 400, 300, "gress")
    draw_pixel(200, 150, "reid")
    draw_text("Braw!", 20, 100, 24, rgba(255, 255, 255, 200))
}

screen_close()
```
