# collections

Class-based and list-based collection helpers: `Stack`, `Queue`, `Deque`, `HaversSet`, `Counter` and `DefaultDict`, plus list utilities (`chunk`, `unique`, `flatten`, `group_by`, `partition`, zip helpers) and a set of plain-list functions (`stack_*`, `queue_*`, `set_*`, `counter*`) that work on ordinary lists and dicts.

Import: `fetch "stdlib/collections"`

Pure mdhavers; it runs on every backend. Loading the module prints `Collections module loaded! Yer data's in guid hauns!`. It defines global `min(a, b)` and `max(a, b)` that take exactly two arguments and shadow any built-ins of the same name. Helpers starting with `_` are internal and not documented (this module has none).

## Functions

### pair(a, b)
Returns `[a, b]`.

### fst(p)
Returns `p[0]`.

### snd(p)
Returns `p[1]`.

### triple(a, b, c)
Returns `[a, b, c]`. (Not the same as `triple(n)` in [functional](./functional.md), which multiplies by 3.)

### zip_lists(a, b)
Returns a list of `[a[i], b[i]]` pairs, as long as the shorter list.

### unzip_list(pairs)
Splits a list of pairs into `[firsts, seconds]`.

### group_by(items, key_func)
Returns a dict whose keys are `tae_string(key_func(item))` and whose values are lists of items. Example: `group_by([1,2,3,4], |x| x % 2)` gives `{"1": [1, 3], "0": [2, 4]}`.

### partition(items, predicate)
Returns `[matches, others]` (a two-element list). The version in [functional](./functional.md) returns a dict instead.

### chunk(items, n)
Splits into lists of `n`; the last chunk may be shorter. `chunk([1,2,3,4,5], 2)` gives `[[1, 2], [3, 4], [5]]`. With `n` of 0 or less nothing is ever emitted until the end, so the result is one chunk of everything.

### flatten(nested)
Flattens one level only: list elements are spliced in, non-list elements are kept.

### unique(items)
Returns the items with duplicates removed, first occurrence kept, order preserved (uses `HaversSet`, so O(n squared)).

### min(a, b)
Returns the smaller of two values.

### max(a, b)
Returns the larger of two values.

### stack_new()
Returns a new empty list to use as a stack.

### stack_push(stack, item)
Appends `item`.

### stack_pop(stack)
Removes and returns the last item. On an empty list the built-in `yank` hurls `"Cannae yank fae an empty list!"` (wrapped in the usual error text).

### stack_peek(stack)
Returns the last item, or `naething` when empty.

### stack_empty(stack)
`aye` when the list has no items.

### queue_new()
Returns a new empty list to use as a queue.

### queue_enqueue(queue, item)
Appends `item`.

### queue_dequeue(queue)
Removes and returns the first item, or `naething` when empty. Shifts every element, so it is O(n).

### queue_peek(queue)
Returns the first item, or `naething`.

### queue_empty(queue)
`aye` when empty.

### set_new()
Returns a new empty list to use as a set.

### set_add(set, item)
Adds `item` if absent. Returns `aye` if added, `nae` if it was already there.

### set_remove(set, item)
Removes the first match in place. Returns `aye` if removed, `nae` if not found.

### set_contains(set, item)
`aye` if present.

### set_union(a, b)
New list with all of `a`, then the items of `b` not already in the result.

### set_intersection(a, b)
New list of items of `a` that are in `b`.

### set_difference(a, b)
New list of items of `a` that are not in `b`.

### counter(items)
Returns a dict mapping `tae_string(item)` to its count.

### counter_get(counts, item)
Count for `item`, or 0.

### counter_most_common(counts, n)
Returns up to `n` `[key, count]` pairs, highest count first.

## Classes

### Stack
Last in, first out. Constructor: `Stack()`. Field: `items` (list, top is last).
- `push(item)` - add on top. Returns `naething` (not chainable).
- `pop()` - remove and return the top, or `naething` when empty.
- `peek()` - top item or `naething`.
- `is_empty()`, `size()`, `clear()`.

### Queue
First in, first out. Constructor: `Queue()`. Field: `items`.
- `enqueue(item)`, `dequeue()` (front or `naething`), `front()`, `back()`, `is_empty()`, `size()`, `clear()`.

### Deque
Double-ended queue. Constructor: `Deque()`. Field: `items`.
- `push_front(item)`, `push_back(item)`, `pop_front()`, `pop_back()` (both `naething` when empty), `front()`, `back()`, `is_empty()`, `size()`. No `clear`.

### HaversSet
Set backed by a list; membership uses `==`. Constructor: `HaversSet()`. Field: `items`.
- `add(item)` - only if absent. `remove(item)`, `contains(item)`.
- `union(other)`, `intersection(other)`, `difference(other)` - each returns a new `HaversSet`; `other` must be a `HaversSet` (union reads `other.items`, the others call `other.contains`).
- `is_subset(other)`, `is_empty()`, `size()`, `tae_list()` (the backing list itself, not a copy).

### Counter
Counts occurrences, keyed by `tae_string(item)` (so `1` and `"1"` share a key). Constructor: `Counter()`. Field: `counts` (dict).
- `add(item)` - increment by one. `count_all(items)` - add each item of a list.
- `get(item)` - count or 0. `total()` - sum of counts.
- `most_common(n = 10)` - list of `[key, count]` pairs, highest first. Keys are strings.

### DefaultDict
Dict with a fallback value for missing keys. Constructor: `DefaultDict(default_val = naething)`. Fields: `data`, `default_val`.
- `get(key)` - stored value, else `default_val` (the key is not inserted). `set(key, value)`, `has(key)`, `keys()`, `values()`.

## Example

```mdhavers
fetch "stdlib/collections"

ken s = Stack()
s.push(1)
s.push(2)
blether s.pop()

ken c = Counter()
c.count_all(["a", "b", "a"])
blether c.most_common(1)

blether chunk([1, 2, 3, 4, 5], 2)
blether unique([1, 2, 2, 3])
```

## Compared with structures

[structures](./structures.md) offers `Stack`, `Queue`, `Deque` and `Counter` under the same names, so fetching both makes the later fetch win. Differences:

| | collections | structures |
|---|---|---|
| Method chaining | mutators return `naething` | mutators return `masel`, so calls chain |
| Set class | `HaversSet` (list, `==` equality, `contains`) | `Set` (dict keyed by `tae_string`, `has`) |
| `Counter.most_common` | `[key, count]` pairs | dicts `{"key", "count"}`; skips zero counts |
| Extras | `DefaultDict`, plain-list helpers, `chunk`, `unique`, `flatten`, zip helpers | `PriorityQueue`, `RingBuffer`, `LRUCache`, `make_*` factories, `tae_list()` on every class |
| `Queue` peek | `front()` and `back()` | `peek()` only |

Prefer collections for list utilities and the plain-list functions. Prefer structures when you need chaining, a priority queue, a ring buffer or an LRU cache.
