# MDH-BUG-FLU-00087 — Native: user functions named pow/atan2/strtod/strncmp collide with libc lookups

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T03:53:00Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T043029Z-099ddc31
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00087-run-fix-20261023T043029Z-099ddc31
- **Owner base:** 00cc38d439be0c554b257d8c632416d733e897e6
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T04:30:29Z
- **Owner until:** 2026-10-23T08:30:29Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:53:00Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the native follow-ups lane 2026-10-23 (code reading, not yet reproduced): src/llvm/codegen.rs looks up libc helpers by bare name (get_function("pow") etc.), so a user `dae pow(...)` is declared under the same symbol and either shadows the libc function codegen calls or fails to link. User `main` is now mangled to __mdh_user_main (MDH-BUG-FLU-00071); the same mangling should cover every user function (or at least every libc name codegen uses).

## Fix

<unfixed — raised only>

## Notes
