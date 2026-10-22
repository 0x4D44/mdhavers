# MDH-BUG-FLU-00046 — WASM audio import table mismatch (midi_pit_rin_roond vs midi_pit_tune)

- **State:** Open
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium)

## Observation

Inferred: compiler imports midi_pit_rin_roond (wasm_compiler.rs:194), runner registers midi_pit_tune (wasm_runner.rs:1227); instantiation fails.

## Fix

<unfixed — raised only>

## Notes
