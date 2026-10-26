# MDH-BUG-FLU-00105 — interpreter_native_builtins_coverage dies with SIGTRAP on macOS in the default full build

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-24T02:13:46Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-c4906e33
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00105-run-verify-20261026T233955Z-c4906e33
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:cdc269b1813e1070936d1a6c581310515cfd16553b1a40f817ea34274018cb4d
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T02:13:46Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:58:28Z, deltic:auto role=fix run=fix-20261024T025822Z-0e25f899 branch=task/bug-MDH-BUG-FLU-00105-run-fix-20261024T025822Z-0e25f899 code=1c57be6 gate=manual)

## Observation

Found by the lead's full-suite sweep 2026-10-24 on flux (macOS, trunk b0bb4c7, cargo test --features wasm_runner --no-fail-fast): the whole test binary aborts with signal 5 (SIGTRAP) while starting raylib, which on macOS must run on the main thread; cargo test runs tests on worker threads. Every other target passed. Likely pre-existing (also seen by lane SEM-I 2026-10-23). Fix: gate the graphics-touching tests off macOS or run them in a harness=false main-thread runner.

Evidence fingerprint: `manual:v1:interpreter-native-builtins-coverage-dies-with--b81f4d1b15a4ddfa`


## Fix

<unfixed — raised only>

## Notes
