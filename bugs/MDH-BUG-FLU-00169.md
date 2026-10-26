# MDH-BUG-FLU-00169 — JS: destructuring with a rest element in the middle emits invalid JavaScript

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:45Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T050440Z-414c5191
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00169-run-fix-20261026T050440Z-414c5191
- **Owner base:** 0ce2a5bc5cf9ba91c13f5b84148dcd8cfea0ba99
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T05:04:40Z
- **Owner until:** 2026-10-26T09:04:40Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:45Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26 (executed): `ken [x, ...mid, y] = [10, 20, 30, 40, 50]` / `blether mid` prints [20, 30, 40] in the interpreter, native and WASM; JS emits `let [x, ...mid, y] = ...` and node fails with a SyntaxError. Fix: lower a middle rest to slice() calls in src/compiler.rs; parity case.

Evidence fingerprint: `manual:v1:js-destructuring-with-a-rest-element-in-the-mid-8ffa714f6151d74e`


## Fix

<unfixed — raised only>

## Notes
