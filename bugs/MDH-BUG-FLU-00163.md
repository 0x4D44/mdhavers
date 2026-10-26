# MDH-BUG-FLU-00163 — Native: ken inside a nested block overwrites the outer variable of the same name

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-26T05:03:17Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-e9071c14
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00163-run-verify-20261026T164142Z-e9071c14
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:d3e29067723859f61d0818e1f7ec40f17f589030c7ed4edb810a08777c761cec
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:17Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T06:49:56Z, deltic:auto role=fix run=fix-20261026T050440Z-9b9dd345 branch=task/bug-MDH-BUG-FLU-00163-run-fix-20261026T050440Z-9b9dd345 code=5c3046a gate=manual)

## Observation

Found by lane H1 2026-10-26, reproduced by the lead (trunk 1755fc8): `ken x = 10` then `{ ken x = 30 }` then `blether x` prints 10 in the interpreter, JS and WASM but 30 natively (also with MDH_RC=1). Same in a `gin aye { ken x = 30 }` block and a `fer i in 0..2 { ken x = 30 }` body; inside a function `ken x=1 { ken x=2 } gie x` returns 2 natively, 1 in the interpreter. `whiles` bodies are fine. Guess: block scoping in src/llvm/codegen.rs reuses the outer variable's slot for a shadowing `ken`. Fix: a fresh slot per block scope, restored at block exit; tests for bare block, gin, fer, nested functions and closures capturing the inner variable.

Evidence fingerprint: `manual:v1:native-ken-inside-a-nested-block-overwrites-the-6b0c04a9e2cdce78`


## Fix

<unfixed — raised only>

## Notes
