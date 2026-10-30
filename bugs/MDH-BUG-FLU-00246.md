# MDH-BUG-FLU-00246 — scots_lang whit_means ignores WEATHER_WORDS

- **State:** Open
- **Priority:** Could
- **Severity:** Low
- **Area:** stdlib
- **Raised:** 2026-10-30T07:15:52Z
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
- **State history:** Open (2026-10-30T07:15:52Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

`whit_means("smirr")`, `("snell")`, `("haar")` give `Dinnae ken...`. scots_lang.braw:110 WEATHER_WORDS is never merged (MORE_SCOTS_WORDS is, at :737-739).

Reproduced with the interpreter (0.3.137) from the repo root with MDHAVERS_HOME set, after `fetch "stdlib/<module>"`, by the stdlib docs review on 2026-10-30. Repro programs: the review worktree .scratch/rv/.

Evidence fingerprint: `manual:v1:scots-lang-whit-means-ignores-weather-words`


## Fix

<unfixed — raised only>

## Notes
