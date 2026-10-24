# MDH-BUG-FLU-00108 — brak/haud inside a block expression: interpreter ignores them, JS emits an illegal break

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-24T02:38:53Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T213341Z-2e4fee5b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00108-run-fix-20261024T213341Z-2e4fee5b
- **Owner base:** aade7cd397c51481ad75e9804b8f1968562ab322
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T21:33:41Z
- **Owner until:** 2026-10-25T01:33:41Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:38:53Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by the lead's review 2026-10-24. src/interpreter.rs eval_block_expr (~12574) discards Break/Continue; JS compile_block_expr puts break inside an arrow IIFE and node rejects the whole program ("Illegal break statement", executed). Repro: fer x in [1, 2, 3] { ken y = { gin x == 2 { brak }; x }  blether y }. Interpreter prints 1 2 3. Needs a language decision (reject in the parser, or propagate like gie per MDH-BUG-FLU-00094).

Evidence fingerprint: `manual:v1:brak-haud-inside-a-block-expression-interpreter-b75f3ef44f3cda10`


## Fix

<unfixed — raised only>

## Notes
