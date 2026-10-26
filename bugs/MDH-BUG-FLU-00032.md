# MDH-BUG-FLU-00032 — LSP hover panics on lines containing multi-byte characters

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** lsp
- **Raised:** 2026-10-22T23:24:09Z
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
- **State history:** Open (2026-10-22T23:24:09Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:48:33Z, deltic:auto role=fix run=fix-20261023T024826Z-87bb119f branch=task/bug-MDH-BUG-FLU-00032-run-fix-20261023T024826Z-87bb119f code=77f950a gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-77586d14 verified_fix_run=fix-20261023T024826Z-87bb119f verdict=close)

## Observation

Inferred (high confidence, code reading): lsp/main.rs:305 checks the column against the byte length but indexes a Vec<char>; LSP columns are UTF-16 offsets. A panic kills the server.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-77586d14 verified_fix_run=fix-20261023T024826Z-87bb119f verdict=close)

Verifier note: Manual verify: [o1b sol] 77f950a: multibyte tests 2 passed; 40 real LSP hover requests on é/emoji line answered, gin hover correct. Counting emoji as one unit fails utf16_col_to_char_index_counts_surrogate_pairs assertion (Some(2) vs Some(1)); restored clean.

## Notes
