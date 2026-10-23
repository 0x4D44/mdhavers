# MDH-BUG-FLU-00080 — Anonymous dae expressions and keyword-named lambda params fail to parse in shipped files

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T031558Z-b83feee4
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00080-run-fix-20261023T031558Z-b83feee4
- **Owner base:** b882e5924321d219b640d4a06c1f7af1c228844c
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:15:58Z
- **Owner until:** 2026-10-23T07:15:58Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the parser lane (pre-existing): benchmarks/stress/stability_test.braw:25 uses `dae(acc, x) {...}` as an expression; stdlib/tri.braw:23 uses `|thing|` where thing is a keyword.

## Fix

<unfixed — raised only>

## Notes
