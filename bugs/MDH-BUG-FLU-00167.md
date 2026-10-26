# MDH-BUG-FLU-00167 — JS backend lacks many interpreter builtins (stoater, dicht, wee, bonnie, is_a, braw, scottify...)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:35Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-3a39d61d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00167-run-verify-20261026T225713Z-3a39d61d
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:419c3f1676339f7246ba53a91c91dff4158158e3d7e1c6ad9dbcbf42590e353b
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:35Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:22Z, deltic:auto role=fix run=fix-20261026T050440Z-aa7b66d3 branch=task/bug-MDH-BUG-FLU-00167-run-fix-20261026T050440Z-aa7b66d3 code=b1616f5 gate=manual)

## Observation

Found by lane H1 2026-10-26 (executed): these raise a JS ReferenceError while the interpreter and native work: stoater, dicht, wee, bonnie, is_a, braw, scottify, redd_up, sclaff, drookit, gallus, wrang_sort, scunner_check, backside_forrit, stacktrace. E.g. stoater([3,9,2]) -> 9, dicht([1,2,999,3], 2) -> [1, 2, 3], wee(10,25) -> 10, bonnie(3) -> "~~~ 3 ~~~", is_a(3, "integer") -> aye. Examples prelude_demo, list_functions, scots_words and spread fail under JS because of it. Fix: add each to runtime/js/core_runtime.js + RUNTIME_BINDINGS, matching the interpreter, with js_golden_parity coverage.

Evidence fingerprint: `manual:v1:js-backend-lacks-many-interpreter-builtins-stoa-84166675178f6352`


## Fix

<unfixed — raised only>

## Notes
