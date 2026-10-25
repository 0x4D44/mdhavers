# MDH-BUG-FLU-00154 — Native modules and --emit-llvm output hard-code the x86_64-linux target triple and data layout

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-25T22:16:12Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261025T230506Z-37196da3
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00154-run-fix-20261025T230506Z-37196da3
- **Owner base:** ce8941947a4674f55a88360030d631b9820621fc
- **Owner fingerprint:** -
- **Owner since:** 2026-10-25T23:05:06Z
- **Owner until:** 2026-10-26T03:05:06Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-25T22:16:12Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane U4 2026-10-25 (observed on arm64 macOS, LLVM 15 and 19): src/llvm/codegen.rs:799 sets target triple x86_64-unknown-linux-gnu and an x86 data layout on every module, whatever the host. Native builds still produce working arm64 binaries (the triple is overridden later or tolerated), but --emit-llvm output is misleading and any target-specific optimisation runs against the wrong layout. Fix: use TargetMachine::get_default_triple() and the target machine's data layout, keeping i64 8-byte alignment where the runtime relies on it.

Evidence fingerprint: `manual:v1:native-modules-and-emit-llvm-output-hard-code-t-2eeb23291cb22339`


## Fix

<unfixed — raised only>

## Notes
