# scots_lang

Scots language utilities: random greetings, farewells, exclamations, proverbs, toasts, song lines and insults; dictionary lookups for Scots words, foods, creatures, Gaelic phrases, clan mottos and mdhavers keywords; number, day and month names; random name and place generators; an English-to-Scots text converter; and a small chat-bot class.

Import: `fetch "stdlib/scots_lang"`

Pure mdhavers, no native built-ins. Loading the module prints `Scots language module loaded! Gie yer code some flavour!`. No function hurls an error. Lookups that fail return a Scots "dinnae ken" string, `naething` (`gaelic_phrase`), or `???`. The `random_*` functions use `dram`, so every list item can be chosen. Helpers starting with `_` are internal and not documented.

## Constants

- Lists of strings: `GREETINGS`, `FAREWELLS`, `EXCLAMATIONS`, `POSITIVE_EXCLAMATIONS`, `NEGATIVE_EXCLAMATIONS`, `PROVERBS`, `INSULTS`, `MILD_INSULTS`, `TOASTS`, `SONG_LINES`, `PLACE_PREFIXES`, `PLACE_SUFFIXES`, `SCOTS_FIRST_NAMES_MALE`, `SCOTS_FIRST_NAMES_FEMALE`, `SCOTS_SURNAMES`.
- `NUMBER_WORDS`: Scots words for 0 to 12. `SCOTS_DAYS`: 7 day names starting Sunday. `SCOTS_MONTHS`: 12 month names.
- Dictionaries: `WEATHER_WORDS`, `SCOTS_WORDS` (used by `whit_means` and `find_scots_words`; at load time every `MORE_SCOTS_WORDS` entry is merged into it), `MORE_SCOTS_WORDS`, `CLAN_MOTTOS`, `PROGRAMMING_TERMS`, `SCOTTISH_FOODS`, `CREATURES`, `GAELIC_PHRASES`. `WEATHER_WORDS` is data only; no function reads it. `MORE_SCOTS_WORDS` is only read by the load-time merge.

## Functions

### random_greeting()
A random greeting.

### random_farewell()
A random farewell.

### random_exclamation()
A random exclamation.

### random_proverb()
A random proverb.

### positive_reaction()
A random positive exclamation.

### negative_reaction()
A random negative exclamation.

### whit_means(word)
Meaning of a word in `SCOTS_WORDS` (case-insensitive), or `Dinnae ken whit '<word>' means!`. It does not search `WEATHER_WORDS`. Because of the load-time merge, `MORE_SCOTS_WORDS` entries are found too (a shared key such as `stoorie` gets the `MORE_SCOTS_WORDS` meaning).

### find_scots_words(contains_text)
List of the words in `SCOTS_WORDS` (including the merged `MORE_SCOTS_WORDS`) that contain the text (case-insensitive substring).

### tae_scots_number(n)
The Scots word for an integer 0 to 12 (`yin`, `twa`, ...), otherwise `tae_string(n)`.

### scots_day(day_index)
Day name for 0 (Soonday) to 6, otherwise `???`.

### scots_month(month_index)
Month name for 0 (Januar) to 11, otherwise `???`.

### make_scots(text)
Converts English text to Scots with a fresh `ScotsConverter`. See the class for the limits.

### random_scots_name(gender = "any")
A random `"First Surname"`. `gender` is `"male"`, `"female"`, or anything else for either.

### random_place_name()
A made-up place name from a prefix and suffix, for example `Inverwick`.

### clan_motto(clan_name)
The motto for a clan in `CLAN_MOTTOS` (exact, case-sensitive), or `Motto unknown`.

### explain_keyword(keyword)
Describes an mdhavers keyword from `PROGRAMMING_TERMS` (for example `gin` is `if (condition check)`), or `'<keyword>' is nae a kent keyword`.

### random_insult(mild = aye)
A random mild insult, or a ruder one if `mild` is `nae`.

### random_toast()
A random toast.

### random_song_line()
A random line of a Scottish song.

### describe_food(food_name)
Description from `SCOTTISH_FOODS` (case-insensitive on the argument; the `Irn-Bru` key is mixed case, so it is not found), or `Dinnae ken whit <name> is!`.

### describe_creature(creature_name)
Description from `CREATURES` (exact, case-sensitive such as `"Nessie"`), or `Nae heard o' that beastie!`.

### gaelic_phrase(english_key)
Despite its name, the argument is the Gaelic phrase: returns its English meaning from `GAELIC_PHRASES` (case-insensitive), or `naething`. Keys with a question mark, such as `Ciamar a tha thu?`, need it included.

### random_gaelic()
A random `Gaelic - English` line.

### scots_fortune()
A random proverb, toast, song line or piece of wisdom, prefixed with a label for the first three.

## Classes

### ScotsConverter
`ScotsConverter()`. Field `replacements`: dictionary of English to Scots words (54 entries, including `know` to `ken`, `have` to `hae`, `with` to `wi'`, `good` to `guid`).

- `convert(text)`: returns the text with each replacement applied in turn, as a substring replace of the lower-case and the capitalised form. It is not word-aware, so a short key such as `no` or `not` will also change the inside of longer words.
- `add_replacement(english, scots)`: adds or overrides a pair and returns the converter.

### ScotsChat
`ScotsChat(name = "Wee Bot")`. Fields: `name`, `mood` (`"happy"`).

- `greet()`: a greeting plus `Ah'm <name>!`.
- `farewell()`: a random farewell.
- `react(is_positive)`: a positive or negative exclamation.
- `share_wisdom()`: `Here's some wisdom fer ye: <proverb>`.
- `explain(word)`: `'<word>' means: <meaning>` using `whit_means`.

## Example

```mdhavers
fetch "stdlib/scots_lang"

blether whit_means("Bairn")
blether tae_scots_number(2) + " " + scots_day(0) + " " + scots_month(2)
blether make_scots("I know the Good house")
blether find_scots_words("ai")
blether clan_motto("Graham")
blether ScotsChat("Jock").explain("laddie")
```

## Compared with scots

[scots](./scots.md) is the smaller module: fewer lists, a `VOCABULARY` of about 40 words, Burns quotes, a lesson printer, `blessing`, `clan_info` and the crude `scottify`. `scots_lang` is the fuller one. It has more data and lookups, number, day and month names, name and place generators, and a configurable `ScotsConverter` (`make_scots` is a better translator than `scottify`, though still substring-based). Its random pickers can return any item, whereas `scots`'s `random_*` functions never pick the last item. Both define `GREETINGS`, `FAREWELLS`, `PROVERBS`, `INSULTS`, `whit_means` and the `random_*` names, so fetch only one in a program. Prefer `scots_lang` unless you need `random_burns`, `scots_lesson`, `blessing` or `clan_info`.
