# MDH-BUG-FLU-00014 — Negative range start parses as negated range: fer i in -1..2 fails

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-803f0b7e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00014-run-verify-20261026T233955Z-803f0b7e
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:0ce78c77aa610e74654f1073a7ff321fd338f78462415a1c51d9825a8793e562
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:50:15Z, deltic:auto role=fix run=fix-20261023T015010Z-5b0fb5a7 branch=task/bug-MDH-BUG-FLU-00014-run-fix-20261023T015010Z-5b0fb5a7 code=8204122 gate=manual)

## Observation

Confirmed: "Cannae negate a list" type error. Ranges are attached inside primary (parser.rs:1358 maybe_range) so -1..2 is -(1..2). Breaks examples/showcases/game_of_life/game_of_life.braw:89.

## Fix

<unfixed — raised only>

## Notes
