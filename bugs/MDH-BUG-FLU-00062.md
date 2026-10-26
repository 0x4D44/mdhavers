# MDH-BUG-FLU-00062 — C runtime __mdh_replace size computation is unchecked

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-22T23:53:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-9a531fd5
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00062-run-verify-20261026T171637Z-9a531fd5
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:3039903c64f33fd84d60aab6fa7d61a8b4ddbc051c145d672f1a6a21331dff42
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
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
