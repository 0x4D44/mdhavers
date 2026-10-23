# MDH-BUG-FLU-00030 — mdhavers fmt turns float literals into integers

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:32Z, deltic:auto role=fix run=fix-20261023T015827Z-85bf68c7 branch=task/bug-MDH-BUG-FLU-00030-run-fix-20261023T015827Z-85bf68c7 code=6e06c31 gate=manual)

## Observation

Confirmed: 1.0 is written as 1 (changes / semantics), 1e300 as a 301-digit integer that no longer lexes. ast.rs:410 f64 Display.

## Fix

<unfixed — raised only>

## Notes
