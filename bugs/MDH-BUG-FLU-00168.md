# MDH-BUG-FLU-00168 — JS: the pipe operator with a lambda returns the function instead of calling it

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:40Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-10444372
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00168-run-verify-20261026T225713Z-10444372
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:8525f30df3597756af72f0fc905be383da1ab04d3f59addd21703e0a80702b16
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:40Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:28Z, deltic:auto role=fix run=fix-20261026T050440Z-36dd2ff0 branch=task/bug-MDH-BUG-FLU-00168-run-fix-20261026T050440Z-36dd2ff0 code=b1616f5 gate=manual)

## Observation

Found by lane H1 2026-10-26 (executed): `ken d = 5 |> |x| x * 2` / `blether d` prints 10 in the interpreter, native and WASM, but `<dae d>` in JS. Reproduced in examples/features/pipe.braw. Guess: Pipe codegen in src/compiler.rs does not call a lambda right-hand side. Fix plus a js_golden_parity case.

Evidence fingerprint: `manual:v1:js-the-pipe-operator-with-a-lambda-returns-the--56f5a4579989454a`


## Fix

<unfixed — raised only>

## Notes
