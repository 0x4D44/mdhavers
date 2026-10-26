# MDH-BUG-FLU-00188 — Native: __mdh_make_string returns a string with a NULL pointer on out-of-memory

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T11:40:50Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T122015Z-c681ecb3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00188-run-fix-20261026T122015Z-c681ecb3
- **Owner base:** 18d4631d79b84c4c82c5241f8661935d53c62389
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T12:20:15Z
- **Owner until:** 2026-10-26T16:20:15Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T11:40:50Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane N11 2026-10-26 (code read): __mdh_obj_strdup in runtime/mdh_runtime.c returns NULL when allocation fails and __mdh_make_string wraps it into a string value with a NULL pointer, which later code dereferences. Raising there would longjmp through Rust callers of __mdh_make_string, so this needs a status-returning variant for the Rust runtime (and for the remaining OOM-only raises from Rust listed in the MDH-BUG-FLU-00183 fix: __mdh_dict_set, __mdh_list_push, *_set_owned, push_owned, bytes_new) and a raise for compiled callers.

Evidence fingerprint: `manual:v1:native-mdh-make-string-returns-a-string-with-a--abc6f18d441da457`


## Fix

<unfixed — raised only>

## Notes
