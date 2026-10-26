# MDH-BUG-FLU-00128 — Native: fer over a dict or creel iterates the handle as a list instead of raising

- **State:** Closed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-24T22:20:47Z
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
- **State history:** Open (2026-10-24T22:20:47Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T22:42:33Z, deltic:auto role=fix run=fix-20261024T222513Z-7529395c branch=task/bug-MDH-BUG-FLU-00128-run-fix-20261024T222513Z-7529395c code=ad4c55f gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-c7ed1769 verified_fix_run=fix-20261024T222513Z-7529395c verdict=close)

## Observation

Found by lane N5 2026-10-24 (executed on the old native build): compile_for_iterable in src/llvm/codegen.rs (~27430) sends every non-string value to compile_for_list_impl, so fer x in a dict or creel reads the handle as a list. Repro: ken c = creel(["a","b","c"])  fer x in c { blether x } -> native prints a float, <object> x6, naething; the interpreter raises "Cannae iterate ower a dict/creel". With tombstoned creels (bug 91 fix) it can read further and crash. Fix: raise the interpreter error for dict/creel handles at run time.

Evidence fingerprint: `manual:v1:native-fer-over-a-dict-or-creel-iterates-the-ha-dd3335514900414a`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-c7ed1769 verified_fix_run=fix-20261024T222513Z-7529395c verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=ad4c55f; interpreter/native repro both raised Cannae iterate ower a creel at line 2; fer_over_a_dict tests passed 2/2; mutant reversed list-tag gate and both test assertions failed (garbage iteration/exit None vs expected errors).

## Notes
