# MDH-BUG-FLU-00016 — Parser takes exponential time on nested brace expressions (dict/block backtracking)

- **State:** Open
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
- **State history:** Open (2026-10-22T23:24:07Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Confirmed: 26 levels of `0+{` nesting did not finish within 10s. primary() parses `{` as a dict key, rewinds on no colon and re-parses as a block (parser.rs:1271-1286). The LSP re-parses on every keystroke so it hangs.

## Fix

<unfixed — raised only>

## Notes
