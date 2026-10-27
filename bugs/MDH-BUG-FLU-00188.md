# MDH-BUG-FLU-00188 — Native: __mdh_make_string returns a string with a NULL pointer on out-of-memory

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T11:40:50Z
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
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-26T11:40:50Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T14:03:11Z, deltic:auto role=fix run=fix-20261026T122015Z-c681ecb3 branch=task/bug-MDH-BUG-FLU-00188-run-fix-20261026T122015Z-c681ecb3 code=6cf14c5 gate=manual) -> Open (2026-10-27T02:35:24Z, deltic:manual role=verify run=verify-20261027T013251Z-2302915c verified_fix_run=fix-20261026T122015Z-c681ecb3 verdict=doubt reason=reopen-error-then-after-but-running-out-of-memory-making-a-string-raises-the-too)

## Observation

Found by lane N11 2026-10-26 (code read): __mdh_obj_strdup in runtime/mdh_runtime.c returns NULL when allocation fails and __mdh_make_string wraps it into a string value with a NULL pointer, which later code dereferences. Raising there would longjmp through Rust callers of __mdh_make_string, so this needs a status-returning variant for the Rust runtime (and for the remaining OOM-only raises from Rust listed in the MDH-BUG-FLU-00183 fix: __mdh_dict_set, __mdh_list_push, *_set_owned, push_owned, bytes_new) and a raise for compiled callers.

Evidence fingerprint: `manual:v1:native-mdh-make-string-returns-a-string-with-a--abc6f18d441da457`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T02:35:24Z, deltic:manual run=verify-20261027T013251Z-2302915c verified_fix_run=fix-20261026T122015Z-c681ecb3 verdict=doubt)

Verifier note: Manual verify: [le luna] 6cf14c5; fixed env_get(MDH_BIG) repro returned the too-big error then “after”. But running_out_of_memory_making_a_string_raises_the_too_big_error and shared compiled_string_copies_raise_the_too_big_error both stayed green after removing only __mdh_make_string’s NULL guard; they miss this C hunk.

## Notes
