# MDH-BUG-FLU-00022 — F-string containing a double-quoted string literal fails to lex (README example broken)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: README.md:455 example f"Cities: {["Edinburgh", "Glasgow"]}" fails to parse; FString regex (token.rs:181) stops at the first inner quote.

## Fix

<unfixed — raised only>

## Notes
