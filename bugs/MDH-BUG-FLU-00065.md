# MDH-BUG-FLU-00065 — LSP hover range starts at the cursor instead of the word start

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** lsp
- **Raised:** 2026-10-22T23:53:42Z
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
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Observed by the L7 worker: src/lsp/main.rs handle_hover builds the range from the cursor column rather than the start of the hovered word.

## Fix

<unfixed — raised only>

## Notes
