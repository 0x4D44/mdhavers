# MDH-BUG-FLU-00169 — JS: destructuring with a rest element in the middle emits invalid JavaScript

- **State:** Closed
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
- **State history:** Open (2026-10-26T05:03:45Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:34Z, deltic:auto role=fix run=fix-20261026T050440Z-414c5191 branch=task/bug-MDH-BUG-FLU-00169-run-fix-20261026T050440Z-414c5191 code=b1616f5 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-cca26030 verified_fix_run=fix-20261026T050440Z-414c5191 verdict=close)

## Observation

Found by lane H1 2026-10-26 (executed): `ken [x, ...mid, y] = [10, 20, 30, 40, 50]` / `blether mid` prints [20, 30, 40] in the interpreter, native and WASM; JS emits `let [x, ...mid, y] = ...` and node fails with a SyntaxError. Fix: lower a middle rest to slice() calls in src/compiler.rs; parity case.

Evidence fingerprint: `manual:v1:js-destructuring-with-a-rest-element-in-the-mid-8ffa714f6151d74e`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-cca26030 verified_fix_run=fix-20261026T050440Z-414c5191 verdict=close)

Verifier note: Manual verify: [lb luna] b1616f5; js_golden_parity passed 1/1 and middle-rest repro matched interpreter/JS (10, [20, 30, 40], 50); mutant disabling lowering failed the fixture with Node SyntaxError: Rest element must be last.

## Notes
