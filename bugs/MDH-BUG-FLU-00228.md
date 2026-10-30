# MDH-BUG-FLU-00228 — replace_all hangs when old is empty or new contains old (strings and text)

- **State:** Open
- **Priority:** Could
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:06Z
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
- **State history:** Open (2026-10-30T07:14:06Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Under both `stdlib/strings` and `stdlib/text`: `replace_all("abc", "", "x")` and `replace_all("aaa", "a", "aa")` never return (5 s timeout). Expected `xaxbxcx` (or a hurl) and `aaaaaa`. strings.braw:254-261 and text.braw:528-537 rescan from index 0 after each replacement.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:replace-all-hangs-when-old-is-empty-or-new-cont-b8800d715f50b59d`


## Fix

<unfixed — raised only>

## Notes
