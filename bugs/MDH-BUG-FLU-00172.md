# MDH-BUG-FLU-00172 — JS: log, tan and similar float builtins differ from the interpreter in the last digit

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-26T05:03:59Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T050440Z-8e62b090
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00172-run-fix-20261026T050440Z-8e62b090
- **Owner base:** 0ce2a5bc5cf9ba91c13f5b84148dcd8cfea0ba99
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T05:04:40Z
- **Owner until:** 2026-10-26T09:04:40Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T05:03:59Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane H1 2026-10-26 (executed): `blether log(3)` and `blether tan(2.5)` print 1.0986122886681098 and -0.7470222972386602 in the interpreter and native, but 1.0986122886681096 and -0.7470222972386603 in JS (Math.log/Math.tan vs Rust libm). Decide: accept (document that JS floats may differ in the last place) or route these through a shared implementation; goldens should avoid printing full-precision transcendental results.

Evidence fingerprint: `manual:v1:js-log-tan-and-similar-float-builtins-differ-fr-05cee71f84399255`


## Fix

<unfixed — raised only>

## Notes
