# MDH-BUG-FLU-00163 — Native: ken inside a nested block overwrites the outer variable of the same name

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-26T05:03:17Z
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
- **State history:** Open (2026-10-26T05:03:17Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T06:49:56Z, deltic:auto role=fix run=fix-20261026T050440Z-9b9dd345 branch=task/bug-MDH-BUG-FLU-00163-run-fix-20261026T050440Z-9b9dd345 code=5c3046a gate=manual) -> Closed (2026-10-26T17:16:10Z, deltic:manual role=verify run=verify-20261026T164142Z-e9071c14 verified_fix_run=fix-20261026T050440Z-9b9dd345 verdict=close)

## Observation

Found by lane H1 2026-10-26, reproduced by the lead (trunk 1755fc8): `ken x = 10` then `{ ken x = 30 }` then `blether x` prints 10 in the interpreter, JS and WASM but 30 natively (also with MDH_RC=1). Same in a `gin aye { ken x = 30 }` block and a `fer i in 0..2 { ken x = 30 }` body; inside a function `ken x=1 { ken x=2 } gie x` returns 2 natively, 1 in the interpreter. `whiles` bodies are fine. Guess: block scoping in src/llvm/codegen.rs reuses the outer variable's slot for a shadowing `ken`. Fix: a fresh slot per block scope, restored at block exit; tests for bare block, gin, fer, nested functions and closures capturing the inner variable.

Evidence fingerprint: `manual:v1:native-ken-inside-a-nested-block-overwrites-the-6b0c04a9e2cdce78`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T17:16:10Z, deltic:manual run=verify-20261026T164142Z-e9071c14 verified_fix_run=fix-20261026T050440Z-9b9dd345 verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] 5c3046a: top_level_blocks_do_not_overwrite_outer_variables passed in plain/RC modes; CLI preserves 10,10,10 and function 1. Disabling fresh shadow-binding slots failed native/interpreter parity: outer 30 vs 10 (plus gin/fer cases). Restored; diff empty.

## Notes
