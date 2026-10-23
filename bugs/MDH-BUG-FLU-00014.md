# MDH-BUG-FLU-00014 — Negative range start parses as negated range: fer i in -1..2 fails

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015010Z-5b0fb5a7
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00014-run-fix-20261023T015010Z-5b0fb5a7
- **Owner base:** a8d2935e910c6181f9727f9331db5182370c20a5
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:50:10Z
- **Owner until:** 2026-10-23T05:50:10Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: "Cannae negate a list" type error. Ranges are attached inside primary (parser.rs:1358 maybe_range) so -1..2 is -(1..2). Breaks examples/showcases/game_of_life/game_of_life.braw:89.

## Fix

<unfixed — raised only>

## Notes
