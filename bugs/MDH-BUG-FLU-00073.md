# MDH-BUG-FLU-00073 — Native caught hurl text lacks the interpreter's "Hurled at line N:" prefix

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-c828cbb3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00073-run-verify-20261026T171637Z-c828cbb3
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:fbebce09cc5f07b6d2fb787ef42778eb3bdc250911dfd9cd7ba74515c7f590a3
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
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
