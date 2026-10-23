# MDH-BUG-FLU-00088 — Native: top-level globals use bare symbol names and can collide with C symbols

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T04:31:21Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T220208Z-7855b16d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00088-run-fix-20261023T220208Z-7855b16d
- **Owner base:** 237e213eb0509bda6de9ead23acf03bef043059f
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T22:02:08Z
- **Owner until:** 2026-10-24T02:02:08Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T04:31:21Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the native hardening lane 2026-10-23 (code reading): src/llvm/codegen.rs ~4205 and ~12849 emit user globals under their bare name, so `ken main = 1` or `ken stdout = ...` can clash with C runtime/libc symbols. Same class as MDH-BUG-FLU-00087 (functions, now mangled to __mdh_fn_<name>); globals need the same treatment.

## Fix

<unfixed — raised only>

## Notes
