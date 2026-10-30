# poetry

Scots verse generator. It fills fixed templates (haiku, a Burns-style verse, a limerick, a toast, a proverb) with random words from Scots word banks, and prints the result. Choices come from a small built-in pseudo-random generator that you can seed, so output is repeatable.

Import: `fetch "stdlib/poetry"`

Pure mdhavers, no native built-ins. Loading the module prints `Poetry module loaded! Ready tae compose!`. No function hurls its own errors. The module keeps its random state in a global `poetry_seed` (starts at 12345), and defines a global helper `repeat_char`, which also exists in [ascii_art](./ascii_art.md).

## Constants

Word banks (lists of strings): `SCOTS_NOUNS`, `SCOTS_ADJECTIVES`, `SCOTS_VERBS`, `SCOTS_ADVERBS`, `SCOTS_EXCLAMATIONS`, `NATURE_WORDS`, `EMOTIONS`.

Template lists: `HAIKU_5`, `HAIKU_7`, `BURNS_LINES_A`, `BURNS_LINES_B`, `LIMERICK_A` to `LIMERICK_D`, `TOAST_OPENINGS`, `TOAST_MIDDLES`, `TOAST_ENDINGS`, `PROVERBS`.

Templates use the placeholders `{adj}`, `{noun}`, `{verb}` and `{exclaim}`; each occurrence is replaced by a separate random pick. `SCOTS_ADVERBS`, `NATURE_WORDS` and `EMOTIONS` are not used by any generator.

## Functions

### next_random()
Advances the linear-congruential generator and returns the new value (0 to 99999).

### pick(lst)
A pseudo-random element of the list.

### pick_n(lst, n)
A list of `n` elements chosen without repeating an index while there is room.

### set_poetry_seed(s)
Sets the generator state. Call it with the same number to get the same poems again. It has no `gie`; it returns the new seed (the value assigned), not `naething`.

### generate_haiku()
Returns a list of 3 lines (5, 7, 5 pattern; the syllable counts are approximate).

### fill_template(template)
Returns the template string with every `{adj}`, `{noun}`, `{verb}` and `{exclaim}` replaced by random words.

### replace_first_occurrence(s, old, new)
Returns `s` with the first occurrence of `old` replaced by `new`.

### generate_burns_verse()
Returns a list of 4 lines (A, B, A, B templates). The rhyme is only a hope.

### generate_limerick()
Returns a list of 5 lines.

### generate_toast()
Returns a list of 3 parts: opening, middle, ending.

### generate_proverb()
Returns one proverb string.

### generate_poem(style = "haiku")
Returns a list of lines for `style`: `"haiku"`, `"burns"`, `"limerick"`, `"toast"` or `"proverb"` (a one-element list). Any other style gives a haiku.

### print_poem(lines, title = naething)
Prints the lines indented, with an optional underlined title. Returns `naething`.

### repeat_char(c, n)
A string of `c` repeated `n` times.

### poetry_session()
Prints a banner and one poem of every kind. Not interactive despite the name.

## Example

```mdhavers
fetch "stdlib/poetry"

set_poetry_seed(42)
print_poem(generate_haiku(), "Haiku")
blether generate_proverb()
blether generate_poem("toast")
```
