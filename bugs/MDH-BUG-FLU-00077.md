# MDH-BUG-FLU-00077 — JS backend sort uses its own ordering, diverging from the interpreter on mixed types

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T034502Z-95b64d22
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00077-run-fix-20261023T034502Z-95b64d22
- **Owner base:** ce3530acebdbf583d63d134917380d9fff301546
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:45:02Z
- **Owner until:** 2026-10-23T07:45:02Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the interpreter lane: src/compiler.rs ~258 sort helper does not follow the interpreter's new total order (numbers, then strings, then others).

## Fix

<unfixed — raised only>

## Notes
