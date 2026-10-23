# MDH-BUG-FLU-00076 — mdhavers build -o /abs/path prints the run hint as .//abs/path

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** cli
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:51:37Z, deltic:auto role=fix run=fix-20261023T035131Z-ee82b4ae branch=task/bug-MDH-BUG-FLU-00076-run-fix-20261023T035131Z-ee82b4ae code=6029d44 gate=manual)

## Observation

Found by the showcase lane: the "Tae run" hint prefixes ./ even to absolute paths.

## Fix

<unfixed — raised only>

## Notes
