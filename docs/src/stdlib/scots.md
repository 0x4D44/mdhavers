# scots

A small collection of Scots culture: greetings, farewells, proverbs, Burns quotes, insults, a vocabulary and a Gaelic phrasebook, clan descriptions, plus a few helper functions to pick random lines and look words up. Everything is plain data and strings.

Import: `fetch "stdlib/scots"`

Pure mdhavers, no native built-ins. Loading the module prints `Scots module loaded! Alba gu bràth!`. No function hurls an error; unknown lookups return a Scots "dinnae ken" string. The `random_*` functions pick with `jammy(0, len - 1)` and, as `jammy`'s upper bound is exclusive, never choose the last item of their list (`toast` and `blessing` do the same).

## Constants

- `GREETINGS`, `FAREWELLS`, `PROVERBS`, `BURNS_QUOTES`, `INSULTS`: lists of strings.
- `VOCABULARY`: dictionary of Scots word to English meaning (about 40 words).
- `GAELIC`: dictionary of Gaelic phrase (lower case) to English.
- `CLANS`: dictionary of clan name (capitalised, such as `"Campbell"`) to a description.

## Functions

### random_greeting()
A random item from `GREETINGS`.

### random_farewell()
A random item from `FAREWELLS`.

### random_proverb()
A random item from `PROVERBS`.

### random_burns()
A random item from `BURNS_QUOTES`.

### random_insult()
A random item from `INSULTS` (all in good fun).

### whit_means(word)
The meaning of a Scots word from `VOCABULARY` (case-insensitive; `braw` is defined twice, so the later meaning `fine / excellent` wins), or `Ah dinnae ken whit '<word>' means!`.

### gaelic_tae_english(phrase)
The English for a Gaelic phrase in `GAELIC` (case-insensitive for ASCII letters only: `lower` leaves `À` alone, so `"SLÀINTE MHATH"` is not found but `"SLàINTE MHATH"` is), or `Ah dinnae ken that Gaelic phrase!`.

### get_vocabulary()
Returns the `VOCABULARY` dictionary.

### get_gaelic()
Returns the `GAELIC` dictionary.

### scots_lesson()
Prints a boxed lesson: a proverb, a Burns quote, five vocabulary words and a Gaelic phrase. Returns `naething`.

### scottify(message)
Replaces some English words with Scots ones (yes, no, know, small, to, from, good, old, own and others). It matches lower-case text only, so `"Yes No"` is left unchanged. It is a crude substring replace, not a word-aware translator, and the replacements apply in order, so for example `no` is changed before `know` is: `"I know"` becomes `"I knaew"`, and `"about"` or `"tomorrow"` are changed inside longer words. See [scots_lang](./scots_lang.md) for `make_scots`.

### toast()
A random Scottish toast (a string).

### blessing()
A random Scottish blessing (a string).

### clan_info(name)
The description for a clan in `CLANS` (exact, case-sensitive match such as `"Bruce"`), or `Ah dinnae ken that clan!`.

## Example

```mdhavers
fetch "stdlib/scots"

blether whit_means("BRAW")
blether whit_means("xyz")
blether gaelic_tae_english("Tapadh Leat")
blether clan_info("Bruce")
blether scottify("a good house")
```

## Compared with scots_lang

[scots_lang](./scots_lang.md) is the larger, newer sibling: it uses `dram` so every list item can be chosen, has a bigger dictionary, a word-search, numbers, days and months, name and place generators, a `ScotsConverter` class, and food, creature and clan-motto lookups. `scots` has a few things `scots_lang` lacks: `BURNS_QUOTES` and `random_burns`, `scots_lesson`, `blessing`, `clan_info`, a `GAELIC` dictionary lookup, and the crude `scottify`. They share the names `GREETINGS`, `FAREWELLS`, `PROVERBS`, `INSULTS`, `random_greeting`, `random_farewell`, `random_proverb`, `random_insult` and `whit_means` with different data and, for `random_insult`, a different signature, so fetch only one of them in a program. Prefer `scots_lang` for general use and `scots` when you want Burns quotes or the lesson.
