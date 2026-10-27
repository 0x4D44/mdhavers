# MDH-BUG-FLU-00003 — json_parse on deeply nested input aborts the process (unbounded recursion)

- **State:** Closed
- **Priority:** Should
- **Severity:** High
- **Area:** interpreter
- **Raised:** 2026-10-22T23:24:06Z
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
- **State history:** Open (2026-10-22T23:24:06Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T01:48:25Z, deltic:auto role=fix run=fix-20261023T014819Z-d15847ec branch=task/bug-MDH-BUG-FLU-00003-run-fix-20261023T014819Z-d15847ec code=d84823a gate=manual) -> Open (2026-10-26T22:19:14Z, deltic:manual role=verify run=verify-20261026T214519Z-8989a73c verified_fix_run=fix-20261023T014819Z-d15847ec verdict=doubt reason=reopen-error-selected-pass-on-trunk-depth-guard-disabled-and-512mib-test-stack-s) -> Fixed (2026-10-27T04:50:12Z, deltic:auto role=fix run=fix-20261027T015222Z-2875ddc8 branch=task/bug-MDH-BUG-FLU-00003-run-fix-20261027T015222Z-2875ddc8 code=cbb72cf gate=manual) -> Closed (2026-10-27T05:22:54Z, deltic:manual role=verify run=verify-20261027T045159Z-00045f20 verified_fix_run=fix-20261027T015222Z-2875ddc8 verdict=close)

## Observation

Confirmed: json_parse("[" * 200000) -> stack overflow abort (interpreter.rs JSON parser ~12537-12645 recurses with no depth limit). Should raise a catchable error.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T22:19:14Z, deltic:manual run=verify-20261026T214519Z-8989a73c verified_fix_run=fix-20261023T014819Z-d15847ec verdict=doubt)

Verifier note: Manual verify: [verifier gpt-6.1-sol (Codex), recorded by lead] d84823a; json_parse_of_deep_nesting_is_a_catchable_error selected/pass on trunk; depth guard disabled and 512MiB test stack: same test still passes (caught then 1). Input has 200000 opening brackets but no closers, so EOF satisfies it. CLI repro also still passes mutant: test does not cover the fix.

### Verification summary (2026-10-27T05:22:54Z, deltic:manual run=verify-20261027T045159Z-00045f20 verified_fix_run=fix-20261027T015222Z-2875ddc8 verdict=close)

Verifier note: Manual verify: [lf luna] d84823a; 200k-open-bracket CLI repro returns catchable nested-too-deep error; json_parse_depth_guard_rejects_balanced_nesting_past_the_cap selected/pass; guard-removed mutant failed the 600-deep assertion (no error); restored pass; diff clean

## Notes
