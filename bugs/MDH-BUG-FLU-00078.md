# MDH-BUG-FLU-00078 — Match range patterns cannot be inclusive (..=)

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T031547Z-64f71d57
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00078-run-fix-20261023T031547Z-64f71d57
- **Owner base:** 44f34f5272c39f61e2cd12e711ba3a07a91639af
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:15:47Z
- **Owner until:** 2026-10-23T07:15:47Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the parser lane: Pattern::Range has no inclusive flag, so `whan 1..=5` is not expressible; needs an AST change.

## Fix

<unfixed — raised only>

## Notes
