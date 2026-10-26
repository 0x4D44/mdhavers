# MDH-BUG-FLU-00009 — sort() leaves mixed int/float lists unsorted

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-1b8cf31d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00009-run-verify-20261026T224201Z-1b8cf31d
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:e78b058006b457ced96402ada8cf63fec529ff946af0e413fe7a8d2cd782ef58
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:31Z, deltic:auto role=fix run=fix-20261023T014925Z-ae485e2a branch=task/bug-MDH-BUG-FLU-00009-run-fix-20261023T014925Z-ae485e2a code=d84823a gate=manual)

## Observation

Confirmed: sort([3, 1.5, 2]) -> [3, 1.5, 2]; comparator (interpreter.rs ~5951) returns Equal for Integer vs Float and mixed types. Inconsistent ordering may also panic in newer std sort (not observed).

## Fix

<unfixed — raised only>

## Notes
