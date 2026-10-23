# MDH-BUG-FLU-00081 — Native: toss_in/heave_oot overwrite the storage of boxed variables

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T03:06:28Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T035143Z-f07eb672
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00081-run-fix-20261023T035143Z-f07eb672
- **Owner base:** 235a458de45868c9298cf9e392041a4b298aefeb
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T03:51:43Z
- **Owner until:** 2026-10-23T07:51:43Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:06:28Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the LLVM lane 2026-10-23 (code reading, src/llvm/codegen.rs ~17068): the list push/pop helpers store their result over the variable's storage slot without respecting boxing, so a boxed (captured or shadow-synced) list variable can be clobbered. Needs a native repro to confirm the user-visible symptom.

## Fix

<unfixed — raised only>

## Notes
