# MDH-BUG-FLU-00048 — LLVM shadow caches are not invalidated on reassignment (stale lists, heap overflow, use-after-free)

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:04:31Z, deltic:auto role=fix run=fix-20261023T030425Z-04261bc7 branch=task/bug-MDH-BUG-FLU-00048-run-fix-20261023T030425Z-04261bc7 code=8e071f4 gate=manual)

## Observation

Inferred (high confidence; to confirm natively): int_shadows/list_ptr_shadows/string_len_shadows/string_cap_shadows are set per variable name and not reset on Expr::Assign (codegen.rs:12994-13220). Reassigned list -> writes to old list; string cap shadow -> in-place append into a read-only literal or smaller buffer; aliased in-place self-append -> realloc frees the alias (UAF); retyped variable -> strlen on an int.

## Fix

<unfixed — raised only>

## Notes
