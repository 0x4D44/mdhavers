# MDH-BUG-FLU-00044 — compile_to_wat returns Ok for invalid modules

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** wasm
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T214519Z-b35799d3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00044-run-verify-20261026T214519Z-b35799d3
- **Owner base:** 40936bf267283acf21ab790379d91007fd6c2e00
- **Owner fingerprint:** sha256:9bb5f3298308347aac9b32ff1bafa1e0cb277c25ec1e397ecbece611809a7c8c
- **Owner since:** 2026-10-26T21:45:19Z
- **Owner until:** 2026-10-27T01:45:19Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:57:41Z, deltic:auto role=fix run=fix-20261023T025736Z-56aedf7c branch=task/bug-MDH-BUG-FLU-00044-run-fix-20261023T025736Z-56aedf7c code=53428a5 gate=manual)

## Observation

Inferred (high confidence): functions-only programs export a missing $main (wasm_compiler.rs:560-566, the README example); user `dae main` duplicates $main; default params and unknown callees (any builtin) and top-level vars used in functions produce unresolved ids. No test validates the WAT.

## Fix

<unfixed — raised only>

## Notes
