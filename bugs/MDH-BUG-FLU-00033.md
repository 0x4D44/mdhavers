# MDH-BUG-FLU-00033 — LSP sends no response to unhandled requests and advertises a stub go-to-definition

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** lsp
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T024839Z-0b3b7ab8
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00033-run-fix-20261023T024839Z-0b3b7ab8
- **Owner base:** 66506488e916d77395268ab536d499e79979e8e2
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:48:39Z
- **Owner until:** 2026-10-23T06:48:39Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: lsp/main.rs:150 drops unknown requests (protocol violation, clients can hang); definition always returns None (main.rs:240) while advertised.

## Fix

<unfixed — raised only>

## Notes
