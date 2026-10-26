# MDH-BUG-FLU-00079 — Deeply nested acyclic list equality and long import chains are not covered by the stack guard

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** interpreter
- **Raised:** 2026-10-23T02:06:58Z
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
- **State history:** Open (2026-10-23T02:06:58Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T03:23:28Z, deltic:auto role=fix run=fix-20261023T032322Z-8648a049 branch=task/bug-MDH-BUG-FLU-00079-run-fix-20261023T032322Z-8648a049 code=a920344 gate=manual) -> Closed (2026-10-26T23:48:13Z, deltic:manual role=verify run=verify-20261026T224201Z-f73a8c2c verified_fix_run=fix-20261023T032322Z-8648a049 verdict=close)

## Observation

Found by the interpreter lane: list == recursion on very deep (non-cyclic) nesting has no depth bound, and module import chains recurse through fetch without the call-depth/stack guard.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T23:48:13Z, deltic:manual run=verify-20261026T224201Z-f73a8c2c verified_fix_run=fix-20261023T032322Z-8648a049 verdict=close)

Verifier note: Manual verify: LEAD: close - equality mutant fails assert; import-guard mutant reproduces the recorded overflow abort. Verifier: [la luna] a920344; 4 interpreter_robustness tests passed. A 1,000-depth equality cap made deep_list_equality_does_not_overflow_the_stack fail `assert!(same)`; removing load_module's stack guard/counter made long_import_chain abort with SIGABRT, not an assertion.

## Notes
