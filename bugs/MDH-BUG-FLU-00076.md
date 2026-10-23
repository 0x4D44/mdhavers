# MDH-BUG-FLU-00076 — mdhavers build -o /abs/path prints the run hint as .//abs/path

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** cli
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T035131Z-ee82b4ae
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00076-run-fix-20261023T035131Z-ee82b4ae
- **Owner base:** e8625012cc835f4b04b48f94c75a2bd68796dbfd
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:51:31Z
- **Owner until:** 2026-10-23T07:51:31Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the showcase lane: the "Tae run" hint prefixes ./ even to absolute paths.

## Fix

<unfixed — raised only>

## Notes
