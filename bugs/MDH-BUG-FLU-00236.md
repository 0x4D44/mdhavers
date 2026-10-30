# MDH-BUG-FLU-00236 — adventure first-visit text never shown on move

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:57Z
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
- **State history:** Open (2026-10-30T07:14:57Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`b.with_first_visit("FIRST VISIT TEXT")` then `north` prints the room without the first-visit text. adventure.braw:520 marks the room visited before `describe()`.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:adventure-first-visit-text-never-shown-on-move`


## Fix

<unfixed — raised only>

## Notes
