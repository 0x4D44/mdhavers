# MDH-BUG-FLU-00021 — F-string holes silently drop trailing tokens and report errors at line 1

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:51:30Z, deltic:auto role=fix run=fix-20261023T015124Z-d729e4e3 branch=task/bug-MDH-BUG-FLU-00021-run-fix-20261023T015124Z-d729e4e3 code=8204122 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-eb2e2097 verified_fix_run=fix-20261023T015124Z-d729e4e3 verdict=close)

## Observation

Confirmed: f"{a b}" prints a; f"{x:>5}" ignores the spec (parser.rs:1591 never checks the sub-parser reached EOF). An error inside a hole on line 4 is reported at line 1 (re-lexed from scratch, parser.rs:1589).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-eb2e2097 verified_fix_run=fix-20261023T015124Z-d729e4e3 verdict=close)

Verifier note: Manual verify: [lc luna] 8204122: fstring_hole_* unit tests passed; removing leftover-token check failed `parse(...).is_err()` for `{a b}`, and line-shift mutant changed expected line 3 to 1. Source diff restored clean.

## Notes
