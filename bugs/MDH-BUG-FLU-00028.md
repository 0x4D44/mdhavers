# MDH-BUG-FLU-00028 — mdhavers fmt writes `fetch "x" as m` which does not parse

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:11Z, deltic:auto role=fix run=fix-20261023T015806Z-7b19ab19 branch=task/bug-MDH-BUG-FLU-00028-run-fix-20261023T015806Z-7b19ab19 code=6e06c31 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-37de1008 verified_fix_run=fix-20261023T015806Z-7b19ab19 verdict=close)

## Observation

Confirmed: formatter.rs:224 prints `as` instead of `tae`. A unit test (formatter.rs:851-860) asserts the buggy output. Breaks examples/basics/modules_demo.braw on fmt.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-37de1008 verified_fix_run=fix-20261023T015806Z-7b19ab19 verdict=close)

Verifier note: Manual verify: [o1b sol] 6e06c31: fmt_import_alias_uses_tae selected/pass; alias tae reverted to as; checked-format assertion refuses parse-invalid alias. CLI emits fetch "lib/maths" tae m and --check passes; restored clean.

## Notes
