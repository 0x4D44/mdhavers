# MDH-BUG-FLU-00139 — Native tak/drap reject strings that the interpreter accepts

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T06:49:39Z
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
- **State history:** Open (2026-10-25T06:49:39Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T07:13:19Z, deltic:auto role=fix run=fix-20261025T065500Z-6e69a66c branch=task/bug-MDH-BUG-FLU-00139-run-fix-20261025T065500Z-6e69a66c code=bfaa64f gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-33b664f6 verified_fix_run=fix-20261025T065500Z-6e69a66c verdict=close)

## Observation

Found by lane N8 2026-10-25 (executed) in an examples sweep: examples/builtins/list_functions.braw exits 1 natively at tak('Hello World', 5) with "Och! Type error in 'tak': got string"; the interpreter accepts strings (first/last n characters) and exits 0. drap likely fails the same way. Fix the native inline/runtime tak and drap to handle strings like the interpreter.

Evidence fingerprint: `manual:v1:native-tak-drap-reject-strings-that-the-interpreter-accepts`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-33b664f6 verified_fix_run=fix-20261025T065500Z-6e69a66c verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=bfaa64f; list_functions.braw interpreter/native both returned character lists; tak_and_drap_take_strings_as_character_lists selected and passed 1/1; mutant restoring list-only tag gate failed assertion at tests/llvm_hardening.rs:63: Some(1) vs Some(0), stderr Type error in tak.

## Notes
