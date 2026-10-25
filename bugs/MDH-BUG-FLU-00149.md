# MDH-BUG-FLU-00149 — Native ordering comparisons accept a bool against an int; the interpreter raises

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T09:01:03Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T090823Z-73018e88
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00149-run-fix-20261025T090823Z-73018e88
- **Owner base:** 479cd4bc38160c71ecc5cf8d225a17b0fcc4126e
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T09:08:23Z
- **Owner until:** 2026-10-25T13:08:23Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T09:01:03Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S11 2026-10-25 (executed): native >=, <= and < accept a bool compared with an integer, so atween(aye, 0, 2) returns aye natively, while the interpreter raises "Cannae compare bool wi' integer". Check every ordering operator (<, <=, >, >=) for bool/int, bool/float, string/int and naething/int mixes against the interpreter, and raise the same type error natively.

Evidence fingerprint: `manual:v1:native-ordering-comparisons-accept-a-bool-again-92518863e6a2c1b4`


## Fix

<unfixed — raised only>

## Notes
