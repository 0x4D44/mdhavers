# patterns

Simple wildcard matching plus a grab-bag of functional "when" matchers and Some/None and Left/Right helpers. `matches(text, pattern)` supports `*` (any run of characters), `?` (exactly one character) and `[abc]` (one of the listed characters; no ranges and no negation).

Import: `fetch "stdlib/patterns"`

Backend: pure mdhavers (uses `whit_kind`, `dict_has`, `tail`, `contains`, `split`, `tae_int`, `tae_float`). Loading the module prints `Patterns module loaded! Ready tae match!`. Nothing here uses `hurl`; failed lookups return `naething` or a `none()` value. Helpers starting with `_` are internal and not documented (there are none in this file).

## Functions

**Wildcard matching**

### matches(text, pattern)
`aye` if the whole of `text` matches `pattern`. Note the argument order: text first.

### match_helper(text, pattern, ti, pi)
Recursive worker for `matches`; call it with `ti = 0` and `pi = 0`. A `[` with no closing `]` never matches.

### find_pattern(text, pattern)
Index of the first position where `pattern` matches a prefix of the rest of the text, or `-1`.

### starts_with_pattern(text, pattern)
`aye` if `text` starts with something matching `pattern` (a `*` is appended when the pattern has none).

### substring_from(s, start)
Characters from `start` to the end.

### replace_pattern(text, pattern, replacement)
Limited. A pattern with no wildcard replaces every literal occurrence. A pattern ending in `*` only replaces occurrences of the text before the `*` (the `*` does not consume extra characters: `replace_pattern("foobar foo", "foo*", "X")` is `Xbar X`). Any other wildcard pattern (for example `a?c`) returns `text` unchanged. A pattern that is just `*` (empty prefix) never advances and loops forever.

### substring_to(s, finish)
Characters from the start up to `finish`.

### starts_with_str(s, prefix)
Plain prefix test.

### ends_with_str(s, suffix)
Plain suffix test.

### replace_all_str(s, old, new)
Literal replace-all in a single pass (safe when `new` contains `old`).

### glob_match(filename, pattern)
`matches(filename, pattern)`. Filename first. (The function of the same name in [patterns_match](./patterns_match.md) takes the pattern first.)

### glob_filter(filenames, pattern)
The filenames that match.

**Looks-like checks**

### is_word(s)
Non-empty and ASCII letters only.

### looks_like_email(s)
Exactly one `@`, non-empty parts either side, and a `.` in the domain.

### looks_like_url(s)
Starts with `http://` or `https://`.

### looks_like_postcode(s)
Only checks that the trimmed length is 5 to 8 characters; very rough.

**Extraction**

### extract_words(text)
List of runs of ASCII letters and apostrophes.

### extract_numbers(text)
List of numbers (always floats, via `tae_float`) found in the text. A run with two dots such as `1.2.3` makes `tae_float` hurl `Cannae turn '1.2.3' intae a float`.

**Type matchers.** Each calls `handler(val)` and returns its result when the type matches, otherwise `naething`.

### when_string(val, handler)
### when_int(val, handler)
### when_float(val, handler)
### when_list(val, handler)
### when_dict(val, handler)
### when_bool(val, handler)
Type names tested are `string`, `integer`, `float`, `list`, `dict` and `bool`.

### when_nil(val, handler)
When `val` is `naething`, calls `handler()` with no argument.

**Value matchers**

### when_equal(val, expected, handler)
Calls `handler(val)` if equal.

### when_one_of(val, options, handler)
Calls `handler(val)` if `val` is in `options`.

### when_pred(val, predicate, handler)
Calls `handler(val)` if `predicate(val)` is true.

**Number matchers** (non-numbers give `naething`)

### when_between(val, low, high, handler)
Inclusive range.

### when_positive(val, handler)
### when_negative(val, handler)
### when_zero(val, handler)
### when_even(val, handler)
### when_odd(val, handler)
`when_even` and `when_odd` accept integers only.

**List matchers**

### when_toom(val, handler)
Empty list; calls `handler()` with no argument.

### when_no_toom(val, handler)
Non-empty list; `handler(list)`.

### when_single(val, handler)
One-element list; `handler(element)`.

### when_cons(val, handler)
Non-empty list; `handler(head, tail)`.

### when_pair(val, handler)
Two-element list; `handler(a, b)`.

### when_triple(val, handler)
Three-element list; `handler(a, b, c)`.

**Dict matchers**

### when_has_key(val, key, handler)
`handler(val[key])` if the dict has the key.

### when_has_keys(val, required_keys, handler)
`handler(val)` if the dict has every listed key.

**Some / None.** Values are dicts: `some(x)` is `{"_some": aye, "value": x}` and `none()` is `{"_none": aye}`.

### some(val)
Wraps a value.

### none()
The empty marker.

### is_some(val)
### is_none(val)
Type-safe checks (non-dicts give `nae`).

### unwrap_or(val, default)
The wrapped value, or `default` if `val` is not a Some.

### map_some(val, mapper)
Applies `mapper` inside a Some; passes a None through.

### when_some(val, handler)
`handler(value)` for a Some, else `naething`.

### when_none(val, handler)
`handler()` for a None, else `naething`.

**Left / Right.** `left(x)` is `{"_left": aye, "value": x}` and `right(x)` is `{"_right": aye, "value": x}`.

### left(val)
Failure side.

### right(val)
Success side.

### is_left(val)
### is_right(val)
Checks.

### when_left(val, handler)
### when_right(val, handler)
Call `handler(value)` on the matching side, else `naething`.

**Safe operations** (return Some or None)

### safe_divide(a, b)
None when `b` is zero.

### safe_get(list, index)
None when out of range or not a list.

### safe_lookup(dict, key)
None when the key is missing or not a dict.

### safe_parse_int(s)
Some(int) or None.

### safe_parse_float(s)
Some(float) or None.

## Compared with patterns_match

Both modules offer `matches` and `glob_match`, but with opposite argument orders for `glob_match`:

| | `patterns` | [patterns_match](./patterns_match.md) |
| --- | --- | --- |
| `glob_match` | `glob_match(filename, pattern)` | `glob_match(pattern, text)` |
| `matches` | `matches(text, pattern)` | `matches(text, pattern)` |
| Wildcards | `*`, `?`, `[abc]` (recursive) | `*`, `?` only (iterative) |
| Extras | `when_*` matchers, Some/None, Left/Right, safe parsers, `looks_like_*` | `Pattern`, `Template`, `Tokenizer` classes, `match_any`/`match_all`, extraction between delimiters |

Fetching both in one program makes the second `glob_match` replace the first, silently swapping the argument order. Use `patterns` for character classes and the functional helpers, `patterns_match` for filtering, tokenizing and templating. For a standalone glob test, always pass the arguments in the order the module you fetched expects.

## Example

```mdhavers
fetch "stdlib/patterns"

blether matches("hello.txt", "*.txt")
blether glob_filter(["a.txt", "b.md", "c.txt"], "*.txt")
blether safe_divide(10, 4)
blether unwrap_or(safe_divide(1, 0), -1)
blether when_pair([1, 2], |a, b| a + b)
blether extract_numbers("a 12 b 3.5")
```
