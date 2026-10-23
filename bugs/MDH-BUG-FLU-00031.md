# MDH-BUG-FLU-00031 — mdhavers fmt breaks f-strings ({{ }} escapes and inner quotes)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015838Z-1de6b86d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00031-run-fix-20261023T015838Z-1de6b86d
- **Owner base:** ce0c73f9f53d995a4d698ad48bf9485245283deb
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:58:38Z
- **Owner until:** 2026-10-23T05:58:38Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: f"{{braces}} {d['k']}" -> f"{braces} {d["k"]}" which no longer parses (formatter.rs:567).

## Fix

<unfixed — raised only>

## Notes
