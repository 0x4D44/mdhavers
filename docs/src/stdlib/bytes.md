# bytes

Thin wrappers around the built-in byte-buffer functions (`bytes`, `bytes_from_string`, `bytes_len`, `bytes_slice`, `bytes_get`, `bytes_set`, `bytes_append` and the big-endian readers and writers), under friendlier names, plus `bytes_to_string`. It depends on the interpreter's byte-buffer built-ins (interpreter only as far as verified here).

Import: `fetch "stdlib/bytes"`

No load message is printed. The module has no classes or constants. Errors come from the built-ins and are passed through unchanged, for example `bytes_get() index oot o' bounds`, `bytes_set() value must be between 0 and 255`, `bytes_read_u16be() out o' bounds` and `bytes_write_u32be() value out o' range`.

Buffers are shared, mutable objects: `bytes_set_o`, `write_u16be`, `write_u32be` and `bytes_append_o` change the buffer you pass in. `bytes_append_o` extends its first argument in place and returns that same buffer; `bytes_slice_o` returns a new buffer.

## Functions

### make_bytes(size)
Zero-filled buffer of `size` bytes.

### bytes_from_str(s)
Buffer holding the bytes of a string.

### bytes_len_o(b)
Number of bytes.

### bytes_slice_o(b, start, end)
New buffer of bytes `[start, end)`.

### bytes_get_o(b, idx)
Byte at `idx` as an integer 0 to 255.

### bytes_set_o(b, idx, val)
Sets byte `idx` to `val` (0 to 255).

### bytes_append_o(b, other)
Appends `other` to `b` in place and returns `b`.

### read_u16be(b, off)
Reads an unsigned 16-bit big-endian integer at offset `off`.

### read_u32be(b, off)
Reads an unsigned 32-bit big-endian integer.

### write_u16be(b, off, val)
Writes a 16-bit big-endian integer.

### write_u32be(b, off, val)
Writes a 32-bit big-endian integer.

### bytes_to_string(b)
Turns a buffer into a string by mapping each byte to a character with `chr` (so it is right for ASCII; bytes above 127 are not decoded as UTF-8). If `b` is not a buffer it returns `tae_string(b)`.

## Example

```mdhavers
fetch "stdlib/bytes"

ken b = make_bytes(4)
bytes_set_o(b, 0, 65)
write_u16be(b, 1, 16963)
blether bytes_get_o(b, 0)
blether read_u16be(b, 1)
blether bytes_to_string(bytes_slice_o(b, 0, 3))
blether bytes_len_o(b)
```
