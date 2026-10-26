# MDH-BUG-FLU-00137 — Native binaries segfault on examples/scottish/scots_fun.braw and scots_words.braw

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** native
- **Raised:** 2026-10-25T06:33:34Z
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
- **State history:** Open (2026-10-25T06:33:34Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T06:54:54Z, deltic:auto role=fix run=fix-20261025T063408Z-d07563ed branch=task/bug-MDH-BUG-FLU-00137-run-fix-20261025T063408Z-d07563ed code=a262039 gate=manual) -> Closed (2026-10-26T17:16:10Z, deltic:manual role=verify run=verify-20261026T164142Z-4cd9129e verified_fix_run=fix-20261025T063408Z-d07563ed verdict=close)

## Observation

Found by lane R1 2026-10-25, reproduced by the lead on trunk 769e76d (debug compiler, default flags, MDH_RC unset): mdhavers build examples/scottish/scots_fun.braw -o x && ./x </dev/null exits 139 (SIGSEGV) with no output; same for examples/scottish/scots_words.braw. The interpreter runs both to completion (scots_fun prints 79 lines starting "=== Scots Vocabulary Functions ===", first section "haverin (is it nonsense?)"). No stdout reached the pipe, so the crash point is unknown (output may be buffered). Bisect the program to the first crashing builtin.

Evidence fingerprint: `manual:v1:native-binaries-segfault-on-examples-scottish-s-2132e2c005f8f620`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T17:16:10Z, deltic:manual run=verify-20261026T164142Z-4cd9129e verified_fix_run=fix-20261025T063408Z-d07563ed verdict=close)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] a262039: cannie_checks_validity_like_the_interpreter passed. Original scots_fun/scots_words native runs exit 0 with 79/48 lines, matching interpreter completion. Restoring cannie-to-inline-trim dispatch failed run_ok assertion None vs Some(0). Restored; diff empty.

## Notes
