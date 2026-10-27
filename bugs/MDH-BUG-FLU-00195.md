# MDH-BUG-FLU-00195 — Interpreter has no clear builtin, though native does

- **State:** Closed
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
- **State history:** Open (2026-10-26T19:24:07Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T00:56:38Z, deltic:auto role=fix run=fix-20261026T214549Z-1a7fa5b0 branch=task/bug-MDH-BUG-FLU-00195-run-fix-20261026T214549Z-1a7fa5b0 code=97fb62e gate=manual) -> Closed (2026-10-27T02:35:24Z, deltic:manual role=verify run=verify-20261027T013251Z-cfbcc6bf verified_fix_run=fix-20261026T214549Z-1a7fa5b0 verdict=close)

## Observation

Found by lane R5 2026-10-26 (executed): ken zs = [1] then blether len(clear(zs)) works natively but fails in the interpreter (mdhavers run) as an undefined name. Decision round 4 keeps native clear list-only; add a matching list-only clear to the interpreter (and JS/WASM if missing), with parity tests.

Evidence fingerprint: `manual:v1:interpreter-has-no-clear-builtin-though-native-does`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T02:35:24Z, deltic:manual run=verify-20261027T013251Z-cfbcc6bf verified_fix_run=fix-20261026T214549Z-1a7fa5b0 verdict=close)

Verifier note: Manual verify: [le luna] fix 97fb62e; repro `clear([1])` printed 0; golden_tests_interpreter passed. Reversing the interpreter registration made it fail the fixture assertion: list_clear.braw reported UndefinedVariable { name: "clear" }.

## Notes
