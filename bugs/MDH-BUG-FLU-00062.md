# MDH-BUG-FLU-00062 — C runtime __mdh_replace size computation is unchecked

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-22T23:53:42Z
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
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:52:26Z, deltic:auto role=fix run=fix-20261023T035219Z-e43a21b4 branch=task/bug-MDH-BUG-FLU-00062-run-fix-20261023T035219Z-e43a21b4 code=6029d44 gate=manual)

## Observation

Inferred by the L6 worker: mdh_runtime.c ~5880/5920 computes count*new_len without an overflow check; only very large inputs could wrap, but the result is a heap overflow.

## Fix

<unfixed — raised only>

## Notes
