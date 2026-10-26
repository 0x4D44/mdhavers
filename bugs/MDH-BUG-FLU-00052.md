# MDH-BUG-FLU-00052 — C runtime string repeat computes len*count unchecked (heap overflow)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-2a1c4dd0
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00052-run-verify-20261026T164142Z-2a1c4dd0
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:875402aca09349fc8ba5254c061537170a135c73c3fa5409502d7bf5ac5904a5
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:05:18Z, deltic:auto role=fix run=fix-20261023T030512Z-6e409407 branch=task/bug-MDH-BUG-FLU-00052-run-fix-20261023T030512Z-6e409407 code=8e071f4 gate=manual)

## Observation

Inferred: inline_repeat and mdh_runtime.c:333-337 wrap the size, then strcat-loop overflows; also O(n^2).

## Fix

<unfixed — raised only>

## Notes
