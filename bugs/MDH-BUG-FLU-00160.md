# MDH-BUG-FLU-00160 — Interpreter dict delete is O(n) per key, so remove-heavy loops are quadratic

- **State:** Closed
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
- **State history:** Open (2026-10-26T02:17:31Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T03:56:35Z, deltic:auto role=fix run=fix-20261026T024153Z-87f9ce1f branch=task/bug-MDH-BUG-FLU-00160-run-fix-20261026T024153Z-87f9ce1f code=72144ea gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-1a702f2b verified_fix_run=fix-20261026T024153Z-87f9ce1f verdict=close)

## Observation

Found by lane S17 2026-10-26 (code read): DictValue::remove in src/value.rs:888 shifts the entries vector and rebuilds the whole index on every call, so heave_oot(dict, k) and dict_remove in a loop over n keys cost O(n^2) in the interpreter (and the WASM host, which uses it). Native uses tombstones (bug 91) and is linear. Fix: tombstones or an IndexMap-style swap-remove that keeps insertion order (order matters: goldens pin iteration order), with a timing test on 100k deletes.

Evidence fingerprint: `manual:v1:interpreter-dict-delete-is-o-n-per-key-so-remov-4b90d8eab0255422`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-1a702f2b verified_fix_run=fix-20261026T024153Z-87f9ce1f verdict=close)

Verifier note: Manual verify: [lb luna] 72144ea; dict_delete_100k_keys_is_not_quadratic passed 0.68s; heave_oot_100k_keys_finishes_quickly passed (output 0, 3.04s); mutant compaction every 1000 deletes failed timing assertion: 7.61s > 5s.

## Notes
