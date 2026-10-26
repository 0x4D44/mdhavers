# MDH-BUG-FLU-00190 — Native: a list of function values never equals itself ([len] == [len] is nae)

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T15:32:28Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T222520Z-5a4a6b82
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00190-run-fix-20261026T222520Z-5a4a6b82
- **Owner base:** 7215f90ac09506ad957c41c70d82bca1bef4a9b4
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T22:25:20Z
- **Owner until:** 2026-10-27T02:25:20Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T15:32:28Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Arthur decision round 4 (2026-10-26): [len] == [len] should be aye. Natively, single function values compare equal (len == len is aye after MDH-BUG-FLU-00162) but list equality on lists holding function values returns nae, unlike the interpreter. Fix element equality for function values in the native list/dict equality path; parity test.

Evidence fingerprint: `manual:v1:native-a-list-of-function-values-never-equals-i-88640b9ce7b20118`


## Fix

<unfixed — raised only>

## Notes
