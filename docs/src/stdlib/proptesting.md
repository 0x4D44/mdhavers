# proptesting

Property-based testing: random data generators, a runner (`forall`, `forall2`, `forall3`) that checks a property against many random inputs, ready-made property builders, and a pass/fail summary. Pure mdhavers on top of the built-in `jammy`, so it works on every backend. There is no shrinking and no seeding.

Import: `fetch "stdlib/proptesting"`

Importing prints `Property testing module loaded! Gie yer code a skelping!`. Important: `jammy(low, high)` excludes `high`, and several generators pass the upper bound straight through. See the notes under each generator. A property passes when it returns a truthy value; if it returns a falsy value or fails with an error, the run stops at the first failure and counts as failed. Helpers starting with `_` are internal and not documented.

## Constants

The counters `__prop_passed`, `__prop_failed` and `__prop_shrinks` track how many `forall` calls passed or failed; `prop_summary` reads them and `prop_reset` clears them.

## Functions

### gen_int(min_val = -1000, max_val = 1000)
Integer from `min_val` up to but not including `max_val`.

### gen_pos_int(max_val = 1000)
Integer from 1 up to but not including `max_val`.

### gen_nat(max_val = 1000)
Integer from 0 up to but not including `max_val`.

### gen_float(min_val = -1000.0, max_val = 1000.0)
Float in the range, in steps of 0.0001 of the span.

### gen_bool()
Intended to give `aye` or `nae`, but it is `jammy(0, 1) == 1`, which is always `nae` (see the note below).

### gen_char()
Random letter or digit; the final character `9` is never produced.

### gen_lower()
Random lowercase letter (never `z`).

### gen_upper()
Random uppercase letter (never `Z`).

### gen_digit()
Random digit character (never `9`).

### gen_string(length = 10)
String of `gen_char()` characters.

### gen_alpha_string(length = 10)
String of letters. Because `gen_bool()` is always `nae`, it currently yields only upper case letters.

### gen_int_list(length = 10, min_val = -100, max_val = 100)
List of `gen_int` values.

### gen_int_list_var(min_len = 0, max_len = 20, min_val = -100, max_val = 100)
`gen_int_list` with a random length from `min_len` up to but not including `max_len`. `min_len` must be below `max_len`, or `jammy` fails with `jammy() needs min < max, ya numpty!`.

### gen_sorted_list(length = 10, min_val = -100, max_val = 100)
Sorted `gen_int_list`.

### gen_one_of(options)
Random element of `options`; the last option is never chosen. A one-element list fails with `jammy() needs min < max, ya numpty!`.

### gen_pair(gen_a, gen_b)
`[gen_a(), gen_b()]`; the arguments are zero-argument functions.

### forall(name, generator, property, num_tests = 100)
Prints `Property: <name>`, then calls `generator()` up to `num_tests` times and checks `property(input)`. On success prints `Passed n/n tests` and returns `aye`. On the first failure prints the zero-based trial number, the input and any error text, then returns `nae`. Updates the pass or fail counters.

### forall2(name, gen_a, gen_b, property, num_tests = 100)
Like `forall` for a two-argument property `property(a, b)`.

### forall3(name, gen_a, gen_b, gen_c, property, num_tests = 100)
Like `forall` for a three-argument property.

### prop_involution(f)
Property builder: `f(f(x)) == x`.

### prop_idempotent(f)
Property builder: `f(x) == f(f(x))`.

### prop_commutative(f)
Property builder over a pair `[a, b]`: `f(a, b) == f(b, a)`. Use with a pair generator.

### prop_associative(f)
Property builder over a triple `[a, b, c]`: `f(f(a, b), c) == f(a, f(b, c))`.

### prop_preserves_length(f)
Property builder: `len(f(x)) == len(x)`.

### prop_reverse_involution()
Property: reversing a list twice gives the list back.

### prop_sort_idempotent()
Property: sorting a sorted list changes nothing.

### prop_sorted_ordered()
Property: `sort` output is in order.

### prop_sort_preserves_length()
Property: `sort` keeps the length.

### prop_add_commutative()
Property over a pair: `a + b == b + a`.

### prop_mul_commutative()
Property over a pair: `a * b == b * a`.

### prop_add_identity()
Property: `x + 0 == x`.

### prop_mul_identity()
Property: `x * 1 == x`.

### prop_mul_zero()
Property: `x * 0 == 0`.

### prop_concat_length()
Property over a pair of strings: `len(a + b) == len(a) + len(b)`.

### prop_case_preserves_length()
Property: `upper` and `lower` keep the length.

### prop_summary()
Prints a boxed results table and returns `aye` if no property failed.

### prop_reset()
Zeroes the counters.

## Example

```mdhavers
fetch "stdlib/proptesting"

forall("addition commutes", || [gen_int(), gen_int()], prop_add_commutative(), 25)
forall("sorting is ordered", || gen_int_list(), prop_sorted_ordered(), 25)
forall2("sum of naturals is natural", || gen_nat(50), || gen_nat(50), |a, b| a + b >= 0, 25)
blether prop_summary()
```

## Note on gen_bool

`gen_bool()` always returns `nae`, and the other single-character generators and `gen_one_of` leave out their last choice. Use `|| maybe()` from [chance](./chance.md) when you need a fair boolean generator.
