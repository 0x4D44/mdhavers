# MDH-BUG-FLU-00073 — Native caught hurl text lacks the interpreter's "Hurled at line N:" prefix

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T02:06:58Z
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:51:13Z, deltic:auto role=fix run=fix-20261023T035107Z-6191175c branch=task/bug-MDH-BUG-FLU-00073-run-fix-20261023T035107Z-6191175c code=6029d44 gate=manual)

## Observation

Found by the showcase lane: hae_a_bash binds "boom" natively but "Hurled at line N: boom" in the interpreter.

## Fix

<unfixed — raised only>

## Notes
