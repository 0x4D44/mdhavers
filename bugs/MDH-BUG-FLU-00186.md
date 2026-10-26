# MDH-BUG-FLU-00186 — Native stack guard: prior-handler chaining gaps on macOS and untested paths

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T09:23:41Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261026T122015Z-8ae40c1a
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00186-run-fix-20261026T122015Z-8ae40c1a
- **Owner base:** 18d4631d79b84c4c82c5241f8661935d53c62389
- **Owner fingerprint:** -
- **Owner since:** 2026-10-26T12:20:15Z
- **Owner until:** 2026-10-26T16:20:15Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T09:23:41Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Follow-ups accepted at landing of MDH-BUG-FLU-00164 (Astra review pass 4, 2026-10-26): (1) Darwin strips SA_RESETHAND from the action sigaction returns, so a one-shot SIGSEGV/SIGBUS handler installed before the runtime runs on every delivery; and a kill()-sent SIGSEGV has the same siginfo as a real fault, so under a prior SIG_IGN it takes the default action (runtime/mdh_runtime.c __mdh_stack_chain comments). Investigate a Darwin-specific way to recover both, or document as permanent. (2) Add a compile-time __atomic_always_lock_free assertion for the one-shot claim flag. (3) Tests in tests/llvm_recursion_depth.rs do not cover concurrent one-shot claims or a prior handler whose sa_mask includes its own signal with SA_NODEFER; add them.

Evidence fingerprint: `manual:v1:native-stack-guard-prior-handler-chaining-gaps--aa120714aaf68491`


## Fix

<unfixed — raised only>

## Notes
