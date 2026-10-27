# MDH-BUG-FLU-00058 — datetime golden test fails: duration helpers return microseconds where milliseconds are expected

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-22T23:24:12Z
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
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:49:56Z, deltic:auto role=fix run=fix-20261023T024950Z-5fe67c8a branch=task/bug-MDH-BUG-FLU-00058-run-fix-20261023T024950Z-5fe67c8a code=77f950a gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-24eb1df6 verified_fix_run=fix-20261023T024950Z-5fe67c8a verdict=close)

## Observation

Confirmed: tests/golden/stdlib/datetime/datetime_functions.braw expects 60000/60/1, interpreter prints 60000000/60000/1000.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-24eb1df6 verified_fix_run=fix-20261023T024950Z-5fe67c8a verdict=close)

Verifier note: Manual verify: [lc luna] 77f950a: golden_tests_interpreter selected and passed (266 golden tests); reverting datetime input 1060→61000 failed Output mismatch (expected 60000/60/1, actual 60000000/60000/1000). Restored clean.

## Notes
