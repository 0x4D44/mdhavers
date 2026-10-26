# MDH-BUG-FLU-00152 — Native thread_spawn leaks its function and argument list under MDH_RC=1

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T10:32:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T194306Z-71d19e4e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00152-run-verify-20261026T194306Z-71d19e4e
- **Owner base:** 36750bc8afd262a99873992d919d9ed250946aef
- **Owner fingerprint:** sha256:c34420f859c339c13635960fe064eaa3f3a55808c257cc8973012c25d769049f
- **Owner since:** 2026-10-26T19:43:06Z
- **Owner until:** 2026-10-26T23:43:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T10:32:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T11:20:09Z, deltic:auto role=fix run=fix-20261025T104640Z-4d48f78c branch=task/bug-MDH-BUG-FLU-00152-run-fix-20261025T104640Z-4d48f78c code=2839ad8 gate=manual)

## Observation

Found by lane R3 2026-10-25 (executed): under native refcounting (MDH_RC=1) every thread_spawn keeps t->func and t->args in its thread record (runtime/mdh_runtime.c __mdh_thread_spawn ~4046) and nothing releases them after join. MDH_RC_REPORT=1 on examples/features/threading_basics.braw reports list=11 live at exit; in a stress program the leaked argument lists keep the whole shared graph alive (261 objects). Fix: release func and args when the thread finishes or is joined (taking care that the thread function still needs them while running).

Evidence fingerprint: `manual:v1:native-thread-spawn-leaks-its-function-and-argu-f5dce939c8afc9ff`


## Fix

<unfixed — raised only>

## Notes
