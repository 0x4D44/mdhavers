# MDH-BUG-FLU-00172 — JS: log, tan and similar float builtins differ from the interpreter in the last digit

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-26T05:03:59Z
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
- **State history:** Open (2026-10-26T05:03:59Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T05:56:53Z, deltic:auto role=fix run=fix-20261026T050440Z-8e62b090 branch=task/bug-MDH-BUG-FLU-00172-run-fix-20261026T050440Z-8e62b090 code=ec9e931 gate=manual) -> Open (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-05513f9c verified_fix_run=fix-20261026T050440Z-8e62b090 verdict=doubt reason=reopen-regression-covers-the-documented-tolerance-behavior-so-test-does-not-cove) -> Fixed (2026-10-27T04:51:07Z, deltic:auto role=fix run=fix-20261027T015317Z-0dd4b00f branch=task/bug-MDH-BUG-FLU-00172-run-fix-20261027T015317Z-0dd4b00f code=895148e gate=manual) -> Closed (2026-10-27T05:23:43Z, deltic:manual role=verify run=verify-20261027T045159Z-24a92ca1 verified_fix_run=fix-20261027T015317Z-0dd4b00f verdict=close)

## Observation

Found by lane H1 2026-10-26 (executed): `blether log(3)` and `blether tan(2.5)` print 1.0986122886681098 and -0.7470222972386602 in the interpreter and native, but 1.0986122886681096 and -0.7470222972386603 in JS (Math.log/Math.tan vs Rust libm). Decide: accept (document that JS floats may differ in the last place) or route these through a shared implementation; goldens should avoid printing full-precision transcendental results.

Evidence fingerprint: `manual:v1:js-log-tan-and-similar-float-builtins-differ-fr-05cee71f84399255`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-05513f9c verified_fix_run=fix-20261026T050440Z-8e62b090 verdict=doubt)

Verifier note: Manual verify: [lb luna] ec9e931 documents the accepted divergence; repro confirms interpreter log(3)/tan(2.5) differ from JS in the last digit. parity_js_maths_builtins_mdh_bug_flu_00100 passed 1/1 but only tests log(1)/tan(0); no regression covers the documented tolerance behavior, so test does not cover the fix.

### Verification summary (2026-10-27T05:23:43Z, deltic:manual run=verify-20261027T045159Z-24a92ca1 verified_fix_run=fix-20261027T015317Z-0dd4b00f verdict=close)

Verifier note: Manual verify: Independent Opus review (lead landed this): close+residual - deleting the docs hunk is not a meaningful mutant (docs are not executable); the bug was settled by the decision to compare rounded output, which 895148e enforces; meaningful fails-before is perturbing a JS Math result by >1e-9, which fails the assertion. Residual (test silently passes when node is missing) raised separately. Verifier: [lf luna] ec9e931/895148e; parity_js_transcendentals_agree_when_rounded_mdh_bug_flu_00172 passes and full outputs differ only in last place; removing the documentation hunk leaves the test green (1/1): test does not cover the fix; docs restored

## Notes
