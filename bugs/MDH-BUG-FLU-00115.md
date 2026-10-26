# MDH-BUG-FLU-00115 — stdlib prelude pad_left/pad_right shadow the native builtins with a stricter, different signature

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-24T03:17:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-daf70e4d
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00115-run-verify-20261026T224201Z-daf70e4d
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:96c78f02a0499a10652e35a86a208a95abb0bb8f8b2811857094185b1ef3387a
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-24T03:17:12Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T21:56:47Z, deltic:auto role=fix run=fix-20261024T213341Z-cebe72f1 branch=task/bug-MDH-BUG-FLU-00115-run-fix-20261024T213341Z-cebe72f1 code=f207ce5 gate=manual)

## Observation

Found by lane J6 2026-10-24 (executed): `target/debug/mdhavers run` loads stdlib/prelude.braw, whose pad_left/pad_right take exactly 3 args, so pad_right("ab", 5) fails with "Function 'pad_right' expects 3 arguments but ye gave it 2", and pad_right("ab", 5, 1) returns "ab111". The native builtins (src/interpreter.rs define_natives) accept 2-3 args and the library interpreter used by tests behaves differently from the CLI. Decide which signature wins and make the prelude match (or drop the prelude copies).

Evidence fingerprint: `manual:v1:stdlib-prelude-pad-left-pad-right-shadow-the-na-cc2b7f46afa8a877`


## Fix

<unfixed — raised only>

## Notes
