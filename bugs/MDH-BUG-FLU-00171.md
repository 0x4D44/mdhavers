# MDH-BUG-FLU-00171 — Interpreter: drap with a negative count returns wrong data, and tak(7, 0) returns []

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-26T05:03:54Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-632469da
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00171-run-verify-20261026T225713Z-632469da
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:ca272bd4447c3d96dcc22fa58e4c73889571612b79f5da3fe6ff1dbcbc79eaa6
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:54Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T06:25:06Z, deltic:auto role=fix run=fix-20261026T050440Z-50f6bb31 branch=task/bug-MDH-BUG-FLU-00171-run-fix-20261026T050440Z-50f6bb31 code=e8c4a76 gate=manual)

## Observation

Found by lane H1 2026-10-26 (executed; S8 noted the drap quirk on 2026-10-25): `drap([3, 1, 2], -1)` gives [2, 3, 1, 2] and `drap("Hello", -1)` gives [o, H, e, l, l, o] in the interpreter (native, JS and WASM give the input unchanged); `tak(7, 0)` returns [] in the interpreter while the other backends reject a non-list. Fix the interpreter (src/interpreter.rs tak/drap): clamp negative counts to 0 and raise the usual type error for a non-list, non-string; parity test across backends.

Evidence fingerprint: `manual:v1:interpreter-drap-with-a-negative-count-returns--e3c8275577752663`


## Fix

<unfixed — raised only>

## Notes
