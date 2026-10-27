# MDH-BUG-FLU-00020 — Integer literal overflow reported as unknown token

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:51:19Z, deltic:auto role=fix run=fix-20261023T015114Z-220e94ff branch=task/bug-MDH-BUG-FLU-00020-run-fix-20261023T015114Z-220e94ff code=8204122 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-6f3a6fec verified_fix_run=fix-20261023T015114Z-220e94ff verdict=close)

## Observation

Confirmed: `ken big = 9223372036854775808` -> "Ah dinnae ken whit '9223372036854775808' is". token.rs:159 callback returns None; InvalidNumber exists.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-6f3a6fec verified_fix_run=fix-20261023T015114Z-220e94ff verdict=close)

Verifier note: Manual verify: [lc luna] 8204122: test_integer_literal_overflow_is_invalid_number passed; removing the overflow branch made it fail with `UnkentToken` for 9223372036854775808 at column 11. Source diff restored clean.

## Notes
