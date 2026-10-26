# MDH-BUG-FLU-00150 — Native RC leaks: blether of a float and multi-character split leak strings under MDH_RC=1

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T09:59:18Z
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
- **State history:** Open (2026-10-25T09:59:18Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T10:37:06Z, deltic:auto role=fix run=fix-20261025T100351Z-e1910576 branch=task/bug-MDH-BUG-FLU-00150-run-fix-20261025T100351Z-e1910576 code=8497359 gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-4463199a verified_fix_run=fix-20261025T100351Z-e1910576 verdict=close)

## Observation

Found by lane R2 2026-10-25 (executed, MDH_RC=1 leak harness): blether 1.5 then ken parts = split("a--b", "--") reports string=3 live at exit; one string per blether of a float (tests/golden/basics/literals/floats.braw leaks one per line) and two from the multi-character-delimiter path of split. These are neither cycles nor exception paths, so they are real missing releases in the float display path and split's multi-char branch. Only affects programs compiled with MDH_RC=1 (off by default).

Evidence fingerprint: `manual:v1:native-rc-leaks-blether-of-a-float-and-multi-ch-51cf4bf96684070b`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-4463199a verified_fix_run=fix-20261025T100351Z-e1910576 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=8497359; MDH_RC=1 repro prints 1.5/2 and reports string=0; both leak_nothing tests selected and passed; mutant removed float_str and multi-char split releases, first test assertion saw string=3 vs 0 (mixed-concat/read_lines test stayed green).

## Notes
