# json

A pure-mdhavers JSON serialiser and parser, with a dotted-path lookup (`json_get`) and a type namer. The source notes that it is a basic parser and that complex JSON should use native functions.

Import: `fetch "stdlib/json"`

Backend: pure mdhavers (uses `whit_kind`, `keys`, `split`, `join`, `wheesht`, `tae_int`, `tae_float`). Loading the module prints `JSON module loaded! Ready tae parse an' stringify!`. Nothing here uses `hurl`; the parser signals bad input by returning `naething`.

Warning: in the current source the serialiser checks `whit_kind` against `"nil"`, `"boolean"` and `"number"`, but the interpreter reports `naething`, `bool`, `integer` and `float`. As a result `tae_json` writes numbers, booleans and null as quoted strings (`3` becomes `"3"`, `aye` becomes `"aye"`), and `json_type` returns `unknown` for numbers, booleans and null. Strings, lists and dicts work correctly. Parsing is not affected. Use the output of `tae_json` for string-only data until this is fixed.

Helpers starting with `_` are internal and not documented (there are none in this file). The parser internals (`mak_parse_state`, `current_char`, `advance`, `skip_ws`, `parse_value` and friends) are public but listed briefly.

## Functions

### tae_json(value, indent = 0, pretty = nae)
Converts a value to a JSON string. Lists become arrays, dicts become objects with `": "` after each key, strings are escaped. With `pretty = aye`, output is spread over lines with two-space indentation. See the warning above for numbers, booleans and `naething`.

### json_escape_string(s)
Returns `s` in double quotes with `"`, `\`, newline, tab and carriage return escaped. Other control characters are left alone.

### json_array(lst, indent, pretty)
Serialises a list (`[]` when empty).

### json_object(obj, indent, pretty)
Serialises a dict (`{}` when empty).

### json_whitespace(n)
A string of `n` spaces.

### pretty_json(value)
`tae_json(value, 0, aye)`.

### parse_json(s)
Parses a JSON text and returns the value: dict, list, string, integer, float, `aye`/`nae`, or `naething` for `null`. Most malformed input yields `naething` (a missing value inside an object, as in `{"a":}`, is stored as `naething` instead), but `naething` is also the result for a valid `null`. Trailing text after the first value is ignored (`[1] junk` gives `[1]`). Numbers with no `.` become integers; exponents are not supported (`1e5` gives `1`), and the `\u` escape is not decoded (`\u0041` gives `u0041`).

### parse_value(state)
Parses whichever value starts at the current position of a parse state.

### parse_obj(state)
Parses an object; `naething` on a syntax error.

### parse_arr(state)
Parses an array; `naething` on a syntax error.

### parse_str(state)
Parses a string; `naething` if unterminated. Understands `\n`, `\t`, `\r`, `\"` and `\\`.

### parse_num(state)
Parses an integer or float.

### parse_true(state)
### parse_false(state)
### parse_null(state)
Parse the literals `true`, `false` and `null`; `naething` if the text does not match (`parse_null` returns `naething` either way).

### mak_parse_state(s)
Creates the parse state dict `{"input", "pos", "len"}`.

### current_char(state)
Character at the current position, or `naething` at the end.

### advance(state)
Moves the position on by one.

### skip_ws(state)
Skips spaces, newlines, tabs and carriage returns.

### is_digit(c)
`aye` if `c` is a single digit character.

### is_digits(s)
`aye` if `s` is non-empty and all digits.

### json_get(obj, path)
Looks up a dotted path such as `"users.0.name"`. All-digit parts index lists; other parts look up dict keys. Returns `naething` if any step is missing.

### is_valid_json(s)
Intended to validate JSON text. It parses `s` and always returns `aye`, so it does not detect bad JSON.

### json_type(value)
Returns `array`, `object` or `string` for lists, dicts and strings; `unknown` for everything else (see the warning above).

## Example

```mdhavers
fetch "stdlib/json"

ken doc = parse_json("{\"name\": \"Jock\", \"tags\": [\"a\", \"b\"], \"age\": 30}")
blether json_get(doc, "tags.1")
blether json_get(doc, "age")
blether tae_json(doc["tags"])
blether tae_json({"msg": "braw \"day\""})
```
