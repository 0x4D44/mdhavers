# MDH-BUG-FLU-00032 — LSP hover panics on lines containing multi-byte characters

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** lsp
- **Raised:** 2026-10-22T23:24:09Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-77586d14
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00032-run-verify-20261026T214519Z-77586d14
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:047ab18317633327f3fc6c0038bb234fe3418e8af583f0679c42eba64aaebcbb
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:48:33Z, deltic:auto role=fix run=fix-20261023T024826Z-87bb119f branch=task/bug-MDH-BUG-FLU-00032-run-fix-20261023T024826Z-87bb119f code=77f950a gate=manual)

## Observation

Inferred (high confidence, code reading): lsp/main.rs:305 checks the column against the byte length but indexes a Vec<char>; LSP columns are UTF-16 offsets. A panic kills the server.

## Fix

<unfixed — raised only>

## Notes
