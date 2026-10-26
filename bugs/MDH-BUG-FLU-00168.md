# MDH-BUG-FLU-00168 — JS: the pipe operator with a lambda returns the function instead of calling it

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** js
- **Raised:** 2026-10-26T05:03:40Z
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
- **State history:** Open (2026-10-26T05:03:40Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26 (executed): `ken d = 5 |> |x| x * 2` / `blether d` prints 10 in the interpreter, native and WASM, but `<dae d>` in JS. Reproduced in examples/features/pipe.braw. Guess: Pipe codegen in src/compiler.rs does not call a lambda right-hand side. Fix plus a js_golden_parity case.

Evidence fingerprint: `manual:v1:js-the-pipe-operator-with-a-lambda-returns-the--56f5a4579989454a`


## Fix

<unfixed — raised only>

## Notes
