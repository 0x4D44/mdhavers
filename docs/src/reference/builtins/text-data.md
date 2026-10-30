# Text and Data Built-ins

String helpers, character codes, regular expressions, JSON, and a few Scots text toys. All are registered by the interpreter in `src/interpreter.rs`.

Strings are indexed by character, not by byte, except where a row says otherwise. A wrong argument type gives an error of the form `name() needs a string` (the exact text is given where it differs).

**Backends.** Checked by compiling and running this page's Examples block. JS (`compile`, run under Node) and LLVM (`build`) printed the same output as the interpreter. WASM (`wasm`) compiles every call, passing each built-in by name to the host runner (`__mdh_builtinN`, see `src/wasm_compiler.rs` and `src/wasm_runner.rs`); the WASM output was not run, so treat WASM as untested. Functions not used in the Examples block are untested on the compiled backends.

## Searching and slicing

| Call | Returns | Notes |
|------|---------|-------|
| `char_at(text, index)` | 1-character string | A negative index counts from the end. Error: `Index <i> oot o' bounds fer string o' length <n>`; `char_at() needs a string`; `char_at() needs an integer index`. |
| `substring(text, start, end)` | string | `end` is exclusive. Negative values count from the end; out-of-range values are clamped, so it never fails on bounds. Error: `substring() needs a string`, `substring() needs integer indices`. |
| `index_of(text, needle)` | integer | Character index of the first match, or `-1`. Also works on a list: the index of the first equal element, or `-1`. Error: `index_of() needs a string/list and a value`. |
| `last_index_of(text, needle)` | integer | Character index of the last match, or `-1`. Error: `last_index_of() needs two strings`. |
| `starts_wi(text, prefix)` | boolean | Scots spelling of `starts_with`. Error: `starts_wi() needs two strings`. |
| `ends_wi(text, suffix)` | boolean | Scots spelling of `ends_with`. Error: `ends_wi() needs two strings`. |
| `starts_with(text, prefix)` | boolean | Error: `starts_with() needs two strings`. |
| `ends_with(text, suffix)` | boolean | Error: `ends_with() needs two strings`. |
| `replace(text, from, to)` | string | Replaces every occurrence. Error: `replace() needs three strings`. |
| `repeat(text, count)` | string | Error: `Cannae repeat a negative number o' times!`; `repeat() needs a string and an integer`. |
| `skelp(text, size)` | list of strings | Cuts into pieces of `size` characters; the last may be shorter. Errors: `skelp() size must be positive`; `skelp() needs a string and size`; `skelp() needs integer size`. |
| `words(text)` | list of strings | Splits on spaces, tabs and newlines, dropping empty pieces. Error: `words() needs a string`. |

## Testing and tidying

| Call | Returns | Notes |
|------|---------|-------|
| `is_digit(text)` | boolean | True for a non-empty string of ASCII digits. |
| `is_alpha(text)` | boolean | True for a non-empty string of ASCII letters only. |
| `is_space(text)` | boolean | True for a non-empty string of spaces, tabs, `\n` or `\r`. |
| `is_blank(text)` | boolean | True for an empty string or whitespace only (note that `is_space("")` is false). |
| `is_empty(value)` | boolean | Works on a string, list or dict. Error: `is_empty() needs a string, list, or dict`. |
| `trim(text)` | string | Removes leading and trailing space, tab, `\n`, `\r`. |
| `trim_start(text)` | string | Leading side only. |
| `trim_end(text)` | string | Trailing side only. |
| `wheesht_aw(text)` | string | Trims and collapses every run of whitespace to one space. |
| `capitalize(text)` | string | Upper-cases the first byte if it is an ASCII letter; the rest is untouched. |
| `title(text)` | string | Upper-cases the first letter of each word and lower-cases the rest (ASCII only). A non-string argument hurls `title() needs a string`. |

## Character codes

| Call | Returns | Notes |
|------|---------|-------|
| `ord(text)` | integer | Code point of the first character. Error: `Cannae get ord o' empty string!`; `ord() needs a string`. |
| `chr(code)` | 1-character string | Error: `chr() needs a valid Unicode codepoint (0 to 1114111), got <n>`; `Invalid Unicode codepoint: <n>` for a surrogate (55296 to 57343); `chr() needs an integer`. |

## Scots text toys

| Call | Returns | Notes |
|------|---------|-------|
| `geggie(text)` | string | The first and last characters joined (`""` for an empty string). |
| `banter(a, b)` | string | Weaves two strings together one character at a time; the longer one's tail is kept. Error: `banter() needs two strings`. |
| `blether_format(template, values)` | string | Replaces each `{key}` in the template with the matching dict value. Unmatched placeholders stay as they are. Errors: `blether_format needs a template string`, `blether_format needs a dictionary o' values`. |
| `haver()` | string | One random line of nonsense, such as `Dinnae fash yersel!`. |
| `slainte()` | string | One random Scottish toast. |

## Regular expressions

Patterns use the Rust `regex` syntax. A bad pattern gives `Invalid regex '<pattern>': <err>`. Match dictionaries hold `match` (the text), and `start` and `end` (byte offsets, not character indices).

| Call | Returns | Notes |
|------|---------|-------|
| `regex_test(text, pattern)` | boolean | Does the pattern match anywhere? |
| `regex_match(text, pattern)` | dict or `naething` | The first match as `{match, start, end}`, or `naething`. |
| `regex_match_all(text, pattern)` | list of dicts | Every non-overlapping match. |
| `regex_replace(text, pattern, replacement)` | string | Replaces every match. `$1` style group references work in `replacement`. |
| `regex_replace_first(text, pattern, replacement)` | string | Replaces the first match only. |
| `regex_split(text, pattern)` | list of strings | Splits wherever the pattern matches. |

Each takes strings for every argument, and reports `regex_test() needs a string` or `regex_test() needs a pattern string` (with its own name) otherwise. `regex_replace` and `regex_replace_first` also report `regex_replace() needs a replacement string` (with their own name).

## JSON

| Call | Returns | Notes |
|------|---------|-------|
| `json_parse(text)` | value | Objects become dicts, arrays lists, `null` becomes `naething`. Error: `json_parse() expects a string, ya numpty!`, plus parse errors. |
| `json_stringify(value)` | string | JSON on one line, with a space after each `,` and `:` (`{"a": [1, 2]}`). `naething` becomes `null`. |
| `json_pretty(value)` | string | Indented JSON, two spaces per level. |
| `json_stringify_pretty(value)` | string | Alias for `json_pretty`. |

## Examples

```mdhavers
blether char_at("hello", -1)
blether substring("hello", 1, -1)
blether index_of("hello", "l")
blether last_index_of("hello", "l")
blether words("  a  b\tc ")
blether title("hello wORLD")
blether wheesht_aw("  a   b  c ")
blether skelp("abcdefg", 3)
blether banter("abc", "12345")
blether ord("A")
blether chr(66)
blether blether_format("Hullo {name}, ye are {age}", {"name": "Hamish", "age": 42})

blether regex_test("abc123", "[0-9]+")
blether regex_match("abc123def", "[0-9]+")
blether regex_match("abc", "[0-9]+")
blether regex_replace("a1b22", "[0-9]+", "#")
blether regex_replace_first("a1b22", "[0-9]+", "#")
blether regex_split("a1b22c", "[0-9]+")

ken j = json_parse("{\"a\": [1, 2, {\"b\": null}], \"c\": true}")
blether json_stringify(j)
blether json_pretty({"x": 1, "y": [1, 2]})
```
