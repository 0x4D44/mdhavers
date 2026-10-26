# MDH-BUG-FLU-00190 — Native: a list of function values never equals itself ([len] == [len] is nae)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T15:32:28Z
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
- **State history:** Open (2026-10-26T15:32:28Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T22:26:17Z, deltic:auto role=fix run=fix-20261026T222520Z-5a4a6b82 branch=task/bug-MDH-BUG-FLU-00190-run-fix-20261026T222520Z-5a4a6b82 code=e312295 gate=manual)

## Observation

Arthur decision round 4 (2026-10-26): [len] == [len] should be aye. Natively, single function values compare equal (len == len is aye after MDH-BUG-FLU-00162) but list equality on lists holding function values returns nae, unlike the interpreter. Fix element equality for function values in the native list/dict equality path; parity test.

Evidence fingerprint: `manual:v1:native-a-list-of-function-values-never-equals-i-88640b9ce7b20118`


## Fix

<unfixed — raised only>

## Notes
