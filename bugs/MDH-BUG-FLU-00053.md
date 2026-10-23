# MDH-BUG-FLU-00053 — C runtime dict insert copies the whole dict and never frees (quadratic)

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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T04:30:47Z, deltic:auto role=fix run=fix-20261023T043041Z-aa3781ce branch=task/bug-MDH-BUG-FLU-00053-run-fix-20261023T043041Z-aa3781ce code=c9f3fd5 gate=manual)

## Observation

Inferred: mdh_runtime.c:3857-3875 with gc_stub (no free).

## Fix

<unfixed — raised only>

## Notes
