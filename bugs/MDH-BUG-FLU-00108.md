# MDH-BUG-FLU-00108 — brak/haud inside a block expression: interpreter ignores them, JS emits an illegal break

- **State:** Fixed
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
- **State history:** Open (2026-10-24T02:38:53Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T21:56:40Z, deltic:auto role=fix run=fix-20261024T213341Z-2e4fee5b branch=task/bug-MDH-BUG-FLU-00108-run-fix-20261024T213341Z-2e4fee5b code=f207ce5 gate=manual)

## Observation

Found by the lead's review 2026-10-24. src/interpreter.rs eval_block_expr (~12574) discards Break/Continue; JS compile_block_expr puts break inside an arrow IIFE and node rejects the whole program ("Illegal break statement", executed). Repro: fer x in [1, 2, 3] { ken y = { gin x == 2 { brak }; x }  blether y }. Interpreter prints 1 2 3. Needs a language decision (reject in the parser, or propagate like gie per MDH-BUG-FLU-00094).

Evidence fingerprint: `manual:v1:brak-haud-inside-a-block-expression-interpreter-b75f3ef44f3cda10`


## Fix

<unfixed — raised only>

## Notes
