# MDH-BUG-FLU-00190 — Native: a list of function values never equals itself ([len] == [len] is nae)

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T15:32:28Z
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
- **State history:** Open (2026-10-26T15:32:28Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T22:26:17Z, deltic:auto role=fix run=fix-20261026T222520Z-5a4a6b82 branch=task/bug-MDH-BUG-FLU-00190-run-fix-20261026T222520Z-5a4a6b82 code=e312295 gate=manual) -> Closed (2026-10-27T02:35:24Z, deltic:manual role=verify run=verify-20261027T013251Z-0fb63f19 verified_fix_run=fix-20261026T222520Z-5a4a6b82 verdict=close)

## Observation

Arthur decision round 4 (2026-10-26): [len] == [len] should be aye. Natively, single function values compare equal (len == len is aye after MDH-BUG-FLU-00162) but list equality on lists holding function values returns nae, unlike the interpreter. Fix element equality for function values in the native list/dict equality path; parity test.

Evidence fingerprint: `manual:v1:native-a-list-of-function-values-never-equals-i-88640b9ce7b20118`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T02:35:24Z, deltic:manual run=verify-20261027T013251Z-0fb63f19 verified_fix_run=fix-20261026T222520Z-5a4a6b82 verdict=close)

Verifier note: Manual verify: [le luna] fix e312295 adds the assertion; llvm_builtin_values::lists_and_dicts_of_function_values_compare_like_the_interpreter passed. Reversing the builtin wrapper cache from 9025063 made the test fail native-vs-interpreter equality: native began “nae\nnae” where expected was “aye\naye”.

## Notes
