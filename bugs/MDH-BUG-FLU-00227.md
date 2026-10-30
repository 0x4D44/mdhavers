# MDH-BUG-FLU-00227 — json is_valid_json always returns aye

- **State:** Open
- **Priority:** Could
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:00Z
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
- **State history:** Open (2026-10-30T07:14:00Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`is_valid_json("{{{ nope")` and `is_valid_json("")` both give `aye`. stdlib/json.braw:432-436 ignores the parse result and returns `aye`.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:json-is-valid-json-always-returns-aye`


## Fix

<unfixed — raised only>

## Notes
