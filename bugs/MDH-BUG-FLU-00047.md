# MDH-BUG-FLU-00047 — wasm_runner host does f64 arithmetic, nils type errors and never frees its store

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** wasm
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-94fca55b
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00047-run-verify-20261026T233955Z-94fca55b
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:33d65e735c2b69e7ced12d9864e8d1952312c31f55c5865c19d995763035a758
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
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
