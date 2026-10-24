# MDH-BUG-FLU-00091 — Native dict inline reads scan linearly; dict remove and heave_oot copy the whole dict

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T213341Z-844c3ee0
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00091-run-fix-20261024T213341Z-844c3ee0
- **Owner base:** aade7cd397c51481ad75e9804b8f1968562ab322
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T21:33:41Z
- **Owner until:** 2026-10-25T01:33:41Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T04:31:22Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the native hardening lane 2026-10-23: inserts are now amortised O(1) with a hash index (MDH-BUG-FLU-00053), but compile_dict_index's inline read path still scans entries, and dict_remove / heave_oot still rebuild the dict per call (quadratic for remove-heavy loops).

## Fix

<unfixed — raised only>

## Notes
