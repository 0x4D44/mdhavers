# MDH-BUG-FLU-00022 — F-string containing a double-quoted string literal fails to lex (README example broken)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015135Z-eb4f690e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00022-run-fix-20261023T015135Z-eb4f690e
- **Owner base:** 8893d9bb5f5af5c3a041001a6e8f6e180a8947a4
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:51:35Z
- **Owner until:** 2026-10-23T05:51:35Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: README.md:455 example f"Cities: {["Edinburgh", "Glasgow"]}" fails to parse; FString regex (token.rs:181) stops at the first inner quote.

## Fix

<unfixed — raised only>

## Notes
