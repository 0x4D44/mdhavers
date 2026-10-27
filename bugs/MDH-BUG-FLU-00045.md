# MDH-BUG-FLU-00045 — WAT string literals with non-ASCII characters are corrupted

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** wasm
- **Raised:** 2026-10-22T23:24:10Z
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:57:53Z, deltic:auto role=fix run=fix-20261023T025747Z-1f8a1cd5 branch=task/bug-MDH-BUG-FLU-00045-run-fix-20261023T025747Z-1f8a1cd5 code=53428a5 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-d29394db verified_fix_run=fix-20261023T025747Z-1f8a1cd5 verdict=close)

## Observation

Inferred: escape_wat_string (wasm_compiler.rs:1152) writes \xx of the code point instead of UTF-8 bytes while passing the byte length.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-d29394db verified_fix_run=fix-20261023T025747Z-1f8a1cd5 verdict=close)

Verifier note: Manual verify: [lc luna] 53428a5: non_ascii_string_literals_* (2 selected) passed; char-code-point mutant failed emitted-byte and round-trip assertions, output corrupted `£5/café/€/日本語 😀`. Source restored clean.

## Notes
