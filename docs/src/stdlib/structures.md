# structures

Chainable data structures: `Stack`, `Queue`, `Deque`, `Set`, `PriorityQueue`, `RingBuffer`, `LRUCache` and `Counter`, plus four factory functions. Almost every mutating method returns the object itself, so calls can be chained.

Import: `fetch "stdlib/structures"`

Pure mdhavers; it runs on every backend. Loading the module prints `Structures module loaded! Yer data's aw organised noo!`. Helpers starting with `_` are internal and not documented (`PriorityQueue._sort` and `LRUCache._touch`).

## Functions

### make_stack()
Returns `Stack()`.

### make_queue()
Returns `Queue()`.

### make_set(items = [])
Returns a `Set` with every item of `items` added.

### make_counter(items = [])
Returns `Counter(items)`.

## Classes

### Stack
Last in, first out. Constructor: `Stack()`. Field: `items`.
- `push(item)` - returns `masel`. `pop()` - top or `naething`. `peek()` - top or `naething`.
- `is_empty()`, `size()`, `clear()` (returns `masel`), `tae_list()` (backing list, top is last).

### Queue
First in, first out. Constructor: `Queue()`. Field: `items`.
- `enqueue(item)` (returns `masel`), `dequeue()` (front or `naething`), `peek()` (front or `naething`), `is_empty()`, `size()`, `clear()`, `tae_list()`.

### Deque
Double-ended queue. Constructor: `Deque()`. Field: `items`.
- `push_front(item)`, `push_back(item)` - return `masel`.
- `pop_front()`, `pop_back()`, `peek_front()`, `peek_back()` - `naething` when empty.
- `is_empty()`, `size()`, `clear()`, `tae_list()`.

### Set
Unique elements, keyed by `tae_string(item)`. Constructor: `Set()`. Field: `items` (dict).
- `add(item)`, `remove(item)`, `clear()` - return `masel`. `has(item)`.
- `size()`, `is_empty()`, `tae_list()` (new list).
- `union(other)`, `intersection(other)`, `difference(other)` - new `Set`. `is_subset(other)`, `is_superset(other)`.

Removal stores the marker string `"__REMOVED__"` rather than deleting the key, so that exact string cannot be stored as a member: `Set().add("__REMOVED__").has("__REMOVED__")` is `nae`. Items that stringify the same (`1` and `"1"`) are the same member.

### PriorityQueue
Lowest priority number comes out first. Constructor: `PriorityQueue(comparator = naething)`; the comparator is stored but never used. Fields: `items` (list of `{"item", "priority"}` dicts), `comparator`.
- `enqueue(item, priority)` - returns `masel`. Re-sorts with a bubble sort on every insert, so it is O(n squared) per call; it is not a true heap.
- `dequeue()` - the item with the lowest priority, or `naething`. `peek()` - same, without removing.
- `is_empty()`, `size()`, `clear()`.

### RingBuffer
Fixed-capacity circular buffer; pushing when full overwrites the oldest item. Constructor: `RingBuffer(capacity)`. Fields: `capacity`, `items`, `head`, `tail`, `count`.
- `push(item)` - returns `masel`. `pop()` - removes the oldest or `naething`. `peek()` - oldest or `naething`.
- `is_empty()`, `is_full()`, `size()`, `get_capacity()`, `tae_list()` (oldest to newest).

### LRUCache
Least-recently-used cache. Constructor: `LRUCache(capacity)`. Fields: `capacity`, `cache`, `order`.
- `get(key)` - value (and marks it recently used), or `naething`. `set(key, value)` - returns `masel`; at capacity the oldest key is evicted. `has(key)`, `size()`, `clear()`.
- Keys are compared by `tae_string(key)`. An evicted key is set to `naething` in `cache` rather than removed, and a stored `naething` value counts as missing for `has` and `size`.

### Counter
Counts occurrences. Constructor: `Counter(items = [])`, which adds each item. Field: `counts`.
- `add(item, count = 1)` - returns `masel`. `get(item)` - count or 0. `remove(item)` - sets the count to 0 (returns `masel`). `total()`.
- `most_common(n = 10)` - list of `{"key": key, "count": count}` dicts, highest first, zero counts skipped.
- `tae_dict()` - dict of key to count, zero counts skipped.

## Example

```mdhavers
fetch "stdlib/structures"

ken s = Stack()
s.push(1).push(2)
blether s.tae_list()

ken pq = PriorityQueue()
pq.enqueue("low", 5).enqueue("high", 1)
blether pq.dequeue()

ken rb = RingBuffer(3)
rb.push(1).push(2).push(3).push(4)
blether rb.tae_list()

ken c = Counter(["a", "b", "a"])
blether c.most_common(1)
```

## Compared with collections

See [collections](./collections.md). Both define `Stack`, `Queue`, `Deque` and `Counter`, so fetching both makes the later fetch win; pick one. Here the methods chain, `Counter.most_common` returns dicts, and `Set` is hash-keyed rather than list-backed. Extra classes: `PriorityQueue`, `RingBuffer`, `LRUCache`. The collections module is the one with the list helpers (`chunk`, `unique`, `flatten`, `group_by`), `DefaultDict` and the plain-list `stack_*`/`queue_*`/`set_*` functions.
