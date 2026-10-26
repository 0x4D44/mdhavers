# MDH-BUG-FLU-00149 — Native ordering comparisons accept a bool against an int; the interpreter raises

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T09:01:03Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-4812f126
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00149-run-verify-20261026T182021Z-4812f126
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:0474808af4e4b4b8afc1104f2ba6c4859ab5729fc8bb92db5b65d8f7cc247927
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T09:01:03Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T09:30:50Z, deltic:auto role=fix run=fix-20261025T090823Z-73018e88 branch=task/bug-MDH-BUG-FLU-00149-run-fix-20261025T090823Z-73018e88 code=eccfe2e gate=manual)

## Observation

Found by lane S11 2026-10-25 (executed): native >=, <= and < accept a bool compared with an integer, so atween(aye, 0, 2) returns aye natively, while the interpreter raises "Cannae compare bool wi' integer". Check every ordering operator (<, <=, >, >=) for bool/int, bool/float, string/int and naething/int mixes against the interpreter, and raise the same type error natively.

Evidence fingerprint: `manual:v1:native-ordering-comparisons-accept-a-bool-again-92518863e6a2c1b4`


## Fix

<unfixed — raised only>

## Notes
