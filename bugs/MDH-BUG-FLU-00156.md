# MDH-BUG-FLU-00156 — Interactive examples crash with a type error at end of input instead of ending cleanly

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** examples
- **Raised:** 2026-10-25T23:04:51Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T230506Z-6ac0a8b4
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00156-run-fix-20261025T230506Z-6ac0a8b4
- **Owner base:** ce8941947a4674f55a88360030d631b9820621fc
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T23:05:06Z
- **Owner until:** 2026-10-26T03:05:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T23:04:51Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S15 2026-10-25 after MDH-BUG-FLU-00136 (speir returns naething at EOF), observed with stdin closed: examples/games/{adventure,guessing_game,dungeon_crawler,hangman,haggis_adventure,highland_heist,pub_quiz,scottish_blackjack,word_scramble}.braw, examples/apps/todo_app.braw and examples/scottish/scots_calculator.braw exit rc=1 with 'wheesht() expects a string' or 'Cannae turn naething intae an integer'. stdlib cli/menus/adventure input helpers likely the same (not run). They no longer loop forever, but should treat naething from speir as 'input ended' and finish with a short goodbye, exit 0.

Evidence fingerprint: `manual:v1:interactive-examples-crash-with-a-type-error-at-2686184b861e2675`


## Fix

<unfixed — raised only>

## Notes
