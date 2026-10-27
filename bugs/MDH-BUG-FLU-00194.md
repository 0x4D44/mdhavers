# MDH-BUG-FLU-00194 — cargo build --all-targets fails: wasm_tak_drap_strings uses wasm_runner without a feature gate

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-26T19:24:00Z
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
- **State history:** Open (2026-10-26T19:24:00Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T00:56:30Z, deltic:auto role=fix run=fix-20261026T214549Z-9b7fd66c branch=task/bug-MDH-BUG-FLU-00194-run-fix-20261026T214549Z-9b7fd66c code=094641b gate=manual) -> Closed (2026-10-27T02:37:15Z, deltic:manual role=verify run=verify-20261027T013251Z-19a47685 verified_fix_run=fix-20261026T214549Z-9b7fd66c verdict=close)

## Observation

Found by lane R5 2026-10-26 (executed): tests/wasm_tak_drap_strings.rs:6 uses mdhavers::wasm_runner without #![cfg(feature = "wasm_runner")], so cargo build --all-targets with default features fails with E0432 (unresolved import). Gate the file like the other wasm_* tests (lead added this file in 11d6d03).

Evidence fingerprint: `manual:v1:cargo-build-all-targets-fails-wasm-tak-drap-str-8fd0ca838fc53f40`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T02:37:15Z, deltic:manual run=verify-20261027T013251Z-19a47685 verified_fix_run=fix-20261026T214549Z-9b7fd66c verdict=close)

Verifier note: Manual verify: Independent Opus review (lead landed this): close - build-configuration bug; proof is cargo check --all-targets going E0432 -> pass, and removing the cfg gate restores E0432; no runtime test can see a feature gate. Residual (no gate runs --all-targets / --cfg coverage checks) raised separately. Verifier: [le luna] fix 094641b; cargo check --all-targets passed and wasm_tak_drap_strings::tak_and_drap_accept_strings passed 1/1. Removing the file-level cfg gate left that test green, so it does not cover the all-targets build fix; the compile error alone is not fails-before evidence.

## Notes
