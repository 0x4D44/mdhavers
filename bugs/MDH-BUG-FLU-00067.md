# MDH-BUG-FLU-00067 — Debug-build evaluator uses ~45 KB of native stack per mdhavers call

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** interpreter
- **Raised:** 2026-10-23T02:06:57Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T224201Z-9851ac4e
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00067-run-verify-20261026T224201Z-9851ac4e
- **Owner base:** 5122a4de097a3cb949560c57a78e74948858cacb
- **Owner fingerprint:** sha256:3b00827cd98cecadc67eb600c998e91dbb34843f70c2b8b90c832011b6a04e9a
- **Owner since:** 2026-10-26T22:42:01Z
- **Owner until:** 2026-10-27T02:42:01Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T02:06:57Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:23:16Z, deltic:auto role=fix run=fix-20261023T032310Z-c098bf45 branch=task/bug-MDH-BUG-FLU-00067-run-fix-20261023T032310Z-c098bf45 code=a920344 gate=manual)

## Observation

Measured by the interpreter lane 2026-10-23: a debug build reaches only ~165 nested mdhavers calls on the 8 MB macOS main thread (96 with hae_a_bash per level) before the new StackOverflow guard fires; a 2 MB test thread fits a few dozen. The CLI runs programs on a 512 MiB thread except macOS graphics builds (raylib/GLFW need the main thread), which is the default Mac build. The evaluate/execute frames are very large (big match arms inlined); boxing or outlining bulky arms would raise the practical depth. The parser has the same issue (~22-25 KB per nesting level in debug, limit set to 64).

## Fix

<unfixed — raised only>

## Notes
