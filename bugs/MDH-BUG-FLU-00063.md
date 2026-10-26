# MDH-BUG-FLU-00063 — Value Debug formatting recurses forever on cyclic containers

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:53:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-37cc430e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00063-run-verify-20261026T224201Z-37cc430e
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:72f5d1d5748e7d425c1261423d472a6bdc4c2e65013634d7a85776d088a8f919
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
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
