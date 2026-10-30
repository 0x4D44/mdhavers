# text

Text processing: case converters (camel, snake, kebab...), counting and analysis, padding and wrapping, a simple English-to-Scots word swapper, a few generators, and similarity helpers.

Import: `fetch "stdlib/text"`

Backend: pure mdhavers (uses the built-ins `chars`, `scran`, `index_of`, `keys`, `jammy`, `wheesht`). Loading the module prints `Text module loaded! Words are the threads that weave yer thoughts!`. Nothing here uses `hurl`. Helpers starting with `_` are internal and not documented (there are none in this file).

Words are split on single spaces only; empty words are dropped by the case converters.

## Functions

### title_case(text)
First letter of each word upper-case, the rest lower-case. Empty words are dropped.

### sentence_case(text)
First character upper-case, everything else lower-case.

### camel_case(text)
`"hello wee world"` gives `helloWeeWorld`.

### pascal_case(text)
`"hello wee world"` gives `HelloWeeWorld`.

### snake_case(text)
Lower-case words joined by `_`.

### screaming_snake_case(text)
Upper-case words joined by `_`.

### kebab_case(text)
Lower-case words joined by `-`.

### swap_case(text)
Flips the case of each letter.

### word_count(text)
Number of non-blank space-separated words.

### char_count(text)
Characters excluding spaces, newlines and tabs.

### line_count(text)
Number of lines (`0` for an empty string; a trailing newline adds a line).

### sentence_count(text)
Counts the characters `.`, `!` and `?`.

### word_frequency(text)
Dict of lower-cased, punctuation-stripped word to count.

### clean_word(word)
Keeps only ASCII letters and digits.

### is_letter(c)
`aye` if `c` is an ASCII letter (single-character string expected).

### is_digit(c)
`aye` if `c` is an ASCII digit.

### has_key(dict, key)
`aye` if `key` is among `keys(dict)`.

### reverse_text(text)
Reversed string.

### repeat_text(text, n)
`text` repeated `n` times.

### pad_left(text, width, char = " ")
Left-pad to `width`.

### pad_right(text, width, char = " ")
Right-pad to `width`.

### center_text(text, width, char = " ")
Centre in `width`; odd extra padding goes on the right.

### truncate(text, max_len, ellipsis = "...")
Returns `text` if it fits. Otherwise cuts it to `max_len` including the ellipsis; if `max_len` is no more than the ellipsis length the result is just the ellipsis.

### word_wrap(text, width)
Greedy wrap on spaces; lines are joined with `\n`. A word longer than `width` gets its own line.

### scots_replacements()
Returns the dict of English-to-Scots word pairs (52 entries, for example `"child": "bairn"`, `"know": "ken"`).

### tae_scots(text)
Swaps known words for Scots ones, keeping a leading capital and trailing punctuation. `"Hello, I know the little child from home."` gives `Hello, I ken the wee bairn frae hame.`

### add_scots_flair(text)
Always returns `"Och, " + text + ", ye ken?"` (deterministic by design).

### is_alpha(text)
Non-empty and all ASCII letters.

### is_numeric(text)
Non-empty and all ASCII digits.

### is_alphanumeric(text)
Non-empty and all letters or digits.

### is_blank(text)
Empty or whitespace only.

### is_all_upper(text)
No lower-case letters and non-empty. Non-letters are ignored.

### is_all_lower(text)
No upper-case letters and non-empty.

### count_substring(text, sub)
Non-overlapping count.

### replace_first(text, old, new)
Replaces the first match using `index_of`.

### replace_all(text, old, new)
Replaces every match. An empty `old`, or a `new` that contains `old`, never finishes.

### lorem_scots(sentences = 5)
Joins the first `sentences` proverbs from a fixed list of 15, cycling if more are asked for.

### generate_text_id(length = 8)
Random lower-case alphanumeric string.

### similarity(text1, text2)
Intended as the fraction of matching positions (0 to 1). In practice it uses integer division, so it returns `0` unless the strings are identical (then `1`); `1.0` is returned only when both are empty.

### is_similar(text1, text2, threshold = 0.8)
`similarity(...) >= threshold`. Because of the issue above this is true only for identical strings.

### text_stats(text)
Dict with `characters`, `characters_no_space`, `words`, `lines`, `sentences` and `is_blank`.

### display_stats(text)
Prints a short report built from `text_stats`.

## Compared with strings

See [strings](./strings.md). `text` offers case converters and analysis; `strings` offers searching (`find_index`, `find_all`) and `starts_wi`/`ends_wi`. Shared names behave slightly differently (`title_case` here lower-cases the tails of words and collapses repeated spaces; `count_substring` is non-overlapping). Avoid fetching both in one program.

## Example

```mdhavers
fetch "stdlib/text"

blether camel_case("hello wee world")
blether kebab_case("Braw Wee Scone")
blether tae_scots("I know the little child")
blether word_wrap("the quick brown fox jumps over", 12)
blether word_frequency("the cat, the dog")
```
