# MDH-BUG-FLU-00025 — mdhavers fmt deletes every comment and blank line and overwrites the file

- **State:** Fixed
- **Priority:** Should
- **Severity:** Critical
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-4cd10813
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00025-run-verify-20261026T214519Z-4cd10813
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:ca3e564e39fa517afdad43e057fcf106ffd2416b367c32f0c520cb9bf1ae5736
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
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
