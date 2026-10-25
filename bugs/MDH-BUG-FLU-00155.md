# MDH-BUG-FLU-00155 — Native has placeholder bodies for builtins the interpreter implements (compose, group_by...)

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-25T23:01:52Z
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
- **State history:** Open (2026-10-25T23:01:52Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S16 2026-10-25 while fixing MDH-BUG-FLU-00145 (read every arm in src/llvm/codegen.rs compile_call): these names are real interpreter builtins but the native arm is a placeholder returning nil, a constant or the first argument, so native silently gives wrong results: dae_times, screen_end, key_down, key_pressed, assert_that, group_by, compose, nae_that, swatch, same, apply_n, constant, efter, the Scots unit converters (scots_miles_tae_km, scots_pints_tae_litres, stanes_tae_kg), screen_width/screen_height (fixed 800/600). List and reasons in wrk_docs/2026.10.25 - INV - Native-only builtins kept and removed.md. Fix: implement each natively to match the interpreter (graphics/key ones may raise a clear 'no graphics in native' error instead), with a parity test per name.

Evidence fingerprint: `manual:v1:native-has-placeholder-bodies-for-builtins-the--ce56768377cf741d`


## Fix

<unfixed — raised only>

## Notes
