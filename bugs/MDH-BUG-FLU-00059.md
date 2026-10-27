# MDH-BUG-FLU-00059 — Makefile and setup-deps.sh are Linux-only

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** build
- **Raised:** 2026-10-22T23:24:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261027T061519Z-12ec0895
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00059-run-verify-20261027T061519Z-12ec0895
- **Owner base:** d6edeceae6882c391a0c0409f8a5cb1a285747d0
- **Owner fingerprint:** sha256:fea8d8bde61a9678ccb82445c09a5fb63f9ea27b1059079d4e3b4ad4050a6602
- **Owner since:** 2026-10-27T06:15:19Z
- **Owner until:** 2026-10-27T10:15:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=2, indeterminate=0
- **State history:** Open (2026-10-22T23:24:12Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:50:07Z, deltic:auto role=fix run=fix-20261023T025001Z-c9390bf4 branch=task/bug-MDH-BUG-FLU-00059-run-fix-20261023T025001Z-c9390bf4 code=77f950a gate=manual) -> Open (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-7c468c3a verified_fix_run=fix-20261023T025001Z-c9390bf4 verdict=doubt reason=reopen-regression-test-exists-rg-found-none-help-would-not-exercise-installation) -> Fixed (2026-10-27T04:50:42Z, deltic:auto role=fix run=fix-20261027T015241Z-1735d534 branch=task/bug-MDH-BUG-FLU-00059-run-fix-20261027T015241Z-1735d534 code=75953fe gate=manual) -> Open (2026-10-27T05:22:54Z, deltic:manual role=verify run=verify-20261027T045159Z-a8b677de verified_fix_run=fix-20261027T015241Z-1735d534 verdict=doubt reason=reopen-fails-its-llvm-assertion-without-the-makefile-detector-setup-deps-stub-de) -> Fixed (2026-10-27T06:12:22Z, deltic:auto role=fix run=fix-20261027T055908Z-e6e86616 branch=task/bug-MDH-BUG-FLU-00059-run-fix-20261027T055908Z-e6e86616 code=a2f3c32 gate=manual)

## Observation

Inferred: Makefile:10 zstd detection uses /usr/lib/x86_64-linux-gnu so macOS always reports LLVM disabled yet cargo build still builds full; scripts/setup-deps.sh supports apt/dnf/pacman/zypper only.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-7c468c3a verified_fix_run=fix-20261023T025001Z-c9390bf4 verdict=doubt)

Verifier note: Manual verify: [la luna] 77f950a adds Darwin/Homebrew branches; make status on this Mac reports LLVM 19 enabled and zstd yes. No Makefile/setup-deps regression test exists (rg found none); --help would not exercise installation detection. Fix appears plausible but its platform behavior is untested.

### Verification summary (2026-10-27T05:22:54Z, deltic:manual run=verify-20261027T045159Z-a8b677de verified_fix_run=fix-20261027T015241Z-1735d534 verdict=doubt)

Verifier note: Manual verify: [lf luna] 77f950a/75953fe; make_status_finds_homebrew_llvm_and_zstd_on_darwin fails its LLVM assertion without the Makefile detector; setup-deps stub detects brew; removing its Darwin branch yields Unsupported package manager while makefile_detection remains 2/2: test does not cover the script fix

## Notes
