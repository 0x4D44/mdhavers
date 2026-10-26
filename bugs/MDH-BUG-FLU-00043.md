# MDH-BUG-FLU-00043 — JS backend diverges from interpreter on numbers, output format, truthiness and equality

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** js
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-ab9ebb55
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00043-run-verify-20261026T214519Z-ab9ebb55
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:d341af516665b87561b7fd37fdc61f7e86f2f2e97b4136a28fcd772674e91bf1
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T22:21:50Z, deltic:auto role=fix run=fix-20261023T222144Z-c2967f3b branch=task/bug-MDH-BUG-FLU-00043-run-fix-20261023T222144Z-c2967f3b code=7fbcda1 gate=manual)

## Observation

Confirmed under node: 7/2 -> 3.5; divide by zero -> Infinity instead of error; blether prints true/null/[ 1, 'a' ] instead of aye/naething/[1, a]; [] truthy; [1]==[1] false; catch binds Error object not message; dict literal {k: 1} uses the name k not its value.

## Fix

<unfixed — raised only>

## Notes
