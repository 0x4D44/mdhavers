# MDH-BUG-FLU-00098 — JS: gie in a block expression inside a default parameter escapes uncaught

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-23T22:22:08Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-60ca1a43
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00098-run-verify-20261026T225713Z-60ca1a43
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:51466fea138c50d2f7d3db967d3b235bef86a1688703abc2b810d96cfa3d55df
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T22:22:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T01:38:25Z, deltic:auto role=fix run=fix-20261024T013820Z-57948697 branch=task/bug-MDH-BUG-FLU-00098-run-fix-20261024T013820Z-57948697 code=50f0079 gate=manual)

## Observation

Found by lane G94 (MDH-BUG-FLU-00094 fix, 8b1d2ff) 2026-10-23. A block-expression gie in a dae default parameter value throws the per-call sentinel before the function body's try/catch is entered, so it escapes as an uncaught exception. The interpreter returns from the called function.

Evidence fingerprint: `manual:v1:js-gie-in-a-block-expression-inside-a-default-p-9af8d25bd734db08`


## Fix

<unfixed — raised only>

## Notes
