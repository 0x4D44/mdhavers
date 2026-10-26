# MDH-BUG-FLU-00045 — WAT string literals with non-ASCII characters are corrupted

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** wasm
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-d29394db
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00045-run-verify-20261026T233955Z-d29394db
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:24e82741c6a6c117c802dac82b2823097c452bd34f19506aa3fcee968f57bb31
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:57:53Z, deltic:auto role=fix run=fix-20261023T025747Z-1f8a1cd5 branch=task/bug-MDH-BUG-FLU-00045-run-fix-20261023T025747Z-1f8a1cd5 code=53428a5 gate=manual)

## Observation

Inferred: escape_wat_string (wasm_compiler.rs:1152) writes \xx of the code point instead of UTF-8 bytes while passing the byte length.

## Fix

<unfixed — raised only>

## Notes
