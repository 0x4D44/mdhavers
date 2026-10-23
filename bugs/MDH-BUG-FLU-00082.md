# MDH-BUG-FLU-00082 — C runtime __mdh_replace computes the result size without overflow checks

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T03:06:28Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T035207Z-d349c40a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00082-run-fix-20261023T035207Z-d349c40a
- **Owner base:** 7fbd2a1f2be1d7fa700fa9f4b980224089bbfdc9
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:52:07Z
- **Owner until:** 2026-10-23T07:52:07Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:06:28Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the LLVM lane 2026-10-23: runtime/mdh_runtime.c __mdh_replace multiplies occurrence count by replacement length with unchecked size_t arithmetic; the repeat/pad paths were hardened (MDH-BUG-FLU-00052) but replace was not. A huge replacement count can under-allocate and overflow the heap.

## Fix

<unfixed — raised only>

## Notes
