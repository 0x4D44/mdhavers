# MDH-BUG-FLU-00014 — Negative range start parses as negated range: fer i in -1..2 fails

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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:50:15Z, deltic:auto role=fix run=fix-20261023T015010Z-5b0fb5a7 branch=task/bug-MDH-BUG-FLU-00014-run-fix-20261023T015010Z-5b0fb5a7 code=8204122 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-803f0b7e verified_fix_run=fix-20261023T015010Z-5b0fb5a7 verdict=close)

## Observation

Confirmed: "Cannae negate a list" type error. Ranges are attached inside primary (parser.rs:1358 maybe_range) so -1..2 is -(1..2). Breaks examples/showcases/game_of_life/game_of_life.braw:89.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-803f0b7e verified_fix_run=fix-20261023T015010Z-5b0fb5a7 verdict=close)

Verifier note: Manual verify: [lc luna] 8204122: negative_range_start_is_negated_number_not_negated_range passed; primary-level mutant failed expected AST match, yielding Unary(Negate, Range) for -1..2. Source diff restored clean.

## Notes
