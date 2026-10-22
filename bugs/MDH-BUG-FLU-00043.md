# MDH-BUG-FLU-00043 — JS backend diverges from interpreter on numbers, output format, truthiness and equality

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed under node: 7/2 -> 3.5; divide by zero -> Infinity instead of error; blether prints true/null/[ 1, 'a' ] instead of aye/naething/[1, a]; [] truthy; [1]==[1] false; catch binds Error object not message; dict literal {k: 1} uses the name k not its value.

## Fix

<unfixed — raised only>

## Notes
