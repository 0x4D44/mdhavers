# MDH-BUG-FLU-00111 — Native structural equality recurses on the C stack

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-24T02:39:05Z
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
- **State history:** Open (2026-10-24T02:39:05Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by the lead's review 2026-10-24 (read). __mdh_eq_in in runtime/mdh_runtime.c is cycle-safe but recursive, so comparing lists/dicts nested ~10^6 deep should crash natively; the interpreter's walker is iterative.

Evidence fingerprint: `manual:v1:native-structural-equality-recurses-on-the-c-stack`


## Fix

<unfixed — raised only>

## Notes
