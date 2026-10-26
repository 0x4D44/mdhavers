# MDH-BUG-FLU-00057 — Test suite fails outside the Linux full-feature configuration

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** tests
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-276ff4cc
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00057-run-verify-20261026T233955Z-276ff4cc
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:8b14e9c1b2bae6150cdef07bac93bdb7d620cb2beab09a9cbb8c4db7e9f9b60c
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
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
