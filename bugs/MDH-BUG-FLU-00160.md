# MDH-BUG-FLU-00160 — Interpreter dict delete is O(n) per key, so remove-heavy loops are quadratic

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-26T02:17:31Z
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
- **State history:** Open (2026-10-26T02:17:31Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T03:56:35Z, deltic:auto role=fix run=fix-20261026T024153Z-87f9ce1f branch=task/bug-MDH-BUG-FLU-00160-run-fix-20261026T024153Z-87f9ce1f code=72144ea gate=manual)

## Observation

Found by lane S17 2026-10-26 (code read): DictValue::remove in src/value.rs:888 shifts the entries vector and rebuilds the whole index on every call, so heave_oot(dict, k) and dict_remove in a loop over n keys cost O(n^2) in the interpreter (and the WASM host, which uses it). Native uses tombstones (bug 91) and is linear. Fix: tombstones or an IndexMap-style swap-remove that keeps insertion order (order matters: goldens pin iteration order), with a timing test on 100k deletes.

Evidence fingerprint: `manual:v1:interpreter-dict-delete-is-o-n-per-key-so-remov-4b90d8eab0255422`


## Fix

<unfixed — raised only>

## Notes
