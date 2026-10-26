# MDH-BUG-FLU-00011 — jammy() always returns the same value on macOS (time-seeded one-step LCG)

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:49:53Z, deltic:auto role=fix run=fix-20261023T014948Z-4aca2c4a branch=task/bug-MDH-BUG-FLU-00011-run-fix-20261023T014948Z-4aca2c4a code=d84823a gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-c79e960c verified_fix_run=fix-20261023T014948Z-4aca2c4a verdict=close)

## Observation

Confirmed on macOS: 20 calls of jammy(0, 8) all return 1. The LCG is seeded from wall-clock nanos (microsecond resolution on macOS) and copied 10 times across interpreter.rs (~5977).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-c79e960c verified_fix_run=fix-20261023T014948Z-4aca2c4a verdict=close)

Verifier note: Manual verify: [la luna] d84823a; jammy_gives_varied_values passed. Restored time-seeded LCG mutant selected the same value 200 times; assertion failed `only 1 distinct values from jammy(0, 8)`. Restored; source diff clean.

## Notes
