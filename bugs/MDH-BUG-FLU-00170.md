# MDH-BUG-FLU-00170 — JS and WASM: tak and drap reject a string argument

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:50Z
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
- **State history:** Open (2026-10-26T05:03:50Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:47Z, deltic:auto role=fix run=fix-20261026T050440Z-91651552 branch=task/bug-MDH-BUG-FLU-00170-run-fix-20261026T050440Z-91651552 code=9aab318 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-a76b3347 verified_fix_run=fix-20261026T050440Z-91651552 verdict=close)

## Observation

Found by lane H1 2026-10-26 (executed): `blether tak("Hello", 2)` prints [H, e] in the interpreter and native but JS and WASM abort with "tak() needs a list and an integer"; same for drap("Hello", 6). Breaks examples/list_functions.braw on JS and WASM. Fix: accept strings (as character lists, matching the interpreter) in the JS runtime and WASM helpers; parity cases.

Evidence fingerprint: `manual:v1:js-and-wasm-tak-and-drap-reject-a-string-argument`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-a76b3347 verified_fix_run=fix-20261026T050440Z-91651552 verdict=close)

Verifier note: Manual verify: [lb luna] 9aab318 WASM (later shared string support e8c4a76), b1616f5 JS; repro printed [H, e]/[] in interpreter, JS, WASM. JS test passed 1/1; WASM tests 2/2. Removing String arms failed out.error=None assertion; removing JS cps handling failed on tak() type error.

## Notes
