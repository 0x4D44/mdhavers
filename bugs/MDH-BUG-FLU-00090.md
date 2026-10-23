# MDH-BUG-FLU-00090 — Native error line tracking is a single global shared by all threads

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:22Z
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
- **State history:** Open (2026-10-23T04:31:22Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the native hardening lane 2026-10-23: __mdh_current_line (runtime/mdh_runtime.c) is process-global, so an error raised in a spawned thread can report another thread's line. Also an error in a loop condition reports the last body statement's line. Should be thread-local.

## Fix

<unfixed — raised only>

## Notes
