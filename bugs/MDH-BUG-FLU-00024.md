# MDH-BUG-FLU-00024 — _ rejected as loop variable, lambda or function parameter

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015156Z-7c946d96
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00024-run-fix-20261023T015156Z-7c946d96
- **Owner base:** 7da41e1de9df3b9850814f7e88b8d4d3f6188821
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:51:56Z
- **Owner until:** 2026-10-23T05:51:56Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: fer _ in 0..3 -> parse error. token.rs:255 / expect_identifier.

## Fix

<unfixed — raised only>

## Notes
