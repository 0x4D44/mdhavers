# prelude

The prelude holds the most commonly used helper functions: null checks, small number helpers, predicate-based list functions, a few string helpers, function combinators, debug printers, Scots phrases and a handful of unit conversions. The source header says it is "automatically loaded fer every mdhavers program"; in testing, names such as `wee`, `ilka` and `mony` resolved in a script that had no `fetch`.

Import: `fetch "stdlib/prelude"` (only needed if you want to load it explicitly).

Backend: pure mdhavers, so it follows the interpreter. It leans on built-ins (`heid`, `bum`, `wheesht`, `gaun`, `tumble`, `jammy`, `the_noo`, `whit_kind`, `mak_siccar`). Several names that look like prelude functions are really built-ins and are not defined in the `.braw` file: `is_even`, `is_odd`, `tak`, `drap`, `pad_left`, `pad_right` and `assert_equal`.

Nothing in this module uses `hurl`. The one failure path is `assert_that`, which calls the built-in `mak_siccar`. Printing helpers write to standard output with `blether`. Helpers starting with `_` are internal and not documented (there are none in this file).

## Functions

**Basics**

### is_nowt(val)
`aye` if `val` is `naething`.

### is_somethin(val)
`aye` if `val` is not `naething`.

### or_else(val, default)
Returns `default` when `val` is `naething`, otherwise `val`.

**Numbers**

### wee(a, b)
The smaller of two values.

### muckle(a, b)
The larger of two values.

### atween(val, wee_end, muckle_end)
`aye` if `wee_end <= val <= muckle_end` (inclusive).

### hauld_atween(val, wee_end, muckle_end)
Clamps `val` into the range `wee_end..muckle_end`.

**Lists**

### is_toom(list)
`aye` if the list is empty.

### tak_while(list, pred)
Leading items for which `pred(item)` is true; stops at the first failure.

### drap_while(list, pred)
Drops leading items while `pred(item)` is true and returns the rest.

### coont_gin(list, pred)
Counts items for which `pred` is true. The built-in `coont(list, value)` counts exact matches instead.

### ilka(list, pred)
`aye` if every item passes `pred` (`aye` for an empty list).

### fin(list, pred)
First item that passes `pred`, or `naething`.

### fin_index(list, pred)
Index of the first item that passes `pred`, or `-1`.

### zip_up(a, b)
List of `[a[i], b[i]]` pairs, as long as the shorter list.

### mony(val, n)
A list holding `val` repeated `n` times.

### hauf_by(list, pred)
Splits into `[matching, others]`. The built-in `split_by(list, "even")` takes string predicates; this takes a function.

### group_by(list, key_fn)
Dict of `tae_string(key_fn(item))` to the list of items with that key. Keys are always strings.

### mode(list)
Most common item (compared by `tae_string`). Returns the first item to reach the top count on ties, and `naething` for an empty list.

### transform_an_combine(list, mapper, reducer, start)
`tumble(gaun(list, mapper), start, reducer)`: map, then fold.

### first(list)
Alias for `heid(list)`.

### last(list)
Alias for `bum(list)`.

**Strings**

### is_baw(s)
`aye` if the string is empty or only whitespace.

### repeat_say(s, n)
`s` repeated `n` times.

### centre(s, width, fill)
Centres `s` in `width` using `fill` (all three arguments are required). Returns `s` unchanged if it is already wide enough. Any extra padding goes on the right.

### is_nummer(s)
`aye` if `tae_float(s)` succeeds.

### is_hale_nummer(s)
`aye` if `tae_int(s)` succeeds.

**Combinators**

### compose(f, g)
Returns a function `x -> f(g(x))`.

### efter(f, g)
Returns a function `x -> g(f(x))` (the reverse order of `compose`).

### apply_n(f, n, start)
Applies `f` to `start` `n` times.

### dae_times(n, val, func)
Same idea as `apply_n` with a different argument order: `func` applied `n` times to `val`.

### same(x)
Identity function. It is defined twice in the file with the same body.

### constant(val)
Returns a function that ignores its argument and returns `val`.

### nae_that(pred)
Returns the negation of a predicate.

**Debugging**

### swatch(label, val)
Prints `[swatch] label: val` and returns `val`.

### spy(label, val)
Prints `[label] val` and returns `val`.

### inspect(val)
Prints `<kind> val` (kind from `whit_kind`) and returns `val`.

### mak_siccar_that(condition, message)
If `condition` is false, prints `ASSERTION FAILED: message`. It does not stop the program.

### assert_that(condition, message)
If `condition` is false, calls the built-in `mak_siccar`, which raises an error such as `Jings crivvens!: Mak siccar failed at line 585! bad` (the line is the one inside the prelude; the leading exclamation is picked at random).

### tak_time(func)
Calls `func()` and returns `[result, milliseconds]`.

**Ranges**

### make_range(start, finish)
Inclusive list `start..finish`.

### make_range_step(start, finish, step)
Inclusive list with a step. A negative step counts down; a zero step gives an empty list.

**Validation**

### is_braw(val)
`nae` for `naething`, an empty string or an empty list; `aye` otherwise (so `0` is braw).

### needsae_be_toom(val, name)
Returns `val` if it is braw. Otherwise prints `Och! <name> cannae be empty, ya numpty!` and returns `naething`.

### is_positive(n)
`n > 0`.

### is_negative(n)
`n < 0`.

**Scots phrases and conversions**

### scots_number(n)
Word for 0 to 10: `zero`, `ane`, `twa`, `three`, `fower`, `five`, `sax`, `seeven`, `echt`, `nine`, `ten` (only some are Scots); `tae_string(n)` for anything else.

### ordinal(n)
`1st`, `2nd`, `3rd`, `4th`, with `11th`-`13th` handled.

### wheech(msg)
Prints `!! MSG !!`.

### dinnae_fash(msg)
Prints `Dinnae fash yersel! msg`.

### pure_dead_brilliant(whit)
Prints `Pure dead brilliant, that whit!`.

### scots_wisdom()
A random proverb (uses `jammy`).

### scots_greetin()
A random greeting.

### scots_farewell()
A random farewell.

### scots_exclaim()
A random exclamation.

### scots_miles_tae_km(miles)
`miles * 1.8`.

### scots_pints_tae_litres(pints)
`pints * 1.7`.

### stanes_tae_kg(stanes)
`stanes * 6.35`.

## Example

```mdhavers
fetch "stdlib/prelude"

ken evens = tak_while([2, 4, 6, 7, 8], |n| n % 2 == 0)
blether evens
blether hauld_atween(15, 1, 10)
blether ordinal(22)
blether group_by([1, 2, 3, 4], |n| n % 2)
blether compose(|x| x + 1, |x| x * 2)(5)
```
