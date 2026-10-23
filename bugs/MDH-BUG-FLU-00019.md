# MDH-BUG-FLU-00019 — mak_siccar without a comma silently drops the message

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** parser
- **Raised:** 2026-10-22T23:24:08Z
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
- **State history:** Open (2026-10-22T23:24:08Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:51:09Z, deltic:auto role=fix run=fix-20261023T015103Z-af70d6da branch=task/bug-MDH-BUG-FLU-00019-run-fix-20261023T015103Z-af70d6da code=8204122 gate=manual)

## Observation

Confirmed: `mak_siccar 1 == 2 "msg"` reports "Assertion failed" without the message; the string is parsed as a separate statement (parser.rs:441 never checks statement end).

## Fix

<unfixed — raised only>

## Notes
