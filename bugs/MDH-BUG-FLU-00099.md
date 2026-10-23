# MDH-BUG-FLU-00099 — Five interpreter test_audio_* lib tests fail in the default full build (raylib audio stub)

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** audio
- **Raised:** 2026-10-23T22:22:13Z
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
- **State history:** Open (2026-10-23T22:22:13Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane G94 2026-10-23: cargo test --lib test_audio -- --test-threads=1 fails 5 tests (src/interpreter.rs ~22802, 22839, 22892, 23028, 23035). With graphics enabled the raylib audio backend wins and its soond_spiel is a stub (src/audio/backend_raylib.rs:282) that never errors, while the tests expect miniaudio's errors. Tests should be gated on the backend or the stub should match. Not confirmed on a pre-change trunk build (inference: unrelated code).

Evidence fingerprint: `manual:v1:five-interpreter-test-audio-lib-tests-fail-in-t-4e2221517ebc6ca0`


## Fix

<unfixed — raised only>

## Notes
