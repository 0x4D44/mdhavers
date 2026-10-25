# MDH-BUG-FLU-00150 — Native RC leaks: blether of a float and multi-character split leak strings under MDH_RC=1

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T09:59:18Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T100351Z-e1910576
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00150-run-fix-20261025T100351Z-e1910576
- **Owner base:** 4dd0b5af505e70b1b6c1ef8c8726ba20a33813ec
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T10:03:51Z
- **Owner until:** 2026-10-25T14:03:51Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T09:59:18Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane R2 2026-10-25 (executed, MDH_RC=1 leak harness): blether 1.5 then ken parts = split("a--b", "--") reports string=3 live at exit; one string per blether of a float (tests/golden/basics/literals/floats.braw leaks one per line) and two from the multi-character-delimiter path of split. These are neither cycles nor exception paths, so they are real missing releases in the float display path and split's multi-char branch. Only affects programs compiled with MDH_RC=1 (off by default).

Evidence fingerprint: `manual:v1:native-rc-leaks-blether-of-a-float-and-multi-ch-51cf4bf96684070b`


## Fix

<unfixed — raised only>

## Notes
