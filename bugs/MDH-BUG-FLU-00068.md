# MDH-BUG-FLU-00068 — Native: a whiles loop corrupts function parameters not used inside the loop

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T02:06:57Z
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
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:05:53Z, deltic:auto role=fix run=fix-20261023T030547Z-a7d84499 branch=task/bug-MDH-BUG-FLU-00068-run-fix-20261023T030547Z-a7d84499 code=8e071f4 gate=manual) -> Closed (2026-10-26T18:20:05Z, deltic:manual role=verify run=verify-20261026T171637Z-dd1adf71 verified_fix_run=fix-20261023T030547Z-a7d84499 verdict=close)

## Observation

Found by the showcase lane 2026-10-23 (native only; interpreter correct): `dae f(t) { ken i = 0` / `whiles i < 2 { i = i + 1 }` / `gie t }` then `blether f("str")` prints a pointer-like integer. Suspected loop-exit sync_all_shadows boxing int copies. Passed to the LLVM lane for checking.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T18:20:05Z, deltic:manual run=verify-20261026T171637Z-dd1adf71 verified_fix_run=fix-20261023T030547Z-a7d84499 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 8e071f4; string_parameter_survives_a_loop_in_the_function: 1 passed; restored parameter shadow and loop sync failed output assertion: 4341399632 vs str.

## Notes
