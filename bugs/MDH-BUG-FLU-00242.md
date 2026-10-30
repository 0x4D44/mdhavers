# MDH-BUG-FLU-00242 — structures Set.remove stores a __REMOVED__ sentinel

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:15:32Z
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
- **State history:** Open (2026-10-30T07:15:32Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`Set().add("__REMOVED__")` then `has` gives `nae` and `size` 0; after `add(1)`,`remove(1)` the items dict holds `"1": __REMOVED__`. structures.braw:212 (sentinel checked at :221,:230,:246).

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:structures-set-remove-stores-a-removed-sentinel`


## Fix

<unfixed — raised only>

## Notes
