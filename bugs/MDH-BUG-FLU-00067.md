# MDH-BUG-FLU-00067 — Debug-build evaluator uses ~45 KB of native stack per mdhavers call

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-23T02:06:57Z
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
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:23:16Z, deltic:auto role=fix run=fix-20261023T032310Z-c098bf45 branch=task/bug-MDH-BUG-FLU-00067-run-fix-20261023T032310Z-c098bf45 code=a920344 gate=manual) -> Closed (2026-10-27T02:35:24Z, deltic:manual role=verify run=verify-20261027T013251Z-2d6f9231 verified_fix_run=fix-20261023T032310Z-c098bf45 verdict=close)

## Observation

Measured by the interpreter lane 2026-10-23: a debug build reaches only ~165 nested mdhavers calls on the 8 MB macOS main thread (96 with hae_a_bash per level) before the new StackOverflow guard fires; a 2 MB test thread fits a few dozen. The CLI runs programs on a 512 MiB thread except macOS graphics builds (raylib/GLFW need the main thread), which is the default Mac build. The evaluate/execute frames are very large (big match arms inlined); boxing or outlining bulky arms would raise the practical depth. The parser has the same issue (~22-25 KB per nesting level in debug, limit set to 64).

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T02:35:24Z, deltic:manual run=verify-20261027T013251Z-2d6f9231 verified_fix_run=fix-20261023T032310Z-c098bf45 verdict=close)

Verifier note: Manual verify: [le luna] a920344; recursion_depth_on_an_8mib_thread_is_practical and recursion_depth_with_try_on_an_8mib_thread_is_practical passed (2 selected). A temporary 45 KiB frame pad made both fail their depth assertions at 139/400 and 51/200; restored, diff clean.

## Notes
