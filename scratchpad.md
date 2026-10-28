# Scratchpad

- [ ] 2026-10-28 runtime/platform/screen_macos.c:mdh_scr_grab the mouse stays decoupled and hidden after Cmd-Tab while grabbed; release the grab on focus loss and restore it on return (Bogle stage 1 review).
- [ ] 2026-10-28 runtime/platform/screen_macos.c:mdh_scr_open sets contents on AppKit's own backing layer; a bridge-owned layer (setLayer: or a sublayer) would rule out AppKit redrawing over it. Also finishLaunching runs again on every reopen (probably harmless). Bogle stage 1 review.
- [ ] 2026-10-28 runtime/platform/screen_none.c:8 under MSVC a static .CRT$XCU pointer can be dropped by the optimiser, leaving the main-thread id 0 so every bridge call hurls; use /include or a non-static symbol when the Windows bridge (stage 6) lands.
- [ ] 2026-10-28 src/llvm/codegen/rc.rs:2242 rc_touches_slot's `is_slot` follows loads and extractvalue but not inttoptr, so `__mdh_obj_realloc(inttoptr(extractvalue(load slot)))` (in-place string append, src/llvm/codegen.rs:16310) is not seen as touching the slot. Harmless today (the append stores the result straight back); found by the R6 review.
