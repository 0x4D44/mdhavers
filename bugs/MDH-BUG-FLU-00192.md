# MDH-BUG-FLU-00192 — Prelude duplicates is_even, is_odd, assert_equal differ from native builtins

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** prelude
- **Raised:** 2026-10-26T15:32:39Z
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
- **State history:** Open (2026-10-26T15:32:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T22:26:30Z, deltic:auto role=fix run=fix-20261026T222551Z-2f3cc478 branch=task/bug-MDH-BUG-FLU-00192-run-fix-20261026T222551Z-2f3cc478 code=9b8ace6 gate=manual)

## Observation

Arthur decision round 4 (2026-10-26): is_even, is_odd and assert_equal exist both in the CLI prelude and as native builtins with different edge cases. Native wins (as decision 147): remove them from the prelude so the interpreter uses the builtins, aligning any interpreter builtin edge cases with native; parity tests.

Evidence fingerprint: `manual:v1:prelude-duplicates-is-even-is-odd-assert-equal--ad2d4d66436b722c`


## Fix

<unfixed — raised only>

## Notes
