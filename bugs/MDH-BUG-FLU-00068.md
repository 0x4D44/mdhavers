# MDH-BUG-FLU-00068 — Native: a whiles loop corrupts function parameters not used inside the loop

- **State:** Open
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
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the showcase lane 2026-10-23 (native only; interpreter correct): `dae f(t) { ken i = 0` / `whiles i < 2 { i = i + 1 }` / `gie t }` then `blether f("str")` prints a pointer-like integer. Suspected loop-exit sync_all_shadows boxing int copies. Passed to the LLVM lane for checking.

## Fix

<unfixed — raised only>

## Notes
