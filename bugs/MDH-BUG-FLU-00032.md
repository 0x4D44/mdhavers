# MDH-BUG-FLU-00032 — LSP hover panics on lines containing multi-byte characters

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** lsp
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T024826Z-87bb119f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00032-run-fix-20261023T024826Z-87bb119f
- **Owner base:** 2e7dba33efadf75cea1bcbb5260182b4d03f8ab1
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:48:26Z
- **Owner until:** 2026-10-23T06:48:26Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred (high confidence, code reading): lsp/main.rs:305 checks the column against the byte length but indexes a Vec<char>; LSP columns are UTF-16 offsets. A panic kills the server.

## Fix

<unfixed — raised only>

## Notes
