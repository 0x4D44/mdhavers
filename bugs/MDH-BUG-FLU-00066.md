# MDH-BUG-FLU-00066 — stdlib/functional.braw does not parse (lambda returning a dict literal)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** stdlib
- **Raised:** 2026-10-23T02:06:57Z
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
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed 2026-10-23 on trunk before and after the parser fixes: `mdhavers check stdlib/functional.braw` fails "Unexpected ':' at line 408". `|item| {"item": ...}` parses the lambda body `{` as a block, never a dict, so the shipped module cannot be fetched. Same text in examples/lib/functional.braw:408. Either the parser should treat a lambda body `{` with a top-level `key:` as a dict (the expression-position disambiguation already does this) or the module should wrap it in parens.

## Fix

<unfixed — raised only>

## Notes
