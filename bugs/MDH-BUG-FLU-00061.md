# MDH-BUG-FLU-00061 — LLVM toss_in/heave_oot store their result into a boxed variable's raw storage

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
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
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:52:01Z, deltic:auto role=fix run=fix-20261023T035155Z-040c74c9 branch=task/bug-MDH-BUG-FLU-00061-run-fix-20261023T035155Z-040c74c9 code=6029d44 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-bc2a88ab verified_fix_run=fix-20261023T035155Z-040c74c9 verdict=close)

## Observation

Inferred by the L6 worker (code reading, 2026-10-23): codegen.rs ~17068 stores the toss_in/heave_oot result straight into the variable's alloca even when the variable is boxed (captured by a closure), so the closure and the outer scope diverge or the box pointer is overwritten.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-bc2a88ab verified_fix_run=fix-20261023T035155Z-040c74c9 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 6029d44; creel_updates_keep_a_captured_variable_boxed: 1 passed; mutant overwrote captured cell, run_ok assertion failed with Type error in toss_in: got bool.

## Notes
