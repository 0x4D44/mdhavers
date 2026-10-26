# MDH-BUG-FLU-00170 — JS and WASM: tak and drap reject a string argument

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:50Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T050440Z-91651552
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00170-run-fix-20261026T050440Z-91651552
- **Owner base:** 0ce2a5bc5cf9ba91c13f5b84148dcd8cfea0ba99
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T05:04:40Z
- **Owner until:** 2026-10-26T09:04:40Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:50Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26 (executed): `blether tak("Hello", 2)` prints [H, e] in the interpreter and native but JS and WASM abort with "tak() needs a list and an integer"; same for drap("Hello", 6). Breaks examples/list_functions.braw on JS and WASM. Fix: accept strings (as character lists, matching the interpreter) in the JS runtime and WASM helpers; parity cases.

Evidence fingerprint: `manual:v1:js-and-wasm-tak-and-drap-reject-a-string-argument`


## Fix

<unfixed — raised only>

## Notes
