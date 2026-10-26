# MDH-BUG-FLU-00160 — Interpreter dict delete is O(n) per key, so remove-heavy loops are quadratic

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-26T02:17:31Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T024153Z-87f9ce1f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00160-run-fix-20261026T024153Z-87f9ce1f
- **Owner base:** ccf8d410c9b13cbf39d2bee64a93505d6b6ed448
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T02:41:53Z
- **Owner until:** 2026-10-26T06:41:53Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T02:17:31Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S17 2026-10-26 (code read): DictValue::remove in src/value.rs:888 shifts the entries vector and rebuilds the whole index on every call, so heave_oot(dict, k) and dict_remove in a loop over n keys cost O(n^2) in the interpreter (and the WASM host, which uses it). Native uses tombstones (bug 91) and is linear. Fix: tombstones or an IndexMap-style swap-remove that keeps insertion order (order matters: goldens pin iteration order), with a timing test on 100k deletes.

Evidence fingerprint: `manual:v1:interpreter-dict-delete-is-o-n-per-key-so-remov-4b90d8eab0255422`


## Fix

<unfixed — raised only>

## Notes
