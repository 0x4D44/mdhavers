# MDH-BUG-FLU-00251 — LLVM: function in an earlier-fetched module cannot call one from a later-fetched module

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** llvm
- **Raised:** 2026-10-30T13:48:36Z
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
- **State history:** Open (2026-10-30T13:48:36Z, raised via `deltic bugs new`)

## Observation

Observed 2026-10-30 with a release build of mdhavers 0.3.134 (macOS arm64). Repro, three files in one directory:
a.braw:  dae uses_helper() { gie helper() + 1 }
c.braw:  dae helper() { gie 41 }
main.braw:  fetch "a" / fetch "c" / blether uses_helper()
`mdhavers run main.braw` prints 42. `MDH_RC=0 mdhavers build -O0 main.braw --output m` fails: 'Och! Compilation went tits up: Undefined variable: helper'. Expected: the native build compiles and prints 42, as the interpreter does (backend parity). A forward reference within a single file compiles natively, so the failure is specific to cross-module order. Found when games/bogle/engine/doors.braw called map_cross from engine/map.braw without fetching it: the interpreter fixtures passed and every native build of a Bogle fixture failed (bogle_unit would have caught it).

## Fix

<unfixed — raised only>

## Notes
