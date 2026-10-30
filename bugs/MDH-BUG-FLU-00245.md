# MDH-BUG-FLU-00245 — test.braw run_suite counts skipped tests as passed

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:15:47Z
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
- **State history:** Open (2026-10-30T07:15:47Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

One `su.skip(...)` plus one passing test: `run_suite(su)` prints `Suite: 2 passed, 0 failed`. Expected 1 passed, 1 skipped. test.braw:251 with :306-307.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:test-braw-run-suite-counts-skipped-tests-as-passed`


## Fix

<unfixed — raised only>

## Notes
