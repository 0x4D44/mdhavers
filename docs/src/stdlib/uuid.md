# uuid

Identifier helpers: random version-4 UUIDs, short random IDs, numeric IDs, counters (`SequentialId`, `PrefixedId`), timestamp-based and sortable IDs, UUID validation and URL slugs. Randomness comes from the built-in `jammy`, so IDs are unique in practice but not cryptographically secure. Works on every backend; the timestamp functions use the built-in `noo()`.

Import: `fetch "stdlib/uuid"`

Importing prints `UUID module loaded! Ready tae gie things unique names!`. No function hurls a custom error. Helpers starting with `_` are internal and not documented.

## Functions

### uuid4()
Random UUID v4 string, lowercase, 36 characters (`xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`, where `y` is `8`, `9`, `a` or `b`).

### simple_id(length = 8)
Random string of lowercase letters and digits.

### numeric_id(length = 10)
Random string of digits (a string, so leading zeros survive).

### timestamp_id()
`<noo()>-<4 random chars>`.

### sortable_id()
Timestamp left-padded with zeros to 15 characters, a dash, then 8 random characters, so IDs sort by creation time.

### is_uuid(s)
`aye` if `s` is a 36-character v4-style UUID: dashes in the right places, version `4`, variant `8`/`9`/`a`/`b`, other characters hex. Hex digits may be upper or lower case, but the variant digit must be lower case.

### user_id_generator()
`PrefixedId("usr")`.

### session_id_generator()
`PrefixedId("sess")`.

### order_id_generator()
`PrefixedId("ord")`.

### transaction_id_generator()
`PrefixedId("txn")`.

### slugify(text)
Lowercases, keeps only `a-z` and `0-9`, turns runs of spaces, `-` and `_` into a single `-`, drops other characters (including non-ASCII letters) and trims a trailing dash.

### unique_slug(text)
`slugify(text)` plus `-` and a 6-character `simple_id`.

## Classes

### SequentialId
`SequentialId(prefix = "", start = 1)`; fields `prefix`, `counter`.

- `next()`: returns `prefix + counter` and advances the counter.
- `current()`: the last ID issued (`counter - 1`).
- `peek()`: the ID `next()` would return.
- `reset(start = 1)`: sets the counter.

### PrefixedId
`PrefixedId(prefix, separator = "_")`; fields `prefix`, `separator`, `counter`.

- `next()`: `prefix + separator + simple_id(8)` and bumps the counter.
- `next_numeric()`: `prefix + separator + counter` after incrementing, so the first is `prefix_1`.

## Example

```mdhavers
fetch "stdlib/uuid"

ken id = uuid4()
blether len(id)
blether is_uuid(id)

ken seq = SequentialId("ORD-", 100)
blether seq.next()
blether seq.next()
blether seq.peek()

blether slugify("Hello, World_ of Scots!")
blether PrefixedId("usr").next_numeric()
```

## Compared with chance

[chance](./chance.md) also has `generate_uuid()`, which produces a random hex string in UUID layout without version or variant digits. Use `uuid4()` from this module when you need a real v4 shape (it passes `is_uuid`).
