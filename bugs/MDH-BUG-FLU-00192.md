# MDH-BUG-FLU-00192 — Prelude duplicates is_even, is_odd, assert_equal differ from native builtins

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** prelude
- **Raised:** 2026-10-26T15:32:39Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T153300Z-118e81ba
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00192-run-fix-20261026T153300Z-118e81ba
- **Owner base:** 4ab0a76055547c9686ddcd004755e5b3afd1e1c0
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T15:33:00Z
- **Owner until:** 2026-10-26T19:33:00Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T15:32:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Arthur decision round 4 (2026-10-26): is_even, is_odd and assert_equal exist both in the CLI prelude and as native builtins with different edge cases. Native wins (as decision 147): remove them from the prelude so the interpreter uses the builtins, aligning any interpreter builtin edge cases with native; parity tests.

Evidence fingerprint: `manual:v1:prelude-duplicates-is-even-is-odd-assert-equal--ad2d4d66436b722c`


## Fix

<unfixed — raised only>

## Notes
