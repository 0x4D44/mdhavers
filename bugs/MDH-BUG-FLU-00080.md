# MDH-BUG-FLU-00080 — Anonymous dae expressions and keyword-named lambda params fail to parse in shipped files

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-1788b04e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00080-run-verify-20261026T233955Z-1788b04e
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:856b65c93292d9005c1ef1c57c6ef9f317f6ddc8576742624c06018494aa937e
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:16:04Z, deltic:auto role=fix run=fix-20261023T031558Z-b83feee4 branch=task/bug-MDH-BUG-FLU-00080-run-fix-20261023T031558Z-b83feee4 code=e5fbb29 gate=manual)

## Observation

Found by the parser lane (pre-existing): benchmarks/stress/stability_test.braw:25 uses `dae(acc, x) {...}` as an expression; stdlib/tri.braw:23 uses `|thing|` where thing is a keyword.

## Fix

<unfixed — raised only>

## Notes
