# chance

Randomness helpers with a Scottish flavour: ranges, coins, dice (including `2d6+3` notation), list picking, playing cards, a few games, simple distributions, fortunes and name generation. Everything is built on the built-in `jammy(low, high)` (integer from `low` up to but not including `high`) and `shuffle`, so it works on every backend. Results are not cryptographically secure and there is no way to seed them from this module.

Import: `fetch "stdlib/chance"`

Importing prints `Chance module loaded! Fortune favours the braw!`. No function here hurls a custom error. Helpers starting with `_` are internal and not documented.

## Constants

- `SUITS`: `["Herts", "Diamonds", "Clubs", "Spades"]`.
- `RANKS`: `"Ace"`, `"Twa"`, `"Three"`, `"Fower"`, `"Five"`, `"Sax"`, `"Seeven"`, `"Echt"`, `"Nine"`, `"Ten"`, `"Knave"`, `"Queen"`, `"King"`.
- `SCOTTISH_FIRST_NAMES` and `SCOTTISH_LAST_NAMES`: name lists used by `random_scottish_name`.

## Functions

### roll_between(wee_end, muckle_end)
Random integer from `wee_end` to `muckle_end`, both inclusive.

### random_float()
Random float from 0 up to just under 1, in steps of 0.0001.

### random_float_between(wee_end, muckle_end)
Random float between the two bounds (same 0.0001 granularity).

### flip_coin()
`"heads"` or `"tails"`.

### flip_bawbee()
`"heid"` or `"tail"`.

### maybe()
`aye` or `nae` with equal chance.

### chance(probability)
`aye` with the given probability (0.0 to 1.0).

### roll_die(sides)
Integer from 1 to `sides` inclusive.

### roll_d6()
`roll_die(6)`.

### roll_d20()
`roll_die(20)`.

### roll_dice(count, sides)
Sum of `count` dice.

### roll_notation(notation)
Parses `"2d6"`, `"3d8+5"` or `"1d20-2"` and rolls it. Returns 0 if there is no `d`. Bad numbers make `tae_int` fail with the usual runtime error.

### roll_advantage(sides)
Higher of two rolls.

### roll_disadvantage(sides)
Lower of two rolls.

### roll_exploding(sides)
Rolls again and adds whenever the maximum comes up.

### pick_one(list)
Random element; `naething` for an empty list.

### pick_many(list, n)
`n` picks with replacement.

### pick_unique(list, n)
`n` distinct positions without replacement. If `n >= len(list)` it returns `shuffle(list)` of everything.

### pick_weighted(items, weights)
Picks an item with probability proportional to its weight.

### mak_deck()
52 cards, each a dictionary `{"rank": ..., "suit": ...}`, ordered by suit then rank.

### shuffle_deck(deck)
`shuffle(deck)`.

### draw_cards(deck, n)
Takes `n` cards from the top. Returns `{"drawn": [...], "deck": [...]}` with the remaining cards; the input deck is not changed.

### card_tae_string(card)
Text like `Ace o' Herts`.

### card_value(card)
Blackjack-style value: Ace 11, Twa to Ten as the number, Knave, Queen and King 10; 0 for an unknown rank.

### heids_or_tails(guess)
Flips a bawbee and returns `{"result", "guess", "won"}`. Guess `"heid"` or `"tail"` (case-insensitive); `"heads"` and `"tails"` can never win.

### roll_initiative(modifier = 0)
d20 plus modifier.

### saving_throw(dc, modifier = 0)
Returns `{"roll", "modifier", "total", "dc", "success", "nat_20", "nat_1"}`; success is `total >= dc`.

### spin_slots()
Three random symbols. Returns `{"slots": [...], "jackpot": ..., "two_match": ...}`.

### random_normal(mean = 0.0, std_dev = 1.0)
Normally distributed value by the Box-Muller method, using `log_approx`.

### log_approx(x)
Rough natural logarithm. Returns -999999 for `x <= 0` and 0 for `x == 1`. Accurate only to a few decimals; prefer the built-in `log` for real work.

### bernoulli(p)
`aye` with probability `p`.

### binomial(n, p)
Number of successes in `n` Bernoulli trials.

### read_fortune()
A random fortune string.

### ask_the_haggis(question)
Magic 8-ball answer; the question is ignored.

### random_scottish_name()
`"First Last"` from the name lists.

### generate_uuid()
Random string shaped like a UUID (8-4-4-4-12 hex digits). Unlike [uuid](./uuid.md)'s `uuid4` it does not set version or variant digits.

## Example

```mdhavers
fetch "stdlib/chance"

blether roll_between(1, 6) >= 1
blether roll_notation("2d6+3") >= 5
blether len(mak_deck())
blether card_tae_string(mak_deck()[0])
blether pick_one([])
blether len(generate_uuid())
```

## Compared with uuid

[chance](./chance.md)'s `generate_uuid` is a quick random hex string. [uuid](./uuid.md) offers a proper version-4 `uuid4`, validation (`is_uuid`), short IDs, sequential IDs and slugs; prefer it for identifiers.
