# MDH-BUG-FLU-00015 — x + 1..5 parses as x + (1..5)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-22T23:24:07Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015020Z-fef124c3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00015-run-fix-20261023T015020Z-fef124c3
- **Owner base:** 7aea2a79a07290b4ec3dfb3ab9d15b0a7fba32b4
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:50:20Z
- **Owner until:** 2026-10-23T05:50:20Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: "Cannae add integer an' list". Range binds at primary level (parser.rs:1149,1358) instead of a binary level below additive.

## Fix

<unfixed — raised only>

## Notes
