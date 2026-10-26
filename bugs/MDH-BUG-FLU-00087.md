# MDH-BUG-FLU-00087 — Native: user functions named pow/atan2/strtod/strncmp collide with libc lookups

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T03:53:00Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-25c09c5b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00087-run-verify-20261026T171637Z-25c09c5b
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:53d30274fe2887f4de11f0eea3664737e3fdb7597fead7b112569019351691a5
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:53:00Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T04:30:35Z, deltic:auto role=fix run=fix-20261023T043029Z-099ddc31 branch=task/bug-MDH-BUG-FLU-00087-run-fix-20261023T043029Z-099ddc31 code=c9f3fd5 gate=manual)

## Observation

Found by the native follow-ups lane 2026-10-23 (code reading, not yet reproduced): src/llvm/codegen.rs looks up libc helpers by bare name (get_function("pow") etc.), so a user `dae pow(...)` is declared under the same symbol and either shadows the libc function codegen calls or fails to link. User `main` is now mangled to __mdh_user_main (MDH-BUG-FLU-00071); the same mangling should cover every user function (or at least every libc name codegen uses).

## Fix

<unfixed — raised only>

## Notes
