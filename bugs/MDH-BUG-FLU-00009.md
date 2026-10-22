# MDH-BUG-FLU-00009 — sort() leaves mixed int/float lists unsorted

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

Confirmed: sort([3, 1.5, 2]) -> [3, 1.5, 2]; comparator (interpreter.rs ~5951) returns Equal for Integer vs Float and mixed types. Inconsistent ordering may also panic in newer std sort (not observed).

## Fix

<unfixed — raised only>

## Notes
