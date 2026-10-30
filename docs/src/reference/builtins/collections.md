# Collection, Maths and Misc Built-ins

Creels (sets), list helpers, higher-order list functions, random numbers, a few maths functions, atomics and some Scots checkers. All are registered by the interpreter in `src/interpreter.rs`. The higher-order ones (`hunt`, `ony`, `aw`) run in `Interpreter::call_builtin_hof`.

List functions never change the list you pass in. They return a new list. Creel functions that add or remove items (`toss_in`) change the creel in place.

**Backends.** Checked by compiling and running this page's Examples block. JS (`compile`, run under Node) and LLVM (`build`) printed the same output as the interpreter, except that `atomic_cas` (like every atomic) throws `atomic_cas() isnae available in JS - it's fer the interpreter an' native builds only` in JS. WASM (`wasm`) compiles every call: first-order built-ins are passed by name to the host runner (`__mdh_builtinN`, see `src/wasm_compiler.rs` and `src/wasm_runner.rs`) and `hunt`, `ony` and `aw` run in the host; the WASM output was not run, so treat WASM as untested. Functions not used in the Examples block are untested on the compiled backends.

## Creels (sets)

A creel holds each value once. Items are compared by value.

| Call | Returns | Notes |
|------|---------|-------|
| `creel(list)` | creel | Builds a creel from a list. A creel passed in is returned as it is. Error: `creel() needs a list tae make a set fae`. |
| `empty_creel()` | creel | A new empty creel. |
| `toss_in(creel, item)` | the same creel | Adds `item` in place. Error: `toss_in() needs a creel (set)`. |
| `is_in_creel(creel, item)` | boolean | Error: `is_in_creel() needs a creel (set)`. |
| `creels_thegither(a, b)` | creel | Union. Error: `creels_thegither() needs two creels`. |
| `creels_baith(a, b)` | creel | Intersection. Error: `creels_baith() needs two creels`. |
| `creels_differ(a, b)` | creel | Items in `a` but not in `b`. Error: `creels_differ() needs two creels`. |
| `creel_tae_list(creel)` | list | Items sorted by their printed text, so `10` sorts before `9`. Error: `creel_tae_list() needs a creel`. |
| `is_subset(a, b)` | boolean | Is every item of `a` in `b`? Error: `is_subset() needs two creels`. |
| `is_superset(a, b)` | boolean | Does `a` hold every item of `b`? Error: `is_superset() needs two creels`. |
| `is_disjoint(a, b)` | boolean | True when they share no item. Error: `is_disjoint() needs two creels`. |

## Lists

| Call | Returns | Notes |
|------|---------|-------|
| `uniq(list)` | list | Drops repeats, keeping the first of each. Compares the debug text of each item. Error: `uniq() needs a list`. |
| `unique(list)` | list | Same result by comparing values directly. Error: `unique() needs a list`. |
| `drookit(list)` | boolean | True when the list holds a duplicate. Error: `drookit() needs a list`. |
| `redd_up(list)` | list | Removes every `naething`. Error: `redd_up() needs a list`. |
| `grup_runs(list)` | list of lists | Groups runs of equal neighbours: `[1, 1, 2, 1]` gives `[[1, 1], [2], [1]]`. Error: `grup_runs() needs a list`. |
| `zip(a, b)` | list of pairs | Pairs items up and stops at the shorter list. Error: `zip() needs two lists`. |
| `interleave(a, b)` | list | Alternates items; the longer list's tail is kept. Error: `interleave() needs two lists`. |
| `split_by(list, predicate)` | `[matching, rest]` | `predicate` is a string: `"even"`, `"odd"`, `"positive"`, `"negative"`, `"truthy"`, `"nil"`, `"string"` or `"number"`. Errors: `Unknown predicate '<p>'. Try: even, odd, positive, negative, truthy, nil, string, number`; `split_by() needs a list and a predicate string`. |
| `chynge(list, index, value)` | list | A copy with `value` inserted at `index`; a negative index counts from the end. Errors: `Index <i> oot o' bounds fer list o' length <n>`; `chynge() needs an integer index`; `chynge() needs a list`. |
| `dicht(list, index)` | list | A copy with the item at `index` removed. Errors as for `chynge`, with `dicht()`. |
| `index_of(list, value)` | integer | Index of the first equal item, or `-1` (see also [text and data](text-data.md)). |
| `backside_forrit(list)` | list | Reversed copy; also reverses a string. Error: `backside_forrit() expects a list or string`. |
| `bampot_mode(list)` | list | A shuffled copy (random order). Error: `bampot_mode() needs a list`. |

