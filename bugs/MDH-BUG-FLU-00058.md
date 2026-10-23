# MDH-BUG-FLU-00058 — datetime golden test fails: duration helpers return microseconds where milliseconds are expected

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-22T23:24:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T024950Z-5fe67c8a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00058-run-fix-20261023T024950Z-5fe67c8a
- **Owner base:** b43531d175e0ecfd50979b968fdf471b2740c19c
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:49:50Z
- **Owner until:** 2026-10-23T06:49:50Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: tests/golden/stdlib/datetime/datetime_functions.braw expects 60000/60/1, interpreter prints 60000000/60000/1000.

## Fix

<unfixed — raised only>

## Notes
