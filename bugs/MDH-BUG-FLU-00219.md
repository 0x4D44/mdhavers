# MDH-BUG-FLU-00219 — Native: build --emit-llvm cannot find a sibling module unless run from the source's directory

- **State:** Open
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-29T16:53:47Z
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
- **State history:** Open (2026-10-29T16:53:47Z, raised via `deltic bugs new --land`)

## Observation

## Observation

`mdhavers build <dir>/main.braw --emit-llvm` fails to resolve `fetch` of a module next to the source file when run from another directory; the same build without `--emit-llvm` succeeds. Found while emitting Bogle's IR (`games/bogle/shell/native.braw` fetches "frame") to investigate MDH-BUG-FLU-00218. macOS arm64, debug compiler from 0.3.130 plus an unrelated uncommitted change.

## Repro

`sub/helper.braw`: `dae twice(x) { gie x * 2 }`
`sub/main.braw`: `fetch "helper"` / `blether twice(21)`

From the parent directory:
- `mdhavers build sub/main.braw -o m && ./m` prints `42`.
- `mdhavers build sub/main.braw --emit-llvm -o m.ll`: `Compilation went tits up: Cannot find module to import: helper`.
- From inside `sub/`, `mdhavers build main.braw --emit-llvm -o m.ll` succeeds.

## Expected vs actual

Expected: `--emit-llvm` resolves imports as the native build does. Actual: it resolves them against the current directory only.

## Pointers (not a diagnosis)

`LLVMCompiler::compile_to_ir` (src/llvm/compiler.rs) does not call `codegen.set_source_path`, which the native path does.

Evidence fingerprint: `manual:v1:native-build-emit-llvm-cannot-find-a-sibling-mo-6c507a2e6f2ea5cb`


## Fix

<unfixed — raised only>

## Notes
