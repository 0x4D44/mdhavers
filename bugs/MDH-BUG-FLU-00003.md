# MDH-BUG-FLU-00003 — json_parse on deeply nested input aborts the process (unbounded recursion)

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261027T015222Z-2875ddc8
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00003-run-fix-20261027T015222Z-2875ddc8
- **Owner base:** 709f64b306bed2515cd7f543dba1afa2d290d7df
- **Owner fingerprint:** -
- **Owner since:** 2026-10-27T01:52:22Z
- **Owner until:** 2026-10-27T05:52:22Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=1, indeterminate=0
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:25Z, deltic:auto role=fix run=fix-20261023T014819Z-d15847ec branch=task/bug-MDH-BUG-FLU-00003-run-fix-20261023T014819Z-d15847ec code=d84823a gate=manual) -> Open (2026-10-26T22:19:14Z, deltic:manual role=verify run=verify-20261026T214519Z-8989a73c verified_fix_run=fix-20261023T014819Z-d15847ec verdict=doubt reason=reopen-error-selected-pass-on-trunk-depth-guard-disabled-and-512mib-test-stack-s)

## Observation

Confirmed: json_parse("[" * 200000) -> stack overflow abort (interpreter.rs JSON parser ~12537-12645 recurses with no depth limit). Should raise a catchable error.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:19:14Z, deltic:manual run=verify-20261026T214519Z-8989a73c verified_fix_run=fix-20261023T014819Z-d15847ec verdict=doubt)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] d84823a; json_parse_of_deep_nesting_is_a_catchable_error selected/pass on trunk; depth guard disabled and 512MiB test stack: same test still passes (caught then 1). Input has 200000 opening brackets but no closers, so EOF satisfies it. CLI repro also still passes mutant: test does not cover the fix.

## Notes
