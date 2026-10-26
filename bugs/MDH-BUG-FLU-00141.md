# MDH-BUG-FLU-00141 — Native: repeat_char undefined in examples/stdlib/tui_demo.braw

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T06:49:48Z
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
- **State history:** Open (2026-10-25T06:49:48Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-25T07:13:29Z, deltic:auto role=fix run=fix-20261025T065500Z-e445faaa branch=task/bug-MDH-BUG-FLU-00141-run-fix-20261025T065500Z-e445faaa code=bfaa64f gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-922bea0a verified_fix_run=fix-20261025T065500Z-e445faaa verdict=close)

## Observation

Found by lane N8 2026-10-25 (executed): examples/stdlib/tui_demo.braw exits 1 natively with "'repeat_char' hasnae been defined yet at line 370"; the interpreter exits 0. Find where repeat_char comes from (stdlib module import or prelude) and why native does not see it.

Evidence fingerprint: `manual:v1:native-repeat-char-undefined-in-examples-stdlib-tui-demo-braw`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-922bea0a verified_fix_run=fix-20261025T065500Z-e445faaa verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=bfaa64f; tui_demo.braw completed in interpreter/native; module_function_shadowing_a_nested_imports_function_is_defined selected and passed 1/1; mutant removed declared_here snapshot and restored self.functions lookup, test failed assertion: rep undefined at line 6, exit Some(1) vs Some(0).

## Notes
