# MDH-BUG-FLU-00240 — config Config.delete leaves the key present with naething

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:15:21Z
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
- **State history:** Open (2026-10-30T07:15:21Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

After `set("a",1)`, `delete("a")`: `has("a")` is `aye` and `get("a","dflt")` is `naething`; a deleted key with a default returns `naething` instead of the default. config.braw:53-58.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:config-config-delete-leaves-the-key-present-with-naething`


## Fix

<unfixed — raised only>

## Notes
