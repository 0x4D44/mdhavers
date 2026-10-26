# MDH-BUG-FLU-00090 — Native error line tracking is a single global shared by all threads

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-d741d40b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00090-run-verify-20261026T171637Z-d741d40b
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:b779763135c0411e3d19601f0fb34fd016410dff553f52cafe84f913b7cbbf57
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T04:31:22Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T22:24:45Z, deltic:auto role=fix run=fix-20261024T213341Z-ea604bf0 branch=task/bug-MDH-BUG-FLU-00090-run-fix-20261024T213341Z-ea604bf0 code=3f5bb72 gate=manual)

## Observation

Found by the native hardening lane 2026-10-23: __mdh_current_line (runtime/mdh_runtime.c) is process-global, so an error raised in a spawned thread can report another thread's line. Also an error in a loop condition reports the last body statement's line. Should be thread-local.

## Fix

<unfixed — raised only>

## Notes
