# MDH-BUG-FLU-00005 — String repeat / pad / bytes with huge counts panic with capacity overflow

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
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
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: "ab" * 9223372036854775807 -> panic "capacity overflow" (interpreter.rs ~11997, 6132, 8026). Large non-overflowing counts abort on allocation failure. Should raise a Scots error.

## Fix

<unfixed — raised only>

## Notes
