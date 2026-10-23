# MDH-BUG-FLU-00062 — C runtime __mdh_replace size computation is unchecked

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-22T23:53:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T035219Z-e43a21b4
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00062-run-fix-20261023T035219Z-e43a21b4
- **Owner base:** 79f22a1e301244240be26e42fbb604e959a8e013
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:52:19Z
- **Owner until:** 2026-10-23T07:52:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred by the L6 worker: mdh_runtime.c ~5880/5920 computes count*new_len without an overflow check; only very large inputs could wrap, but the result is a heap overflow.

## Fix

<unfixed — raised only>

## Notes
