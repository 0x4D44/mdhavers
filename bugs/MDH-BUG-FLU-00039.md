# MDH-BUG-FLU-00039 — JS compiler emits JS reserved words and runtime helper names verbatim

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:16:52Z, deltic:auto role=fix run=fix-20261023T021647Z-ce08c85a branch=task/bug-MDH-BUG-FLU-00039-run-fix-20261023T021647Z-ce08c85a code=3e24904 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-d370557b verified_fix_run=fix-20261023T021647Z-ce08c85a verdict=close)

## Observation

Confirmed under node: mdhavers identifiers new/class/var/function produce SyntaxError; top-level `dae len` or `ken max` collides with the destructured runtime const ("Identifier 'len' has already been declared"). compiler.rs:587, 597.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-d370557b verified_fix_run=fix-20261023T021647Z-ce08c85a verdict=close)

Verifier note: Manual verify: [o1b sol] 3e24904: parity_js_identifier_hygiene_mdh_bug_flu_00039 selected/pass; shared root-hunk mutant: reserved-name mangling disabled; parity assertion catches JS SyntaxError. CLI/Node repro prints 2/99/5; restored clean.

## Notes
