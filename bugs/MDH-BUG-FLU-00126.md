# MDH-BUG-FLU-00126 — JS: inst == literal compiles to === and skips the instance's equality overload

- **State:** Open
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
- **State history:** Open (2026-10-24T05:44:33Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane J8 (f83f560) 2026-10-24: == against a string or bool literal still compiles to a raw === (an older fast path), so an instance's __same_as__-style overload is not called; the interpreter calls it. Also: every obj.m() now goes through __havers.callM (extra call + rest array), not benchmarked.

Evidence fingerprint: `manual:v1:js-inst-literal-compiles-to-and-skips-the-insta-435c3b6e402ec2ca`


## Fix

<unfixed — raised only>

## Notes
