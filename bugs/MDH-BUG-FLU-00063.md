# MDH-BUG-FLU-00063 — Value Debug formatting recurses forever on cyclic containers

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
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
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:23:04Z, deltic:auto role=fix run=fix-20261023T032258Z-470c8642 branch=task/bug-MDH-BUG-FLU-00063-run-fix-20261023T032258Z-470c8642 code=a920344 gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-37cc430e verified_fix_run=fix-20261023T032258Z-470c8642 verdict=close)

## Observation

Inferred by the L2 worker: Value derives Debug, so {:?} on a self-referential list/dict recurses without limit (stack overflow). Display is being made cycle-safe; Debug is not, and is reachable from error paths that use {:?}.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-37cc430e verified_fix_run=fix-20261023T032258Z-470c8642 verdict=close)

Verifier note: Manual verify: LEAD: close - mutant reproduces the recorded stack overflow (SIGABRT); the crash is the bug's symptom. Verifier: [la luna] a920344; 4 interpreter_robustness Debug tests passed baseline. Removing DebugEnter's repeated-container check made debug_of_a_self_referential_list_terminates abort with stack overflow (SIGABRT) before an assertion; fails-before evidence does not meet the assertion requirement.

## Notes
