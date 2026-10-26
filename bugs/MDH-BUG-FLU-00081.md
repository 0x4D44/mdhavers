# MDH-BUG-FLU-00081 — Native: toss_in/heave_oot overwrite the storage of boxed variables

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T03:06:28Z
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
- **State history:** Open (2026-10-23T03:06:28Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:51:49Z, deltic:auto role=fix run=fix-20261023T035143Z-f07eb672 branch=task/bug-MDH-BUG-FLU-00081-run-fix-20261023T035143Z-f07eb672 code=6029d44 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-9de98a0a verified_fix_run=fix-20261023T035143Z-f07eb672 verdict=close)

## Observation

Found by the LLVM lane 2026-10-23 (code reading, src/llvm/codegen.rs ~17068): the list push/pop helpers store their result over the variable's storage slot without respecting boxing, so a boxed (captured or shadow-synced) list variable can be clobbered. Needs a native repro to confirm the user-visible symptom.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-9de98a0a verified_fix_run=fix-20261023T035143Z-f07eb672 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 6029d44; creel_updates_keep_a_captured_variable_boxed: 1 passed; mutant overwrote captured cell, run_ok assertion failed with Type error in toss_in: got bool.

## Notes
