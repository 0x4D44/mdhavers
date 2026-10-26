# MDH-BUG-FLU-00078 — Match range patterns cannot be inclusive (..=)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-17c64e03
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00078-run-verify-20261026T233955Z-17c64e03
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:90fe24fa7a95e23501e392365a6d1db8e2d3b74a9db3927e53bf76f35c6d38ad
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
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
