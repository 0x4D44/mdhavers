# strings

Small string utilities written in plain mdhavers: checks (`is_digits`, `starts_wi`), case changes, padding, searching, replacing and a few Scots-flavoured decorators.

Import: `fetch "stdlib/strings"`

Backend: pure mdhavers, so it works wherever the interpreter does. Loading the module prints `Strings module loaded! Words are bonnie!`. Nothing here uses `hurl`; bad input either returns a neutral value or, for a non-string argument, raises the interpreter's own type error. Helpers starting with `_` are internal and not documented (there are none in this file).

Warning: `replace_all(s, "", x)` never finishes, because the empty needle always matches at index 0.

## Functions

### is_blank(s)
`aye` if `s` is empty or whitespace only.

### starts_wi(s, prefix)
`aye` if `s` starts with `prefix`.

### ends_wi(s, suffix)
`aye` if `s` ends with `suffix`.

### is_digits(s)
`aye` if `s` is non-empty and all ASCII digits.

### is_letters(s)
`aye` if `s` is non-empty and all ASCII letters.

### is_alnum(s)
`aye` if `s` is non-empty and all ASCII letters or digits.

### capitalise(s)
Upper-cases the first character and leaves the rest alone. `""` stays `""`.

### title_case(s)
Splits on single spaces and runs `capitalise` on each word. Other letters keep their case, and repeated spaces are kept.

### swap_case(s)
Flips the case of ASCII letters.

### repeat_str(s, n)
`s` repeated `n` times (`""` for `n <= 0`).

### reverse_str(s)
Reversed string.

### pad_left(s, width, char = " ")
Left-pads to `width`. Returns `s` if already long enough. A native built-in of the same name also exists (see [prelude](./prelude.md)).

### pad_right(s, width, char = " ")
Right-pads to `width`.

### center(s, width, char = " ")
Centres `s`; any odd extra padding goes on the right.

### substring(s, start, finish)
Characters from `start` (inclusive) to `finish` (exclusive), clipped to the string length.

### find_index(s, needle)
Index of the first match, `-1` if none, `0` for an empty needle.

### find_all(s, needle)
List of indexes of every match. Matches may overlap: `find_all("aaaa", "aa")` is `[0, 1, 2]`.

### count_str(s, needle)
`len(find_all(s, needle))`, so overlapping matches count.

### replace_first(s, old, new)
Replaces the first occurrence.

### replace_all(s, old, new)
Replaces every occurrence, rescanning the result each time. If `new` contains `old` (for example `replace_all("a", "a", "aa")`) it never finishes; an empty `old` also never finishes.

### remove_all(s, chars)
Removes every character of `s` that appears in the string `chars`. This is a character set, not a substring.

### word_count(s)
Number of space-separated, non-blank words.

### first_words(s, n)
The first `n` words, joined by single spaces.

### truncate(s, max_len, suffix = "...")
Returns `s` if it fits; otherwise cuts it so the result, including `suffix`, is `max_len` long.

### shouty(s)
Upper-case plus `!`.

### wee_voice(s)
Lower-case wrapped in `...`.

### add_greeting(s)
`"Haud on - " + s`.

### add_signoff(s)
`s + " - Cheerio!"`.

### is_empty(s)
`len(s) == 0`.

### capitalize(s)
Alias for `capitalise`.

### reverse_string(s)
Alias for `reverse_str`.

### is_palindrome(s)
Trims and lower-cases `s`, then compares it with its reverse. Spaces and punctuation inside the string still count.

### remove_char(s, ch)
`replace(s, ch, "")` using the built-in `replace`.

### count_substr(s, sub)
Counts overlapping occurrences by sliding a window with `scran`.

## Compared with text

[text](./text.md) defines several functions with the same names (`title_case`, `swap_case`, `word_count`, `pad_left`, `pad_right`, `truncate`, `is_blank`, `replace_first`, `replace_all`) plus case converters and analysis. Differences: `text` goes through `chars()`/`scran` and drops empty words, so `text.title_case` also lower-cases the rest of each word and collapses repeated spaces; `count_substring` in `text` counts non-overlapping matches whereas `count_str` and `count_substr` here count overlapping ones. Prefer `strings` for tiny predicates and searching, `text` for case conversion and analysis. Do not `fetch` both in one program unless you want the later one to win for the shared names.

## Example

```mdhavers
fetch "stdlib/strings"

blether title_case("braw wee scone")
blether pad_left("7", 3, "0")
blether find_all("abcabc", "bc")
blether truncate("a long sentence indeed", 10)
blether is_palindrome("Racecar")
```
