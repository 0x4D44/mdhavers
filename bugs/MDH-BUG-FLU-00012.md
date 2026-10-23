# MDH-BUG-FLU-00012 — json_parse rejects integers beyond i64 instead of falling back to float

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T014959Z-c69570a1
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00012-run-fix-20261023T014959Z-c69570a1
- **Owner base:** 1d02e9461ef841f791f52d2ffd3a01ab18a4d41d
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:49:59Z
- **Owner until:** 2026-10-23T05:49:59Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: json_parse("12345678901234567890") -> "Invalid integer". interpreter.rs ~12727.

## Fix

<unfixed — raised only>

## Notes
