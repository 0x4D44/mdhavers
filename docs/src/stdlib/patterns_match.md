# patterns_match

Wildcard and word-pattern utilities: a `glob_match` for `*` and `?`, a `Pattern` object (with case-insensitive mode), a `Template` placeholder renderer and a `Tokenizer`, plus filters and delimiter extraction.

Import: `fetch "stdlib/patterns_match"`

Backend: pure mdhavers. It uses the built-ins `char_at`, `index_of`, `scran`, `words`, `is_space`, `starts_wi`, `ends_wi`, `replace`, `keys` and `contains`. Loading the module prints `Patterns module loaded! Ready tae find whit ye're lookin' fer!`. Nothing here uses `hurl`. Helpers starting with `_` are internal and not documented (there are none in this file).

## Functions

### glob_match(pattern, text)
Iterative glob test with `*` and `?`. The pattern comes first: `glob_match("*.txt", "a.txt")` is `aye`. Square-bracket classes are not supported (`[` matches only a literal `[`).

### contains_pattern(text, pattern)
`aye` if the pattern matches starting at any position of `text` (the match runs to the end of the remaining text, so end the pattern with `*` to allow a trailing remainder). A pattern with no wildcards is a plain substring test.

### find_all_occurrences(text, search)
Indexes of every non-overlapping occurrence of `search`.

### count_occurrences(text, search)
Number of those occurrences.

### match_any(text, patterns)
`aye` if any pattern in the list matches `text`.

### match_all(text, patterns)
`aye` if every pattern matches `text`.

### filter_by_pattern(list, pattern)
Items whose `tae_string` matches the pattern.

### exclude_by_pattern(list, pattern)
Items whose `tae_string` does not match.

### extract_between(text, start_delim, end_delim)
The text between the first `start_delim` and the next `end_delim`, or `naething` if either is missing.

### extract_all_between(text, start_delim, end_delim)
List of every such piece, in order. Caveat: it finds the next end delimiter from the start of the remaining text, not after the start delimiter, so an end delimiter that appears before the first start delimiter makes a piece repeat (`extract_all_between("] x [a] and [b]", "[", "]")` gives `[a, a, b]`).

### starts_with_word(text, word)
`text` starts with `word` and the next character is end of text, a space, or one of `. , ! ?`.

### ends_with_word(text, word)
Mirror image for the end of the text.

### contains_word(text, word)
`aye` if `word` is one of `words(text)`.

### make_pattern(pattern_str)
`Pattern(pattern_str)`.

### make_template(template_str)
`Template(template_str)`.

### make_tokenizer()
`Tokenizer()`.

### matches(text, pattern)
`glob_match(pattern, text)`: text first, unlike `glob_match`.

### wildcard_filter(items, pattern)
Alias for `filter_by_pattern`.

## Classes

### Pattern
`Pattern(pattern)` stores the pattern and an empty `flags` dict. Fields: `pattern`, `flags`.

| Method | Notes |
| --- | --- |
| `ignore_case()` | Turns on case-insensitive matching; returns `masel`. |
| `test(text)` | `aye` if `text` matches (via `glob_match`). |
| `find_all(text)` | The words of `text` (split with `words`) that match. When case-insensitive the original words are returned. |
| `replace_matches(text, replacement)` | Replaces matching words with `replacement`; the result is re-joined with single spaces. |

### Template
`Template(template_str)` uses `{{` and `}}` as the default delimiters. Fields: `template`, `placeholder_start`, `placeholder_end`.

| Method | Notes |
| --- | --- |
| `with_delimiters(start, end)` | Sets custom delimiters; returns `masel`. |
| `render(values)` | Replaces `start + key + end` for every key in the dict, using `tae_string` on the value. Unknown placeholders stay in place. |
| `placeholders()` | List of the names found in the template. |

### Tokenizer
`Tokenizer()` splits on space, tab and newline by default. Fields: `separators`, `keep_separators`.

| Method | Notes |
| --- | --- |
| `add_separator(sep)` | Adds one separator character; returns `masel`. |
| `set_separators(seps)` | Replaces the separator list; returns `masel`. |
| `keep()` | Keeps separators as their own tokens; returns `masel`. |
| `tokenize(text)` | Returns the list of tokens. Separators are single characters. |

## Compared with patterns

[patterns](./patterns.md) is the other pattern module. The big trap is `glob_match`: it is `glob_match(filename, pattern)` there and `glob_match(pattern, text)` here. `matches(text, pattern)` is text-first in both. This module has no `[abc]` classes; `patterns` does. This module has classes (`Pattern`, `Template`, `Tokenizer`) and list helpers; `patterns` has `when_*` matchers and Some/None/Left/Right helpers. Do not fetch both in one program, because the later `glob_match` and `matches` replace the earlier ones.

## Example

```mdhavers
fetch "stdlib/patterns_match"

blether glob_match("*.txt", "notes.txt")
ken p = make_pattern("H*").ignore_case()
blether p.find_all("Hello wee world hi")
ken t = make_template("Hi {{name}}")
blether t.render({"name": "Jock"})
blether make_tokenizer().tokenize("a b\tc")
blether extract_all_between("[a] and [b]", "[", "]")
```
