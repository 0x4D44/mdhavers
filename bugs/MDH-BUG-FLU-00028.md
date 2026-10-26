# MDH-BUG-FLU-00028 — mdhavers fmt writes `fetch "x" as m` which does not parse

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-37de1008
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00028-run-verify-20261026T214519Z-37de1008
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:78c2a449efcd83996034f9be1a2d1e4866d61fc1c1c4401d921c1db1d7241dbb
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
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
