# MDH-BUG-FLU-00046 — WASM audio import table mismatch (midi_pit_rin_roond vs midi_pit_tune)

- **State:** Closed
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
- **State history:** Open (2026-10-22T23:24:10Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-23T02:58:04Z, deltic:auto role=fix run=fix-20261023T025758Z-3d1b8c52 branch=task/bug-MDH-BUG-FLU-00046-run-fix-20261023T025758Z-3d1b8c52 code=53428a5 gate=manual) -> Closed (2026-10-27T01:51:48Z, deltic:manual role=verify run=verify-20261026T233955Z-6d81b49f verified_fix_run=fix-20261023T025758Z-3d1b8c52 verdict=close)

## Observation

Inferred: compiler imports midi_pit_rin_roond (wasm_compiler.rs:194), runner registers midi_pit_tune (wasm_runner.rs:1227); instantiation fails.

## Fix

<unfixed — raised only>

### Verification summary (2026-10-27T01:51:48Z, deltic:manual run=verify-20261026T233955Z-6d81b49f verified_fix_run=fix-20261023T025758Z-3d1b8c52 verdict=close)

Verifier note: Manual verify: [lc luna] 53428a5: every_audio_import_instantiates_and_runs_in_the_host_runner passed (1 selected); renaming runner `midi_pit_rin_roond` to stale `midi_pit_tune` failed with unknown import `env::midi_pit_rin_roond`. Source mutant restored clean.

## Notes
