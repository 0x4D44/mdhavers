# MDH-BUG-FLU-00026 — mdhavers fmt replaces else-if chains with `ither { ... }`

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:57:50Z, deltic:auto role=fix run=fix-20261023T015745Z-f382a506 branch=task/bug-MDH-BUG-FLU-00026-run-fix-20261023T015745Z-f382a506 code=6e06c31 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-3a5272f9 verified_fix_run=fix-20261023T015745Z-f382a506 verdict=close)

## Observation

Confirmed: output `} ither { ... }` written back to the file; formatter.rs:131 -> format_stmt_single returns "..." for Stmt::If (formatter.rs:368).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-3a5272f9 verified_fix_run=fix-20261023T015745Z-f382a506 verdict=close)

Verifier note: Manual verify: [o1b sol] 6e06c31: fmt_else_if_chains_are_kept selected/pass; else-if lowering restored to placeholder; checked-format assertion refuses unparseable .... CLI retains ither gin nae and --check passes; restored clean.

## Notes
