# MDH-BUG-FLU-00082 — C runtime __mdh_replace computes the result size without overflow checks

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T03:06:28Z
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
- **State history:** Open (2026-10-23T03:06:28Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:52:14Z, deltic:auto role=fix run=fix-20261023T035207Z-d349c40a branch=task/bug-MDH-BUG-FLU-00082-run-fix-20261023T035207Z-d349c40a code=6029d44 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-aa05480a verified_fix_run=fix-20261023T035207Z-d349c40a verdict=close)

## Observation

Found by the LLVM lane 2026-10-23: runtime/mdh_runtime.c __mdh_replace multiplies occurrence count by replacement length with unchecked size_t arithmetic; the repeat/pad paths were hardened (MDH-BUG-FLU-00052) but replace was not. A huge replacement count can under-allocate and overflow the heap.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-aa05480a verified_fix_run=fix-20261023T035207Z-d349c40a verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 6029d44; huge_replace_hurls_instead_of_writing_through_a_failed_allocation: 1 passed; unchecked-size/raw-allocation mutant crashed, run_ok exit assertion got None vs Some(0).

## Notes
