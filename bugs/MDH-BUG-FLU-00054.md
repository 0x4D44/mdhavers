# MDH-BUG-FLU-00054 — Native handles are raw pointers: forged mutex/thread handles segfault

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T030524Z-2c5a5401
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00054-run-fix-20261023T030524Z-2c5a5401
- **Owner base:** 5c8feaaf8b0cbce207c01abbcf5419f4b926bd28
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:05:24Z
- **Owner until:** 2026-10-23T07:05:24Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: mdh_runtime.c:3228-3260; interpreter raises a catchable "Unknown mutex handle".

## Fix

<unfixed — raised only>

## Notes
