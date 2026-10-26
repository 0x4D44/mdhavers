# MDH-BUG-FLU-00088 — Native: top-level globals use bare symbol names and can collide with C symbols

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T04:31:21Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-25ccf4af
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00088-run-verify-20261026T171637Z-25ccf4af
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:556cc972b710a9bf9b62b606055cbc6a7445fbf65a6cad21dccb6cddc0c80e6a
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T04:31:21Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T22:02:15Z, deltic:auto role=fix run=fix-20261023T220208Z-7855b16d branch=task/bug-MDH-BUG-FLU-00088-run-fix-20261023T220208Z-7855b16d code=5404d6a gate=manual)

## Observation

Found by the native hardening lane 2026-10-23 (code reading): src/llvm/codegen.rs ~4205 and ~12849 emit user globals under their bare name, so `ken main = 1` or `ken stdout = ...` can clash with C runtime/libc symbols. Same class as MDH-BUG-FLU-00087 (functions, now mangled to __mdh_fn_<name>); globals need the same treatment.

## Fix

<unfixed — raised only>

## Notes
