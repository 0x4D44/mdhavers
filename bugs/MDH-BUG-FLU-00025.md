# MDH-BUG-FLU-00025 — mdhavers fmt deletes every comment and blank line and overwrites the file

- **State:** Fixed
- **Priority:** Should
- **Severity:** Critical
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:08Z
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:57:39Z, deltic:auto role=fix run=fix-20261023T015733Z-6cc8128d branch=task/bug-MDH-BUG-FLU-00025-run-fix-20261023T015733Z-6cc8128d code=6e06c31 gate=manual)

## Observation

Confirmed: comments are skipped by the lexer (token.rs:300) and the formatter prints only the AST; main.rs fmt writes the result back in place.

## Fix

<unfixed — raised only>

## Notes
