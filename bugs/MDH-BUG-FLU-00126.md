# MDH-BUG-FLU-00126 — JS: inst == literal compiles to === and skips the instance's equality overload

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** js
- **Raised:** 2026-10-24T05:44:33Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T225713Z-e9e85b59
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00126-run-verify-20261026T225713Z-e9e85b59
- **Owner base:** 6cca1b9724e089b14af0018739faf304b5548a5a
- **Owner fingerprint:** sha256:a2ec460bf06df2d3cdedf730c0b5f8929a09d06d69683071e0563905cfe9bdce
- **Owner since:** 2026-10-26T22:57:13Z
- **Owner until:** 2026-10-27T02:57:13Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T05:44:33Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T06:25:22Z, deltic:auto role=fix run=fix-20261024T062516Z-7c117884 branch=task/bug-MDH-BUG-FLU-00126-run-fix-20261024T062516Z-7c117884 code=ad9b34c gate=manual)

## Observation

Found by lane J8 (f83f560) 2026-10-24: == against a string or bool literal still compiles to a raw === (an older fast path), so an instance's __same_as__-style overload is not called; the interpreter calls it. Also: every obj.m() now goes through __havers.callM (extra call + rest array), not benchmarked.

Evidence fingerprint: `manual:v1:js-inst-literal-compiles-to-and-skips-the-insta-435c3b6e402ec2ca`


## Fix

<unfixed — raised only>

## Notes
