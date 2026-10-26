# MDH-BUG-FLU-00126 — JS: inst == literal compiles to === and skips the instance's equality overload

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T05:44:33Z
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
- **State history:** Open (2026-10-24T05:44:33Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T06:25:22Z, deltic:auto role=fix run=fix-20261024T062516Z-7c117884 branch=task/bug-MDH-BUG-FLU-00126-run-fix-20261024T062516Z-7c117884 code=ad9b34c gate=manual) -> Closed (2026-10-26T23:39:42Z, deltic:manual role=verify run=verify-20261026T225713Z-e9e85b59 verified_fix_run=fix-20261024T062516Z-7c117884 verdict=close)

## Observation

Found by lane J8 (f83f560) 2026-10-24: == against a string or bool literal still compiles to a raw === (an older fast path), so an instance's __same_as__-style overload is not called; the interpreter calls it. Also: every obj.m() now goes through __havers.callM (extra call + rest array), not benchmarked.

Evidence fingerprint: `manual:v1:js-inst-literal-compiles-to-and-skips-the-insta-435c3b6e402ec2ca`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:39:42Z, deltic:manual run=verify-20261026T225713Z-e9e85b59 verified_fix_run=fix-20261024T062516Z-7c117884 verdict=close)

Verifier note: Manual verify: [lb luna] ad9b34c (implementation aa6444f); compiler::test_literal_equality_routes_through_overload_helpers and js_instance_equality_overload_runs_against_literals passed; old OR fast path failed both, JS returned `nae` instead of `same-called`.

## Notes
