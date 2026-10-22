# MDH-BUG-FLU-00061 — LLVM toss_in/heave_oot store their result into a boxed variable's raw storage

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-22T23:53:42Z
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
- **State history:** Open (2026-10-22T23:53:42Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred by the L6 worker (code reading, 2026-10-23): codegen.rs ~17068 stores the toss_in/heave_oot result straight into the variable's alloca even when the variable is boxed (captured by a closure), so the closure and the outer scope diverge or the box pointer is overwritten.

## Fix

<unfixed — raised only>

## Notes
