# MDH-BUG-FLU-00161 — Native graphics builtins (draw_*, get_mouse_*, screen_update...) silently return nil

- **State:** Open
- **Priority:** Should
- **Severity:** Medium
- **Area:** native
- **Raised:** 2026-10-26T03:04:10Z
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
- **State history:** Open (2026-10-26T03:04:10Z, raised via `deltic bugs new --land` model=claude-opus-5-5@medium)

## Observation

Found by lane S19 2026-10-26 (code read, src/llvm/codegen.rs ~23985): native compile_call has placeholder arms for screen_update, draw_*, get_mouse_*, is_key_pressed, screen_fps, set_fps and similar graphics names that return nil, so a graphics program builds natively and silently does nothing. Bug 155 made screen_width/screen_height/key_down/key_pressed raise 'Nae windae! Screen an' keyboard functions cannae be used in native code' via __mdh_no_window; apply the same to every remaining graphics placeholder (or implement via raylib if native graphics is wanted - Arthur's call), with a test listing them.

Evidence fingerprint: `manual:v1:native-graphics-builtins-draw-get-mouse-screen--c91c547199374e06`


## Fix

<unfixed — raised only>

## Notes
