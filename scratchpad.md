# Scratchpad

- [ ] 2026-10-28 src/llvm/codegen/rc.rs:2242 rc_touches_slot's `is_slot` follows loads and extractvalue but not inttoptr, so `__mdh_obj_realloc(inttoptr(extractvalue(load slot)))` (in-place string append, src/llvm/codegen.rs:16310) is not seen as touching the slot. Harmless today (the append stores the result straight back); found by the R6 review.
