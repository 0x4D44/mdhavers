# MDH-BUG-FLU-00081 — Native: toss_in/heave_oot overwrite the storage of boxed variables

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T03:06:28Z
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
- **State history:** Open (2026-10-23T03:06:28Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Found by the LLVM lane 2026-10-23 (code reading, src/llvm/codegen.rs ~17068): the list push/pop helpers store their result over the variable's storage slot without respecting boxing, so a boxed (captured or shadow-synced) list variable can be clobbered. Needs a native repro to confirm the user-visible symptom.

## Fix

<unfixed — raised only>

## Notes
