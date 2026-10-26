# MDH-BUG-FLU-00094 — gie inside a block expression sets the block value instead of returning from the function

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-23T21:37:10Z
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
- **State history:** Open (2026-10-23T21:37:10Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T22:21:39Z, deltic:auto role=fix run=fix-20261023T222133Z-85f54b47 branch=task/bug-MDH-BUG-FLU-00094-run-fix-20261023T222133Z-85f54b47 code=8b1d2ff gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-c4d70db0 verified_fix_run=fix-20261023T222133Z-85f54b47 verdict=close)

## Observation

Found by the semantics lanes 2026-10-23. The DEC doc (2026.10.23 - DEC, row 64) says a block's value is its last expression and gie still returns from the enclosing function. Native (lane SEM-N) now does that. The interpreter and JS still catch gie inside a {...} block expression as the block's value, so dae f() { ken b = { gie 4 }; gie 9 } returns 9 on interpreter/JS and 4 natively. Also top-level ken b = { gie 4 } is accepted on interpreter/JS; under the DEC it should be ReturnOutsideFunction (F4).

Evidence fingerprint: `manual:v1:gie-inside-a-block-expression-sets-the-block-va-00b29cb4775c510f`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-c4d70db0 verified_fix_run=fix-20261023T222133Z-85f54b47 verdict=close)

Verifier note: Manual verify: [la luna] 883d221 (recorded 8b1d2ff is version-only); parity_js_gie_in_block_expression_returns_from_function_mdh_bug_flu_00094 and top_level_gie_is_return_outside_function_decision_f4 selected and passed. Repros: 4,51,3,5,-1,oot,42. Returning block gie as a value failed parity assertion (9 vs 4); parser mutant accepted top-level gie and failed. Restored clean.

## Notes
