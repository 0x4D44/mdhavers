# MDH-BUG-FLU-00086 — Native runtime errors lack the interpreter's " at line N" location

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T03:52:56Z
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
- **State history:** Open (2026-10-23T03:52:56Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T04:30:58Z, deltic:auto role=fix run=fix-20261023T043052Z-0316ab2e branch=task/bug-MDH-BUG-FLU-00086-run-fix-20261023T043052Z-0316ab2e code=c9f3fd5 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-e4d90fb3 verified_fix_run=fix-20261023T043052Z-0316ab2e verdict=close)

## Observation

Found by the native follow-ups lane 2026-10-23: e.g. native divide-by-zero prints the Scots message without " at line N", while the interpreter includes the line. Caught hurls now carry "Hurled at line N:" (MDH-BUG-FLU-00073), but runtime-raised errors do not.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-e4d90fb3 verified_fix_run=fix-20261023T043052Z-0316ab2e verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] c9f3fd5; caught_runtime_errors_match_the_interpreter_text and uncaught_runtime_error_reports_its_line: each 1 passed; line=0 mutant failed expected-output assertion (messages lacked at line N).

## Notes
