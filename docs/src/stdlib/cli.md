# cli

Building blocks for command-line programs without needing [colors](./colors.md): small `cli_*` style helpers, cursor control, box drawing, a `ProgressBar` and `Spinner` that return strings, a `CLITable`, line prompts, status messages, lists, trees and key/value printing.

Import: `fetch "stdlib/cli"`

Backend note: needs a terminal for the cursor and prompt functions; the prompts read from standard input with `speir`. If input has ended (`speir` gives `naething`) each prompt prints an empty line and then `Input's ended - cheerio, and haste ye back!` and returns its default, or `naething` where there is none. Loading prints `CLI module loaded! Ready tae build bonnie command-line tools!`. Helpers starting with `_` (`_style`, and `CLITable._calc_widths` / `_pad`) are internal and not documented. No function hurls its own errors; an unknown box or spinner style name fails with the interpreter's ordinary error.

## Constants

- `BOX_CHARS`: dict of border sets for the styles `"single"`, `"double"` and `"rounded"`, each with keys `tl tr bl br h v`.
- `SPINNER_STYLES`: dict of frame lists for `"dots"`, `"line"`, `"arrows"`, `"bounce"`, `"growing"`, `"circle"`, `"square"`, `"clock"`, `"moon"`.

## Functions

### cli_speir_or_awa(prompt)
Reads a line with `speir`; on end of input prints an empty line, then the cheerio message, and returns `naething`.

### cli_bold(text)
Returns `text` in bold (ANSI), with a reset.
### cli_dim(text)
Dim.
### cli_italic(text)
Italic.
### cli_underline(text)
Underlined.
### cli_red(text)
Red.
### cli_green(text)
Green.
### cli_yellow(text)
Yellow.
### cli_blue(text)
Blue.
### cli_magenta(text)
Magenta.
### cli_cyan(text)
Cyan.

### clear_screen()
Prints the clear-and-home escape.
### move_cursor(row, col)
Prints the cursor-position escape.
### hide_cursor()
Prints the hide-cursor escape.
### show_cursor()
Prints the show-cursor escape.
### clear_line()
Prints the erase-line escape.

### draw_box(width, height, style = "single")
Returns a list of lines forming an empty box of the given outer size. Does not print.

### print_box(content, padding = 1, style = "single")
Prints a box around a list of lines `content`, padded on each side by `padding` spaces.

### print_banner(text, style = "double")
Splits `text` on `\n` and prints it in a box with padding 2.

### print_header(text, char = "=")
Prints `text` between two lines of `char` as long as the text.

### print_divider(width = 40, char = "─")
Prints a line of `width` copies of `char`.

### hr(width = 40, style = "single")
Prints a rule: `"double"` uses `═`, `"dashed"` uses `╌`, anything else `─`.

### make_progress(total, width = 40)
Returns `ProgressBar(total, width)`.

### make_spinner(message = "", style = "dots")
Returns `Spinner(message, style)`.

### make_table()
Returns an empty `CLITable`.

### prompt_text(message, default = naething)
Asks `message [default]: `. Returns the trimmed answer; an empty answer gives `default`; end of input gives `default`.

### prompt_confirm(message, default = aye)
Asks `message [Y/n]: ` (or `[y/N]`). `y`, `yes`, `aye` give `aye`; `n`, `no`, `nae` give `nae`; anything else (including empty or end of input) gives `default`.

### prompt_choice(message, choices)
Prints a numbered list and asks `Enter number: ` until a valid number is typed; returns the chosen element. Prints `Invalid choice, try again.` for a number out of range and `Please enter a number.` for text. Returns `naething` at end of input.

### prompt_password(message)
Reads a trimmed line. It does not hide input: the module cannot disable echo.

### print_success(message)
Prints a green check and `message`.
### print_error(message)
Red cross.
### print_warning(message)
Yellow warning sign.
### print_info(message)
Blue info sign.
### print_step(step, total, message)
Prints a cyan `[step/total]` then `message`.

### print_list(items, bullet = "•")
Prints each item as `  • item`.
### print_numbered(items)
Prints `  1. item`, `  2. item`, ...

### print_tree(tree, prefix = "", is_last = aye)
Prints a tree from a dict `{"name": "...", "children": [ ... ]}` using `├──`, `└──` and `│` connectors; `children` is optional. The top node is drawn with a `└──` connector.

### print_kv(key, value, separator = ":")
Prints the bold key, the separator and the value as a string.

### print_dict(dict, indent = 0)
Prints each key bold with its value; nested dicts are printed indented by two spaces per level.

## Classes

### ProgressBar
`ProgressBar(total, width = 40)`. Fields: `total`, `current` (0), `width`, `fill_char` (`█`), `empty_char` (`░`), `show_percent`, `show_count`, `prefix`, `suffix`. Builder methods return the bar: `with_fill(char)`, `with_empty(char)`, `with_prefix(prefix)`, `with_suffix(suffix)`, `hide_percent()`, `hide_count()`, `update(value)`.

- `increment(amount = 1)`: adds and clamps at `total`; returns the bar.
- `render()`: returns a string such as `[█████░░░░░] 50% (5/10)`. A `total` of 0 gives 0%.
- `display()`: prints `\r` plus `render()`.
- `is_complete()`: `aye` once `current >= total`.

### Spinner
`Spinner(message = "", style = "dots")`. Fields: `message`, `frames`, `current_frame`, `running`.

- `with_style(style)`, `with_message(msg)`: chainable setters.
- `next_frame()`: advances and returns the new frame character.
- `render()`: returns frame plus space plus message.
- `display()`: prints `\r` plus `render()`, then advances.
- `success(msg = naething)`, `failure(msg = naething)`, `warning(msg = naething)`: print a green check, red cross or yellow warning with `msg`, defaulting to the spinner's message.

### CLITable
`CLITable()`. Fields: `headers`, `rows`, `col_widths`, `border_style`, `header_color` (stored but headers are always cyan).

- `set_headers(headers)`, `add_row(row)`, `set_rows(rows)`, `with_border(style)`: chainable; widths are recalculated by `set_headers`, `add_row` and `set_rows` (not by `with_border`).
- `render()`: returns the list of lines. Cells are centred with one space of slack; headers are cyan.
- `display()`: prints every line from `render()`.

## Compared with other modules

- [colors](./colors.md) has the full colour set; `cli_*` is only bold, dim, italic, underline and six colours, but needs no other module.
- [menus](./menus.md) also has `ProgressBar`, `Spinner` and `Table` classes that print directly and interactive `Menu`, `Form` and inputs; fetching `cli` and `menus` together makes the later `ProgressBar` and `Spinner` win.
- [tui](./tui.md) prints widgets directly; `cli` returns strings and lines where it can, which is easier to test.
- [argparse](./argparse.md) parses the command line, which `cli` does not.

## Example

```mdhavers
fetch "stdlib/cli"

print_header("Report")
print_success("loaded")
print_list(["porridge", "haggis"])

ken bar = make_progress(10, 10)
bar.update(5)
blether bar.render()

ken t = make_table()
t.set_headers(["Name", "Age"]).add_row(["Ann", 30])
t.display()

print_tree({"name": "root", "children": [{"name": "a"}, {"name": "b"}]})
```
