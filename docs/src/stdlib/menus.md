# menus

Interactive prompts for terminal programs: a numbered `Menu`, `confirm`, validated `TextInput`, range-checked `NumberInput`, `ChoiceInput`, a `Form` builder that chains them, plus simple `ProgressBar`, `Spinner` and `Table` classes and quick `ask`, `ask_int`, `ask_float` and `choose` helpers.

Import: `fetch "stdlib/menus"`

Backend note: reads standard input with `speir`, so it needs a terminal or piped input (interpreter only in practice). If input ends, the prompt prints `Input's ended - cheerio, and haste ye back!`, sets a module flag (`menus_state[0]`) and returns `naething` (or the default, for `confirm`). Loading prints `Menus module loaded! Ready tae ask some questions!`. Helpers starting with `_` are internal; the one public-looking helper `menus_speir_or_awa` is the shared line reader and is documented below for completeness. Pages checked by running with piped input; only the example is check-only.

## Constants

- `menus_state`: one-element list `[nae]`; becomes `[aye]` once input has ended. `Menu.show_required` stops looping when it is set.
- `SPINNER_FRAMES`: `["|", "/", "-", "\\"]`.

## Functions

### menus_speir_or_awa(prompt)
Reads one line with `speir`. On end of input prints a blank line and the cheerio message, sets `menus_state[0]` to `aye`, and returns `naething`.

### confirm(question, default = nae)
Asks `question [Y/n] ` (or `[y/N]`). `y`, `yes`, `aye` give `aye`; `n`, `no`, `nae` give `nae`; empty or end of input gives `default`; other text prints `Didnae understand that. Using default.` and gives `default`.

### ask(prompt)
Returns the raw line for `prompt: `, or `naething` at end of input.

### ask_int(prompt)
`NumberInput(prompt).ask()`: an integer, re-asking only for range errors (none set here). Text that is not a number hurls `tae_int`'s error (`Cannae turn '<text>' intae an integer`).

### ask_float(prompt)
Same with `as_float()`.

### choose(prompt, options)
Shows a numbered `ChoiceInput` for the list `options` and returns the chosen element.

## Classes

### Menu
`Menu(title = "Menu")`. Fields: `title`, `options` (list of `{"label", "value"}`), `show_numbers` (aye).

- `add(label, value = naething)`: adds an option whose value is `value`, or the label when `value` is omitted. Returns the menu.
- `add_all(options)`: adds each label. Returns the menu.
- `show()`: prints the title, an underline and the numbered options, asks `Enter yer choice: `, and returns the chosen option's value, or `naething` for a number out of range or end of input. Text that is not a number hurls `tae_int`'s error.
- `show_required()`: calls `show` until a valid choice, printing `Och, that's no' a valid choice. Try again!` between attempts; stops on end of input.
- `clear()`: removes all options; returns the menu.

### TextInput
`TextInput(prompt)`. Fields: `prompt`, `validators`, `default`, `required`.

- `with_default(value)`, `required()`, `add_validator(fn)`: chainable. A validator takes the text and returns `aye` when valid or an error message otherwise.
- `validate(value)`: runs the validators; returns `aye` or the first non-`aye` result.
- `ask()`: shows `prompt [default]: `; returns the text. Empty input gives the default, else repeats with `This field is required!` when required, else `""`. Failed validation prints `Invalid: <message>` and asks again. End of input gives `naething`.

### NumberInput
`NumberInput(prompt)`. Fields: `prompt`, `min_val`, `max_val`, `default`, `is_float`.

- `min(value)`, `max(value)`, `with_default(value)`, `as_float()`: chainable.
- `ask()`: returns an integer (or float). Empty input returns the default if set. Out-of-range input prints `Value must be at least <min>` or `Value must be at most <max>` and asks again. Non-numeric text hurls the `tae_int` or `tae_float` error rather than re-asking. End of input gives `naething`.

### ChoiceInput
`ChoiceInput(prompt)`. Fields: `prompt`, `choices`, `default_idx`.

- `add(label, value = naething)`, `add_all(choices)`: chainable.
- `with_default(idx)`: default choice, counted from 1; chainable.
- `ask()`: prints the numbered choices (marking the default), asks `Enter choice: `, returns the value. An out-of-range number prints `Invalid choice, try again!` and asks again; text that is not a number (or empty input with no default) hurls `tae_int`'s error. End of input gives `naething`.

### ProgressBar
`ProgressBar(total, width = 40)`. Fields: `total`, `width`, `current`, `label` (`"Progress"`).

- `with_label(label)`: chainable.
- `update(current)`, `tick()`: set or increment, then print the bar with a leading `\r`. Both return the bar.
- `render()`: prints `\rlabel: [bar] N% (current/total)`. A `total` of 0 is a division-by-zero error.
- `complete()`: sets `current` to `total`, renders, prints a newline.

### Spinner
`Spinner(message = "Loading")`. Fields: `message`, `frame`.

- `spin()`: prints the next `|/-\` frame and `message...`.
- `done(final_message = "Done!")`: prints a check mark and the message. `fail(error_message = "Failed!")`: prints a cross and the message.

### Form
`Form(title = "Form")`. Fields: `title`, `fields`, `values`. Adders are chainable and take the field `name` (the key in the result) and a `prompt`:

- `text(name, prompt, required = nae)`, `number(name, prompt, min = naething, max = naething)`, `yesno(name, prompt, default = nae)`, `choice(name, prompt, options)`.
- `submit()`: prints the title and asks each field in order with the matching input class; returns the dict of values (also stored in `values`).
- `get(name)`: the stored value or `naething`. `all()`: the whole dict.

### Table
`Table(headers)`. Fields: `headers`, `rows`, `col_widths`.

- `add_row(row)`, `add_rows(rows)`: chainable; widths grow to fit.
- `show()`: prints a `+---+` grid. The cells are left-aligned.

This class has the same name as [table](./table.md)'s `Table`; do not fetch both in one program.

## Compared with cli and tui

[cli](./cli.md) has similar prompts (`prompt_text`, `prompt_confirm`, `prompt_choice`) as plain functions and returns its `ProgressBar`, `Spinner` and table as strings and lines; `menus` bundles classes with validation and a `Form`, and prints directly. [tui](./tui.md) is display only. Prefer `menus` for multi-field input, `cli` for one-off prompts. The `ProgressBar`, `Spinner` and `Table` names clash with `cli` and `table`.

## Example

_Checked, not run: needs a live peer/device._

```mdhavers
fetch "stdlib/menus"

ken m = Menu("Pick one").add("Porridge", 1).add("Haggis", 2)
ken choice = m.show()
blether choice

ken name = TextInput("Name").with_default("Bob").ask()
blether name
```
