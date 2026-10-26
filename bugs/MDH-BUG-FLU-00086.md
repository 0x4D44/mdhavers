# MDH-BUG-FLU-00086 — Native runtime errors lack the interpreter's " at line N" location

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T03:52:56Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-e4d90fb3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00086-run-verify-20261026T171637Z-e4d90fb3
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:970aa944ef5f085afe4e45a2202ea87b0beb93b2a49954267e9efafa58081e00
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:52:56Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T04:30:58Z, deltic:auto role=fix run=fix-20261023T043052Z-0316ab2e branch=task/bug-MDH-BUG-FLU-00086-run-fix-20261023T043052Z-0316ab2e code=c9f3fd5 gate=manual)

## Observation

Found by the native follow-ups lane 2026-10-23: e.g. native divide-by-zero prints the Scots message without " at line N", while the interpreter includes the line. Caught hurls now carry "Hurled at line N:" (MDH-BUG-FLU-00073), but runtime-raised errors do not.

## Fix

<unfixed — raised only>

## Notes
