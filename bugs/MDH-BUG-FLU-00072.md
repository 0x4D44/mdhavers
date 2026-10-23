# MDH-BUG-FLU-00072 — Native: functions cannot reference globals defined later in the file

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T220155Z-449b24f2
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00072-run-fix-20261023T220155Z-449b24f2
- **Owner base:** f88b63d1c8a9f669e13748e68cf00df06ccf3927
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T22:01:55Z
- **Owner until:** 2026-10-24T02:01:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the showcase lane: a function using a global declared after it fails natively with "Undefined variable: X"; the interpreter resolves it at call time. May be a deliberate restriction; needs a decision and documentation either way.

## Fix

<unfixed — raised only>

## Notes
