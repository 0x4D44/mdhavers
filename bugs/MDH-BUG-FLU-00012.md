# MDH-BUG-FLU-00012 — json_parse rejects integers beyond i64 instead of falling back to float

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:50:04Z, deltic:auto role=fix run=fix-20261023T014959Z-c69570a1 branch=task/bug-MDH-BUG-FLU-00012-run-fix-20261023T014959Z-c69570a1 code=d84823a gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-a0b57932 verified_fix_run=fix-20261023T014959Z-c69570a1 verdict=close)

## Observation

Confirmed: json_parse("12345678901234567890") -> "Invalid integer". interpreter.rs ~12727.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-a0b57932 verified_fix_run=fix-20261023T014959Z-c69570a1 verdict=close)

Verifier note: Manual verify: [la luna] d84823a; json_parse_big_integer_falls_back_to_float passed. Removing the integer-to-float fallback made the valid input error `Invalid integer: 12345678901234567890`, failing the regression test. Restored; source diff clean.

## Notes
