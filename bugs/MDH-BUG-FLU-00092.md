# MDH-BUG-FLU-00092 — WASM host store never frees strings, lists or dicts

- **State:** Fixed
- **Priority:** Should
- **Severity:** Low
- **Area:** wasm
- **Raised:** 2026-10-23T04:31:22Z
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
- **State history:** Open (2026-10-23T04:31:22Z, raised via `deltic bugs new` model=claude-opus-5-5@medium) -> Fixed (2026-10-24T02:18:21Z, deltic:auto role=fix run=fix-20261024T021815Z-ec3adc88 branch=task/bug-MDH-BUG-FLU-00092-run-fix-20261024T021815Z-ec3adc88 code=c6b0df3 gate=manual)

## Observation

Found by the WASM runner lane 2026-10-23: numbers are now inline or interned (MDH-BUG-FLU-00047), but every string, list and dict allocated by the wasm_runner host stays in the store for the life of the run, so long-running string-heavy programs grow without bound.

## Fix

<unfixed — raised only>

## Notes
