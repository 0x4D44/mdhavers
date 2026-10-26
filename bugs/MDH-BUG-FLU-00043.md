# MDH-BUG-FLU-00043 — JS backend diverges from interpreter on numbers, output format, truthiness and equality

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T22:21:50Z, deltic:auto role=fix run=fix-20261023T222144Z-c2967f3b branch=task/bug-MDH-BUG-FLU-00043-run-fix-20261023T222144Z-c2967f3b code=7fbcda1 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-ab9ebb55 verified_fix_run=fix-20261023T222144Z-c2967f3b verdict=close)

## Observation

Confirmed under node: 7/2 -> 3.5; divide by zero -> Infinity instead of error; blether prints true/null/[ 1, 'a' ] instead of aye/naething/[1, a]; [] truthy; [1]==[1] false; catch binds Error object not message; dict literal {k: 1} uses the name k not its value.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-ab9ebb55 verified_fix_run=fix-20261023T222144Z-c2967f3b verdict=close)

Verifier note: Manual verify: [o1b sol] 7fbcda1(version only); actual 3e24904/4b18cb3: parity_js_values_mdh_bug_flu_00043 1 passed; integer division reverted to JS Number division; parity assertion catches 3.5 vs 3. CLI/Node prints 3/aye/naething/[1, a]/falsy/equality aye/colour key/zero-div error matching interpreter; restored clean.

## Notes
