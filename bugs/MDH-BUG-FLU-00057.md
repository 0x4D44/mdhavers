# MDH-BUG-FLU-00057 — Test suite fails outside the Linux full-feature configuration

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** tests
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed on macOS: `--features minimal` had 38 failing tests: audio unit tests and tests/audio_stub_coverage.rs assume the audio feature/message text; tests/interpreter_dtls.rs, interpreter_srtp.rs and interpreter_native_ok_or_branches_coverage.rs need dtls but are gated on native; cli_repl_readline_interrupt test gets BrokenPipe.

## Fix

<unfixed — raised only>

## Notes
