# MDH-BUG-FLU-00027 — mdhavers fmt replaces gin/fer/whiles inside lambda block bodies with ...

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015755Z-12d8c291
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00027-run-fix-20261023T015755Z-12d8c291
- **Owner base:** f169c3ef9d1fcbb40509b50d016fe5f062445335
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:57:55Z
- **Owner until:** 2026-10-23T05:57:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: a lambda `|n| { gin ... }` is written as `...`. formatter.rs:600 uses format_stmt_single.

## Fix

<unfixed — raised only>

## Notes
