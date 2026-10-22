# MDH-BUG-FLU-00011 — jammy() always returns the same value on macOS (time-seeded one-step LCG)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** -
- **Owner role:** -
- **Owner run:** -
- **Owner host:** -
- **Owner branch:** -
- **Owner base:** -
- **Owner fingerprint:** -
- **Owner since:** -
- **Owner until:** -
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed on macOS: 20 calls of jammy(0, 8) all return 1. The LCG is seeded from wall-clock nanos (microsecond resolution on macOS) and copied 10 times across interpreter.rs (~5977).

## Fix

<unfixed — raised only>

## Notes
