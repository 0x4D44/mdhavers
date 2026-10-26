# MDH-BUG-FLU-00027 — mdhavers fmt replaces gin/fer/whiles inside lambda block bodies with ...

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** formatter
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:58:01Z, deltic:auto role=fix run=fix-20261023T015755Z-12d8c291 branch=task/bug-MDH-BUG-FLU-00027-run-fix-20261023T015755Z-12d8c291 code=6e06c31 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-64cc6d0b verified_fix_run=fix-20261023T015755Z-12d8c291 verdict=close)

## Observation

Confirmed: a lambda `|n| { gin ... }` is written as `...`. formatter.rs:600 uses format_stmt_single.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-64cc6d0b verified_fix_run=fix-20261023T015755Z-12d8c291 verdict=close)

Verifier note: Manual verify: [o1b sol] 6e06c31: fmt_lambda_block_bodies_keep_nested_statements selected/pass; block formatter restored to placeholder; expected-success assertion catches lost body/comment. CLI retains gin/fer inside lambda and --check passes; restored clean.

## Notes
