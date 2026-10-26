# MDH-BUG-FLU-00083 — Pre-existing llvm_comprehensive_tests failures: fankle_cov x3, dict_merge, out_of_bounds_list, lines

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T03:06:29Z
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
- **State history:** Open (2026-10-23T03:06:29Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T21:50:52Z, deltic:auto role=fix run=fix-20261023T215045Z-74532bde branch=task/bug-MDH-BUG-FLU-00083-run-fix-20261023T215045Z-74532bde code=664d308 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-377d88dd verified_fix_run=fix-20261023T215045Z-74532bde verdict=close)

## Observation

Found by the LLVM lane 2026-10-23: these 6 tests fail on origin/master before and after the LLVM safety fixes (10 more fail on lambda-as-argument, MDH-BUG-FLU-00070). The suite was only run to ~5300/7126 within the time box; the tail is unverified.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-377d88dd verified_fix_run=fix-20261023T215045Z-74532bde verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] record code 664d308 is version-only; test repair 1afd2be; all six cited tests each 1 passed. Old lines(path) repro emits 1 vs expected 3; old fankle(nested) build says interleave expects 2 arguments (test-only fix, no product hunk).

## Notes
