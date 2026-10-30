# MDH-BUG-FLU-00233 — benchmark treats tick() nanoseconds as microseconds

- **State:** Open
- **Priority:** Could
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:39Z
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
- **State history:** Open (2026-10-30T07:14:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`ken s = tick(); bide(50); format_time(tick() - s)` shows `55 s` for ~55 ms. tick returns nanoseconds (src/interpreter.rs:8650); benchmark.braw:59-68 assumes microseconds.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:benchmark-treats-tick-nanoseconds-as-microseconds`


## Fix

<unfixed — raised only>

## Notes
