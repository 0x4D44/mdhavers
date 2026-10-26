# MDH-BUG-FLU-00044 — compile_to_wat returns Ok for invalid modules

- **State:** Closed
- **Priority:** Should
- **Severity:** High
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:57:41Z, deltic:auto role=fix run=fix-20261023T025736Z-56aedf7c branch=task/bug-MDH-BUG-FLU-00044-run-fix-20261023T025736Z-56aedf7c code=53428a5 gate=manual) -> Closed (2026-10-26T22:40:14Z, deltic:manual role=verify run=verify-20261026T214519Z-b35799d3 verified_fix_run=fix-20261023T025736Z-56aedf7c verdict=close)

## Observation

Inferred (high confidence): functions-only programs export a missing $main (wasm_compiler.rs:560-566, the README example); user `dae main` duplicates $main; default params and unknown callees (any builtin) and top-level vars used in functions produce unresolved ids. No test validates the WAT.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:40:14Z, deltic:manual run=verify-20261026T214519Z-b35799d3 verified_fix_run=fix-20261023T025736Z-56aedf7c verdict=close)

Verifier note: Manual verify: [o1b sol] 53428a5: functions_only_program_exports_a_real_main 1 passed; conditional main-emission mutant fails WAT validation assertion: unknown func $main. CLI functions-only/main/default/global/builtin WAT runs: empty/7/42/42/3; unknown callee rejected; restored clean.

## Notes
