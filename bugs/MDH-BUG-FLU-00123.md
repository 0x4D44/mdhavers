# MDH-BUG-FLU-00123 — WASM closures: a second ken in the same scope makes a fresh cell instead of rebinding

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T04:51:08Z
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
- **State history:** Open (2026-10-24T04:51:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T06:14:36Z, deltic:auto role=fix run=fix-20261024T061429Z-9fbea918 branch=task/bug-MDH-BUG-FLU-00123-run-fix-20261024T061429Z-9fbea918 code=c436abd gate=manual) -> Closed (2026-10-27T01:32:14Z, deltic:manual role=verify run=verify-20261027T001425Z-04b80963 verified_fix_run=fix-20261024T061429Z-9fbea918 verdict=close)

## Observation

Documented divergence from lane W4 (WASM function values, 2026-10-24, see wrk_docs/2026.10.24 - DES - WASM function values and closures.md): closures created before a repeated ken x keep the old cell, while the interpreter rebinds in place. Also: WASM function values must be called with exactly their arity (no defaults), nested dae cannot have default parameters, and block expressions are supported only as lambda bodies.

Evidence fingerprint: `manual:v1:wasm-closures-a-second-ken-in-the-same-scope-ma-3ce057110420a511`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:32:14Z, deltic:manual run=verify-20261027T001425Z-04b80963 verified_fix_run=fix-20261024T061429Z-9fbea918 verdict=close)

Verifier note: Manual verify: [ld luna] fix c436abd (source 703a6e9); a_second_ken_in_the_same_scope_rebinds_the_shared_cell passed; disabling same-scope rebind failed parity (1[PATH] vs 2[PATH]).

## Notes
