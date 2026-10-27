# MDH-BUG-FLU-00196 — RC borrowing follow-ups: dead-object checks, libc allow-list, regression probes

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T19:42:46Z
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
- **State history:** Open (2026-10-26T19:42:46Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-27T07:49:55Z, deltic:auto role=fix run=fix-20261027T070615Z-7d05bcb2 branch=task/bug-MDH-BUG-FLU-00196-run-fix-20261027T070615Z-7d05bcb2 code=c3d58cf gate=manual)

## Observation

Accepted at review of the R5 RC borrowing round (Opus reviewer, 2026-10-26; source review plus 10 probe programs, all correct): (1) MDH_RC_NOFREE=1 only catches some early frees, since an elided retain leaves nothing to touch the dead object; in debug builds make __mdh_string_concat and the index/field readers check MDH_OBJ_DEAD. (2) rc_call_effect (src/llvm/codegen/rc.rs ~:2044) treats any non-__mdh_ external as harmless except qsort/free/realloc; switch to an explicit allow-list with 'may run code' as the default. (3) Comment that __mdh_list_push may move the items buffer. (4) Add the reviewer's probes as regression tests: (ys[0] = "a"+"b") + zap(); s = s + inner() with a capturing lambda; mg + setmg() across a fetch.

Evidence fingerprint: `manual:v1:rc-borrowing-follow-ups-dead-object-checks-libc-9af0bcc28a2d7150`


## Fix

<unfixed — raised only>

## Notes
