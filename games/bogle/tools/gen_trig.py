#!/usr/bin/env python3
"""Rebuild the committed 4096-entry, signed 16.16 sine table offline."""
import math
from pathlib import Path


def table():
    # Round half away from zero. Mirror one quadrant for exact symmetry.
    quarter = [int(math.sin(i * math.pi / 2048) * 65536 + 0.5) for i in range(1025)]
    return (quarter[:-1] + quarter[:0:-1]
            + [-v for v in quarter[:-1]] + [-v for v in quarter[:0:-1]])


if __name__ == "__main__":
    values = table()
    assert len(values) == 4096
    target = Path(__file__).resolve().parents[1] / "data" / "trig.txt"
    target.write_text("".join(f"{value}\n" for value in values), encoding="ascii")
    checksum = 0
    for i, value in enumerate(values):
        checksum = (checksum + (i + 1) * (value + 65536)) % 4294967291
    print(f"4096 entries; weighted checksum {checksum}")
