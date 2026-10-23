# MDH-BUG-FLU-00051 — C runtime lists with capacity 0 never grow (heap overflow on push)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T030500Z-b3e801ea
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00051-run-fix-20261023T030500Z-b3e801ea
- **Owner base:** 8fc39dcb7c43255e6466a960d2b5045faa50ddc6
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:05:00Z
- **Owner until:** 2026-10-23T07:05:00Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: capacity *= 2 keeps 0 (mdh_runtime.c:1335-1338, 6528, 5704); an empty slice or copy([]) then shove writes past malloc(0).

## Fix

<unfixed — raised only>

## Notes
