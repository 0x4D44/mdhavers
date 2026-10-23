# MDH-BUG-FLU-00070 — Native: calling a lambda passed as a parameter fails with "got function"

- **State:** Open
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
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the showcase lane: `dae c(op) { gie op(1) }` then `blether c(|a| a + 1)` -> "Type error in 'call': got function" natively; named functions passed as parameters work; the interpreter prints 2.

## Fix

<unfixed — raised only>

## Notes
