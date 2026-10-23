# MDH-BUG-FLU-00063 — Value Debug formatting recurses forever on cyclic containers

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:53:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T032258Z-470c8642
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00063-run-fix-20261023T032258Z-470c8642
- **Owner base:** 16e96c7f704e4f54b801898d20837c0c37b1283c
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:22:58Z
- **Owner until:** 2026-10-23T07:22:58Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred by the L2 worker: Value derives Debug, so {:?} on a self-referential list/dict recurses without limit (stack overflow). Display is being made cycle-safe; Debug is not, and is reachable from error paths that use {:?}.

## Fix

<unfixed — raised only>

## Notes
