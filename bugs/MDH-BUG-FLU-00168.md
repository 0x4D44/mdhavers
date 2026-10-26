# MDH-BUG-FLU-00168 — JS: the pipe operator with a lambda returns the function instead of calling it

- **State:** Closed
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
- **State history:** Open (2026-10-26T05:03:40Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:28Z, deltic:auto role=fix run=fix-20261026T050440Z-36dd2ff0 branch=task/bug-MDH-BUG-FLU-00168-run-fix-20261026T050440Z-36dd2ff0 code=b1616f5 gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-10444372 verified_fix_run=fix-20261026T050440Z-36dd2ff0 verdict=close)

## Observation

Found by lane H1 2026-10-26 (executed): `ken d = 5 |> |x| x * 2` / `blether d` prints 10 in the interpreter, native and WASM, but `<dae d>` in JS. Reproduced in examples/features/pipe.braw. Guess: Pipe codegen in src/compiler.rs does not call a lambda right-hand side. Fix plus a js_golden_parity case.

Evidence fingerprint: `manual:v1:js-the-pipe-operator-with-a-lambda-returns-the--56f5a4579989454a`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-10444372 verified_fix_run=fix-20261026T050440Z-36dd2ff0 verdict=close)

Verifier note: Manual verify: [lb luna] b1616f5; js_golden_parity passed 1/1 and pipe_lambda repro printed 10 in interpreter/JS; mutant removing parenthesization failed golden output assertion for tests/golden/functions/pipe_lambda.braw.

## Notes
