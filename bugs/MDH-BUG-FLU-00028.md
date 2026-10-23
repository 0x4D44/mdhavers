# MDH-BUG-FLU-00028 — mdhavers fmt writes `fetch "x" as m` which does not parse

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:11Z, deltic:auto role=fix run=fix-20261023T015806Z-7b19ab19 branch=task/bug-MDH-BUG-FLU-00028-run-fix-20261023T015806Z-7b19ab19 code=6e06c31 gate=manual)

## Observation

Confirmed: formatter.rs:224 prints `as` instead of `tae`. A unit test (formatter.rs:851-860) asserts the buggy output. Breaks examples/basics/modules_demo.braw on fmt.

## Fix

<unfixed — raised only>

## Notes
