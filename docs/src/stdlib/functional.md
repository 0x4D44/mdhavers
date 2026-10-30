# functional

Higher-order list helpers (`map_list`, `filter_list`, `reduce`, `find`, `any`, `all`), function composition and partial application, common predicates, mappers and reducers, iteration utilities, sorting helpers, a `Memoize` wrapper and a chainable `Functor`.

Import: `fetch "stdlib/functional"`

Pure mdhavers; it runs on every backend. Loading the module prints `Functional module loaded! Ready fer some functional havers!`. A lambda body is either a single expression (`|x| x + 1`, which cannot hold a statement such as `blether`) or a `{ }` block that can hold statements (`|x| { blether x }`). Division of two integers truncates, so `average([1,2,3,4])` is `2`, not `2.5`. Helpers starting with `_` are internal and not documented (this module has none).

## Functions

### map_list(list, fn)
New list of `fn(item)` for each item.

### filter_list(list, predicate)
New list of the items for which `predicate` is truthy.

### reduce(list, fn, initial)
Left fold: `acc = fn(acc, item)` starting from `initial`.

### reduce_right(list, fn, initial)
Same, but walks the list in reverse. `fn` still receives `(acc, item)`.

### find(list, predicate)
First matching item, or `naething`.

### find_last(list, predicate)
Last matching item, or `naething`.

### find_index(list, predicate)
Index of the first match, or `-1`.

### any(list, predicate)
`aye` if at least one item matches.

### all(list, predicate)
`aye` if every item matches (`aye` for an empty list).

### none(list, predicate)
`aye` if no item matches.

### count_if(list, predicate)
Number of matching items.

### take_while(list, predicate)
Leading items while the predicate holds.

### drop_while(list, predicate)
Everything from the first item that fails the predicate.

### partition(list, predicate)
Returns the dict `{"match": [...], "no_match": [...]}`. ([collections](./collections.md) has a `partition` that returns a two-element list.)

### group_by(list, key_fn)
Dict from `tae_string(key_fn(item))` to the list of items, e.g. `{"aye": [1, 3], "nae": [2]}` for `is_odd`.

### flat_map(list, fn)
Maps, then splices list results one level; non-list results are kept as they are.

### compose(f, g)
Returns `|x| f(g(x))`.

### pipe_fn(f, g)
Returns `|x| g(f(x))`.

### pipeline(functions)
Returns a function that applies each function in the list, in order.

### apply_n(fn, n, initial)
Applies `fn` to `initial` `n` times.

### partial(fn, first_arg)
Returns `|b| fn(first_arg, b)`.

### partial_right(fn, second_arg)
Returns `|a| fn(a, second_arg)`.

### flip(fn)
Returns `|a, b| fn(b, a)`.

### is_even(n)
`n % 2 == 0`.

### is_odd(n)
`n % 2 != 0`.

### is_positive(n)
`n > 0`.

### is_negative(n)
`n < 0`.

### is_zero(n)
`n == 0`.

### is_empty(val)
`aye` for an empty list, string or dict; `nae` for any other value (including numbers and `naething`).

### is_not_empty(val)
The negation of `is_empty`. Note that non-collections count as "not empty".

### double(n)
`n * 2`.

### triple(n)
`n * 3`. (A different function from `triple(a, b, c)` in [collections](./collections.md).)

### square(n)
`n * n`.

### cube(n)
`n * n * n`.

### increment(n)
`n + 1`.

### decrement(n)
`n - 1`.

### negate(n)
`-n`.

### identity(x)
Returns `x`.

### constant(value)
Returns a zero-argument function that returns `value`.

### sum(list)
Sum, 0 for an empty list.

### product(list)
Product, 1 for an empty list.

### max_of(list)
Largest item, or `naething` for an empty list.

### min_of(list)
Smallest item, or `naething` for an empty list.

### average(list)
`sum / len`; 0 for an empty list. Integer lists use integer division.

### concat_all(lists)
Concatenates a list of lists.

### join_strings(strings, separator = "")
Joins strings with `separator`; `""` for an empty list.

### times(n, fn)
Calls `fn(i)` for `i` from 0 to `n - 1`. Returns nothing.

### generate(n, fn)
List of `fn(i)` for `i` in `0..n`.

### replicate(n, value)
List of `value` repeated `n` times.

### iterate(fn, initial, n)
List of `n` values: `initial`, `fn(initial)`, and so on. With `n` of 0 or 1 it still returns `[initial]`.

### sort_by(list, key_fn)
New list ordered by `key_fn(item)`, ascending (bubble sort, so O(n squared)).

### sort_desc(list)
`reverse(sort(list))`.

### sort_by_desc(list, key_fn)
`reverse(sort_by(list, key_fn))`.

### memoize(fn)
Returns `Memoize(fn)`.

### wrap(value)
Returns `Functor(value)`.

## Classes

### Memoize
Caches a one-argument function. Constructor: `Memoize(fn)`. Fields: `fn`, `cache`.
- `call(arg)` - cached result, keyed by `tae_string(arg)`. Call it as `m.call(x)`; the object is not directly callable.
- `clear_cache()` - returns `masel`. `cache_size()` - number of cached entries.

### Functor
Chainable wrapper around a value. Constructor: `Functor(value)`. Field: `value`.
- `map(fn)` - maps over a list value, or applies `fn` to a non-list value. Returns a new `Functor`.
- `filter(predicate)` - for lists; other values are returned unchanged.
- `reduce(fn, initial)` - for lists, returns a `Functor` holding the folded value; other values unchanged.
- `when(condition, fn)` - `map(fn)` if `condition` is truthy, else the same functor.
- `chain(fn)` - calls `fn(value)`; an instance result is returned as is, anything else is wrapped.
- `unwrap()` - the value.

## Example

```mdhavers
fetch "stdlib/functional"

blether map_list([1, 2, 3], double)
blether filter_list([1, 2, 3, 4], is_even)
blether reduce([1, 2, 3], |a, b| a + b, 0)
blether compose(double, increment)(5)
blether wrap([1, 2, 3]).map(square).filter(|x| x > 1).unwrap()
```
