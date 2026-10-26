# MDH-BUG-FLU-00170 — JS and WASM: tak and drap reject a string argument

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:50Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-a76b3347
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00170-run-verify-20261026T225713Z-a76b3347
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:ac4d9e9e83a792e0f9c74b23a37e854172474c84af1289693e3cb1217d83aee4
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:50Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:47Z, deltic:auto role=fix run=fix-20261026T050440Z-91651552 branch=task/bug-MDH-BUG-FLU-00170-run-fix-20261026T050440Z-91651552 code=9aab318 gate=manual)

## Observation

Found by lane H1 2026-10-26 (executed): `blether tak("Hello", 2)` prints [H, e] in the interpreter and native but JS and WASM abort with "tak() needs a list and an integer"; same for drap("Hello", 6). Breaks examples/list_functions.braw on JS and WASM. Fix: accept strings (as character lists, matching the interpreter) in the JS runtime and WASM helpers; parity cases.

Evidence fingerprint: `manual:v1:js-and-wasm-tak-and-drap-reject-a-string-argument`


## Fix

<unfixed — raised only>

## Notes
