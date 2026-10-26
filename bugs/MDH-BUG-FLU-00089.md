# MDH-BUG-FLU-00089 — Native float modulo by zero raises a type error instead of divide-by-zero

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:21Z
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
- **State history:** Open (2026-10-23T04:31:21Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:43:21Z, deltic:auto role=fix run=fix-20261024T014315Z-26bc9e7e branch=task/bug-MDH-BUG-FLU-00089-run-fix-20261024T014315Z-26bc9e7e code=5cf1a3f gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-a685e43e verified_fix_run=fix-20261024T014315Z-26bc9e7e verdict=close)

## Observation

Found by the native hardening lane 2026-10-23: native `1.5 % 0` reports a type error; the interpreter checks the zero right operand before types and reports divide-by-zero.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-a685e43e verified_fix_run=fix-20261024T014315Z-26bc9e7e verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 67a0617 (recorded 5cf1a3f is version-only); modulo_by_integer_zero_divides_by_zero_whatever_the_left_type: 1 passed; removing runtime and LLVM int-zero checks failed output assertion (Type error vs divide-by-zero for 1.5 % 0).

## Notes
