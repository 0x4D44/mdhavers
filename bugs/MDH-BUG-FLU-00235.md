# MDH-BUG-FLU-00235 — adventure win condition only checked after an unrecognised command

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:51Z
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
- **State history:** Open (2026-10-30T07:14:51Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

With `set_win_condition(|g| g.current_room_id == "b")`, moving `north` into b does not win; `look` does not either; only `xyzzy` (unrecognised) prints `Congratulations! Ye've won!`. adventure.braw:741-746 checks after every handled command has already returned.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:adventure-win-condition-only-checked-after-an-u-b925331d9d987629`


## Fix

<unfixed — raised only>

## Notes
