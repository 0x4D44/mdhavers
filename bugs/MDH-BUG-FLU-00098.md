# MDH-BUG-FLU-00098 — JS: gie in a block expression inside a default parameter escapes uncaught

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-23T22:22:08Z
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
- **State history:** Open (2026-10-23T22:22:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:38:25Z, deltic:auto role=fix run=fix-20261024T013820Z-57948697 branch=task/bug-MDH-BUG-FLU-00098-run-fix-20261024T013820Z-57948697 code=50f0079 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-60ca1a43 verified_fix_run=fix-20261024T013820Z-57948697 verdict=close)

## Observation

Found by lane G94 (MDH-BUG-FLU-00094 fix, 8b1d2ff) 2026-10-23. A block-expression gie in a dae default parameter value throws the per-call sentinel before the function body's try/catch is entered, so it escapes as an uncaught exception. The interpreter returns from the called function.

Evidence fingerprint: `manual:v1:js-gie-in-a-block-expression-inside-a-default-p-9af8d25bd734db08`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-60ca1a43 verified_fix_run=fix-20261024T013820Z-57948697 verdict=close)

Verifier note: Manual verify: [lb luna] 50f0079 (implementation 7335439); parity_js_gie_in_default_parameter_mdh_bug_flu_00098 passed; disabling sentinel wrapper failed with ReferenceError `__gie_1 is not defined` for default, method, and recursive cases.

## Notes
