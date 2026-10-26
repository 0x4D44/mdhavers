# MDH-BUG-FLU-00108 — brak/haud inside a block expression: interpreter ignores them, JS emits an illegal break

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-24T02:38:53Z
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
- **State history:** Open (2026-10-24T02:38:53Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T21:56:40Z, deltic:auto role=fix run=fix-20261024T213341Z-2e4fee5b branch=task/bug-MDH-BUG-FLU-00108-run-fix-20261024T213341Z-2e4fee5b code=f207ce5 gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-c98d2575 verified_fix_run=fix-20261024T213341Z-2e4fee5b verdict=close)

## Observation

Found by the lead's review 2026-10-24. src/interpreter.rs eval_block_expr (~12574) discards Break/Continue; JS compile_block_expr puts break inside an arrow IIFE and node rejects the whole program ("Illegal break statement", executed). Repro: fer x in [1, 2, 3] { ken y = { gin x == 2 { brak }; x }  blether y }. Interpreter prints 1 2 3. Needs a language decision (reject in the parser, or propagate like gie per MDH-BUG-FLU-00094).

Evidence fingerprint: `manual:v1:brak-haud-inside-a-block-expression-interpreter-b75f3ef44f3cda10`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-c98d2575 verified_fix_run=fix-20261024T213341Z-2e4fee5b verdict=close)

Verifier note: Manual verify: [la luna] f207ce5; brak_in_block_expression_is_rejected_before_any_backend selected 1 pass; parser::tests::test_brak_haud_in_block_expression_is_a_parse_error selected 1 pass. Repro now errors `brak` cannot jump out of a block expression. Removing both parser guards made the unit test fail: it parsed the recorded repro as Ok. Restored; source diff clean.

## Notes
