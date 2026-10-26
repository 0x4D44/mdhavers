# MDH-BUG-FLU-00083 — Pre-existing llvm_comprehensive_tests failures: fankle_cov x3, dict_merge, out_of_bounds_list, lines

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T03:06:29Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-377d88dd
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00083-run-verify-20261026T171637Z-377d88dd
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:f59866b5024254e04ffaf1386d481e84a5b50dd49010646d5c7ce45195d4cda1
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:06:29Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T21:50:52Z, deltic:auto role=fix run=fix-20261023T215045Z-74532bde branch=task/bug-MDH-BUG-FLU-00083-run-fix-20261023T215045Z-74532bde code=664d308 gate=manual)

## Observation

Found by the LLVM lane 2026-10-23: these 6 tests fail on origin/master before and after the LLVM safety fixes (10 more fail on lambda-as-argument, MDH-BUG-FLU-00070). The suite was only run to ~5300/7126 within the time box; the tail is unverified.

## Fix

<unfixed — raised only>

## Notes
