# MDH-BUG-FLU-00130 — Native codegen is nondeterministic: sync_all_shadows emits stores in HashMap order

- **State:** Open
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
- **State history:** Open (2026-10-25T00:11:33Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane U2 2026-10-25 (executed), code checked by the lead: sync_all_shadows in src/llvm/codegen.rs (~6302) iterates self.int_shadows.keys() (a HashMap), so the order of the shadow sync stores at loop exit varies run to run. Repro: run target/debug/mdhavers build tests/golden/algorithms/math/sum_range.braw --emit-llvm several times and compare md5 of the IR; 6 runs gave 2 distinct files. Behaviour is unaffected (the stores are independent) but builds are not reproducible and IR diffs are noisy. Fix: sort the names (or keep an ordered map). Check other HashMap walks that emit IR.

Evidence fingerprint: `manual:v1:native-codegen-is-nondeterministic-sync-all-sha-3a6317a1028d939a`


## Fix

<unfixed — raised only>

## Notes
