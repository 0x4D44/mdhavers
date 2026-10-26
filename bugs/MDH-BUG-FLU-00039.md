# MDH-BUG-FLU-00039 — JS compiler emits JS reserved words and runtime helper names verbatim

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-d370557b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00039-run-verify-20261026T214519Z-d370557b
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:f092e85c5e98e182ac3082921ecf363aef577e0193491b3b28e3cdb080fd8c03
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:16:52Z, deltic:auto role=fix run=fix-20261023T021647Z-ce08c85a branch=task/bug-MDH-BUG-FLU-00039-run-fix-20261023T021647Z-ce08c85a code=3e24904 gate=manual)

## Observation

Confirmed under node: mdhavers identifiers new/class/var/function produce SyntaxError; top-level `dae len` or `ken max` collides with the destructured runtime const ("Identifier 'len' has already been declared"). compiler.rs:587, 597.

## Fix

<unfixed — raised only>

## Notes
