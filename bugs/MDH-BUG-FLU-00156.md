# MDH-BUG-FLU-00156 — Interactive examples crash with a type error at end of input instead of ending cleanly

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** examples
- **Raised:** 2026-10-25T23:04:51Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-e0422feb
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00156-run-verify-20261026T224201Z-e0422feb
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:08cdc1d10ed8a6d7256f11c86abe6ed182d1daba90ed61b38e3f185af50a8f56
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T23:04:51Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T01:31:41Z, deltic:auto role=fix run=fix-20261025T230506Z-6ac0a8b4 branch=task/bug-MDH-BUG-FLU-00156-run-fix-20261025T230506Z-6ac0a8b4 code=7d8a875 gate=manual)

## Observation

Found by lane S15 2026-10-25 after MDH-BUG-FLU-00136 (speir returns naething at EOF), observed with stdin closed: examples/games/{adventure,guessing_game,dungeon_crawler,hangman,haggis_adventure,highland_heist,pub_quiz,scottish_blackjack,word_scramble}.braw, examples/apps/todo_app.braw and examples/scottish/scots_calculator.braw exit rc=1 with 'wheesht() expects a string' or 'Cannae turn naething intae an integer'. stdlib cli/menus/adventure input helpers likely the same (not run). They no longer loop forever, but should treat naething from speir as 'input ended' and finish with a short goodbye, exit 0.

Evidence fingerprint: `manual:v1:interactive-examples-crash-with-a-type-error-at-2686184b861e2675`


## Fix

<unfixed — raised only>

## Notes
