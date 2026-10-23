# MDH-BUG-FLU-00018 — gie before } or ; is rejected

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015052Z-20eda298
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00018-run-fix-20261023T015052Z-20eda298
- **Owner base:** 952df4a2d62920522e2eaf48b1477b2a116a3826
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:50:52Z
- **Owner until:** 2026-10-23T05:50:52Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: `dae f() { gie }` -> "Unexpected token: }". return_statement (parser.rs:364) treats only Newline/EOF as no-value.

## Fix

<unfixed — raised only>

## Notes
