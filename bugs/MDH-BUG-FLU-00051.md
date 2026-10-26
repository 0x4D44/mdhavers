# MDH-BUG-FLU-00051 — C runtime lists with capacity 0 never grow (heap overflow on push)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-0195dde1
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00051-run-verify-20261026T164142Z-0195dde1
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:b6786cf705f448a9cc0e4bfc960abcbb429069050b096f94ab97ab1bf6675bab
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:05:06Z, deltic:auto role=fix run=fix-20261023T030500Z-b3e801ea branch=task/bug-MDH-BUG-FLU-00051-run-fix-20261023T030500Z-b3e801ea code=8e071f4 gate=manual)

## Observation

Inferred: capacity *= 2 keeps 0 (mdh_runtime.c:1335-1338, 6528, 5704); an empty slice or copy([]) then shove writes past malloc(0).

## Fix

<unfixed — raised only>

## Notes
