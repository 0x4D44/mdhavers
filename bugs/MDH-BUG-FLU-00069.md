# MDH-BUG-FLU-00069 — Native: list reassigned inside a gin block keeps its old value when indexed

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T02:06:57Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-54109e07
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00069-run-verify-20261026T171637Z-54109e07
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:aafdf3ad23bbb3b6de449bcbf9c0d94ce4369e56dfa2548b1aa101b94daef01c
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:06:06Z, deltic:auto role=fix run=fix-20261023T030559Z-a63f4c39 branch=task/bug-MDH-BUG-FLU-00069-run-fix-20261023T030559Z-a63f4c39 code=8e071f4 gate=manual)

## Observation

Found by the showcase lane: `ken size = [96, 54]` then `gin size != naething { size = [640, 360] }` then `size[0]` gives 96 natively while printing size shows the new list (stale list_ptr shadow).

## Fix

<unfixed — raised only>

## Notes
