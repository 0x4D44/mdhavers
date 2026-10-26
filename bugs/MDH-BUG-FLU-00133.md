# MDH-BUG-FLU-00133 — Native -O1+: writes in a hae_a_bash body are lost when the body hurls

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-25T01:32:39Z
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
- **State history:** Open (2026-10-25T01:32:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T01:38:53Z, deltic:auto role=fix run=fix-20261025T013248Z-1d712276 branch=task/bug-MDH-BUG-FLU-00133-run-fix-20261025T013248Z-1d712276 code=015f5a4 gate=manual) -> Closed (2026-10-26T17:16:10Z, deltic:manual role=verify run=verify-20261026T164142Z-b96748cb verified_fix_run=fix-20261025T013248Z-1d712276 verdict=close)

## Observation

Found by lane R0a 2026-10-25 (executed): at -O1 and above, a local written inside a hae_a_bash body before a hurl (direct or from a called function) is seen with its pre-try value in the handler and after the try. Ints, strings, lists and untyped values are all affected. Cause: LLVM keeps the slots in registers across the setjmp/longjmp path. Probe at -O2: interpreter and -O0 print 11 b [2, 3] 5 7 2; -O2 printed 1 a [1] 1 1 2. Fixed on lane R0a by escape_slots_before in compile_try_catch (src/llvm/codegen.rs), which pins the function's slots in memory before registering the handler.

Evidence fingerprint: `manual:v1:native-o1-writes-in-a-hae-a-bash-body-are-lost--37d3bf9db033ccdd`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T17:16:10Z, deltic:manual run=verify-20261026T164142Z-b96748cb verified_fix_run=fix-20261025T013248Z-1d712276 verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] 015f5a4: try_body_writes_reach_the_handler_for_every_slot_kind passed at -O2; CLI/interpreter print 11,b,[2, 3],5,7. Removing escape_slots_before failed output assertion with original stale values 1,a,[1],1,1. Restored; diff empty.

## Notes
