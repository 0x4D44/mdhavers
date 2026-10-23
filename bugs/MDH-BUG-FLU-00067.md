# MDH-BUG-FLU-00067 — Debug-build evaluator uses ~45 KB of native stack per mdhavers call

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-23T02:06:57Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T032310Z-c098bf45
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00067-run-fix-20261023T032310Z-c098bf45
- **Owner base:** dace99a9d262cf372e780b118599f8ba1e5961cb
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:23:10Z
- **Owner until:** 2026-10-23T07:23:10Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Measured by the interpreter lane 2026-10-23: a debug build reaches only ~165 nested mdhavers calls on the 8 MB macOS main thread (96 with hae_a_bash per level) before the new StackOverflow guard fires; a 2 MB test thread fits a few dozen. The CLI runs programs on a 512 MiB thread except macOS graphics builds (raylib/GLFW need the main thread), which is the default Mac build. The evaluate/execute frames are very large (big match arms inlined); boxing or outlining bulky arms would raise the practical depth. The parser has the same issue (~22-25 KB per nesting level in debug, limit set to 64).

## Fix

<unfixed — raised only>

## Notes
