# MDH-BUG-FLU-00232 — tui menu prints options but never reads input

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:32Z
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
- **State history:** Open (2026-10-30T07:14:32Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`ken r = menu("Pick", ["x","y"])` prints the menu and prompt and returns `naething` without reading stdin. tui.braw:211-221.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:tui-menu-prints-options-but-never-reads-input`


## Fix

<unfixed — raised only>

## Notes
