# MDH-BUG-FLU-00088 — Native: top-level globals use bare symbol names and can collide with C symbols

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T04:31:21Z
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
- **State history:** Open (2026-10-23T04:31:21Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T22:02:15Z, deltic:auto role=fix run=fix-20261023T220208Z-7855b16d branch=task/bug-MDH-BUG-FLU-00088-run-fix-20261023T220208Z-7855b16d code=5404d6a gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-25ccf4af verified_fix_run=fix-20261023T220208Z-7855b16d verdict=close)

## Observation

Found by the native hardening lane 2026-10-23 (code reading): src/llvm/codegen.rs ~4205 and ~12849 emit user globals under their bare name, so `ken main = 1` or `ken stdout = ...` can clash with C runtime/libc symbols. Same class as MDH-BUG-FLU-00087 (functions, now mangled to __mdh_fn_<name>); globals need the same treatment.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-25ccf4af verified_fix_run=fix-20261023T220208Z-7855b16d verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] recorded 5404d6a is version-only; source fix 58ef56c; globals_named_like_c_symbols_work_and_are_mangled: 1 passed; bare-symbol mutant exited abnormally (None), caught by run_ok assertion.

## Notes
