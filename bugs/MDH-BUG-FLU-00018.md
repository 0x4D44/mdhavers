# MDH-BUG-FLU-00018 — gie before } or ; is rejected

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:50:58Z, deltic:auto role=fix run=fix-20261023T015052Z-20eda298 branch=task/bug-MDH-BUG-FLU-00018-run-fix-20261023T015052Z-20eda298 code=8204122 gate=manual)

## Observation

Confirmed: `dae f() { gie }` -> "Unexpected token: }". return_statement (parser.rs:364) treats only Newline/EOF as no-value.

## Fix

<unfixed — raised only>

## Notes
