# MDH-BUG-FLU-00054 — Native handles are raw pointers: forged mutex/thread handles segfault

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:05:30Z, deltic:auto role=fix run=fix-20261023T030524Z-2c5a5401 branch=task/bug-MDH-BUG-FLU-00054-run-fix-20261023T030524Z-2c5a5401 code=8e071f4 gate=manual)

## Observation

Inferred: mdh_runtime.c:3228-3260; interpreter raises a catchable "Unknown mutex handle".

## Fix

<unfixed — raised only>

## Notes
