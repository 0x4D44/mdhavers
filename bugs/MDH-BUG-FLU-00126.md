# MDH-BUG-FLU-00126 — JS: inst == literal compiles to === and skips the instance's equality overload

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T05:44:33Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261024T062516Z-7c117884
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00126-run-fix-20261024T062516Z-7c117884
- **Owner base:** ad9b34c251f683f796cc261e54c3977a57c1ff1a
- **Owner fingerprint:** -
- **Owner since:** 2026-10-24T06:25:16Z
- **Owner until:** 2026-10-24T10:25:16Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T05:44:33Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane J8 (f83f560) 2026-10-24: == against a string or bool literal still compiles to a raw === (an older fast path), so an instance's __same_as__-style overload is not called; the interpreter calls it. Also: every obj.m() now goes through __havers.callM (extra call + rest array), not benchmarked.

Evidence fingerprint: `manual:v1:js-inst-literal-compiles-to-and-skips-the-insta-435c3b6e402ec2ca`


## Fix

<unfixed — raised only>

## Notes
