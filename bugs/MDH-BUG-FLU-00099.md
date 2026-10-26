# MDH-BUG-FLU-00099 — Five interpreter test_audio_* lib tests fail in the default full build (raylib audio stub)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** audio
- **Raised:** 2026-10-23T22:22:13Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-44e6448f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00099-run-verify-20261026T224201Z-44e6448f
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:b8aeaf41fa34808bf41825fad2fb14c45edabded8829e02e886fe3e426b7cb83
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T22:22:13Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:48:50Z, deltic:auto role=fix run=fix-20261024T014845Z-02a6565f branch=task/bug-MDH-BUG-FLU-00099-run-fix-20261024T014845Z-02a6565f code=e3865f0 gate=manual)

## Observation

Found by lane G94 2026-10-23: cargo test --lib test_audio -- --test-threads=1 fails 5 tests (src/interpreter.rs ~22802, 22839, 22892, 23028, 23035). With graphics enabled the raylib audio backend wins and its soond_spiel is a stub (src/audio/backend_raylib.rs:282) that never errors, while the tests expect miniaudio's errors. Tests should be gated on the backend or the stub should match. Not confirmed on a pre-change trunk build (inference: unrelated code).

Evidence fingerprint: `manual:v1:five-interpreter-test-audio-lib-tests-fail-in-t-4e2221517ebc6ca0`


## Fix

<unfixed — raised only>

## Notes
