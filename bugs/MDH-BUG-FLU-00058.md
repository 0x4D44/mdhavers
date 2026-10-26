# MDH-BUG-FLU-00058 — datetime golden test fails: duration helpers return microseconds where milliseconds are expected

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-22T23:24:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-24eb1df6
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00058-run-verify-20261026T233955Z-24eb1df6
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:ab267dc2ed152232969804629751aca7d6f643794f1f47fba71fa5c459d49592
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:49:56Z, deltic:auto role=fix run=fix-20261023T024950Z-5fe67c8a branch=task/bug-MDH-BUG-FLU-00058-run-fix-20261023T024950Z-5fe67c8a code=77f950a gate=manual)

## Observation

Confirmed: tests/golden/stdlib/datetime/datetime_functions.braw expects 60000/60/1, interpreter prints 60000000/60000/1000.

## Fix

<unfixed — raised only>

## Notes
