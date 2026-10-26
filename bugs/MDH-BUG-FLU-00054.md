# MDH-BUG-FLU-00054 — Native handles are raw pointers: forged mutex/thread handles segfault

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-578c0d6b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00054-run-verify-20261026T171637Z-578c0d6b
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:b441712cd4e53b44922988683a13c01ca9f9f61ad90b60cfa85c038a5599cab3
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
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
