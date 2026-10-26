# MDH-BUG-FLU-00169 — JS: destructuring with a rest element in the middle emits invalid JavaScript

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:45Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-cca26030
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00169-run-verify-20261026T225713Z-cca26030
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:7a023f7c189e38e276ff085cab3251535ae4c67d646e8a8fcf194d1c6ceffed2
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:45Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:34Z, deltic:auto role=fix run=fix-20261026T050440Z-414c5191 branch=task/bug-MDH-BUG-FLU-00169-run-fix-20261026T050440Z-414c5191 code=b1616f5 gate=manual)

## Observation

Found by lane H1 2026-10-26 (executed): `ken [x, ...mid, y] = [10, 20, 30, 40, 50]` / `blether mid` prints [20, 30, 40] in the interpreter, native and WASM; JS emits `let [x, ...mid, y] = ...` and node fails with a SyntaxError. Fix: lower a middle rest to slice() calls in src/compiler.rs; parity case.

Evidence fingerprint: `manual:v1:js-destructuring-with-a-rest-element-in-the-mid-8ffa714f6151d74e`


## Fix

<unfixed — raised only>

## Notes
