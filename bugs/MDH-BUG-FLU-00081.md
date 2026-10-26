# MDH-BUG-FLU-00081 — Native: toss_in/heave_oot overwrite the storage of boxed variables

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-23T03:06:28Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-9de98a0a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00081-run-verify-20261026T171637Z-9de98a0a
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:8e6d3cde07689fdffa361c5236adbb88ab169d2666b2ed4bb3bebb8c05ff6f7f
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-23T03:06:28Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:51:49Z, deltic:auto role=fix run=fix-20261023T035143Z-f07eb672 branch=task/bug-MDH-BUG-FLU-00081-run-fix-20261023T035143Z-f07eb672 code=6029d44 gate=manual)

## Observation

Found by the LLVM lane 2026-10-23 (code reading, src/llvm/codegen.rs ~17068): the list push/pop helpers store their result over the variable's storage slot without respecting boxing, so a boxed (captured or shadow-synced) list variable can be clobbered. Needs a native repro to confirm the user-visible symptom.

## Fix

<unfixed — raised only>

## Notes
