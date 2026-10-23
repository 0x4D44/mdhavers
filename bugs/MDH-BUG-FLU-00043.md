# MDH-BUG-FLU-00043 — JS backend diverges from interpreter on numbers, output format, truthiness and equality

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T222144Z-c2967f3b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00043-run-fix-20261023T222144Z-c2967f3b
- **Owner base:** 67c7f61ae463b8b971153922adf79e9d97263e01
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T22:21:44Z
- **Owner until:** 2026-10-24T02:21:44Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed under node: 7/2 -> 3.5; divide by zero -> Infinity instead of error; blether prints true/null/[ 1, 'a' ] instead of aye/naething/[1, a]; [] truthy; [1]==[1] false; catch binds Error object not message; dict literal {k: 1} uses the name k not its value.

## Fix

<unfixed — raised only>

## Notes
