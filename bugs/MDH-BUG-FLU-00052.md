# MDH-BUG-FLU-00052 — C runtime string repeat computes len*count unchecked (heap overflow)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T030512Z-6e409407
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00052-run-fix-20261023T030512Z-6e409407
- **Owner base:** 3c645a280158c81a129e9f0d9a20dcb78ff63f3d
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:05:12Z
- **Owner until:** 2026-10-23T07:05:12Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: inline_repeat and mdh_runtime.c:333-337 wrap the size, then strcat-loop overflows; also O(n^2).

## Fix

<unfixed — raised only>

## Notes
