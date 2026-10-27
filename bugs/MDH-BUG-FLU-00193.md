# MDH-BUG-FLU-00193 — Native codegen: about 35 inline allocations are unchecked (split, list/dict literals, instances)

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T15:38:17Z
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
- **State history:** Open (2026-10-26T15:38:17Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T05:57:50Z, deltic:auto role=fix run=fix-20261027T045241Z-2f69b43a branch=task/bug-MDH-BUG-FLU-00193-run-fix-20261027T045241Z-2f69b43a code=85e9daf gate=manual) -> Closed (2026-10-27T06:15:02Z, deltic:manual role=verify run=verify-20261027T055825Z-a8428af6 verified_fix_run=fix-20261027T045241Z-2f69b43a verdict=close)

## Observation

Found by lane N13 2026-10-26 (code audit while fixing MDH-BUG-FLU-00189, executed repro): src/llvm/codegen.rs still emits about 35 inline malloc/realloc calls with no NULL check, e.g. split's single-character-delimiter path (~:31475, ~:31580), list literals (~:29398, ~:29417), dict literals (~:29901), instance allocation (~:39817). Repro: split of a 1500-character literal on ',' under the debug knob MDH_TEST_ALLOC_FAIL_OVER=1000 exits 139 instead of raising 'ower muckle'. Fixing changes IR widely: route through a checked runtime helper (e.g. __mdh_need) and measure instruction cost; IR diff must be limited to those sites.

Evidence fingerprint: `manual:v1:native-codegen-about-35-inline-allocations-are--fe9d384ed3969bce`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T06:15:02Z, deltic:manual run=verify-20261027T055825Z-a8428af6 verified_fix_run=fix-20261027T045241Z-2f69b43a verdict=close)

Verifier note: Manual verify: [lh luna] 85e9daf; 1500-char comma-split repro exited 0 and printed the too-big error; both focused tests passed (1 selected each); unchecked inline_split sc_final_ptr and compile_dict dict_alloc mutants failed their run.code assertions; 38 checked sites found, with only inline_repeat raw and null-checked.

## Notes
