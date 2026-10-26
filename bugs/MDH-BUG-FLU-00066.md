# MDH-BUG-FLU-00066 — stdlib/functional.braw does not parse (lambda returning a dict literal)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** stdlib
- **Raised:** 2026-10-23T02:06:57Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-9b8d8b8b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00066-run-verify-20261026T214519Z-9b8d8b8b
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:3474dc845f88e40d819d344ff4ac383be8cde5368c0309c865c3ef37487cedf9
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:15:30Z, deltic:auto role=fix run=fix-20261023T031524Z-c93f9577 branch=task/bug-MDH-BUG-FLU-00066-run-fix-20261023T031524Z-c93f9577 code=e5fbb29 gate=manual)

## Observation

Confirmed 2026-10-23 on trunk before and after the parser fixes: `mdhavers check stdlib/functional.braw` fails "Unexpected ':' at line 408". `|item| {"item": ...}` parses the lambda body `{` as a block, never a dict, so the shipped module cannot be fetched. Same text in examples/lib/functional.braw:408. Either the parser should treat a lambda body `{` with a top-level `key:` as a dict (the expression-position disambiguation already does this) or the module should wrap it in parens.

## Fix

<unfixed — raised only>

## Notes
