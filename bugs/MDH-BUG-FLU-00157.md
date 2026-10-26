# MDH-BUG-FLU-00157 — Native build of examples/stdlib/game_demo.braw times out after 120 seconds

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T23:04:55Z
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
- **State history:** Open (2026-10-25T23:04:55Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T01:31:35Z, deltic:auto role=fix run=fix-20261025T230506Z-6b2db5bc branch=task/bug-MDH-BUG-FLU-00157-run-fix-20261025T230506Z-6b2db5bc code=8b0ec87 gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-f95e041b verified_fix_run=fix-20261025T230506Z-6b2db5bc verdict=close)

## Observation

Found by lane S15 2026-10-25 during a native build sweep of examples/ (observed once under load ~90, not rerun): mdhavers build examples/stdlib/game_demo.braw did not finish within 120 s, while other examples build in seconds. Possible compile-time blow-up (prelude/stdlib fixpoint injection, a huge function, or an optimiser pass). Reproduce with a timing at -O0 and -O2 and --emit-llvm size; fix the cause.

Evidence fingerprint: `manual:v1:native-build-of-examples-stdlib-game-demo-braw--5dc229ae0518b37d`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-f95e041b verified_fix_run=fix-20261025T230506Z-6b2db5bc verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 8b0ec87; game_demo.braw native build completed in 105s; both dispatch tests pass. Reversing the shared-dispatch hunk made function_value_calls_do_not_blow_up_ir fail: IR 51,336 lines, limit 36,000.

## Notes
