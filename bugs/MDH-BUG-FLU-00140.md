# MDH-BUG-FLU-00140 — Native backside_forrit rejects a list that the interpreter reverses

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T06:49:44Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-2488edcf
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00140-run-verify-20261026T182021Z-2488edcf
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:c8bfdf5b0e85b0d01ec30300eab370f6254f86b809e2f0e7442c1c8bfff670a3
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T06:49:44Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T07:13:24Z, deltic:auto role=fix run=fix-20261025T065500Z-77cde6ad branch=task/bug-MDH-BUG-FLU-00140-run-fix-20261025T065500Z-77cde6ad code=bfaa64f gate=manual)

## Observation

Found by lane N8 2026-10-25 (executed): examples/builtins/prelude_demo.braw exits 1 natively at backside_forrit(numbers) with "Type error in 'reverse_str': got list"; the interpreter (prelude/builtin) accepts lists and exits 0. Native maps backside_forrit to a string-only reverse.

Evidence fingerprint: `manual:v1:native-backside-forrit-rejects-a-list-that-the--bae3094a14b6346d`


## Fix

<unfixed — raised only>

## Notes
