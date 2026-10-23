# MDH-BUG-FLU-00083 — Pre-existing llvm_comprehensive_tests failures: fankle_cov x3, dict_merge, out_of_bounds_list, lines

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T03:06:29Z
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
- **State history:** Open (2026-10-23T03:06:29Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the LLVM lane 2026-10-23: these 6 tests fail on origin/master before and after the LLVM safety fixes (10 more fail on lambda-as-argument, MDH-BUG-FLU-00070). The suite was only run to ~5300/7126 within the time box; the tail is unverified.

## Fix

<unfixed — raised only>

## Notes
