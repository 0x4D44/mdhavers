# MDH-BUG-FLU-00209 — cfg(coverage) test build has 12 non-network failures (EOF input cases, LLVM wrong-arity tests)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-28T15:23:04Z
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
- **State history:** Open (2026-10-28T15:23:04Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-28T19:37:02Z, deltic:auto role=fix run=fix-20261028T174303Z-8078ac19 branch=task/bug-MDH-BUG-FLU-00209-run-fix-20261028T174303Z-8078ac19 code=5b71ba5 gate=manual)

## Observation

The `--cfg coverage` test build has 12 failures in code unrelated to recent changes.

Observed by the 205 fix worker (Codex gpt-6.1-sol), 2026-10-28, on 0.3.106 plus the 205 fix. The fix touches only the interpreter's module loader, so these are inferred to be on trunk too; not re-run on bare trunk by the lead.

Command: `RUSTFLAGS="--cfg coverage" CARGO_TARGET_DIR=<scratch> cargo test --no-default-features --features cli,llvm,native --lib --tests --no-fail-fast -- --test-threads=6`. Result: 2508 passed, 26 failed, 1 ignored. 14 of the 26 are socket/TLS binds denied by the Codex sandbox (PermissionDenied) and are probably environmental. The other 12:
- 5 EOF/input cases (eof_helpers 4, cli_coverage 1) that conflict with coverage builds disabling `speir` input. These probably date from the round 4 EOF-helpers change (191).
- 7 LLVM coverage tests that still expect calls with missing required arguments to compile (the same pattern MDH-BUG-FLU-00199 fixed in four lib tests).

Full per-target table: the 205 worker's report (.scratch/MDH-BUG-FLU-00205-report.md in that lane, now retired).

Expected: the coverage build is green apart from tests that need network access. MDH-BUG-FLU-00200 (gate these builds) was closed wont-fix, so nothing catches this automatically; lanes run it by hand per lessons_learnt.md.

Evidence fingerprint: `manual:v1:cfg-coverage-test-build-has-12-non-network-fail-4e3be446f2b16617`


## Fix

<unfixed — raised only>

## Notes
