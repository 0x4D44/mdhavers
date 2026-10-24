# MDH-BUG-FLU-00098 — JS: gie in a block expression inside a default parameter escapes uncaught

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-23T22:22:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T013820Z-57948697
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00098-run-fix-20261024T013820Z-57948697
- **Owner base:** 9a5187774f637684a41071a0c13ab36adca48355
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T01:38:20Z
- **Owner until:** 2026-10-24T05:38:20Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T22:22:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane G94 (MDH-BUG-FLU-00094 fix, 8b1d2ff) 2026-10-23. A block-expression gie in a dae default parameter value throws the per-call sentinel before the function body's try/catch is entered, so it escapes as an uncaught exception. The interpreter returns from the called function.

Evidence fingerprint: `manual:v1:js-gie-in-a-block-expression-inside-a-default-p-9af8d25bd734db08`


## Fix

<unfixed — raised only>

## Notes
