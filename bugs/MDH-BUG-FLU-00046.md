# MDH-BUG-FLU-00046 — WASM audio import table mismatch (midi_pit_rin_roond vs midi_pit_tune)

- **State:** Fixed
- **Priority:** Should
- **Severity:** Medium
- **Area:** wasm
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** verify
- **Owner run:** verify-20261026T233955Z-6d81b49f
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00046-run-verify-20261026T233955Z-6d81b49f
- **Owner base:** 3a7321efd6af1e16d34dcd7fed8e7df5ad38c691
- **Owner fingerprint:** sha256:83f78f3b038e2dd96da4a14ff90642e2ae00724f20bb7e12b0127718878ab7d8
- **Owner since:** 2026-10-26T23:39:55Z
- **Owner until:** 2026-10-27T03:39:55Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:58:04Z, deltic:auto role=fix run=fix-20261023T025758Z-3d1b8c52 branch=task/bug-MDH-BUG-FLU-00046-run-fix-20261023T025758Z-3d1b8c52 code=53428a5 gate=manual)

## Observation

Inferred: compiler imports midi_pit_rin_roond (wasm_compiler.rs:194), runner registers midi_pit_tune (wasm_runner.rs:1227); instantiation fails.

## Fix

<unfixed — raised only>

## Notes
