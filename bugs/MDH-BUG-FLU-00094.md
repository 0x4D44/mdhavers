# MDH-BUG-FLU-00094 — gie inside a block expression sets the block value instead of returning from the function

- **State:** Open
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
- **State history:** Open (2026-10-23T21:37:10Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by the semantics lanes 2026-10-23. The DEC doc (2026.10.23 - DEC, row 64) says a block's value is its last expression and gie still returns from the enclosing function. Native (lane SEM-N) now does that. The interpreter and JS still catch gie inside a {...} block expression as the block's value, so dae f() { ken b = { gie 4 }; gie 9 } returns 9 on interpreter/JS and 4 natively. Also top-level ken b = { gie 4 } is accepted on interpreter/JS; under the DEC it should be ReturnOutsideFunction (F4).

Evidence fingerprint: `manual:v1:gie-inside-a-block-expression-sets-the-block-va-00b29cb4775c510f`


## Fix

<unfixed — raised only>

## Notes
