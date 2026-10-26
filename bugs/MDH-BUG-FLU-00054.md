# MDH-BUG-FLU-00054 — Native handles are raw pointers: forged mutex/thread handles segfault

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:05:30Z, deltic:auto role=fix run=fix-20261023T030524Z-2c5a5401 branch=task/bug-MDH-BUG-FLU-00054-run-fix-20261023T030524Z-2c5a5401 code=8e071f4 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-578c0d6b verified_fix_run=fix-20261023T030524Z-2c5a5401 verdict=close)

## Observation

Inferred: mdh_runtime.c:3228-3260; interpreter raises a catchable "Unknown mutex handle".

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-578c0d6b verified_fix_run=fix-20261023T030524Z-2c5a5401 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 8e071f4; forged_native_handles_are_catchable_errors: 1 passed; raw-pointer mutant segfaulted child, triggering run_ok assertion expected exit Some(0), got None.

## Notes
