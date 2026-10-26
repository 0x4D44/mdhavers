# MDH-BUG-FLU-00111 — Native structural equality recurses on the C stack

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-24T02:39:05Z
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
- **State history:** Open (2026-10-24T02:39:05Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T03:04:29Z, deltic:auto role=fix run=fix-20261024T030424Z-533018ed branch=task/bug-MDH-BUG-FLU-00111-run-fix-20261024T030424Z-533018ed code=efb4bb2 gate=manual) -> Closed (2026-10-26T19:40:44Z, deltic:manual role=verify run=verify-20261026T182021Z-7c4d9e08 verified_fix_run=fix-20261024T030424Z-533018ed verdict=close)

## Observation

Found by the lead's review 2026-10-24 (read). __mdh_eq_in in runtime/mdh_runtime.c is cycle-safe but recursive, so comparing lists/dicts nested ~10^6 deep should crash natively; the interpreter's walker is iterative.

Evidence fingerprint: `manual:v1:native-structural-equality-recurses-on-the-c-stack`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T19:40:44Z, deltic:manual run=verify-20261026T182021Z-7c4d9e08 verified_fix_run=fix-20261024T030424Z-533018ed verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] fix=528b015 (history code efb4bb2 is version bump); 200k-deep interpreter/native repro both printed aye; deeply_nested_containers_compare_without_overflowing_the_stack passed 1/1; mutant restored recursive C equality and assertion failed on runtime stack-overflow error after 67.82s.

## Notes
