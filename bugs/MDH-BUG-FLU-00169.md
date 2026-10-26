# MDH-BUG-FLU-00169 — JS: destructuring with a rest element in the middle emits invalid JavaScript

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:45Z
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
- **State history:** Open (2026-10-26T05:03:45Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26 (executed): `ken [x, ...mid, y] = [10, 20, 30, 40, 50]` / `blether mid` prints [20, 30, 40] in the interpreter, native and WASM; JS emits `let [x, ...mid, y] = ...` and node fails with a SyntaxError. Fix: lower a middle rest to slice() calls in src/compiler.rs; parity case.

Evidence fingerprint: `manual:v1:js-destructuring-with-a-rest-element-in-the-mid-8ffa714f6151d74e`


## Fix

<unfixed — raised only>

## Notes
