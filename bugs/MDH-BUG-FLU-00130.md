# MDH-BUG-FLU-00130 — Native codegen is nondeterministic: sync_all_shadows emits stores in HashMap order

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-25T00:11:33Z
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
- **State history:** Open (2026-10-25T00:11:33Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T01:06:10Z, deltic:auto role=fix run=fix-20261025T003027Z-b694fe04 branch=task/bug-MDH-BUG-FLU-00130-run-fix-20261025T003027Z-b694fe04 code=55c90a7 gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-5a4afb7b verified_fix_run=fix-20261025T003027Z-b694fe04 verdict=close)

## Observation

Found by lane U2 2026-10-25 (executed), code checked by the lead: sync_all_shadows in src/llvm/codegen.rs (~6302) iterates self.int_shadows.keys() (a HashMap), so the order of the shadow sync stores at loop exit varies run to run. Repro: run target/debug/mdhavers build tests/golden/algorithms/math/sum_range.braw --emit-llvm several times and compare md5 of the IR; 6 runs gave 2 distinct files. Behaviour is unaffected (the stores are independent) but builds are not reproducible and IR diffs are noisy. Fix: sort the names (or keep an ordered map). Check other HashMap walks that emit IR.

Evidence fingerprint: `manual:v1:native-codegen-is-nondeterministic-sync-all-sha-3a6317a1028d939a`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-5a4afb7b verified_fix_run=fix-20261025T003027Z-b694fe04 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=55c90a7; six sum_range --emit-llvm runs had one SHA-256; native_ir_is_identical_across_compiles passed 1/1; mutant reversed sorted shadow/capture/function walks and test assertion reported IR differs at compile 1.

## Notes
