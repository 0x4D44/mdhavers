# MDH-BUG-FLU-00195 — Interpreter has no clear builtin, though native does

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-26T19:24:07Z
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
- **State history:** Open (2026-10-26T19:24:07Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T00:56:38Z, deltic:auto role=fix run=fix-20261026T214549Z-1a7fa5b0 branch=task/bug-MDH-BUG-FLU-00195-run-fix-20261026T214549Z-1a7fa5b0 code=97fb62e gate=manual)

## Observation

Found by lane R5 2026-10-26 (executed): ken zs = [1] then blether len(clear(zs)) works natively but fails in the interpreter (mdhavers run) as an undefined name. Decision round 4 keeps native clear list-only; add a matching list-only clear to the interpreter (and JS/WASM if missing), with parity tests.

Evidence fingerprint: `manual:v1:interpreter-has-no-clear-builtin-though-native-does`


## Fix

<unfixed — raised only>

## Notes