### Higher-order functions

These take a function (a `dae` or a lambda `|x| ...`) as the second argument.

| Call | Returns | Notes |
|------|---------|-------|
| `hunt(list, fn)` | item or `naething` | The first item for which `fn(item)` is truthy. |
| `ony(list, fn)` | boolean | True if `fn(item)` is truthy for any item. Stops at the first. |
| `aw(list, fn)` | boolean | True if `fn(item)` is truthy for every item. Stops at the first failure. |

## Random numbers

| Call | Returns | Notes |
|------|---------|-------|
| `random()` | float | A number from 0.0 up to, but not including, 1.0. |
| `random_int(min, max)` | integer | Inclusive of both ends. Errors: `random_int() needs integer bounds`; `random_int() min must be <= max`. |
| `random_choice(list)` | item or `naething` | `naething` for an empty list. Error: `random_choice() needs a list`. |

## Maths

`pi`, `e` and `tau` are functions, so write `pi()`.

| Call | Returns | Notes |
|------|---------|-------|
| `pow(base, exponent)` | float | Always a float, even for integers. Error: `pow() needs numbers`. |
| `trunc(number)` | integer | Cuts off the fraction towards zero. Error: `trunc() needs a number`. |
| `log2(number)` | float | Base 2 logarithm. Error: `log2() needs a number`. |
| `pi()` | float | 3.141592653589793 |
| `e()` | float | 2.718281828459045 |
| `tau()` | float | 6.283185307179586 (twice `pi()`) |

## Atomics

| Call | Returns | Notes |
|------|---------|-------|
| `atomic_cas(atomic, expected, desired)` | boolean | Compare-and-swap: if the atomic holds `expected`, it is set to `desired` and the call returns `aye`; otherwise nothing changes and it returns `nae`. Floats are cut to integers. Errors: `atomic_cas() expects atomic handle`; `atomic_cas() expects integer`. Make an atomic with `atomic_new(value)`. Not available in JS output. |

## Scots checkers

| Call | Returns | Notes |
|------|---------|-------|
| `gallus(value)` | boolean | True for a number above 100 or below -100, a string longer than 20 bytes (not characters), or a list of more than 10 items. Anything else is false. |
| `numpty_check(value)` | string | `That's naething, ya numpty!` for `naething`, `Empty string, ya numpty!`, `Empty list, ya numpty!`, otherwise `That's braw!`. |
| `scunner_check(value, type_name)` | `aye` or string | `aye` when the value's type name matches, otherwise `Och, ya scunner! Expected <wanted> but got <actual>`. Type names are the interpreter's, such as `integer`, not `int`. Error: `scunner_check() needs type name as second arg`. |
| `wrang_sort(value, type_name)` | boolean | True when the value is not of that type. Error: `Second arg must be a type name string`. |

## Examples

```mdhavers
ken a = creel([1, 2, 3])
ken b = creel([3, 4])
toss_in(a, 9)
blether is_in_creel(a, 9)
blether creel_tae_list(creels_thegither(a, b))
blether creel_tae_list(creels_baith(a, b))
blether creel_tae_list(creels_differ(a, b))
blether is_subset(creel([1, 2]), a)
blether is_superset(a, creel([1, 2]))
blether is_disjoint(a, creel([7]))
blether creel_tae_list(empty_creel())

blether unique([1, 2, 2, 3, 3, 3])
blether grup_runs([1, 1, 2, 2, 1])
blether redd_up([1, naething, 2])
blether drookit([1, 2, 1])
blether zip([1, 2, 3], ["a", "b"])
blether interleave([1, 2, 3], ["a"])
blether split_by([1, 2, 3, 4], "even")
blether chynge([1, 3], 1, 2)
blether dicht([1, 2, 3], 0)
blether backside_forrit([1, 2, 3])

blether hunt([1, 5, 8], |x| x > 3)
blether ony([1, 2], |x| x > 1)
blether aw([1, 2], |x| x > 1)

blether random_int(4, 4)
blether random_choice([7])
blether random() < 1.0
blether pow(2, 10)
blether trunc(-3.7)
blether log2(8)
blether pi()

blether gallus(500)
blether numpty_check(naething)
blether scunner_check(5, "integer")
blether wrang_sort(5, "string")

ken counter = atomic_new(1)
blether atomic_cas(counter, 1, 5)
blether atomic_cas(counter, 1, 9)
blether atomic_load(counter)
```
