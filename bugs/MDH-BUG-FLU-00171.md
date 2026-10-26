# MDH-BUG-FLU-00171 — Interpreter: drap with a negative count returns wrong data, and tak(7, 0) returns []

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-26T05:03:54Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T050440Z-50f6bb31
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00171-run-fix-20261026T050440Z-50f6bb31
- **Owner base:** 0ce2a5bc5cf9ba91c13f5b84148dcd8cfea0ba99
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T05:04:40Z
- **Owner until:** 2026-10-26T09:04:40Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:54Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26 (executed; S8 noted the drap quirk on 2026-10-25): `drap([3, 1, 2], -1)` gives [2, 3, 1, 2] and `drap("Hello", -1)` gives [o, H, e, l, l, o] in the interpreter (native, JS and WASM give the input unchanged); `tak(7, 0)` returns [] in the interpreter while the other backends reject a non-list. Fix the interpreter (src/interpreter.rs tak/drap): clamp negative counts to 0 and raise the usual type error for a non-list, non-string; parity test across backends.

Evidence fingerprint: `manual:v1:interpreter-drap-with-a-negative-count-returns--e3c8275577752663`


## Fix

<unfixed — raised only>

## Notes
