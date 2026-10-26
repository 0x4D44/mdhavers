# MDH-BUG-FLU-00115 — stdlib prelude pad_left/pad_right shadow the native builtins with a stricter, different signature

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-24T03:17:12Z
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
- **State history:** Open (2026-10-24T03:17:12Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T21:56:47Z, deltic:auto role=fix run=fix-20261024T213341Z-cebe72f1 branch=task/bug-MDH-BUG-FLU-00115-run-fix-20261024T213341Z-cebe72f1 code=f207ce5 gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-daf70e4d verified_fix_run=fix-20261024T213341Z-cebe72f1 verdict=close)

## Observation

Found by lane J6 2026-10-24 (executed): `target/debug/mdhavers run` loads stdlib/prelude.braw, whose pad_left/pad_right take exactly 3 args, so pad_right("ab", 5) fails with "Function 'pad_right' expects 3 arguments but ye gave it 2", and pad_right("ab", 5, 1) returns "ab111". The native builtins (src/interpreter.rs define_natives) accept 2-3 args and the library interpreter used by tests behaves differently from the CLI. Decide which signature wins and make the prelude match (or drop the prelude copies).

Evidence fingerprint: `manual:v1:stdlib-prelude-pad-left-pad-right-shadow-the-na-cc2b7f46afa8a877`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-daf70e4d verified_fix_run=fix-20261024T213341Z-cebe72f1 verdict=close)

Verifier note: Manual verify: [la luna] f207ce5; cli_prelude_does_not_shadow_native_pad_builtins selected 1 pass with `[ab ]`, `[ab***]`, `[ ab]`, `[000ab]`. Re-adding the old 3-arg prelude functions made the test fail its exit-code assertion; stderr says pad_right expects 3 args but got 2. Restored; source diff clean.

## Notes
