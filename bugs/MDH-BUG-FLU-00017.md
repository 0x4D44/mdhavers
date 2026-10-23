# MDH-BUG-FLU-00017 — Deeply nested source aborts the CLI/REPL/LSP with a stack overflow

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T015041Z-d57f194c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00017-run-fix-20261023T015041Z-d57f194c
- **Owner base:** f2c30159f7d45089f3fd10ebffb65239c5ac8b84
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T01:50:41Z
- **Owner until:** 2026-10-23T05:50:41Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: 200000 nested parens -> "thread 'main' has overflowed its stack", exit 134. No parser depth limit.

## Fix

<unfixed — raised only>

## Notes
