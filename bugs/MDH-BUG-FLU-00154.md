# MDH-BUG-FLU-00154 — Native modules and --emit-llvm output hard-code the x86_64-linux target triple and data layout

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** llvm
- **Raised:** 2026-10-25T22:16:12Z
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
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-25T22:16:12Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T01:31:47Z, deltic:auto role=fix run=fix-20261025T230506Z-37196da3 branch=task/bug-MDH-BUG-FLU-00154-run-fix-20261025T230506Z-37196da3 code=c5abe17 gate=manual) -> Open (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-2c133d22 verified_fix_run=fix-20261025T230506Z-37196da3 verdict=doubt reason=reopen-fails-when-the-triple-is-reverted-a-layout-only-mutant-using-the-old-x86)

## Observation

Found by lane U4 2026-10-25 (observed on arm64 macOS, LLVM 15 and 19): src/llvm/codegen.rs:799 sets target triple x86_64-unknown-linux-gnu and an x86 data layout on every module, whatever the host. Native builds still produce working arm64 binaries (the triple is overridden later or tolerated), but --emit-llvm output is misleading and any target-specific optimisation runs against the wrong layout. Fix: use TargetMachine::get_default_triple() and the target machine's data layout, keeping i64 8-byte alignment where the runtime relies on it.

Evidence fingerprint: `manual:v1:native-modules-and-emit-llvm-output-hard-code-t-2eeb23291cb22339`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-2c133d22 verified_fix_run=fix-20261025T230506Z-37196da3 verdict=doubt)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] c5abe17; emitted_ir_uses_the_host_default_triple fails when the triple is reverted; a layout-only mutant using the old x86 layout kept the test green (1 selected). Test asserts only that a datalayout line exists, not the host layout.

## Notes
