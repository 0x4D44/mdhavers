# tui

Print-only terminal widgets: rules, headers, boxes, panels, progress bars, lists, tables, dialogs, cards, columns, form mock-ups, gauges, sparklines and status dots. Every function here prints with `blether` (the exceptions return a string: `repeat_char`, `pad_text`, `bold`, `sparkline`). There is no keyboard input; for real prompts use [menus](./menus.md) or [cli](./cli.md).

Import: `fetch "stdlib/tui"`

Backend note: pure mdhavers; the output uses ANSI escapes, box-drawing characters and some emoji, so it needs a UTF-8 terminal. `tui` itself runs `fetch "colors"`, so [colors](./colors.md) is loaded as well, and loading prints two banner lines (`Colors module loaded! ...` and `TUI module loaded! Makin' yer terminal bonnie!`).

Helpers starting with `_` are internal and not documented (there are none here). None of the functions hurl their own errors.

## Functions

### hr(width = 50, char = "─")
Prints a horizontal rule of `width` copies of `char`.

### hr_double(width = 50)
Same as `hr` with `═`.

### blank()
Prints an empty line.

### centered(text, width = 50)
Prints `text` indented by `(width - len(text)) / 2` spaces (never negative).

### header(title, width = 50)
Rule, centred title, rule.

### fancy_header(title, width = 50)
Three-line double-border banner (`╔═╗`, `║ title ║`, `╚═╝`) with the title centred.

### colored_header(title, color_func, width = 50)
Like `fancy_header` but passes each line through `color_func` (for example `red` from [colors](./colors.md)).

### repeat_char(char, n)
Returns `char` repeated `n` times (does not print).

### box(content, width = 40, style = "single")
Prints a box around `content`; lines split on `\n`, padded (not wrapped) to `width - 2`. `style` `"double"` uses double borders, anything else single.

### panel(title, content, width = 40)
A single-border box with `title` embedded in the top edge.

### progress(value, max_val, width = 30, label = "")
Prints `[<green bar>] N%`, prefixed by `label: ` when `label` is non-empty. The ratio is clamped to 0..1. Careful: with two integers `value / max_val` is integer division, so `progress(5, 10)` shows 0%; pass floats (`5.0`) for fractions.

### spinner(frame, message = "Loading")
Prints one braille spinner frame (`frame` modulo 10) and `message...`. Call it in a loop with an increasing `frame`.

### bullet_list(items, bullet = "•")
Prints each item as `  • item`.

### numbered_list(items, start = 1)
Intended to print `  1. a`, `  2. b`. The counter is incremented after the loop, so every line is numbered `start`.

### menu(title, options)
Prints a `fancy_header`, the options as `[1] ...` and `Enter yer choice (1-N):`. It only displays; it does not read input or return a choice.

### selection_menu(options, selected_idx = 0)
Prints the options with a cyan `▶` marker on `selected_idx`.

### table(headers, rows, col_width = 12)
Prints a bordered table with fixed-width columns; cells are padded or truncated with `…` by `pad_text`. Headers are bold. For a data-oriented table see [table](./table.md).

### pad_text(text, width)
Returns `text` padded with spaces to `width`, or truncated to `width - 1` characters plus `…` when too long.

### bold(text)
Returns `text` wrapped in bold-on/reset escapes. This redefines `bold` from [colors](./colors.md) with the same effect.

### info_box(message, width = 40)
Single-border box with a cyan `ℹ` icon. Message is padded or truncated to `width - 4`.

### success_box(message, width = 40)
Green box with a `✓` icon.

### warning_box(message, width = 40)
Yellow box with a `⚠` icon.

### error_box(message, width = 40)
Red box with a `✗` icon. For the three coloured boxes the message is not truncated; one longer than `width - 4` overflows the border.

### confirm_dialog(question)
Prints a fixed-width dialog `? question` with `[Y] Aye    [N] Nae`. Display only.

### scots_banner(text)
Prints a three-line banner with flag and thistle emoji around `text`.

### thistle_divider(width = 40)
Prints `~~~ 🌿 ~~~` of about `width` characters.

### whisky_toast(message)
Prints a toast with `message` centred and `Slàinte mhath!`.

### card(title, content, width = 40)
Rounded-corner card with a bold title, a divider and the lines of `content`.

### stat_card(label, value, icon = "📊")
Fixed-width four-line box showing `icon label` and the bold `value`.

### columns(items, num_cols = 2, col_width = 20)
Prints `items` in rows of `num_cols` columns, each padded to `col_width`.

### form_field(label, value = "", width = 30)
Prints `  label: [value]`, or a run of underscores when `value` is empty.

### form(title, fields)
Prints a header, then one `form_field` per entry of `fields` (each `[label]` or `[label, value]`), then `[Submit]  [Cancel]`. Display only.

### gauge(label, value, max_val = 100, width = 20)
Prints `label:` then a bar coloured red (below 30%), yellow (below 70%) or green, plus `value/max_val`. Integer division applies as for `progress`: `gauge("cpu", 50)` draws an empty bar; use `50.0` or a float `max_val`.

### sparkline(data, width = 20)
Returns a string of block characters scaled between the minimum and maximum of `data` (empty string for an empty list). The `width` parameter is unused. With integer data the scaling is integer division, so `[1, 5, 3, 9]` gives `▁▁▁█`; use floats for a smooth line.

### status_indicator(label, status)
Prints a dot and `label`: green for `ok`, `online`, `active`; yellow for `warning`, `pending`; red for `error`, `offline`, `inactive`; an empty circle otherwise.

### clear_screen()
Prints the clear-screen escape.

### move_cursor(row, col)
Prints the cursor-position escape.

### hide_cursor()
Prints the hide-cursor escape.

### show_cursor()
Prints the show-cursor escape.

## Compared with other modules

- [colors](./colors.md) supplies the colour functions `tui` uses and has its own `progress_bar` (returns a string) and `box`. `tui` prints directly.
- [cli](./cli.md) covers similar ground (`print_box`, `CLITable`, `ProgressBar`, spinners) but returns lines or objects and adds real prompts. [menus](./menus.md) has interactive widgets.
- Prefer `tui` for quick one-shot decoration, `cli` for an application with prompts and tables, `table` for data manipulation.
- `tui`, `colors` and `cli` all define `clear_screen`, `move_cursor`, `hide_cursor`, `show_cursor`; the last module fetched wins. `tui` also defines `table`, `box` and `hr`, as do other modules.

## Example

```mdhavers
fetch "stdlib/tui"

fancy_header("Hello", 20)
bullet_list(["porridge", "haggis"])
table(["Name", "Age"], [["Ann", 30], ["Bob", 25]], 8)
blether sparkline([1.0, 5.0, 3.0, 9.0])
status_indicator("db", "ok")
```
