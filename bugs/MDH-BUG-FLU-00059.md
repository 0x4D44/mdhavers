# MDH-BUG-FLU-00059 — Makefile and setup-deps.sh are Linux-only

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** build
- **Raised:** 2026-10-22T23:24:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261027T015241Z-1735d534
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00059-run-fix-20261027T015241Z-1735d534
- **Owner base:** 7f4b97dc360742c546b8e8d533826ea51f466dff
- **Owner fingerprint:** -
- **Owner since:** 2026-10-27T01:52:41Z
- **Owner until:** 2026-10-27T05:52:41Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:50:07Z, deltic:auto role=fix run=fix-20261023T025001Z-c9390bf4 branch=task/bug-MDH-BUG-FLU-00059-run-fix-20261023T025001Z-c9390bf4 code=77f950a gate=manual) -> Open (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-7c468c3a verified_fix_run=fix-20261023T025001Z-c9390bf4 verdict=doubt reason=reopen-regression-test-exists-rg-found-none-help-would-not-exercise-installation)

## Observation

Inferred: Makefile:10 zstd detection uses /usr/lib/x86_64-linux-gnu so macOS always reports LLVM disabled yet cargo build still builds full; scripts/setup-deps.sh supports apt/dnf/pacman/zypper only.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-7c468c3a verified_fix_run=fix-20261023T025001Z-c9390bf4 verdict=doubt)

Verifier note: Manual verify: [la luna] 77f950a adds Darwin/Homebrew branches; make status on this Mac reports LLVM 19 enabled and zstd yes. No Makefile/setup-deps regression test exists (rg found none); --help would not exercise installation detection. Fix appears plausible but its platform behavior is untested.

## Notes
