# MDH-BUG-FLU-00078 — Match range patterns cannot be inclusive (..=)

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:15:53Z, deltic:auto role=fix run=fix-20261023T031547Z-64f71d57 branch=task/bug-MDH-BUG-FLU-00078-run-fix-20261023T031547Z-64f71d57 code=e5fbb29 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-17c64e03 verified_fix_run=fix-20261023T031547Z-64f71d57 verdict=close)

## Observation

Found by the parser lane: Pattern::Range has no inclusive flag, so `whan 1..=5` is not expressible; needs an AST change.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-17c64e03 verified_fix_run=fix-20261023T031547Z-64f71d57 verdict=close)

Verifier note: Manual verify: [lc luna] e5fbb29: inclusive_range_pattern_* passed; false-flag mutant made endpoint 3 fall through to `muckle` and formatter dropped `=`. LLVM-specific test build stalled >5m and was stopped; source diff restored clean.

## Notes
