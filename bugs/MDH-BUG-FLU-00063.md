# MDH-BUG-FLU-00063 — Value Debug formatting recurses forever on cyclic containers

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:53:42Z
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
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:23:04Z, deltic:auto role=fix run=fix-20261023T032258Z-470c8642 branch=task/bug-MDH-BUG-FLU-00063-run-fix-20261023T032258Z-470c8642 code=a920344 gate=manual)

## Observation

Inferred by the L2 worker: Value derives Debug, so {:?} on a self-referential list/dict recurses without limit (stack overflow). Display is being made cycle-safe; Debug is not, and is reachable from error paths that use {:?}.

## Fix

<unfixed — raised only>

## Notes
