# MDH-BUG-FLU-00080 — Anonymous dae expressions and keyword-named lambda params fail to parse in shipped files

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the parser lane (pre-existing): benchmarks/stress/stability_test.braw:25 uses `dae(acc, x) {...}` as an expression; stdlib/tri.braw:23 uses `|thing|` where thing is a keyword.

## Fix

<unfixed — raised only>

## Notes
