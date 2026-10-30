# MDH-BUG-FLU-00230 — stdlib numeric helpers truncate with integer division

- **State:** Open
- **Priority:** Could
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-30T07:14:19Z
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
- **State history:** Open (2026-10-30T07:14:19Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Integer `/` truncates (`7 / 2` is `3`). Observed: tui `progress(3,10,10)` shows 0%, `gauge("g",50,100,10)` empty, `sparkline([1..8])` `▁▁▁▁▁▁▁█`; colors `gradient("abcd",16,20)` all colour 16; ascii_art `progress_bar(30,100,10)` empty; maths `mean([1,2])` 1, `median([1,2])` 1, `variance([1,2])` 0; text `similarity("abcd","abxy")` 0; template `format_bytes(1536)` `1 KB`; time `seconds_tae_minutes(90)` 1. Sites: tui.braw:166,481,511; colors.braw:293; ascii_art.braw:292; maths.braw:172,184,220; text.braw:609; template.braw:~172; time.braw:69,79,89.

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:stdlib-numeric-helpers-truncate-with-integer-division`


## Fix

<unfixed — raised only>

## Notes
