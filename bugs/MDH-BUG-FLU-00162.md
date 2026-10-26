# MDH-BUG-FLU-00162 — Native builtin values: two uses of len compare unequal, and clype(len) differs

- **State:** Closed
- **Priority:** Should
- **Severity:** Low
- **Area:** native
- **Raised:** 2026-10-26T03:56:47Z
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
- **State history:** Open (2026-10-26T03:56:47Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium) -> Fixed (2026-10-26T04:59:30Z, deltic:auto role=fix run=fix-20261026T041042Z-1dd68436 branch=task/bug-MDH-BUG-FLU-00162-run-fix-20261026T041042Z-1dd68436 code=9025063 gate=manual) -> Closed (2026-10-26T21:44:08Z, deltic:manual role=verify run=verify-20261026T194306Z-ec0ac31f verified_fix_run=fix-20261026T041042Z-1dd68436 verdict=close)

## Observation

Found by lane S20 2026-10-26, confirmed by the lead (executed, trunk 7142eee): ken f = len; ken g = len; blether f == g prints nae natively, aye in the interpreter, because each use of a builtin as a value emits its own wrapper (compile_builtin_value in src/llvm/codegen.rs). clype(len) prints "[function] function '<native dae len>'" natively and "[native function] native function 'len'" in the interpreter (__mdh_clype in runtime/mdh_runtime.c). Fix: emit one wrapper per builtin name (cache it), and have clype recognise wrapper entries in the function-name table as native functions; parity test.

Evidence fingerprint: `manual:v1:native-builtin-values-two-uses-of-len-compare-u-c707edc3888fea63`


## Fix

<unfixed — raised only>

### Verification summary (2026-10-26T21:44:08Z, deltic:manual run=verify-20261026T194306Z-ec0ac31f verified_fix_run=fix-20261026T041042Z-1dd68436 verdict=close)

Verifier note: Manual verify: [verifier gpt-6-luna max (Codex), recorded by lead] 9025063; identity and clype tests pass on trunk. Reversing cache plus clype recognition made both selected assertions fail: len identity was nae; clype printed [function] instead of [native function].

## Notes
