# MDH-BUG-FLU-00015 — x + 1..5 parses as x + (1..5)

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:50:26Z, deltic:auto role=fix run=fix-20261023T015020Z-fef124c3 branch=task/bug-MDH-BUG-FLU-00015-run-fix-20261023T015020Z-fef124c3 code=8204122 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-74bec51c verified_fix_run=fix-20261023T015020Z-fef124c3 verdict=close)

## Observation

Confirmed: "Cannae add integer an' list". Range binds at primary level (parser.rs:1149,1358) instead of a binary level below additive.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-74bec51c verified_fix_run=fix-20261023T015020Z-fef124c3 verdict=close)

Verifier note: Manual verify: [lc luna] 8204122: range_binds_looser_than_addition passed; primary-level mutant failed expected AST match, yielding x + (1..5). Source diff restored clean.

## Notes
