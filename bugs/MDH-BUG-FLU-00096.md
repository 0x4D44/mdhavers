# MDH-BUG-FLU-00096 — dae body implicit value: interpreter returns last expression, JS and WASM return naething

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-23T21:37:23Z
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
- **Resolution:** by-design
- **State history:** Open (2026-10-23T21:37:23Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Closed (2026-10-24T21:33:15Z, deltic:manual host=flux verdict=close resolution=by-design)

## Observation

Found by the semantics lane SEM-I 2026-10-23. dae f() { 42 } then blether f() prints 42 on the interpreter but naething on JS and WASM; docs/src/learn/03-functions.md says naething. The DEC doc row 64 covers block expressions only. Needs Arthur's decision: should a function body, like a block, yield its last expression?

Evidence fingerprint: `manual:v1:dae-body-implicit-value-interpreter-returns-las-fb76658fb802351f`


## Fix

<unfixed — raised only>

## Notes

### Close note (2026-10-24T21:33:15Z, deltic:manual host=flux verdict=close resolution=by-design)

Close reason: Arthur decided 2026-10-24 (DEC doc, round 2): a dae body keeps explicit gie; without gie it returns naething. Only blocks and lambdas yield their last expression.
