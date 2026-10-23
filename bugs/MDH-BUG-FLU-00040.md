# MDH-BUG-FLU-00040 — JS output fails for ken redeclaration, fer-variable assignment and repeated _ destructuring

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T021658Z-14d6424a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00040-run-fix-20261023T021658Z-14d6424a
- **Owner base:** fc8b0edc199da56ce46fb6a744d3f8fc4fb90f26
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:16:58Z
- **Owner until:** 2026-10-23T06:16:58Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed under node: duplicate `let x` SyntaxError (compiler.rs:597); `for (const v ...)` then assigning v TypeError (:661); `const [_, _]` duplicate binding (:851).

## Fix

<unfixed — raised only>

## Notes
