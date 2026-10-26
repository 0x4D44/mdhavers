# MDH-BUG-FLU-00065 — LSP hover range starts at the cursor instead of the word start

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** lsp
- **Raised:** 2026-10-22T23:53:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-4172a2ce
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00065-run-verify-20261026T233955Z-4172a2ce
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:efc358162f55406336943faa4d602b2af410856f91a6b983422627c9620c69a2
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:45:20Z, deltic:auto role=fix run=fix-20261023T034513Z-e6fa937a branch=task/bug-MDH-BUG-FLU-00065-run-fix-20261023T034513Z-e6fa937a code=54979e9 gate=manual)

## Observation

Observed by the L7 worker: src/lsp/main.rs handle_hover builds the range from the cursor column rather than the start of the hovered word.

## Fix

<unfixed — raised only>

## Notes
