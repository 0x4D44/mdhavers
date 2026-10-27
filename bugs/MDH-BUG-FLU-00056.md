# MDH-BUG-FLU-00056 — stdlib calls functions the interpreter does not define (has_key, slice, tae_json)

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** stdlib
- **Raised:** 2026-10-22T23:24:11Z
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:49:32Z, deltic:auto role=fix run=fix-20261023T024926Z-b94978e1 branch=task/bug-MDH-BUG-FLU-00056-run-fix-20261023T024926Z-b94978e1 code=77f950a gate=manual) -> Open (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-68ccab16 verified_fix_run=fix-20261023T024926Z-b94978e1 verdict=doubt reason=reopen-failed-with-undefined-function-errors-restoring-tae-json-in-swarm-symphon) -> Fixed (2026-10-27T04:50:20Z, deltic:auto role=fix run=fix-20261027T015229Z-57a7fd30 branch=task/bug-MDH-BUG-FLU-00056-run-fix-20261027T015229Z-57a7fd30 code=aa81b64 gate=manual) -> Closed (2026-10-27T05:22:54Z, deltic:manual role=verify run=verify-20261027T045159Z-1432da49 verified_fix_run=fix-20261027T015229Z-57a7fd30 verdict=close)

## Observation

Inferred: stdlib/collections.braw:423 group_by calls has_key (interpreter has dict_has; has_key is an LLVM-only alias, and error.rs:643 suggests it); stdlib/uuid.braw:232 slugify calls slice (LLVM-only; interpreter has scran); swarm_symphony/main.braw:55 calls tae_json without fetching json.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-68ccab16 verified_fix_run=fix-20261023T024926Z-b94978e1 verdict=doubt)

Verifier note: Manual verify: [lc luna] 77f950a: group_by and slugify tests passed; old `has_key`/`slice` mutants failed with undefined-function errors. Restoring `tae_json` in swarm_symphony left all 4 stdlib_smoke tests green: test does not cover that fix.

### Verification summary (2026-10-27T05:22:54Z, deltic:manual run=verify-20261027T045159Z-1432da49 verified_fix_run=fix-20261027T015229Z-57a7fd30 verdict=close)

Verifier note: Manual verify: [lf luna] 77f950a/aa81b64; stdlib_smoke 6/6; mutant restoring tae_json made swarm_symphony_json_output_uses_json_stringify fail with undefined-function error; restored test pass; source diff clean

## Notes
