# MDH-BUG-FLU-00027 — mdhavers fmt replaces gin/fer/whiles inside lambda block bodies with ...

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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:01Z, deltic:auto role=fix run=fix-20261023T015755Z-12d8c291 branch=task/bug-MDH-BUG-FLU-00027-run-fix-20261023T015755Z-12d8c291 code=6e06c31 gate=manual)

## Observation

Confirmed: a lambda `|n| { gin ... }` is written as `...`. formatter.rs:600 uses format_stmt_single.

## Fix

<unfixed — raised only>

## Notes
