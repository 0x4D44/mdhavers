"""Offline fixture generator; the decoder and baker never call this script.

Python's standard-library zlib supplies an independent deflate oracle. The raw
RGBA expectations come directly from the source pixels, before PNG filtering.
Run from any directory with python3 games/bogle/tests/png/generate.py.
"""

from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parent


def chunk(kind, data):
    return (struct.pack(">I", len(data)) + kind + data
            + struct.pack(">I", zlib.crc32(kind + data)))


def paeth(a, b, c):
    p = a + b - c
    distances = (abs(p - a), abs(p - b), abs(p - c))
    return (a, b, c)[distances.index(min(distances))]


def png(name, w, h, colour, pixels, filter_type=0, palette=b"", depth=8,
        interlace=0, corrupt_adler=False):
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[colour]
    stride = w * channels
    assert len(pixels) == stride * h
    filtered = bytearray()
    for y in range(h):
        filtered.append(filter_type)
        for x in range(stride):
            at = y * stride + x
            a = pixels[at - channels] if x >= channels else 0
            b = pixels[at - stride] if y else 0
            c = pixels[at - stride - channels] if y and x >= channels else 0
            predictors = (0, a, b, (a + b) // 2, paeth(a, b, c), 0)
            filtered.append((pixels[at] - predictors[filter_type]) % 256)
    compressed = zlib.compress(filtered)
    if corrupt_adler:
        compressed = compressed[:-1] + bytes([compressed[-1] ^ 1])
    header = struct.pack(">IIBBBBB", w, h, depth, colour, 0, 0, interlace)
    data = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
    if palette:
        data += chunk(b"PLTE", palette)
    data += chunk(b"tEXt", b"fixture\0ancillary chunks are skipped")
    # Every valid fixture exercises IDAT concatenation, including a split header.
    for part in (compressed[:1], compressed[1:5], compressed[5:]):
        data += chunk(b"IDAT", part)
    data += chunk(b"IEND", b"")
    (ROOT / (name + ".png")).write_bytes(data)
    if not (corrupt_adler or interlace or depth != 8 or colour == 4
            or filter_type > 4):
        rgba = bytearray()
        for at in range(0, len(pixels), channels):
            sample = pixels[at:at + channels]
            if colour == 0:
                rgba += sample * 3 + b"\xff"
            elif colour == 3:
                start = sample[0] * 3
                if start + 3 > len(palette):
                    return
                rgba += palette[start:start + 3] + b"\xff"
            elif colour == 2:
                rgba += sample + b"\xff"
            else:
                rgba += sample
        (ROOT / (name + ".rgba")).write_bytes(rgba)


rgb = bytes([0, 255, 17, 250, 4, 99, 12, 88, 200,
             255, 0, 31, 3, 249, 80, 210, 7, 160,
             11, 90, 255, 199, 20, 0, 64, 128, 192])
for f in range(5):
    png(f"filter{f}", 3, 3, 2, rgb, f)
png("grey", 2, 2, 0, bytes([0, 255, 37, 128]), 4)
png("rgba", 2, 3, 6, bytes([255, 0, 255, 0, 1, 2, 3, 127,
                            0, 255, 0, 128, 100, 120, 140, 255,
                            255, 255, 255, 255, 0, 0, 0, 255]), 1)
palette = bytes([0, 0, 0, 255, 0, 255, 11, 22, 33, 200, 240, 17])
png("indexed", 3, 2, 3, bytes([0, 1, 2, 3, 2, 0]), 2, palette)
png("bad_adler", 3, 3, 2, rgb, corrupt_adler=True)
png("interlaced", 3, 3, 2, rgb, interlace=1)
png("bad_depth", 3, 3, 2, rgb, depth=16)
png("bad_colour", 1, 1, 4, bytes([7, 255]))
png("bad_filter", 3, 3, 2, rgb, filter_type=5)
png("bad_index", 1, 1, 3, bytes([4]), palette=palette)
(ROOT / "bad_signature.png").write_bytes(b"not PNG!")
(ROOT / "truncated.png").write_bytes((ROOT / "filter0.png").read_bytes()[:-7])

raw = ((b"abracadabra, braw castle!\x00\xff" * 256) + bytes(range(256)) * 4) * 2
for name, level, strategy, block_type in (
        ("stored", 0, zlib.Z_DEFAULT_STRATEGY, 0),
        ("fixed", 9, zlib.Z_FIXED, 1),
        ("dynamic", 6, zlib.Z_DEFAULT_STRATEGY, 2)):
    compressor = zlib.compressobj(level, zlib.DEFLATED, 15, 8, strategy)
    compressed = compressor.compress(raw) + compressor.flush()
    assert (compressed[2] >> 1) & 3 == block_type, name
    assert zlib.decompress(compressed) == raw
    (ROOT / (name + ".zlib")).write_bytes(compressed)
    (ROOT / (name + ".raw")).write_bytes(raw)
print("generated PNG/RGBA fixtures and three verified deflate block vectors")

# RFC 1951 section 3.2.7: a literal-only dynamic block may have no distance
# codes. Canonical code-length codes are 18=0, 0=10, 1=11. The literal tree
# has two one-bit codes: 'A'=0, end-of-block=1; its one distance length is zero.
literal_only = bytes.fromhex("789c05c081080000000020b6fda54e00420042")
assert zlib.decompress(literal_only) == b"A"
(ROOT / "literal_only.zlib").write_bytes(literal_only)
(ROOT / "literal_only.raw").write_bytes(b"A")
