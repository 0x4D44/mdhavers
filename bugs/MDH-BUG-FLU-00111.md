# MDH-BUG-FLU-00111 — Native structural equality recurses on the C stack

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-24T02:39:05Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T030424Z-533018ed
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00111-run-fix-20261024T030424Z-533018ed
- **Owner base:** e33fde1be5f425c905967fa3b912c1e813e8c19b
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T03:04:24Z
- **Owner until:** 2026-10-24T07:04:24Z
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
