# MDH-BUG-FLU-00061 — LLVM toss_in/heave_oot store their result into a boxed variable's raw storage

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-22T23:53:42Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T171637Z-bc2a88ab
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00061-run-verify-20261026T171637Z-bc2a88ab
- **Owner base:** b5fbdef97d31e05c1d4f8b997c745938b368bb11
- **Owner fingerprint:** sha256:b9492e47ef59555d81be8e5e3f825744bf487471bc75fcbd6bf934d1d6ca93ff
- **Owner since:** 2026-10-26T17:16:37Z
- **Owner until:** 2026-10-26T21:16:37Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:52:01Z, deltic:auto role=fix run=fix-20261023T035155Z-040c74c9 branch=task/bug-MDH-BUG-FLU-00061-run-fix-20261023T035155Z-040c74c9 code=6029d44 gate=manual)

## Observation

Inferred by the L6 worker (code reading, 2026-10-23): codegen.rs ~17068 stores the toss_in/heave_oot result straight into the variable's alloca even when the variable is boxed (captured by a closure), so the closure and the outer scope diverge or the box pointer is overwritten.

## Fix

<unfixed — raised only>

## Notes
