# MDH-BUG-FLU-00057 — Test suite fails outside the Linux full-feature configuration

- **State:** Fixed
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:49:44Z, deltic:auto role=fix run=fix-20261023T024938Z-0c4c279e branch=task/bug-MDH-BUG-FLU-00057-run-fix-20261023T024938Z-0c4c279e code=77f950a gate=manual)

## Observation

Confirmed on macOS: `--features minimal` had 38 failing tests: audio unit tests and tests/audio_stub_coverage.rs assume the audio feature/message text; tests/interpreter_dtls.rs, interpreter_srtp.rs and interpreter_native_ok_or_branches_coverage.rs need dtls but are gated on native; cli_repl_readline_interrupt test gets BrokenPipe.

## Fix

<unfixed — raised only>

## Notes
