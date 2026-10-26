# MDH-BUG-FLU-00142 — Native: prelude higher-order functions ilka and compose misbehave in prelude_demo.braw

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T07:07:21Z
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
- **State history:** Open (2026-10-25T07:07:21Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T07:37:08Z, deltic:auto role=fix run=fix-20261025T071342Z-9a6b3273 branch=task/bug-MDH-BUG-FLU-00142-run-fix-20261025T071342Z-9a6b3273 code=849ef76 gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-d16332eb verified_fix_run=fix-20261025T071342Z-9a6b3273 verdict=close)

## Observation

Found by lane S8 2026-10-25 (executed): in examples/builtins/prelude_demo.braw native ilka(nums, big) prints naething where the interpreter prints nae, and compose(add_one, double) returns something uncallable ("Type error in 'call': got naething") where the interpreter prints 11; output stops matching there, so apply_n, same, constant and nae_that are unchecked. Repro saved by S8 as .scratch/c1.braw in its lane. Likely the native backend lacks or miscompiles these stdlib/prelude.braw functions.

Evidence fingerprint: `manual:v1:native-prelude-higher-order-functions-ilka-and--f028fdfe864f6a82`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-d16332eb verified_fix_run=fix-20261025T071342Z-9a6b3273 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=849ef76; prelude_demo outputs ilka=nae and compose=11 in interpreter/native; 5 prelude_ tests selected and passed; mutant bypassing native_prelude_statements failed 4/5 assertions, primary test saw naething then Type error in call: got naething.

## Notes
