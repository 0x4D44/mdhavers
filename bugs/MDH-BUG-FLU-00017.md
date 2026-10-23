# MDH-BUG-FLU-00017 — Deeply nested source aborts the CLI/REPL/LSP with a stack overflow

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** parser
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:50:47Z, deltic:auto role=fix run=fix-20261023T015041Z-d57f194c branch=task/bug-MDH-BUG-FLU-00017-run-fix-20261023T015041Z-d57f194c code=8204122 gate=manual)

## Observation

Confirmed: 200000 nested parens -> "thread 'main' has overflowed its stack", exit 134. No parser depth limit.

## Fix

<unfixed — raised only>

## Notes
