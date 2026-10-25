# MDH-BUG-FLU-00152 — Native thread_spawn leaks its function and argument list under MDH_RC=1

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T10:32:08Z
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
- **State history:** Open (2026-10-25T10:32:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R3 2026-10-25 (executed): under native refcounting (MDH_RC=1) every thread_spawn keeps t->func and t->args in its thread record (runtime/mdh_runtime.c __mdh_thread_spawn ~4046) and nothing releases them after join. MDH_RC_REPORT=1 on examples/features/threading_basics.braw reports list=11 live at exit; in a stress program the leaked argument lists keep the whole shared graph alive (261 objects). Fix: release func and args when the thread finishes or is joined (taking care that the thread function still needs them while running).

Evidence fingerprint: `manual:v1:native-thread-spawn-leaks-its-function-and-argu-f5dce939c8afc9ff`


## Fix

<unfixed — raised only>

## Notes
