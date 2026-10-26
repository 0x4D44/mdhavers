# MDH-BUG-FLU-00141 — Native: repeat_char undefined in examples/stdlib/tui_demo.braw

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T06:49:48Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T182021Z-922bea0a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00141-run-verify-20261026T182021Z-922bea0a
- **Owner base:** 2f352f49ef4c71b45a0497bf81a04779c622faad
- **Owner fingerprint:** sha256:750ad230b865eabc2efffc04568d35b8e91812502f4876084540c660b086afd3
- **Owner since:** 2026-10-26T18:20:21Z
- **Owner until:** 2026-10-26T22:20:21Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T06:49:48Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T07:13:29Z, deltic:auto role=fix run=fix-20261025T065500Z-e445faaa branch=task/bug-MDH-BUG-FLU-00141-run-fix-20261025T065500Z-e445faaa code=bfaa64f gate=manual)

## Observation

Found by lane N8 2026-10-25 (executed): examples/stdlib/tui_demo.braw exits 1 natively with "'repeat_char' hasnae been defined yet at line 370"; the interpreter exits 0. Find where repeat_char comes from (stdlib module import or prelude) and why native does not see it.

Evidence fingerprint: `manual:v1:native-repeat-char-undefined-in-examples-stdlib-tui-demo-braw`


## Fix

<unfixed — raised only>

## Notes
