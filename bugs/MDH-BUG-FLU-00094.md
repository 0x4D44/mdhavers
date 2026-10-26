# MDH-BUG-FLU-00094 — gie inside a block expression sets the block value instead of returning from the function

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-23T21:37:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-c4d70db0
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00094-run-verify-20261026T224201Z-c4d70db0
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:d5f2952ca6f2904b58cfeb9f5992016e1e52e44afd4459669472199754a36705
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T21:37:10Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T22:21:39Z, deltic:auto role=fix run=fix-20261023T222133Z-85f54b47 branch=task/bug-MDH-BUG-FLU-00094-run-fix-20261023T222133Z-85f54b47 code=8b1d2ff gate=manual)

## Observation

Found by the semantics lanes 2026-10-23. The DEC doc (2026.10.23 - DEC, row 64) says a block's value is its last expression and gie still returns from the enclosing function. Native (lane SEM-N) now does that. The interpreter and JS still catch gie inside a {...} block expression as the block's value, so dae f() { ken b = { gie 4 }; gie 9 } returns 9 on interpreter/JS and 4 natively. Also top-level ken b = { gie 4 } is accepted on interpreter/JS; under the DEC it should be ReturnOutsideFunction (F4).

Evidence fingerprint: `manual:v1:gie-inside-a-block-expression-sets-the-block-va-00b29cb4775c510f`


## Fix

<unfixed — raised only>

## Notes
