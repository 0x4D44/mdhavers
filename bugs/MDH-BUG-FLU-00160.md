# MDH-BUG-FLU-00160 — Interpreter dict delete is O(n) per key, so remove-heavy loops are quadratic

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-26T02:17:31Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-1a702f2b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00160-run-verify-20261026T225713Z-1a702f2b
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:8378f135ff02f87b4f41bc0d0133b2458ba02984d5578b231991fd98dec51bd9
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
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
