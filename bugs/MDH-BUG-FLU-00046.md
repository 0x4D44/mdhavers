# MDH-BUG-FLU-00046 — WASM audio import table mismatch (midi_pit_rin_roond vs midi_pit_tune)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** wasm
- **Raised:** 2026-10-22T23:24:10Z
- **Discovery source:** Agent
- **Owner:** deltic:manual
- **Owner role:** fix
- **Owner run:** fix-20261023T025758Z-3d1b8c52
- **Owner host:** flux
- **Owner branch:** task/bug-MDH-BUG-FLU-00046-run-fix-20261023T025758Z-3d1b8c52
- **Owner base:** 1a87831271f4a6301869d549c38730d5641b6577
- **Owner fingerprint:** -
- **Owner since:** 2026-10-23T02:57:58Z
- **Owner until:** 2026-10-23T06:57:58Z
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: compiler imports midi_pit_rin_roond (wasm_compiler.rs:194), runner registers midi_pit_tune (wasm_runner.rs:1227); instantiation fails.

## Fix

<unfixed — raised only>

## Notes
