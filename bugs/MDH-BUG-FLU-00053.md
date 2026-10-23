# MDH-BUG-FLU-00053 — C runtime dict insert copies the whole dict and never frees (quadratic)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T043041Z-aa3781ce
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00053-run-fix-20261023T043041Z-aa3781ce
- **Owner base:** b836d4219abee9b49fbba035710edcc680d74e69
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T04:30:41Z
- **Owner until:** 2026-10-23T08:30:41Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: mdh_runtime.c:3857-3875 with gc_stub (no free).

## Fix

<unfixed — raised only>

## Notes
