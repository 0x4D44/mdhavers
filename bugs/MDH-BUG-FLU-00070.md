# MDH-BUG-FLU-00070 — Native: calling a lambda passed as a parameter fails with "got function"

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T02:06:57Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-8d3cf0cb
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00070-run-verify-20261026T171637Z-8d3cf0cb
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:e4650477897924b7ab9bb3ed51bb9db0fbda1a4e61c72f179238148eeaa5ef7d
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:50:50Z, deltic:auto role=fix run=fix-20261023T035044Z-4ebc702c branch=task/bug-MDH-BUG-FLU-00070-run-fix-20261023T035044Z-4ebc702c code=6029d44 gate=manual)

## Observation

Found by the showcase lane: `dae c(op) { gie op(1) }` then `blether c(|a| a + 1)` -> "Type error in 'call': got function" natively; named functions passed as parameters work; the interpreter prints 2.

## Fix

<unfixed — raised only>

## Notes
