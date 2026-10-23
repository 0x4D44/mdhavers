# MDH-BUG-FLU-00031 — mdhavers fmt breaks f-strings ({{ }} escapes and inner quotes)

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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:43Z, deltic:auto role=fix run=fix-20261023T015838Z-1de6b86d branch=task/bug-MDH-BUG-FLU-00031-run-fix-20261023T015838Z-1de6b86d code=6e06c31 gate=manual)

## Observation

Confirmed: f"{{braces}} {d['k']}" -> f"{braces} {d["k"]}" which no longer parses (formatter.rs:567).

## Fix

<unfixed — raised only>

## Notes
