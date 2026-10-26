# MDH-BUG-FLU-00093 — Native integer overflow error lacks the interpreter's line number

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-23T04:31:22Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-559e66da
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00093-run-verify-20261026T182021Z-559e66da
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:cddfb4792d99599440d46cb59c486718596d8b31b9384f00eae1258c8b3b6016
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T04:31:22Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:43:10Z, deltic:auto role=fix run=fix-20261024T014305Z-ccbcbaf8 branch=task/bug-MDH-BUG-FLU-00093-run-fix-20261024T014305Z-ccbcbaf8 code=5cf1a3f gate=manual)

## Observation

Found by the native hardening lane 2026-10-23: the interpreter says "too muckle at line N - it's overflowed!" (line mid-message); native overflow errors carry no line. Other native runtime type errors ("Och! Type error in 'x'") also still differ from the interpreter's wording.

## Fix

<unfixed — raised only>

## Notes
