# MDH-BUG-FLU-00025 — mdhavers fmt deletes every comment and blank line and overwrites the file

- **State:** Open
- **Priority:** Should
- **Severity:** Critical
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015733Z-6cc8128d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00025-run-fix-20261023T015733Z-6cc8128d
- **Owner base:** b16e859f1be808ecb7ec7d8e5f5c0aba7cc4ff3a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:57:33Z
- **Owner until:** 2026-10-23T05:57:33Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: comments are skipped by the lexer (token.rs:300) and the formatter prints only the AST; main.rs fmt writes the result back in place.

## Fix

<unfixed — raised only>

## Notes
