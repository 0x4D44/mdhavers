# MDH-BUG-FLU-00076 — mdhavers build -o /abs/path prints the run hint as .//abs/path

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** cli
- **Raised:** 2026-10-23T02:06:58Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-4c6572ed
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00076-run-verify-20261026T224201Z-4c6572ed
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:2e4893e56ecdd68cfc08b4be50a4477ee23c3650e54da80e93242eb65c64151e
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
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
