# MDH-BUG-FLU-00047 — wasm_runner host does f64 arithmetic, nils type errors and never frees its store

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** wasm
- **Raised:** 2026-10-22T23:24:10Z
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T04:18:06Z, deltic:auto role=fix run=fix-20261023T041800Z-d3230ec2 branch=task/bug-MDH-BUG-FLU-00047-run-fix-20261023T041800Z-d3230ec2 code=885d33d gate=manual)

## Observation

Inferred: wasm_runner.rs:573-594 numeric_binop via f64 (7/2 = 3.5, >2^53 precision loss); type errors return handle 0; HostStore grows forever (:61); dicts are HashMap (print order varies).

## Fix

<unfixed — raised only>

## Notes
