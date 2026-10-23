# MDH-BUG-FLU-00026 — mdhavers fmt replaces else-if chains with `ither { ... }`

- **State:** Open
- **Priority:** Should
- **Severity:** Critical
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015745Z-f382a506
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00026-run-fix-20261023T015745Z-f382a506
- **Owner base:** a5d8ebb78259339a05136c4a4758bed523b11e3a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:57:45Z
- **Owner until:** 2026-10-23T05:57:45Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: output `} ither { ... }` written back to the file; formatter.rs:131 -> format_stmt_single returns "..." for Stmt::If (formatter.rs:368).

## Fix

<unfixed — raised only>

## Notes
