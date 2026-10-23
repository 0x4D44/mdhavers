# MDH-BUG-FLU-00028 — mdhavers fmt writes `fetch "x" as m` which does not parse

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015806Z-7b19ab19
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00028-run-fix-20261023T015806Z-7b19ab19
- **Owner base:** 604e12f2981952feb2dcca5c0bbfdb96a0f7fac1
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:58:06Z
- **Owner until:** 2026-10-23T05:58:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: formatter.rs:224 prints `as` instead of `tae`. A unit test (formatter.rs:851-860) asserts the buggy output. Breaks examples/basics/modules_demo.braw on fmt.

## Fix

<unfixed — raised only>

## Notes
