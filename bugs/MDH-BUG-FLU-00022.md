# MDH-BUG-FLU-00022 — F-string containing a double-quoted string literal fails to lex (README example broken)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-7f9708f9
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00022-run-verify-20261026T233955Z-7f9708f9
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:503b4d4f7e8a4a9a9883b3dc32ad8be367435912b2bade3d3459bc1116c2c774
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:51:40Z, deltic:auto role=fix run=fix-20261023T015135Z-eb4f690e branch=task/bug-MDH-BUG-FLU-00022-run-fix-20261023T015135Z-eb4f690e code=8204122 gate=manual)

## Observation

Confirmed: README.md:455 example f"Cities: {["Edinburgh", "Glasgow"]}" fails to parse; FString regex (token.rs:181) stops at the first inner quote.

## Fix

<unfixed — raised only>

## Notes
