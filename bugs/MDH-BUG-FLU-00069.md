# MDH-BUG-FLU-00069 — Native: list reassigned inside a gin block keeps its old value when indexed

- **State:** Fixed
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
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:06:06Z, deltic:auto role=fix run=fix-20261023T030559Z-a63f4c39 branch=task/bug-MDH-BUG-FLU-00069-run-fix-20261023T030559Z-a63f4c39 code=8e071f4 gate=manual)

## Observation

Found by the showcase lane: `ken size = [96, 54]` then `gin size != naething { size = [640, 360] }` then `size[0]` gives 96 natively while printing size shows the new list (stale list_ptr shadow).

## Fix

<unfixed — raised only>

## Notes
