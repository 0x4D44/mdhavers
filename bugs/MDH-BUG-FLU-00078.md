# MDH-BUG-FLU-00078 — Match range patterns cannot be inclusive (..=)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:15:53Z, deltic:auto role=fix run=fix-20261023T031547Z-64f71d57 branch=task/bug-MDH-BUG-FLU-00078-run-fix-20261023T031547Z-64f71d57 code=e5fbb29 gate=manual)

## Observation

Found by the parser lane: Pattern::Range has no inclusive flag, so `whan 1..=5` is not expressible; needs an AST change.

## Fix

<unfixed — raised only>

## Notes
