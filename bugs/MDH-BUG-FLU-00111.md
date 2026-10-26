# MDH-BUG-FLU-00111 — Native structural equality recurses on the C stack

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-24T02:39:05Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-7c4d9e08
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00111-run-verify-20261026T182021Z-7c4d9e08
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:67781a354bf724cbf205199324577d7c2c44091be86022f0553476d3671282e8
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:39:05Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:04:29Z, deltic:auto role=fix run=fix-20261024T030424Z-533018ed branch=task/bug-MDH-BUG-FLU-00111-run-fix-20261024T030424Z-533018ed code=efb4bb2 gate=manual)

## Observation

Found by the lead's review 2026-10-24 (read). __mdh_eq_in in runtime/mdh_runtime.c is cycle-safe but recursive, so comparing lists/dicts nested ~10^6 deep should crash natively; the interpreter's walker is iterative.

Evidence fingerprint: `manual:v1:native-structural-equality-recurses-on-the-c-stack`


## Fix

<unfixed — raised only>

## Notes
