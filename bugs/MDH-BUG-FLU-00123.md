# MDH-BUG-FLU-00123 — WASM closures: a second ken in the same scope makes a fresh cell instead of rebinding

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-24T04:51:08Z
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
- **State history:** Open (2026-10-24T04:51:08Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T06:14:36Z, deltic:auto role=fix run=fix-20261024T061429Z-9fbea918 branch=task/bug-MDH-BUG-FLU-00123-run-fix-20261024T061429Z-9fbea918 code=c436abd gate=manual)

## Observation

Documented divergence from lane W4 (WASM function values, 2026-10-24, see wrk_docs/2026.10.24 - DES - WASM function values and closures.md): closures created before a repeated ken x keep the old cell, while the interpreter rebinds in place. Also: WASM function values must be called with exactly their arity (no defaults), nested dae cannot have default parameters, and block expressions are supported only as lambda bodies.

Evidence fingerprint: `manual:v1:wasm-closures-a-second-ken-in-the-same-scope-ma-3ce057110420a511`


## Fix

<unfixed — raised only>

## Notes
