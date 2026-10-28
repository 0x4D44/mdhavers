# MDH-BUG-FLU-00200 — No gate builds --all-targets or the cfg(coverage) tests, so those builds rot unseen

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** build
- **Raised:** 2026-10-27T02:47:42Z
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
- **Resolution:** wont-fix
- **State history:** Open (2026-10-27T02:47:42Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Closed (2026-10-28T14:09:20Z, deltic:manual host=flux verdict=close resolution=wont-fix)

## Observation

No check that runs before or after a landing builds the `--cfg coverage` test set or `--all-targets`, so builds that only those configurations see rot unnoticed. Two cases in two days (2026-10-26/27):
- MDH-BUG-FLU-00194: tests/wasm_tak_drap_strings.rs (added 11d6d03) used wasm_runner without a feature gate; `cargo build --all-targets` without wasm_runner failed (E0432/E0433), found by a verifier, not a gate.
- The cfg(coverage) lib tests broke on 166's arity change (see the companion bug raised at the same time); found by chance while checking a verifier note.
Independent reviewer's residual on MDH-BUG-FLU-00134 (2026-10-27): "no gate runs RUSTFLAGS=\"--cfg coverage\" cargo check ... --lib --tests or cargo check --all-targets".

Expected: the trunk/drain gate (or .deltic-integrate.toml fallback) includes a compile-only check of both configurations (`cargo check --all-targets` and `RUSTFLAGS='--cfg coverage' cargo check --lib --tests`), cheap relative to the test suite.
Changing gate policy is a configuration decision; see the configure-integration skill.

Evidence fingerprint: `manual:v1:no-gate-builds-all-targets-or-the-cfg-coverage--db242f4011e2a05d`


## Fix

<unfixed — raised only>

## Notes

### Close note (2026-10-28T14:09:20Z, deltic:manual host=flux verdict=close resolution=wont-fix)

Close reason: Arthur 2026-10-28 14:08: skip. The gate stays as is; the cost of building --all-targets and the cfg(coverage) tests on every landing isn't worth it. lessons_learnt.md (Makefile:coverage) tells lanes to run the coverage build after semantic or Value-layout changes.
