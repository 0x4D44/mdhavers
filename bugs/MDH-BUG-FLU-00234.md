# MDH-BUG-FLU-00234 — benchmark compare divides by zero for fast functions

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:45Z
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
- **State history:** Open (2026-10-30T07:14:45Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`dae noop() { gie 1 }` then `compare("a", noop, "b", noop, 10)` prints both at 0 µs then hurls `Ye numpty! Tryin' tae divide by zero at line 115`. benchmark.braw:115 (and :22 when iterations is 0).

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:benchmark-compare-divides-by-zero-for-fast-functions`


## Fix

<unfixed — raised only>

## Notes
