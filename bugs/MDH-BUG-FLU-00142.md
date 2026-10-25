# MDH-BUG-FLU-00142 — Native: prelude higher-order functions ilka and compose misbehave in prelude_demo.braw

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T07:07:21Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T071342Z-9a6b3273
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00142-run-fix-20261025T071342Z-9a6b3273
- **Owner base:** 0a3a176611307838e3d389087d2b9829b13ce4d1
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T07:13:42Z
- **Owner until:** 2026-10-25T11:13:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T07:07:21Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S8 2026-10-25 (executed): in examples/builtins/prelude_demo.braw native ilka(nums, big) prints naething where the interpreter prints nae, and compose(add_one, double) returns something uncallable ("Type error in 'call': got naething") where the interpreter prints 11; output stops matching there, so apply_n, same, constant and nae_that are unchecked. Repro saved by S8 as .scratch/c1.braw in its lane. Likely the native backend lacks or miscompiles these stdlib/prelude.braw functions.

Evidence fingerprint: `manual:v1:native-prelude-higher-order-functions-ilka-and--f028fdfe864f6a82`


## Fix

<unfixed — raised only>

## Notes
