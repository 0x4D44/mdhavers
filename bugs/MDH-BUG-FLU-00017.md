# MDH-BUG-FLU-00017 — Deeply nested source aborts the CLI/REPL/LSP with a stack overflow

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-c06f6667
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00017-run-verify-20261026T214519Z-c06f6667
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:491780fee32933b70040215a9873dba02cd55691a02e508ced17ce793d4a4333
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
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
