# MDH-BUG-FLU-00070 — Native: calling a lambda passed as a parameter fails with "got function"

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T02:06:57Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T035044Z-4ebc702c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00070-run-fix-20261023T035044Z-4ebc702c
- **Owner base:** fd08c6aa45e4fe7a0f40b77be36e5dd5ce1f8fbc
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:50:44Z
- **Owner until:** 2026-10-23T07:50:44Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the showcase lane: `dae c(op) { gie op(1) }` then `blether c(|a| a + 1)` -> "Type error in 'call': got function" natively; named functions passed as parameters work; the interpreter prints 2.

## Fix

<unfixed — raised only>

## Notes
