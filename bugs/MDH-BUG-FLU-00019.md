# MDH-BUG-FLU-00019 — mak_siccar without a comma silently drops the message

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-f15d890a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00019-run-verify-20261026T233955Z-f15d890a
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:d2f137c595f574a0ab32e5c427efaf77f395ef13eb6172f5c41974599800c1cc
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:51:09Z, deltic:auto role=fix run=fix-20261023T015103Z-af70d6da branch=task/bug-MDH-BUG-FLU-00019-run-fix-20261023T015103Z-af70d6da code=8204122 gate=manual)

## Observation

Confirmed: `mak_siccar 1 == 2 "msg"` reports "Assertion failed" without the message; the string is parsed as a separate statement (parser.rs:441 never checks statement end).

## Fix

<unfixed — raised only>

## Notes
