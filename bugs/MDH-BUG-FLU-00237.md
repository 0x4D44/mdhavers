# MDH-BUG-FLU-00237 — adventure give_to never gives the quest reward

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:15:03Z
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
- **State history:** Open (2026-10-30T07:15:03Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`npc.set_quest("coin","sword")` then `give coin to smith` prints the thanks but `has_item("sword")` is `nae`. adventure.braw:616 is a placeholder.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:adventure-give-to-never-gives-the-quest-reward`


## Fix

<unfixed — raised only>

## Notes
