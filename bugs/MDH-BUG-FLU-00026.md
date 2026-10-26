# MDH-BUG-FLU-00026 — mdhavers fmt replaces else-if chains with `ither { ... }`

- **State:** Fixed
- **Priority:** Should
- **Severity:** Critical
- **Area:** formatter
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-3a5272f9
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00026-run-verify-20261026T214519Z-3a5272f9
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:85223b8f028c3dd591c17e616f04302eaf14445d81c8c1f340cec4bee2a4ffa5
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:57:50Z, deltic:auto role=fix run=fix-20261023T015745Z-f382a506 branch=task/bug-MDH-BUG-FLU-00026-run-fix-20261023T015745Z-f382a506 code=6e06c31 gate=manual)

## Observation

Confirmed: output `} ither { ... }` written back to the file; formatter.rs:131 -> format_stmt_single returns "..." for Stmt::If (formatter.rs:368).

## Fix

<unfixed — raised only>

## Notes
