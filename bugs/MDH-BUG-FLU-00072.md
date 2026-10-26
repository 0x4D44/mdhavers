# MDH-BUG-FLU-00072 — Native: functions cannot reference globals defined later in the file

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-1648cdcb
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00072-run-verify-20261026T171637Z-1648cdcb
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:e83367e57bf7174da4c8f4987012d6dbd7992801236a41a557f447d18f8548a6
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T22:02:02Z, deltic:auto role=fix run=fix-20261023T220155Z-449b24f2 branch=task/bug-MDH-BUG-FLU-00072-run-fix-20261023T220155Z-449b24f2 code=5404d6a gate=manual)

## Observation

Found by the showcase lane: a function using a global declared after it fails natively with "Undefined variable: X"; the interpreter resolves it at call time. May be a deliberate restriction; needs a decision and documentation either way.

## Fix

<unfixed — raised only>

## Notes
