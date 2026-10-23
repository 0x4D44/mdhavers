# MDH-BUG-FLU-00040 — JS output fails for ken redeclaration, fer-variable assignment and repeated _ destructuring

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:17:03Z, deltic:auto role=fix run=fix-20261023T021658Z-14d6424a branch=task/bug-MDH-BUG-FLU-00040-run-fix-20261023T021658Z-14d6424a code=3e24904 gate=manual)

## Observation

Confirmed under node: duplicate `let x` SyntaxError (compiler.rs:597); `for (const v ...)` then assigning v TypeError (:661); `const [_, _]` duplicate binding (:851).

## Fix

<unfixed — raised only>

## Notes
