# MDH-BUG-FLU-00065 — LSP hover range starts at the cursor instead of the word start

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** lsp
- **Raised:** 2026-10-22T23:53:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T034513Z-e6fa937a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00065-run-fix-20261023T034513Z-e6fa937a
- **Owner base:** 9f2f2e8e881b7d6085730ca0d396301d2cb42e04
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:45:13Z
- **Owner until:** 2026-10-23T07:45:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Observed by the L7 worker: src/lsp/main.rs handle_hover builds the range from the cursor column rather than the start of the hovered word.

## Fix

<unfixed — raised only>

## Notes
