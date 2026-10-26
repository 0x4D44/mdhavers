# MDH-BUG-FLU-00167 — JS backend lacks many interpreter builtins (stoater, dicht, wee, bonnie, is_a, braw, scottify...)

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:35Z
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
- **State history:** Open (2026-10-26T05:03:35Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:22Z, deltic:auto role=fix run=fix-20261026T050440Z-aa7b66d3 branch=task/bug-MDH-BUG-FLU-00167-run-fix-20261026T050440Z-aa7b66d3 code=b1616f5 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-3a39d61d verified_fix_run=fix-20261026T050440Z-aa7b66d3 verdict=close)

## Observation

Found by lane H1 2026-10-26 (executed): these raise a JS ReferenceError while the interpreter and native work: stoater, dicht, wee, bonnie, is_a, braw, scottify, redd_up, sclaff, drookit, gallus, wrang_sort, scunner_check, backside_forrit, stacktrace. E.g. stoater([3,9,2]) -> 9, dicht([1,2,999,3], 2) -> [1, 2, 3], wee(10,25) -> 10, bonnie(3) -> "~~~ 3 ~~~", is_a(3, "integer") -> aye. Examples prelude_demo, list_functions, scots_words and spread fail under JS because of it. Fix: add each to runtime/js/core_runtime.js + RUNTIME_BINDINGS, matching the interpreter, with js_golden_parity coverage.

Evidence fingerprint: `manual:v1:js-backend-lacks-many-interpreter-builtins-stoa-84166675178f6352`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-3a39d61d verified_fix_run=fix-20261026T050440Z-aa7b66d3 verdict=close)

Verifier note: Manual verify: [lb luna] b1616f5; js_builtin_fixes passed 7/7 and js_golden_parity passed; removing `stoater` made its error regression fail: expected Scots empty-list error, got `stoater is not a function`.

## Notes
