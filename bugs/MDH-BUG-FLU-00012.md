# MDH-BUG-FLU-00012 — json_parse rejects integers beyond i64 instead of falling back to float

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:50:04Z, deltic:auto role=fix run=fix-20261023T014959Z-c69570a1 branch=task/bug-MDH-BUG-FLU-00012-run-fix-20261023T014959Z-c69570a1 code=d84823a gate=manual)

## Observation

Confirmed: json_parse("12345678901234567890") -> "Invalid integer". interpreter.rs ~12727.

## Fix

<unfixed — raised only>

## Notes
