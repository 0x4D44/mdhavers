# MDH-BUG-FLU-00055 — speir truncates input lines at 1023 bytes

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:05:42Z, deltic:auto role=fix run=fix-20261023T030536Z-c58fd48f branch=task/bug-MDH-BUG-FLU-00055-run-fix-20261023T030536Z-c58fd48f code=8e071f4 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-88f6660f verified_fix_run=fix-20261023T030536Z-c58fd48f verdict=close)

## Observation

Inferred: mdh_runtime.c:1151 fixed buffer; the rest stays in stdin.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-88f6660f verified_fix_run=fix-20261023T030536Z-c58fd48f verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 8e071f4; speir_reads_lines_longer_than_1023_bytes: 1 passed; first-chunk-only mutant failed output assertion: 1023+remainder vs 5000,next.

## Notes
