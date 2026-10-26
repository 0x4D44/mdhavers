# MDH-BUG-FLU-00152 — Native thread_spawn leaks its function and argument list under MDH_RC=1

- **State:** Closed
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
- **State history:** Open (2026-10-25T10:32:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T11:20:09Z, deltic:auto role=fix run=fix-20261025T104640Z-4d48f78c branch=task/bug-MDH-BUG-FLU-00152-run-fix-20261025T104640Z-4d48f78c code=2839ad8 gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-71d19e4e verified_fix_run=fix-20261025T104640Z-4d48f78c verdict=close)

## Observation

Found by lane R3 2026-10-25 (executed): under native refcounting (MDH_RC=1) every thread_spawn keeps t->func and t->args in its thread record (runtime/mdh_runtime.c __mdh_thread_spawn ~4046) and nothing releases them after join. MDH_RC_REPORT=1 on examples/features/threading_basics.braw reports list=11 live at exit; in a stress program the leaked argument lists keep the whole shared graph alive (261 objects). Fix: release func and args when the thread finishes or is joined (taking care that the thread function still needs them while running).

Evidence fingerprint: `manual:v1:native-thread-spawn-leaks-its-function-and-argu-f5dce939c8afc9ff`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-71d19e4e verified_fix_run=fix-20261025T104640Z-4d48f78c verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 2839ad8; thread_spawn_releases_its_function_and_arguments selected/pass on trunk; removing the worker func/args release made it fail at tests/llvm_refcount.rs:647: live list=10, expected 0.

## Notes
