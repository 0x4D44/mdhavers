# MDH-BUG-FLU-00180 — Native: a nested dae inside a block reads the global, not the block's shadowing ken

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T07:41:51Z
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
- **State history:** Open (2026-10-26T07:41:51Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T11:36:16Z, deltic:auto role=fix run=fix-20261026T091430Z-15c98ae2 branch=task/bug-MDH-BUG-FLU-00180-run-fix-20261026T091430Z-15c98ae2 code=ea4f5c4 gate=manual)

## Observation

Found by lane S26 2026-10-26 (executed, trunk and after the 179 fix): ken x = 1; dae blk(c) { gin c { ken x = 7; dae g() { gie x }; gie g() } gie x }; blether blk(aye) prints 1 natively, 7 in the interpreter. The 179 fix (shadowing_locals) covers top-level kens in a function body but not block-level kens inside gin/loops/blocks. Extend capture discovery to block-scoped shadowing locals (see 5c3046a block scoping); parity test incl. MDH_RC=1 total=0.

Evidence fingerprint: `manual:v1:native-a-nested-dae-inside-a-block-reads-the-gl-d9b6216222db6a7a`


## Fix

<unfixed — raised only>

## Notes
