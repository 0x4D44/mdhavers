# MDH-BUG-FLU-00024 — _ rejected as loop variable, lambda or function parameter

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:52:01Z, deltic:auto role=fix run=fix-20261023T015156Z-7c946d96 branch=task/bug-MDH-BUG-FLU-00024-run-fix-20261023T015156Z-7c946d96 code=8204122 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-12a6ca5e verified_fix_run=fix-20261023T015156Z-7c946d96 verdict=close)

## Observation

Confirmed: fer _ in 0..3 -> parse error. token.rs:255 / expect_identifier.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-12a6ca5e verified_fix_run=fix-20261023T015156Z-7c946d96 verdict=close)

Verifier note: Manual verify: [lc luna] 8204122: underscore_binders_work_in_loops_lambdas_and_functions passed; removing `expect_binder` support failed parsing `_` as the loop variable on line 3. Source diff restored clean.

## Notes
