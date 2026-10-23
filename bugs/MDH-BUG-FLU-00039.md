# MDH-BUG-FLU-00039 — JS compiler emits JS reserved words and runtime helper names verbatim

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T021647Z-ce08c85a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00039-run-fix-20261023T021647Z-ce08c85a
- **Owner base:** 53a16487c6b1353fd147afd9864bcf57e4efce4a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:16:47Z
- **Owner until:** 2026-10-23T06:16:47Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed under node: mdhavers identifiers new/class/var/function produce SyntaxError; top-level `dae len` or `ken max` collides with the destructured runtime const ("Identifier 'len' has already been declared"). compiler.rs:587, 597.

## Fix

<unfixed — raised only>

## Notes
