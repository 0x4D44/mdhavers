# MDH-BUG-FLU-00015 — x + 1..5 parses as x + (1..5)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-74bec51c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00015-run-verify-20261026T233955Z-74bec51c
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:7859f246386d2f64235ec99d974cd237493ad8099a2087146734c515c524ec18
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:50:26Z, deltic:auto role=fix run=fix-20261023T015020Z-fef124c3 branch=task/bug-MDH-BUG-FLU-00015-run-fix-20261023T015020Z-fef124c3 code=8204122 gate=manual)

## Observation

Confirmed: "Cannae add integer an' list". Range binds at primary level (parser.rs:1149,1358) instead of a binary level below additive.

## Fix

<unfixed — raised only>

## Notes
