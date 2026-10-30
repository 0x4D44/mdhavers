# MDH-BUG-FLU-00244 — statemachine must_trigger prints instead of hurling

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:15:42Z
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
- **State history:** Open (2026-10-30T07:15:42Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Inside `hae_a_bash`, `StateMachine("idle",["idle","busy"]).must_trigger("nope")` prints `StateMachine Error: ...Cannae trigger 'nope' from state 'idle'` and returns `naething`; nothing is caught. statemachine.braw:121-126.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:statemachine-must-trigger-prints-instead-of-hurling`


## Fix

<unfixed — raised only>

## Notes
