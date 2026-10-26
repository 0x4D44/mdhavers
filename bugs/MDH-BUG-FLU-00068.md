# MDH-BUG-FLU-00068 — Native: a whiles loop corrupts function parameters not used inside the loop

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T02:06:57Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-dd1adf71
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00068-run-verify-20261026T171637Z-dd1adf71
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:40729803a90d470786d1dac6e30b4ed04cfc81273eb08467457af161bccf2f11
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:05:53Z, deltic:auto role=fix run=fix-20261023T030547Z-a7d84499 branch=task/bug-MDH-BUG-FLU-00068-run-fix-20261023T030547Z-a7d84499 code=8e071f4 gate=manual)

## Observation

Found by the showcase lane 2026-10-23 (native only; interpreter correct): `dae f(t) { ken i = 0` / `whiles i < 2 { i = i + 1 }` / `gie t }` then `blether f("str")` prints a pointer-like integer. Suspected loop-exit sync_all_shadows boxing int copies. Passed to the LLVM lane for checking.

## Fix

<unfixed — raised only>

## Notes
