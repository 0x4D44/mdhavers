# PNG unit vectors

Created once with Python 3's standard-library `struct` and `zlib`; regenerate with
`python3 games/bogle/tests/png/generate.py`. This is an offline fixture generator.
The game, decoder and asset baker have no Python dependency.

`filter0` through `filter4` contain the same 3 by 3 RGB pixels encoded with None,
Sub, Up, Average and Paeth. The RGB source includes channel wrap-around and zero
left/up predictors. `grey` is 2 by 2, `indexed` is 3 by 2 with a four-entry PLTE,
and `rgba` is 2 by 3 with alpha 0, 127, 128 and 255. Their `.rgba` files are exact
row-major expected pixels, taken from the inputs before filtering/compression.
Every image splits its zlib stream across three IDAT chunks and has a tEXt chunk
to skip. PNG chunk CRCs are valid; the decoder deliberately does not check them.

The `.zlib` vectors use stored, fixed and dynamic blocks; the generator asserts
their first BTYPE and verifies them with `zlib.decompress`. Their `.raw` files
are exact output, with long overlapping matches, NUL and byte 255.
`literal_only.zlib` is a hand-encoded dynamic block with an empty distance tree,
as allowed by [RFC 1951 section 3.2.7](https://www.rfc-editor.org/rfc/rfc1951).
Its exact output is `A`, independently verified with `zlib.decompress`.

Malformed PNGs cover Adler corruption (with a valid chunk CRC), interlace, bit
depth, unsupported colour type, an unknown filter, an out-of-range palette index,
signature corruption and truncation. `games/bogle/tests/png.braw` checks every
output byte and dimensions, and requires these malformed inputs to raise.
