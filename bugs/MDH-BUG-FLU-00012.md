# MDH-BUG-FLU-00012 — json_parse rejects integers beyond i64 instead of falling back to float

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-a0b57932
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00012-run-verify-20261026T224201Z-a0b57932
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:79b44063a9662323902ef47447a9211aad925ffdfce2e60ce14154c2a766d32b
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
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
