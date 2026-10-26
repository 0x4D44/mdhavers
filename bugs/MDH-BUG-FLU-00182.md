# MDH-BUG-FLU-00182 — WASM golden parity times out on showcase_blether_lisp under load

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** tests
- **Raised:** 2026-10-26T08:37:21Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-b2ab774c
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00182-run-verify-20261026T233955Z-b2ab774c
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:330255b60f2b69df55da1840e508245bd02904f5432e3293cfc56328ae136835
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-26T08:37:21Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T11:36:30Z, deltic:auto role=fix run=fix-20261026T091430Z-25c8deaa branch=task/bug-MDH-BUG-FLU-00182-run-fix-20261026T091430Z-25c8deaa code=e1b9d8f gate=manual)

## Observation

Found by lead 2026-10-26 08:35 (executed): tests/wasm_golden_parity.rs and tests/wasm_validity.rs give each golden 30s. showcase_blether_lisp under the debug wasm-run takes ~53e9 instructions (~32s at load average ~170 on the shared Mac; passes at load ~60), so both suites fail on a busy box although the output matches the expected file. Make the per-case limit robust: scale it by case weight, give heavy showcases a longer limit, or measure instructions rather than wall time. Do not drop the case.

Evidence fingerprint: `manual:v1:wasm-golden-parity-times-out-on-showcase-blether-lisp-under-load`


## Fix

<unfixed — raised only>

## Notes
