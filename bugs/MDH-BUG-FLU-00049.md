# MDH-BUG-FLU-00049 — LLVM try/catch leaves dangling jmp_buf on gie/brak and grows the stack in loops

- **State:** Open
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
- **State history:** Open (2026-10-22T23:24:11Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: return/break/continue emit no try_pop (codegen.rs:12278-12310); jmp_buf alloca not in the entry block (35255-35267); runtime try stack is global, fixed at 64 entries, silently drops overflow (mdh_runtime.c:9724-9760) and is shared across threads.

## Fix

<unfixed — raised only>

## Notes
