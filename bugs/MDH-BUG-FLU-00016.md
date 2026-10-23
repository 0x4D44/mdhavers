# MDH-BUG-FLU-00016 — Parser takes exponential time on nested brace expressions (dict/block backtracking)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** parser
- **Raised:** 2026-10-22T23:24:07Z
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:50:36Z, deltic:auto role=fix run=fix-20261023T015031Z-f9e9e0fa branch=task/bug-MDH-BUG-FLU-00016-run-fix-20261023T015031Z-f9e9e0fa code=8204122 gate=manual)

## Observation

Confirmed: 26 levels of `0+{` nesting did not finish within 10s. primary() parses `{` as a dict key, rewinds on no colon and re-parses as a block (parser.rs:1271-1286). The LSP re-parses on every keystroke so it hangs.

## Fix

<unfixed — raised only>

## Notes
