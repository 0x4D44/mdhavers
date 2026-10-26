# MDH-BUG-FLU-00040 — JS output fails for ken redeclaration, fer-variable assignment and repeated _ destructuring

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-222dcc10
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00040-run-verify-20261026T214519Z-222dcc10
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:ddbe048f7c9678c2fb5eebabef197522200c50f774c17600e1023de0c0445afa
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:17:03Z, deltic:auto role=fix run=fix-20261023T021658Z-14d6424a branch=task/bug-MDH-BUG-FLU-00040-run-fix-20261023T021658Z-14d6424a code=3e24904 gate=manual)

## Observation

Confirmed under node: duplicate `let x` SyntaxError (compiler.rs:597); `for (const v ...)` then assigning v TypeError (:661); `const [_, _]` duplicate binding (:851).

## Fix

<unfixed — raised only>

## Notes
