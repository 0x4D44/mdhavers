# MDH-BUG-FLU-00142 — Native: prelude higher-order functions ilka and compose misbehave in prelude_demo.braw

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T07:07:21Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-d16332eb
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00142-run-verify-20261026T182021Z-d16332eb
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:b6838076cda3761a854ecaf83804139dc361b1586257bfd0501907d2b14cc292
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T07:07:21Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T07:37:08Z, deltic:auto role=fix run=fix-20261025T071342Z-9a6b3273 branch=task/bug-MDH-BUG-FLU-00142-run-fix-20261025T071342Z-9a6b3273 code=849ef76 gate=manual)

## Observation

Found by lane S8 2026-10-25 (executed): in examples/builtins/prelude_demo.braw native ilka(nums, big) prints naething where the interpreter prints nae, and compose(add_one, double) returns something uncallable ("Type error in 'call': got naething") where the interpreter prints 11; output stops matching there, so apply_n, same, constant and nae_that are unchecked. Repro saved by S8 as .scratch/c1.braw in its lane. Likely the native backend lacks or miscompiles these stdlib/prelude.braw functions.

Evidence fingerprint: `manual:v1:native-prelude-higher-order-functions-ilka-and--f028fdfe864f6a82`


## Fix

<unfixed — raised only>

## Notes
