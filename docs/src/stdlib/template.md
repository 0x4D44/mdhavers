# template

String building and formatting: a `StringBuilder`, number/currency/byte/duration formatters, text wrapping, ASCII text tables and boxes, and `{{key}}` substitution.

Import: `fetch "stdlib/template"`

Backend: pure mdhavers (uses the built-ins `repeat`, `char_at`, `pooer`, `floor`, `round`, `abs`, `replace`, `keys`). Loading the module prints `Template module loaded! Mak yer strings bonnie!`. Nothing here uses `hurl`. Helpers starting with `_` are internal and not documented (`TextTable` has several).

## Functions

### string_builder()
Returns a new `StringBuilder`.

### pad_left(s, width, ch = " ")
Left-pads `tae_string(s)`, so numbers are accepted.

### pad_right(s, width, ch = " ")
Right-pads `tae_string(s)`.

### pad_center(s, width, ch = " ")
Centres `tae_string(s)`; odd extra padding goes on the right.

### format_number(n, sep = ",")
Groups digits in threes: `format_number(1234567)` is `1,234,567`. Works for integers. Floats are not handled: the decimal point is treated as a digit, so `format_number(-1234.5)` gives `-123,4.5`.

### format_currency(amount, symbol = "£", decimals = 2)
`format_currency(1234.5)` is `£1,234.50`. Known edge cases: a fraction that rounds up to a full unit is not carried (`format_currency(-3.999)` gives `-£3.100`), and `decimals = 0` gives `$5.0` rather than `$5`.

### format_percent(value, decimals = 1)
`format_percent(0.256)` is `25.6%`; `format_percent(0.5, 0)` is `50%`.

### format_bytes(bytes)
Scales to B, KB, MB, GB or TB. Whole bytes are shown as `512 B`. Larger units use integer division on the rounded value, so `1536` gives `1 KB`, not `1.5 KB`.

### format_duration(seconds)
`format_duration(3725)` is `1h 2m 5s`; `90061` is `1d 1h 1m 1s`; `0` is `0s`.

### wrap_text(text, width)
Greedy word wrap, lines joined with `\n`.

### truncate(text, max_len, suffix = "...")
Cuts to `max_len` including `suffix`. If `max_len` is no more than the suffix length it returns just the suffix.

### text_table()
Returns a new `TextTable`.

### box(text, padding = 1)
Draws an ASCII box around (possibly multi-line) text with `padding` blank lines and spaces.

### titled_box(title, text, width = 40)
A box of total `width` with the title centred in the top border. Long lines are cut to fit.

### render_template(template_str, context)
Replaces each `{{key}}` with `tae_string(context[key])`. Keys missing from the context are left as they are.

## Classes

### StringBuilder
`StringBuilder()` takes no arguments and starts empty. Every method that changes it returns `masel`, so calls chain.

| Method | Notes |
| --- | --- |
| `append(s)` | Adds `tae_string(s)`. |
| `appendln(s)` | Adds `tae_string(s)` then a newline. |
| `append_all(items)` | Adds each item as a string. |
| `append_join(items, sep)` | Adds the items separated by `sep`. |
| `clear()` | Empties the builder. |
| `length()` | Total characters held. |
| `is_empty()` | `aye` if nothing has been added. |
| `build()` | Returns the joined string. |
| `tae_string()` | Same as `build()`. |

Field: `parts` (list of string pieces).

### TextTable
`TextTable()` takes no arguments. Fields: `headers`, `rows`, `col_widths`.

| Method | Notes |
| --- | --- |
| `set_headers(headers)` | Sets the header row; returns `masel`. |
| `add_row(row)` | Appends a row (any values; shown with `tae_string`); returns `masel`. |
| `render()` | Returns the table as a string with `+---+` separator lines. Data lines start and end with ` | ` (with a leading and trailing space), so they sit one column off the `+---+` border lines. |
| `tae_string()` | Same as `render()`. |

## Compared with patterns_match

`Template` in [patterns_match](./patterns_match.md) also does `{{key}}` substitution (with custom delimiters and a `placeholders()` method); `render_template` here is the one-line function form. `template` also defines `pad_left`, `pad_right` and `truncate`, which share names with [strings](./strings.md) and [text](./text.md); do not fetch several of them together.

## Example

```mdhavers
fetch "stdlib/template"

ken sb = string_builder()
sb.append("Total: ").append(format_number(1234567)).appendln("")
sb.append(format_duration(3725))
blether sb.build()
blether format_currency(1234.5)
blether render_template("Hello {{name}}!", {"name": "Jock"})
blether box("braw")
```
