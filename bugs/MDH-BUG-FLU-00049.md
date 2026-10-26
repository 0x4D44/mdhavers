# MDH-BUG-FLU-00049 — LLVM try/catch leaves dangling jmp_buf on gie/brak and grows the stack in loops

- **State:** Fixed
- **Priority:** Should
- **Severity:** High
- **Area:** llvm
- **Raised:** 2026-10-22T23:24:11Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T164142Z-219641dc
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00049-run-verify-20261026T164142Z-219641dc
- **Owner base:** b5d5a9fa852316d9d5ef58b28e50e2449fbe0bad
- **Owner fingerprint:** sha256:a9fa5658bf2dd4cc61d9813d0b09078176627a4f64ad9b378caa5bcbf43eee36
- **Owner since:** 2026-10-26T16:41:42Z
- **Owner until:** 2026-10-26T20:41:42Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:04:43Z, deltic:auto role=fix run=fix-20261023T030437Z-98c48c60 branch=task/bug-MDH-BUG-FLU-00049-run-fix-20261023T030437Z-98c48c60 code=8e071f4 gate=manual)

## Observation

Inferred: return/break/continue emit no try_pop (codegen.rs:12278-12310); jmp_buf alloca not in the entry block (35255-35267); runtime try stack is global, fixed at 64 entries, silently drops overflow (mdh_runtime.c:9724-9760) and is shared across threads.

## Fix

<unfixed — raised only>

## Notes
