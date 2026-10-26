# MDH-BUG-FLU-00167 — JS backend lacks many interpreter builtins (stoater, dicht, wee, bonnie, is_a, braw, scottify...)

- **State:** Open
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
- **State history:** Open (2026-10-26T05:03:35Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26 (executed): these raise a JS ReferenceError while the interpreter and native work: stoater, dicht, wee, bonnie, is_a, braw, scottify, redd_up, sclaff, drookit, gallus, wrang_sort, scunner_check, backside_forrit, stacktrace. E.g. stoater([3,9,2]) -> 9, dicht([1,2,999,3], 2) -> [1, 2, 3], wee(10,25) -> 10, bonnie(3) -> "~~~ 3 ~~~", is_a(3, "integer") -> aye. Examples prelude_demo, list_functions, scots_words and spread fail under JS because of it. Fix: add each to runtime/js/core_runtime.js + RUNTIME_BINDINGS, matching the interpreter, with js_golden_parity coverage.

Evidence fingerprint: `manual:v1:js-backend-lacks-many-interpreter-builtins-stoa-84166675178f6352`


## Fix

<unfixed — raised only>

## Notes
