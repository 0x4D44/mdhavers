# MDH-BUG-FLU-00033 — LSP sends no response to unhandled requests and advertises a stub go-to-definition

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** lsp
- **Raised:** 2026-10-22T23:24:09Z
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:48:45Z, deltic:auto role=fix run=fix-20261023T024839Z-0b3b7ab8 branch=task/bug-MDH-BUG-FLU-00033-run-fix-20261023T024839Z-0b3b7ab8 code=77f950a gate=manual)

## Observation

Inferred: lsp/main.rs:150 drops unknown requests (protocol violation, clients can hang); definition always returns None (main.rs:240) while advertised.

## Fix

<unfixed — raised only>

## Notes
