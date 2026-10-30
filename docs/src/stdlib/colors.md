# colors

ANSI escape-code helpers for colourful terminal output: foreground, background, style, 256-colour and RGB helpers, cursor control, a progress bar, a box, rainbow and gradient text, Scottish theme colours and levelled log lines. Every colour helper takes a string and returns it wrapped in the escape codes plus a reset; only the terminal-control and `log_*` functions print.

Import: `fetch "stdlib/colors"`

Backend note: pure mdhavers, no native built-ins, so it works on any backend that can run the interpreter. Loading the module prints `Colors module loaded! Ready tae add some colour!` once.

Helpers starting with `_` are internal and not documented (there are none in this module).

## Constants

- `ESC` (`"\x1b["`) and `RESET` (`"\x1b[0m"`).
- Foreground: `FG_BLACK`, `FG_RED`, `FG_GREEN`, `FG_YELLOW`, `FG_BLUE`, `FG_MAGENTA`, `FG_CYAN`, `FG_WHITE`, and the `FG_BRIGHT_*` set of the same eight names.
- Background: `BG_BLACK` ... `BG_WHITE`, and `BG_BRIGHT_*`.
- Styles: `BOLD`, `DIM`, `ITALIC`, `UNDERLINE`, `BLINK`, `REVERSE`, `HIDDEN`, `STRIKETHROUGH`.

## Functions

### styled(text, style)
Returns `style + text + RESET`. `style` is any of the constants above.

The eight foreground helpers wrap `text` in the matching `FG_*` colour via `styled`.

### black(text)
Foreground black.
### red(text)
Foreground red.
### green(text)
Foreground green.
### yellow(text)
Foreground yellow.
### blue(text)
Foreground blue.
### magenta(text)
Foreground magenta.
### cyan(text)
Foreground cyan.
### white(text)
Foreground white.

### bright_black(text)
Bright foreground black (grey).
### bright_red(text)
Bright red.
### bright_green(text)
Bright green.
### bright_yellow(text)
Bright yellow.
### bright_blue(text)
Bright blue.
### bright_magenta(text)
Bright magenta.
### bright_cyan(text)
Bright cyan.
### bright_white(text)
Bright white.

### bold(text)
Bold style.
### dim(text)
Dim style.
### italic(text)
Italic style.
### underline(text)
Underlined.
### blink(text)
Blinking.
### reverse(text)
Reverse video.
### strikethrough(text)
Struck through.

### bg_black(text)
Background black.
### bg_red(text)
Background red.
### bg_green(text)
Background green.
### bg_yellow(text)
Background yellow.
### bg_blue(text)
Background blue.
### bg_magenta(text)
Background magenta.
### bg_cyan(text)
Background cyan.
### bg_white(text)
Background white.

### style_all(text, styles)
`styles` is a list of escape strings; they are concatenated in order before `text`, then `RESET` is appended. Example: `style_all("x", [BOLD, FG_RED])`.

### bold_red(text)
`BOLD` plus red.
### bold_green(text)
`BOLD` plus green.
### bold_yellow(text)
`BOLD` plus yellow.
### bold_blue(text)
`BOLD` plus blue.
### bold_cyan(text)
`BOLD` plus cyan.
### bold_magenta(text)
`BOLD` plus magenta.

### error_style(text)
Bold red.
### warning_style(text)
Bold yellow.
### success_style(text)
Bold green.
### info_style(text)
Bold cyan.

### fg_256(text, color_code)
Foreground from the 256-colour palette (`ESC[38;5;Nm`). `color_code` is converted with `tae_string`.

### bg_256(text, color_code)
Background from the 256-colour palette (`ESC[48;5;Nm`).

### fg_rgb(text, r, g, b)
True-colour foreground (`ESC[38;2;r;g;bm`).

### bg_rgb(text, r, g, b)
True-colour background (`ESC[48;2;r;g;bm`).

### clear_screen()
Prints `ESC[2J ESC[H` (clear and home the cursor).

### move_cursor(row, col)
Prints the cursor-position escape for `row`, `col`.

### hide_cursor()
Prints the hide-cursor escape.

### show_cursor()
Prints the show-cursor escape.

### save_cursor()
Prints the save-cursor-position escape.

### restore_cursor()
Prints the restore-cursor-position escape.

### clear_line()
Prints the erase-line escape.

### progress_bar(current, total, width = 40, filled_char = "█", empty_char = "░")
Returns (does not print) a string like `[<green bar>] 50%`. The bar is coloured green. Percentage and filled width are floored. Dividing by a `total` of 0 is not guarded.

### box(text, width = 0)
Returns a three-line single-border box around one line of `text`, joined by `\n`. With `width` 0 the box is `len(text) + 4` wide; a larger `width` pads the text. Single line only.

### repeat_char(c, n)
Returns `c` repeated `n` times.

### rainbow(text)
Returns `text` with each non-space character cycling through red, yellow, green, cyan, blue, magenta, ending with `RESET`.

### gradient(text, start_color, end_color)
Intended as a 256-colour gradient between two palette codes. The ratio is `i / len(text)`, which is integer division for integer positions, so every character gets the `start_color` code (see Notes).

### saltire_blue(text)
RGB (0, 91, 153).
### saltire_white(text)
Bright white.
### tartan_red(text)
RGB (180, 18, 20).
### tartan_green(text)
RGB (0, 100, 50).
### tartan_navy(text)
RGB (0, 35, 102).
### tartan_yellow(text)
RGB (255, 204, 0).
### heather(text)
RGB (150, 120, 180).
### whisky(text)
RGB (205, 127, 50).
### highland_green(text)
RGB (34, 139, 34).

### log_error(message)
Prints `[ERROR]` (bold red) then `message`.
### log_warning(message)
Prints `[WARN]` (bold yellow) then `message`.
### log_success(message)
Prints `[OK]` (bold green) then `message`.
### log_info(message)
Prints `[INFO]` (bold cyan) then `message`.
### log_debug(message)
Prints the whole line `[DEBUG] message` dimmed.

## Notes

- None of these functions hurl errors of their own. Missing arguments or a `total` of 0 produce the interpreter's normal errors.
- Known issue: `gradient` gives a single colour because `ratio = i / len_text` is integer division.
- Other terminal modules define the same cursor/screen names: [tui](./tui.md) and [cli](./cli.md).

## Compared with tui and cli

[cli](./cli.md) has its own small `cli_*` colour set that does not need this module; [tui](./tui.md) fetches `colors` and redefines `bold`, `clear_screen`, `move_cursor`, `hide_cursor`, `show_cursor` itself. Prefer `colors` when you only want styled strings; use `cli` for prompts and tables.

## Example

```mdhavers
fetch "stdlib/colors"

blether red("error") + " " + bold("done")
blether progress_bar(5, 10, 10)
blether box("hello")
blether styled("x", BOLD + FG_GREEN)
```
