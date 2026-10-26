# MDH-BUG-FLU-00070 — Native: calling a lambda passed as a parameter fails with "got function"

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T02:06:57Z
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
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:50:50Z, deltic:auto role=fix run=fix-20261023T035044Z-4ebc702c branch=task/bug-MDH-BUG-FLU-00070-run-fix-20261023T035044Z-4ebc702c code=6029d44 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-8d3cf0cb verified_fix_run=fix-20261023T035044Z-4ebc702c verdict=close)

## Observation

Found by the showcase lane: `dae c(op) { gie op(1) }` then `blether c(|a| a + 1)` -> "Type error in 'call': got function" natively; named functions passed as parameters work; the interpreter prints 2.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-8d3cf0cb verified_fix_run=fix-20261023T035044Z-4ebc702c verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 6029d44; lambda_passed_as_parameter_can_be_called: 1 passed; old type-error fallback mutant produced Type error in call: got function, caught by run_ok exit assertion.

## Notes
