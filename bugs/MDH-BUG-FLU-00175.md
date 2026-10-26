# MDH-BUG-FLU-00175 — Five examples use native-only builtins (str, sum, append) so the interpreter cannot run them

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** examples
- **Raised:** 2026-10-26T05:04:22Z
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
- **State history:** Open (2026-10-26T05:04:22Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T06:25:18Z, deltic:auto role=fix run=fix-20261026T050440Z-ffb3bf95 branch=task/bug-MDH-BUG-FLU-00175-run-fix-20261026T050440Z-ffb3bf95 code=e2547ee gate=manual)

## Observation

Found by lane H1 2026-10-26 (executed): examples fractals, game_2048, game_of_life, roguelike and sort_viz call str(3.0), sum([1,2,3]), append(l, 3) or term_width(), which exist natively (kept by bug 145) but are undefined in the interpreter, so `mdhavers run` exits 1. Natively they loop forever on closed stdin and flood output. Fix: use the interpreter's builtins (tae_string, sumaw, shove, ...) in the examples, or add these names to the interpreter if Arthur wants them in the language; make the interactive ones end cleanly at EOF (cf. bug 156).

Evidence fingerprint: `manual:v1:five-examples-use-native-only-builtins-str-sum--e1842501c371dd88`


## Fix

<unfixed — raised only>

## Notes
