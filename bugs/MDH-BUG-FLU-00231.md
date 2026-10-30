# MDH-BUG-FLU-00231 — tui numbered_list numbers every line 1.

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:25Z
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
- **State history:** Open (2026-10-30T07:14:25Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`numbered_list(["a","b","c"])` prints `1. a`, `1. b`, `1. c`. tui.braw:207: the counter increment is outside the loop.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:tui-numbered-list-numbers-every-line-1`


## Fix

<unfixed — raised only>

## Notes
