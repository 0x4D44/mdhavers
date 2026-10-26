//! Reference counting in generated code, stage R1 (`wrk_docs/2026.10.25 - DES - R1 refcount
//! insertion plan.md`, section 3).
//!
//! Every holder of a value owns one reference: an expression's temp slot, a variable slot,
//! a container element, a closure capture, a function's return value. A holder gets its
//! reference with [`CodeGen::rc_take`]: it takes the +1 a fresh value was born with if no
//! other holder has taken it yet, and retains otherwise. So every reference taken is
//! released exactly once by its holder (statement end, overwrite, function epilogue), and a
//! value whose origin is not recognised is retained, which can only leak, never free early.
//!
//! Fresh values are recognised from the IR: a call whose `RUNTIME_OWNERSHIP` row is
//! `Ret::Fresh`, a call to a compiled mdhavers function (they return +1), a value built from
//! an `__mdh_obj_alloc` result, and phis of those.

use super::*;

use inkwell::values::{AsValueRef, BasicValue, CallSiteValue, InstructionOpcode, InstructionValue};

use crate::llvm::ownership::{ownership_of, Ret};
use crate::llvm::types::RC_HEAP_TAG_MASK;

/// How a value's reference is held, as far as the IR shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RcClass {
    /// Nothing to count: an immediate, a static object, or a call whose result is never a
    /// counted object.
    Skip,
    /// Carries a +1 that no holder has taken yet.
    Fresh,
    /// Anything else: a holder must retain it.
    Borrowed,
}

/// One temp scope: a statement (or the function's base scope, index 0).
pub(super) struct RcScope<'ctx> {
    /// Its temp slots.
    pub(super) slots: Vec<PointerValue<'ctx>>,
    /// Where the scope opened: its slots are set to nil there, so a slot is only live
    /// within its statement (an entry-block init would make every temp live across the
    /// whole function, and mem2reg's phis grow with temps x blocks). `None` for the base
    /// scope, whose slots are entry-initialised and released by the epilogue.
    pub(super) opened_at: Option<(BasicBlock<'ctx>, Option<InstructionValue<'ctx>>)>,
    /// Borrowed reads of local slots in this statement (see `rc_borrowed_read`).
    pub(super) reads: Vec<RcRead<'ctx>>,
    /// The values parked in its temp slots (`rc_own_temp`), so `gie` can hand a temp's
    /// reference to the caller instead of retaining and then releasing the temp.
    pub(super) parked: Vec<(BasicValueEnum<'ctx>, PointerValue<'ctx>)>,
    /// Early exits (`gie`, `brak`, `haud`) compiled out of this scope: the last
    /// instruction (if any) of the block the release each emitted ended in. A temp added to the scope afterwards
    /// (`rc_note_write`) is released at these too.
    pub(super) exits: Vec<(BasicBlock<'ctx>, Option<InstructionValue<'ctx>>)>,
}

/// A local variable (or `masel`) read used without a reference of its own: the slot holds
/// one for as long as nothing stores into it. A store into the slot while the read's
/// statement is open (`rc_note_write`) gives the read its reference after all, parked in
/// a temp of that statement.
pub(super) struct RcRead<'ctx> {
    slot: PointerValue<'ctx>,
    load: InstructionValue<'ctx>,
    value: BasicValueEnum<'ctx>,
}

/// Per-function reference-counting state, saved while a nested function is compiled.
#[derive(Default)]
pub(super) struct RcFnState<'ctx> {
    /// The open temp scopes (innermost last; index 0 is the base scope).
    pub(super) scopes: Vec<RcScope<'ctx>>,
    /// The parameter slots bound so far (see `store_param`).
    pub(super) params: Vec<RcParam<'ctx>>,
}

/// A parameter (or capture, or `masel`) slot. Arguments are borrowed: the caller holds
/// them for the whole call, so a slot nothing writes needs no reference of its own. The
/// first write (`store_slot`) makes it an ordinary owned slot: when the function is done,
/// the prologue's store gets a retain and the epilogue releases the slot.
pub(super) struct RcParam<'ctx> {
    slot: PointerValue<'ctx>,
    /// The prologue's store of the argument.
    store: InstructionValue<'ctx>,
    value: BasicValueEnum<'ctx>,
    written: bool,
}

fn value_id(v: impl AsValueRef) -> usize {
    v.as_value_ref() as usize
}

impl<'ctx> CodeGen<'ctx> {
    /// True for an `MdhValue` (the only values that can hold a counted object).
    pub(super) fn rc_is_value(&self, v: BasicValueEnum<'ctx>) -> bool {
        matches!(v, BasicValueEnum::StructValue(s) if s.get_type() == self.types.value_type)
    }

    pub(super) fn rc_active(&self) -> bool {
        self.rc_enabled
            && self.current_function.is_some()
            && !self.current_block_terminated()
            && self.builder.get_insert_block().is_some()
    }

    /// The direct callee of a call instruction.
    fn rc_callee(inst: InstructionValue<'ctx>) -> Option<FunctionValue<'ctx>> {
        if inst.get_opcode() != InstructionOpcode::Call {
            return None;
        }
        CallSiteValue::try_from(inst).ok()?.get_called_fn_value()
    }

    fn rc_operand(inst: InstructionValue<'ctx>, i: u32) -> Option<BasicValueEnum<'ctx>> {
        match inst.get_operand(i) {
            Some(inkwell::values::Operand::Value(v)) => Some(v),
            _ => None,
        }
    }

    /// `v` carries a +1 for whoever takes it (an element moved out of its container):
    /// classify it as fresh.
    pub(super) fn rc_mark_owned(&mut self, v: BasicValueEnum<'ctx>) {
        if self.rc_enabled {
            self.rc_owned_values.insert(value_id(v));
        }
    }

    /// `v` was only used (tested, or stored by a runtime call that took its own
    /// reference): if it is a fresh +1 nobody has taken, release it now. A plain call.
    pub(super) fn rc_consume_fresh(&mut self, v: BasicValueEnum<'ctx>) {
        if !self.rc_active() || !self.rc_is_value(v) {
            return;
        }
        let mut roots = Vec::new();
        if self.rc_roots(v, 0, &mut roots) == RcClass::Fresh {
            self.rc_taken.extend(roots);
            self.rc_release(v);
        }
    }

    /// Classify `v` (see [`RcClass`]).
    pub(super) fn rc_class(&self, v: BasicValueEnum<'ctx>) -> RcClass {
        let mut roots = Vec::new();
        self.rc_roots(v, 0, &mut roots)
    }

    /// Classify `v`, collecting the *roots* of its fresh +1s: the instructions that made the
    /// objects (an allocator call, a call returning an owned value). Different values built
    /// from the same object (a call result and a value rebuilt from its data) share the
    /// root, so the +1 is taken once. Fresh means every heap path ends at an untaken root.
    fn rc_roots(&self, v: BasicValueEnum<'ctx>, depth: u32, roots: &mut Vec<usize>) -> RcClass {
        if depth > 12 {
            return RcClass::Borrowed;
        }
        if self.rc_owned_values.contains(&value_id(v)) {
            return self.rc_fresh_root(value_id(v), roots);
        }
        match v {
            BasicValueEnum::StructValue(sv) => {
                if sv.get_type() != self.types.value_type {
                    return RcClass::Skip;
                }
                if sv.is_const() {
                    // Literals: immediates, nil, and static objects (immortal headers).
                    return RcClass::Skip;
                }
                let Some(inst) = sv.as_instruction_value() else {
                    return RcClass::Borrowed;
                };
                match inst.get_opcode() {
                    InstructionOpcode::InsertValue => self.rc_value_parts_roots(inst, depth, roots),
                    InstructionOpcode::Call => self.rc_call_roots(inst, roots),
                    InstructionOpcode::Phi => self.rc_phi_roots(inst, depth, roots),
                    _ => RcClass::Borrowed,
                }
            }
            BasicValueEnum::IntValue(iv) => {
                if iv.is_const() {
                    return RcClass::Skip;
                }
                let Some(inst) = iv.as_instruction() else {
                    return RcClass::Borrowed;
                };
                match inst.get_opcode() {
                    InstructionOpcode::PtrToInt => match Self::rc_operand(inst, 0) {
                        Some(p) => self.rc_roots(p, depth + 1, roots),
                        None => RcClass::Borrowed,
                    },
                    InstructionOpcode::ExtractValue => match Self::rc_operand(inst, 0) {
                        // The data field of a value: the value's object.
                        Some(agg @ BasicValueEnum::StructValue(_)) => {
                            self.rc_roots(agg, depth + 1, roots)
                        }
                        _ => RcClass::Borrowed,
                    },
                    InstructionOpcode::Phi => self.rc_phi_roots(inst, depth, roots),
                    _ => RcClass::Borrowed,
                }
            }
            BasicValueEnum::PointerValue(pv) => {
                if pv.is_const() {
                    return RcClass::Skip;
                }
                let Some(inst) = pv.as_instruction() else {
                    return RcClass::Borrowed;
                };
                match inst.get_opcode() {
                    InstructionOpcode::Call => {
                        let alloc = Self::rc_callee(inst).is_some_and(|f| {
                            matches!(
                                f.get_name().to_bytes(),
                                b"__mdh_obj_alloc" | b"__mdh_obj_strdup"
                            )
                        });
                        if alloc {
                            self.rc_fresh_root(value_id(inst), roots)
                        } else {
                            RcClass::Borrowed
                        }
                    }
                    InstructionOpcode::IntToPtr => match Self::rc_operand(inst, 0) {
                        Some(i) => self.rc_roots(i, depth + 1, roots),
                        None => RcClass::Borrowed,
                    },
                    InstructionOpcode::Phi => self.rc_phi_roots(inst, depth, roots),
                    _ => RcClass::Borrowed,
                }
            }
            _ => RcClass::Skip,
        }
    }

    fn rc_fresh_root(&self, id: usize, roots: &mut Vec<usize>) -> RcClass {
        if self.rc_taken.contains(&id) {
            RcClass::Borrowed
        } else {
            roots.push(id);
            RcClass::Fresh
        }
    }

    /// A value built with `insertvalue`: a constant non-heap tag or constant data needs
    /// nothing; otherwise its object is whatever its data field comes from.
    fn rc_value_parts_roots(
        &self,
        inst: InstructionValue<'ctx>,
        depth: u32,
        roots: &mut Vec<usize>,
    ) -> RcClass {
        let mut tag = None;
        let mut data = None;
        let mut cur = Some(inst);
        for _ in 0..4 {
            let Some(i) = cur else { break };
            if i.get_opcode() != InstructionOpcode::InsertValue {
                break;
            }
            if let Some(BasicValueEnum::IntValue(field)) = Self::rc_operand(i, 1) {
                match field.get_type().get_bit_width() {
                    8 if tag.is_none() => tag = Some(field),
                    64 if data.is_none() => data = Some(field),
                    _ => {}
                }
            }
            let agg = Self::rc_operand(i, 0);
            // `insertvalue { i8 3, i64 undef }, data, 1`: the tag sits in the constant.
            if let Some(BasicValueEnum::StructValue(c)) = agg {
                if tag.is_none() && c.is_const() {
                    if let Some(BasicValueEnum::IntValue(t)) = c.get_field_at_index(0) {
                        tag = Some(t);
                    }
                }
            }
            cur = agg.and_then(|agg| agg.as_instruction_value());
        }
        if let Some(t) = tag.and_then(|t| t.get_zero_extended_constant()) {
            if t >= 16 || (RC_HEAP_TAG_MASK >> t) & 1 == 0 {
                return RcClass::Skip;
            }
        }
        match data {
            Some(d) => self.rc_roots(d.into(), depth + 1, roots),
            None => RcClass::Borrowed,
        }
    }

    fn rc_call_roots(&self, inst: InstructionValue<'ctx>, roots: &mut Vec<usize>) -> RcClass {
        let Some(callee) = Self::rc_callee(inst) else {
            return RcClass::Borrowed;
        };
        let name = callee.get_name().to_string_lossy();
        if let Some((_, ret)) = ownership_of(&name) {
            return match ret {
                Ret::None | Ret::Static => RcClass::Skip,
                Ret::Fresh => self.rc_fresh_root(value_id(inst), roots),
                _ => RcClass::Borrowed,
            };
        }
        // Compiled mdhavers functions, methods, struct constructors, lambdas and late
        // dispatchers return +1. By symbol too, since a call can be compiled before its
        // callee's body.
        let compiled = ["__mdh_fn_", "__lambda_", "__mdh_late_call_"]
            .iter()
            .any(|p| name.starts_with(p));
        if compiled || self.rc_owned_return_fns.contains(&callee) {
            self.rc_fresh_root(value_id(inst), roots)
        } else {
            RcClass::Borrowed
        }
    }

    fn rc_phi_roots(
        &self,
        inst: InstructionValue<'ctx>,
        depth: u32,
        roots: &mut Vec<usize>,
    ) -> RcClass {
        if self.rc_taken.contains(&value_id(inst)) {
            return RcClass::Borrowed;
        }
        let phi = inkwell::values::PhiValue::try_from(inst).expect("phi");
        let mut class = RcClass::Skip;
        for i in 0..phi.count_incoming() {
            let (iv, _) = phi.get_incoming(i).expect("incoming");
            match self.rc_roots(iv, depth + 1, roots) {
                RcClass::Skip => {}
                RcClass::Fresh => class = RcClass::Fresh,
                RcClass::Borrowed => return RcClass::Borrowed,
            }
        }
        if class == RcClass::Fresh {
            roots.push(value_id(inst));
        }
        class
    }

    /// Get one reference to `v` for a new holder: take its fresh +1, or retain it.
    /// Returns false when `v` holds nothing counted (the holder has nothing to release).
    /// A retain is a plain call (no new blocks), so it is safe anywhere.
    pub(super) fn rc_take(&mut self, v: BasicValueEnum<'ctx>) -> bool {
        self.rc_take_mode(v, false)
    }

    /// `rc_take` with the retain's tag test inline (new blocks: the builder ends in a
    /// new block, so only for code that does not keep the old block for a phi).
    pub(super) fn rc_take_inline(&mut self, v: BasicValueEnum<'ctx>) -> bool {
        self.rc_take_mode(v, true)
    }

    fn rc_take_mode(&mut self, v: BasicValueEnum<'ctx>, inline: bool) -> bool {
        if !self.rc_enabled || !self.rc_is_value(v) {
            return false;
        }
        let mut roots = Vec::new();
        match self.rc_roots(v, 0, &mut roots) {
            RcClass::Skip => false,
            RcClass::Fresh => {
                self.rc_taken.extend(roots);
                true
            }
            RcClass::Borrowed => {
                if !self.rc_take_phi_by_edges(v) {
                    if inline {
                        self.rc_retain_inline(v);
                    } else {
                        self.rc_retain(v);
                    }
                }
                true
            }
        }
    }

    /// `v` is an untaken phi with at least one fresh incoming value.
    fn rc_phi_has_fresh(&self, v: BasicValueEnum<'ctx>) -> bool {
        self.rc_phi_has_fresh_at(v, 0)
    }

    fn rc_phi_has_fresh_at(&self, v: BasicValueEnum<'ctx>, depth: u32) -> bool {
        if depth > 4 {
            return false;
        }
        let BasicValueEnum::StructValue(sv) = v else {
            return false;
        };
        let Some(inst) = sv.as_instruction_value() else {
            return false;
        };
        if inst.get_opcode() != InstructionOpcode::Phi || self.rc_taken.contains(&value_id(inst)) {
            return false;
        }
        let phi = inkwell::values::PhiValue::try_from(inst).expect("phi");
        (0..phi.count_incoming()).any(|i| {
            let (iv, _) = phi.get_incoming(i).expect("incoming");
            let mut roots = Vec::new();
            self.rc_roots(iv, 1, &mut roots) == RcClass::Fresh
                || self.rc_phi_has_fresh_at(iv, depth + 1)
        })
    }

    /// A borrowed phi: make it owned by taking the fresh incomings and retaining the
    /// borrowed ones at the end of their edge's block. Only when every borrowed incoming's
    /// block ends in an unconditional branch (so the retain runs only on that edge).
    /// Returns false when not done.
    fn rc_take_phi_by_edges(&mut self, v: BasicValueEnum<'ctx>) -> bool {
        let sv = v.into_struct_value();
        let Some(inst) = sv.as_instruction_value() else {
            return false;
        };
        if inst.get_opcode() != InstructionOpcode::Phi || self.rc_taken.contains(&value_id(inst)) {
            return false;
        }
        let phi = inkwell::values::PhiValue::try_from(inst).expect("phi");
        let mut plan = Vec::new();
        for i in 0..phi.count_incoming() {
            let (iv, bb) = phi.get_incoming(i).expect("incoming");
            let mut roots = Vec::new();
            let class = self.rc_roots(iv, 1, &mut roots);
            if class == RcClass::Borrowed {
                let Some(term) = bb.get_terminator() else {
                    return false;
                };
                if term.get_opcode() != InstructionOpcode::Br || term.get_num_operands() != 1 {
                    return false;
                }
            }
            plan.push((iv, bb, class, roots));
        }
        self.rc_taken.insert(value_id(inst));
        let retain = self.libc.rc_retain;
        for (iv, bb, class, roots) in plan {
            match class {
                RcClass::Skip => {}
                RcClass::Fresh => self.rc_taken.extend(roots),
                // A phi of fresh and borrowed values (a property read that is a field or a
                // new bound method): normalise it in turn, so its fresh +1s are taken.
                RcClass::Borrowed if self.rc_phi_has_fresh(iv) && self.rc_take_phi_by_edges(iv) => {
                }
                RcClass::Borrowed => {
                    let term = bb.get_terminator().expect("terminator");
                    let b = self.context.create_builder();
                    b.position_before(&term);
                    b.build_call(retain, &[iv.into()], "").unwrap();
                }
            }
        }
        true
    }

    fn rc_emit_inline(&mut self, v: BasicValueEnum<'ctx>, f: FunctionValue<'ctx>, what: &str) {
        let function = self
            .builder
            .get_insert_block()
            .and_then(|b| b.get_parent())
            .expect("rc: no insertion block");
        let sv = v.into_struct_value();
        let tag = self
            .builder
            .build_extract_value(sv, 0, "rc_tag")
            .unwrap()
            .into_int_value();
        let data = self
            .builder
            .build_extract_value(sv, 1, "rc_data")
            .unwrap()
            .into_int_value();
        let is_heap = self.rc_build_heap_test(&self.builder, tag, "rc_");
        let call_bb = self
            .context
            .append_basic_block(function, &format!("rc_{what}"));
        let cont_bb = self
            .context
            .append_basic_block(function, &format!("rc_{what}_done"));
        self.builder
            .build_conditional_branch(is_heap, call_bb, cont_bb)
            .unwrap();
        self.builder.position_at_end(call_bb);
        self.builder.build_call(f, &[data.into()], "").unwrap();
        self.builder.build_unconditional_branch(cont_bb).unwrap();
        self.builder.position_at_end(cont_bb);
    }

    /// `tag` is a counted heap tag (`RC_HEAP_TAG_MASK`): one shift of the mask by the tag.
    /// The shift amount is the tag mod 64, which a 64-bit shift on AArch64 and x86-64 takes
    /// for free; every tag is below 16 and the mask has no bit at 16 or above, so tags
    /// 16..63 test false as before.
    fn rc_build_heap_test(
        &self,
        b: &Builder<'ctx>,
        tag: IntValue<'ctx>,
        prefix: &str,
    ) -> IntValue<'ctx> {
        let i64t = self.types.i64_type;
        let tag64 = b
            .build_int_z_extend(tag, i64t, &format!("{prefix}tag64"))
            .unwrap();
        let shift = b
            .build_and(tag64, i64t.const_int(63, false), &format!("{prefix}shift"))
            .unwrap();
        let bits = b
            .build_right_shift(
                i64t.const_int(RC_HEAP_TAG_MASK, false),
                shift,
                false,
                &format!("{prefix}bits"),
            )
            .unwrap();
        b.build_int_truncate(bits, self.types.bool_type, &format!("{prefix}is_heap"))
            .unwrap()
    }

    /// Retain `v` with a call to `__mdh_retain` (no new blocks; no-op for values that
    /// hold nothing counted).
    pub(super) fn rc_retain(&mut self, v: BasicValueEnum<'ctx>) {
        if !self.rc_enabled || !self.rc_is_value(v) || self.rc_class(v) == RcClass::Skip {
            return;
        }
        let f = self.libc.rc_retain;
        self.builder.build_call(f, &[v.into()], "").unwrap();
    }

    /// Release `v` with a call to `__mdh_release` (no new blocks).
    pub(super) fn rc_release(&mut self, v: BasicValueEnum<'ctx>) {
        if !self.rc_enabled || !self.rc_is_value(v) || self.rc_class(v) == RcClass::Skip {
            return;
        }
        let f = self.libc.rc_release;
        self.builder.build_call(f, &[v.into()], "").unwrap();
    }

    /// Retain with the tag test inline (see `rc_take_inline` for where that is allowed).
    pub(super) fn rc_retain_inline(&mut self, v: BasicValueEnum<'ctx>) {
        if !self.rc_enabled || !self.rc_is_value(v) || self.rc_class(v) == RcClass::Skip {
            return;
        }
        let f = self.libc.rc_retain_obj;
        self.rc_emit_inline(v, f, "retain");
    }

    /// Release with the tag test inline (see `rc_take_inline`).
    pub(super) fn rc_release_inline(&mut self, v: BasicValueEnum<'ctx>) {
        if !self.rc_enabled || !self.rc_is_value(v) || self.rc_class(v) == RcClass::Skip {
            return;
        }
        let f = self.libc.rc_release_obj;
        self.rc_emit_inline(v, f, "release");
    }

    /// A holder's reference to `v` for a statement-level store (`inline`): when a temp of
    /// the statement holds `v`, move that reference (the temp becomes nil) instead of a
    /// retain now and the temp's release when the statement ends. Only for slots no other
    /// thread can reach (not globals, see `store_shared_slot`): after its store the
    /// statement runs no user code (an assignment's value is retained at once), so
    /// nothing can drop the slot's reference while `v` is still used. Otherwise
    /// `rc_take_mode`.
    fn rc_take_or_move(&mut self, v: BasicValueEnum<'ctx>, inline: bool) -> bool {
        if inline && self.rc_move_from_temp(v, false) {
            return true;
        }
        self.rc_take_mode(v, inline)
    }

    /// If a temp holds `v` (of the innermost scope, or with `any_scope` of any open one),
    /// set that temp to nil and forget it: its reference now belongs to the caller.
    /// The temp still holds `v`: a temp is written only where it is parked, right after
    /// its value is made (which dominates every use of `v`), and where its scope closes.
    fn rc_move_from_temp(&mut self, v: BasicValueEnum<'ctx>, any_scope: bool) -> bool {
        if !self.rc_is_value(v) {
            return false;
        }
        let depth = self.rc_state.scopes.len();
        let from = if any_scope {
            0
        } else {
            depth.saturating_sub(1)
        };
        let mut found = None;
        for (si, scope) in self.rc_state.scopes.iter().enumerate().skip(from).rev() {
            if let Some(pi) = scope.parked.iter().rposition(|(pv, _)| *pv == v) {
                found = Some((si, pi));
                break;
            }
        }
        let Some((si, pi)) = found else {
            return false;
        };
        let (_, slot) = self.rc_state.scopes[si].parked.remove(pi);
        let nil = self.make_nil();
        self.builder.build_store(slot, nil).unwrap();
        true
    }

    /// Store `v` into the owned slot `ptr` (a variable, global, cell element or other
    /// holder of one reference): take a reference to `v`, store it, then release the old
    /// value (store before release, DES F6 3.4). Plain calls, no new blocks.
    pub(super) fn store_slot(&mut self, ptr: PointerValue<'ctx>, v: BasicValueEnum<'ctx>) {
        self.store_slot_mode(ptr, v, false);
    }

    /// `store_slot` with inline tag tests, for statement-level variable stores.
    pub(super) fn store_slot_inline(&mut self, ptr: PointerValue<'ctx>, v: BasicValueEnum<'ctx>) {
        self.store_slot_mode(ptr, v, true);
    }

    fn store_slot_mode(&mut self, ptr: PointerValue<'ctx>, v: BasicValueEnum<'ctx>, inline: bool) {
        if !self.rc_enabled || !self.rc_is_value(v) || !self.rc_active() {
            self.builder.build_store(ptr, v).unwrap();
            return;
        }
        self.rc_note_write(ptr);
        if self.rc_is_shared_slot(ptr) {
            self.store_shared_slot(ptr, v, inline);
            return;
        }
        let old = self
            .builder
            .build_load(self.types.value_type, ptr, "rc_old")
            .unwrap();
        self.rc_take_or_move(v, inline);
        self.builder.build_store(ptr, v).unwrap();
        if inline {
            self.rc_release_inline(old);
        } else {
            self.rc_release(old);
        }
    }

    /// A slot other threads can reach by name: an LLVM global (user globals, module
    /// variables). Captured cells are shared too; their element slots are handled where
    /// they are read and written (`box_get`, `box_set`).
    pub(super) fn rc_is_shared_slot(&self, ptr: PointerValue<'ctx>) -> bool {
        // SAFETY: a type query on a live value of this module.
        unsafe { !inkwell::llvm_sys::core::LLVMIsAGlobalVariable(ptr.as_value_ref()).is_null() }
    }

    /// Branch on the runtime's "a thread has been spawned" flag (`__mdh_line_threaded`,
    /// set once and never cleared). Returns (threaded, single, done) blocks, with the
    /// builder at the end of none of them.
    pub(super) fn rc_branch_threaded(
        &mut self,
        what: &str,
    ) -> (BasicBlock<'ctx>, BasicBlock<'ctx>, BasicBlock<'ctx>) {
        let function = self
            .builder
            .get_insert_block()
            .and_then(|b| b.get_parent())
            .expect("rc: no insertion block");
        let flag = match self.module.get_global("__mdh_line_threaded") {
            Some(g) => g,
            None => {
                let g = self
                    .module
                    .add_global(self.types.i32_type, None, "__mdh_line_threaded");
                g.set_linkage(Linkage::External);
                g
            }
        }
        .as_pointer_value();
        let threaded = self
            .builder
            .build_load(self.types.i32_type, flag, "threaded")
            .unwrap()
            .into_int_value();
        let is_threaded = self
            .builder
            .build_int_compare(
                IntPredicate::NE,
                threaded,
                self.types.i32_type.const_zero(),
                "is_threaded",
            )
            .unwrap();
        let slow = self
            .context
            .append_basic_block(function, &format!("{what}_threaded"));
        let fast = self
            .context
            .append_basic_block(function, &format!("{what}_single"));
        let done = self
            .context
            .append_basic_block(function, &format!("{what}_done"));
        self.builder
            .build_conditional_branch(is_threaded, slow, fast)
            .unwrap();
        (slow, fast, done)
    }

    /// Read a shared slot (a global or a captured cell's element) as an owned (+1) value.
    /// Before the first spawn: load and retain. After: `__mdh_shared_load`, which loads
    /// and retains under the slot's lock, so a writer cannot release the value in
    /// between (DEC round 3, R3). New blocks; the result is marked fresh.
    pub(super) fn rc_shared_load(
        &mut self,
        ptr: PointerValue<'ctx>,
        name: &str,
    ) -> BasicValueEnum<'ctx> {
        if !self.rc_active() {
            return self
                .builder
                .build_load(self.types.value_type, ptr, name)
                .unwrap();
        }
        let (slow, fast, done) = self.rc_branch_threaded("shared_rd");
        self.builder.position_at_end(fast);
        let plain = self
            .builder
            .build_load(self.types.value_type, ptr, name)
            .unwrap();
        self.rc_retain_inline(plain);
        let fast_end = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(done).unwrap();
        self.builder.position_at_end(slow);
        let locked = self
            .builder
            .build_call(self.libc.shared_load, &[ptr.into()], name)
            .unwrap()
            .try_as_basic_value()
            .basic()
            .unwrap();
        self.builder.build_unconditional_branch(done).unwrap();
        self.builder.position_at_end(done);
        let phi = self.builder.build_phi(self.types.value_type, name).unwrap();
        phi.add_incoming(&[(&plain, fast_end), (&locked, slow)]);
        let v = phi.as_basic_value();
        self.rc_mark_owned(v);
        v
    }

    /// Store into a shared slot: take a reference to `v`, then swap it in and release the
    /// old value. After the first spawn the swap happens under the slot's lock
    /// (`__mdh_shared_store`). Inline: the flag test is inline and only the threaded
    /// path calls; otherwise one call, which tests the flag itself (no new blocks).
    pub(super) fn store_shared_slot(
        &mut self,
        ptr: PointerValue<'ctx>,
        v: BasicValueEnum<'ctx>,
        inline: bool,
    ) {
        // Never a move from a temp: an assignment's value is used after its store (it is
        // an expression), and once threads exist another thread can replace the global
        // and drop the slot's reference in between; the temp's reference keeps it alive.
        self.rc_take_mode(v, inline);
        if !inline {
            self.builder
                .build_call(self.libc.shared_store, &[ptr.into(), v.into()], "")
                .unwrap();
            return;
        }
        let (slow, fast, done) = self.rc_branch_threaded("shared_wr");
        self.builder.position_at_end(fast);
        let old = self
            .builder
            .build_load(self.types.value_type, ptr, "rc_old")
            .unwrap();
        self.builder.build_store(ptr, v).unwrap();
        self.rc_release_inline(old);
        self.builder.build_unconditional_branch(done).unwrap();
        self.builder.position_at_end(slow);
        self.builder
            .build_call(self.libc.shared_store, &[ptr.into(), v.into()], "")
            .unwrap();
        self.builder.build_unconditional_branch(done).unwrap();
        self.builder.position_at_end(done);
    }

    /// Store an immediate (an Int from a shadow) that needs no reference into `ptr`. A
    /// shared slot under counting takes the locked swap once threads exist, so readers
    /// never see a half-written value; otherwise a plain store, as before.
    pub(super) fn store_immediate_slot(
        &mut self,
        ptr: PointerValue<'ctx>,
        v: BasicValueEnum<'ctx>,
    ) {
        if !self.rc_enabled || !self.rc_active() || !self.rc_is_shared_slot(ptr) {
            self.builder.build_store(ptr, v).unwrap();
            return;
        }
        let (slow, fast, done) = self.rc_branch_threaded("shared_wr");
        self.builder.position_at_end(fast);
        self.builder.build_store(ptr, v).unwrap();
        self.builder.build_unconditional_branch(done).unwrap();
        self.builder.position_at_end(slow);
        self.builder
            .build_call(self.libc.shared_store, &[ptr.into(), v.into()], "")
            .unwrap();
        self.builder.build_unconditional_branch(done).unwrap();
        self.builder.position_at_end(done);
    }

    /// Variable `name` lives in a shared slot and counting is on: its shadow caches (main's
    /// string length and capacity) are only valid until the first spawn, since other
    /// threads' writes do not update them.
    pub(super) fn rc_shadows_need_thread_check(&self, name: &str) -> bool {
        self.rc_enabled
            && self.rc_active()
            && self
                .variables
                .get(name)
                .or(self.globals.get(name))
                .is_some_and(|p| self.rc_is_shared_slot(*p))
    }

    /// Bind a parameter in the prologue. It stays borrowed (no reference, not released)
    /// unless the body writes the slot (see `RcParam`).
    pub(super) fn store_param(&mut self, ptr: PointerValue<'ctx>, v: BasicValueEnum<'ctx>) {
        let store = self.builder.build_store(ptr, v).unwrap();
        if !self.rc_active() || !self.rc_is_value(v) {
            return;
        }
        if let Some(function) = self.current_function {
            if let Some(&i) = self.rc_fn_index.get(&function) {
                self.rc_fn_slots[i].1.retain(|s| *s != ptr);
            }
        }
        self.rc_state.params.push(RcParam {
            slot: ptr,
            store,
            value: v,
            written: false,
        });
    }

    /// A store into `ptr` that releases what it held: a parameter slot becomes owned, and
    /// the open statements' borrowed reads of it take their reference after all.
    pub(super) fn rc_note_write(&mut self, ptr: PointerValue<'ctx>) {
        for p in self.rc_state.params.iter_mut() {
            if p.slot == ptr {
                p.written = true;
            }
        }
        let function = match self.current_function {
            Some(f) => f,
            None => return,
        };
        for si in 0..self.rc_state.scopes.len() {
            let reads = &mut self.rc_state.scopes[si].reads;
            if !reads.iter().any(|r| r.slot == ptr) {
                continue;
            }
            let (hit, kept): (Vec<RcRead<'ctx>>, Vec<RcRead<'ctx>>) = std::mem::take(reads)
                .into_iter()
                .partition(|r| r.slot == ptr);
            self.rc_state.scopes[si].reads = kept;
            for r in hit {
                // Right after the read: retain and park (load the temp's old value,
                // store, release the old one), as `rc_own_temp` would have done there.
                let temp = self.create_entry_block_alloca_raw("rc_tmp");
                self.rc_temp_pins.entry(function).or_default().push(temp);
                self.rc_state.scopes[si].slots.push(temp);
                let b = self.context.create_builder();
                match r.load.get_next_instruction() {
                    Some(next) => b.position_before(&next),
                    None => b.position_at_end(r.load.get_parent().unwrap()),
                }
                b.build_call(self.libc.rc_retain, &[r.value.into()], "")
                    .unwrap();
                let old = b
                    .build_load(self.types.value_type, temp, "rc_tmp_old")
                    .unwrap();
                b.build_store(temp, r.value).unwrap();
                b.build_call(self.libc.rc_release, &[old.into()], "")
                    .unwrap();
                // An early exit compiled before this temp existed (a `gie` in a loop
                // whose condition reads the variable) must release it too. The temp is
                // nil until its statement parks something in it, so this is always safe.
                let exits = self.rc_state.scopes[si].exits.clone();
                for (block, after) in exits {
                    let next = match after {
                        Some(i) => i.get_next_instruction(),
                        None => block.get_first_instruction(),
                    };
                    match next {
                        Some(next) => b.position_before(&next),
                        None => b.position_at_end(block),
                    }
                    let held = b
                        .build_load(self.types.value_type, temp, "rc_tmp_exit")
                        .unwrap();
                    b.build_store(temp, self.make_nil()).unwrap();
                    b.build_call(self.libc.rc_release, &[held.into()], "")
                        .unwrap();
                }
            }
        }
    }

    /// The slot an expression reads that may stay borrowed: a local, unboxed variable or
    /// `masel`, inside a statement. Callees cannot store into such a slot.
    pub(super) fn rc_borrowable_slot(&self, expr: &Expr) -> Option<PointerValue<'ctx>> {
        if !self.rc_active()
            || !self
                .rc_state
                .scopes
                .last()
                .is_some_and(|s| s.opened_at.is_some())
        {
            return None;
        }
        match expr {
            Expr::Variable { name, .. } => {
                let slot = *self.variables.get(name)?;
                if self.boxed_vars.contains(name) || self.globals.get(name) == Some(&slot) {
                    return None;
                }
                Some(slot)
            }
            Expr::Masel { .. } => self.current_masel,
            _ => None,
        }
    }

    /// `value` is the read of `slot` (a load of it): keep it borrowed for this statement.
    pub(super) fn rc_borrowed_read(
        &mut self,
        slot: PointerValue<'ctx>,
        value: BasicValueEnum<'ctx>,
    ) -> bool {
        let Some(load) = value.as_instruction_value() else {
            return false;
        };
        if load.get_opcode() != InstructionOpcode::Load
            || !matches!(
                load.get_operand(0),
                Some(inkwell::values::Operand::Value(p))
                    if p == BasicValueEnum::PointerValue(slot)
            )
        {
            return false;
        }
        match self.rc_state.scopes.last_mut() {
            Some(scope) => {
                scope.reads.push(RcRead { slot, load, value });
                true
            }
            None => false,
        }
    }

    /// The function is done: give each written parameter slot its own reference right
    /// after the prologue binds it, and let the epilogue release it.
    fn rc_own_written_params(&mut self) {
        let params = std::mem::take(&mut self.rc_state.params);
        for p in params.into_iter().filter(|p| p.written) {
            let Some(function) = p.store.get_parent().and_then(|b| b.get_parent()) else {
                continue;
            };
            let b = self.context.create_builder();
            match p.store.get_next_instruction() {
                Some(next) => b.position_before(&next),
                None => b.position_at_end(p.store.get_parent().unwrap()),
            }
            b.build_call(self.libc.rc_retain, &[p.value.into()], "")
                .unwrap();
            match self.rc_fn_index.get(&function) {
                Some(&i) => self.rc_fn_slots[i].1.push(p.slot),
                None => {
                    self.rc_fn_index.insert(function, self.rc_fn_slots.len());
                    self.rc_fn_slots.push((function, vec![p.slot]));
                }
            }
        }
    }

    /// Store `v` into a container element or field that held nothing (fresh memory): take a
    /// reference and store.
    pub(super) fn store_new_element(&mut self, ptr: PointerValue<'ctx>, v: BasicValueEnum<'ctx>) {
        if self.rc_active() {
            self.rc_take(v);
        }
        self.builder.build_store(ptr, v).unwrap();
    }

    /// Register `slot` (an entry-block alloca of the current function, already nil) as a
    /// holder that the function epilogue releases.
    pub(super) fn rc_register_slot(&mut self, slot: PointerValue<'ctx>) {
        let Some(function) = self.current_function else {
            return;
        };
        match self.rc_fn_index.get(&function) {
            Some(&i) => self.rc_fn_slots[i].1.push(slot),
            None => {
                self.rc_fn_index.insert(function, self.rc_fn_slots.len());
                self.rc_fn_slots.push((function, vec![slot]));
            }
        }
    }

    /// The expression-result rule: give `v` an owned reference held by a temp slot of the
    /// innermost temp scope, released when that scope ends.
    pub(super) fn rc_own_temp(&mut self, v: BasicValueEnum<'ctx>) {
        if !self.rc_enabled || !self.rc_is_value(v) || !self.rc_active() {
            return;
        }
        if !self.rc_take_inline(v) {
            return;
        }
        let slot = self.rc_temp_slot();
        // A slot parked again before its scope ends (a loop condition) lets go of the
        // previous value.
        let old = self
            .builder
            .build_load(self.types.value_type, slot, "rc_tmp_old")
            .unwrap();
        self.builder.build_store(slot, v).unwrap();
        self.rc_release_unknown(old);
        if let Some(scope) = self.rc_state.scopes.last_mut() {
            scope.parked.push((v, slot));
        }
    }

    /// A temp slot of the innermost temp scope. Inside a statement it is set to nil where
    /// the statement opened and released where it closes (see `RcScope::opened_at`);
    /// outside one it is an ordinary entry slot the epilogue releases.
    ///
    /// A call site's scratch bindings (`masel`, a capture, a default's parameter, a pipe's
    /// left operand) are not temps: they alias a value an argument already holds for the
    /// call, so they are plain allocas that hold no reference. As holders, each candidate
    /// of a method dispatch would get its own slot released at the statement's end, and
    /// those slots merge as O(candidates^2) phis.
    fn rc_temp_slot(&mut self) -> PointerValue<'ctx> {
        let name = "rc_tmp";
        let in_statement = self
            .rc_state
            .scopes
            .last()
            .is_some_and(|s| s.opened_at.is_some());
        if !in_statement {
            let slot = self.create_entry_block_alloca(name);
            match self.rc_state.scopes.last_mut() {
                Some(scope) => scope.slots.push(slot),
                None => self.rc_state.scopes.push(RcScope {
                    slots: vec![slot],
                    opened_at: None,
                    reads: Vec::new(),
                    parked: Vec::new(),
                    exits: Vec::new(),
                }),
            }
            return slot;
        }
        let slot = self.create_entry_block_alloca_raw(name);
        let function = self.current_function.expect("rc temp outside a function");
        self.rc_temp_pins.entry(function).or_default().push(slot);
        if let Some(scope) = self.rc_state.scopes.last_mut() {
            scope.slots.push(slot);
        }
        slot
    }

    /// Release a value loaded from a holder (its class is unknown by construction).
    fn rc_release_unknown(&mut self, v: BasicValueEnum<'ctx>) {
        let f = self.libc.rc_release_obj;
        self.rc_emit_inline(v, f, "release");
    }

    /// Open a temp scope (a statement).
    pub(super) fn rc_scope_open(&mut self) {
        let opened_at = self
            .builder
            .get_insert_block()
            .map(|b| (b, b.get_last_instruction()));
        self.rc_state.scopes.push(RcScope {
            slots: Vec::new(),
            opened_at,
            reads: Vec::new(),
            parked: Vec::new(),
            exits: Vec::new(),
        });
    }

    /// Close the innermost temp scope: release and clear its temps. A block that has
    /// already ended (`gie`, `brak`, `haud`) released them on its way out
    /// (`rc_release_scopes_from`); a `hurl` leaks them (accepted).
    pub(super) fn rc_scope_close(&mut self) {
        let Some(scope) = self.rc_state.scopes.pop() else {
            return;
        };
        if scope.slots.is_empty() {
            return;
        }
        if let Some((block, after)) = scope.opened_at {
            let b = self.context.create_builder();
            match after.and_then(|i| i.get_next_instruction()) {
                Some(next) => b.position_before(&next),
                None => {
                    // The block was empty when the scope opened: its first instruction
                    // past any entry allocas (temp slots are allocated at the top).
                    let mut first = if after.is_none() {
                        block.get_first_instruction()
                    } else {
                        None
                    };
                    while let Some(i) = first {
                        if i.get_opcode() != InstructionOpcode::Alloca {
                            break;
                        }
                        first = i.get_next_instruction();
                    }
                    match first {
                        Some(i) => b.position_before(&i),
                        None => b.position_at_end(block),
                    }
                }
            }
            let nil = self.make_nil();
            for slot in &scope.slots {
                b.build_store(*slot, nil).unwrap();
            }
        }
        if !self.rc_active() {
            return;
        }
        self.rc_release_slots(&scope.slots);
    }

    pub(super) fn rc_release_slots(&mut self, slots: &[PointerValue<'ctx>]) {
        let nil = self.make_nil();
        for slot in slots.iter().rev() {
            let v = self
                .builder
                .build_load(self.types.value_type, *slot, "rc_tmp_done")
                .unwrap();
            self.builder.build_store(*slot, nil).unwrap();
            self.rc_release_unknown(v);
        }
    }

    /// The temp-scope depth now: what a loop records so `brak`/`haud` release the temps
    /// of the statements inside it.
    pub(super) fn rc_scope_depth(&self) -> usize {
        self.rc_state.scopes.len()
    }

    /// Leaving scopes `depth..` early (`gie` from 0, `brak`/`haud` from the loop's
    /// depth): release their temps on this path. Their statements' closes still run on
    /// the other paths.
    pub(super) fn rc_release_scopes_from(&mut self, depth: usize) {
        if !self.rc_enabled || !self.rc_active() {
            return;
        }
        let slots: Vec<PointerValue<'ctx>> = self.rc_state.scopes
            [depth.min(self.rc_state.scopes.len())..]
            .iter()
            .flat_map(|s| s.slots.iter().copied())
            .collect();
        self.rc_release_slots(&slots);
        if let Some(block) = self.builder.get_insert_block() {
            let exit = (block, block.get_last_instruction());
            let from = depth.min(self.rc_state.scopes.len());
            for scope in self.rc_state.scopes[from..].iter_mut() {
                scope.exits.push(exit);
            }
        }
    }

    /// Start compiling a new function body: fresh temp scopes (one base scope that only
    /// the epilogue releases). Returns the caller's state for [`Self::rc_leave_function`].
    pub(super) fn rc_enter_function(
        &mut self,
        owned_return: Option<FunctionValue<'ctx>>,
    ) -> RcFnState<'ctx> {
        if let Some(f) = owned_return {
            self.rc_owned_return_fns.insert(f);
        }
        let mut fresh = RcFnState::default();
        fresh.scopes.push(RcScope {
            slots: Vec::new(),
            opened_at: None,
            reads: Vec::new(),
            parked: Vec::new(),
            exits: Vec::new(),
        });
        std::mem::replace(&mut self.rc_state, fresh)
    }

    pub(super) fn rc_leave_function(&mut self, saved: RcFnState<'ctx>) {
        if self.rc_enabled {
            self.rc_own_written_params();
        }
        self.rc_state = saved;
    }

    /// In `main`'s prologue, before the runtime starts: turn counting on in the runtime
    /// (`__mdh_rc_enable`, which reads MDH_RC_KINDS). A program compiled without it counts
    /// nothing, whatever its environment says.
    pub(super) fn rc_emit_enable(&mut self) {
        if !self.rc_enabled {
            return;
        }
        let f = self
            .module
            .get_function("__mdh_rc_enable")
            .unwrap_or_else(|| {
                let f = self.module.add_function(
                    "__mdh_rc_enable",
                    self.types.void_type.fn_type(&[], false),
                    Some(Linkage::External),
                );
                let nounwind = self
                    .context
                    .create_enum_attribute(Attribute::get_named_enum_kind_id("nounwind"), 0);
                f.add_attribute(AttributeLoc::Function, nounwind);
                f
            });
        self.builder.build_call(f, &[], "").unwrap();
    }

    /// With counting off, the retain/release declarations are unused: remove them, so the
    /// module is exactly what it was before reference counting existed.
    pub(super) fn rc_drop_unused_declarations(&mut self) {
        if self.rc_enabled {
            return;
        }
        for f in [
            self.libc.rc_retain_obj,
            self.libc.rc_release_obj,
            self.libc.rc_retain,
            self.libc.rc_release,
            self.libc.shared_load,
            self.libc.shared_store,
        ] {
            if f.as_global_value()
                .as_pointer_value()
                .get_first_use()
                .is_none()
            {
                // SAFETY: a declaration with no uses; nothing refers to it after compile().
                unsafe { f.delete() };
            }
        }
    }

    /// With counting on, give retain and release an inline fast path (R4 performance pass).
    /// Every call codegen emitted to `__mdh_retain`/`__mdh_release` (an `MdhValue`) and
    /// `__mdh_rc_retain_obj`/`__mdh_rc_release_obj` (its data) is redirected to an internal,
    /// always-inlined function that adjusts the count itself in the common case: a live,
    /// mortal object of a counted kind, before any thread exists, and for a release one
    /// that does not reach zero. Everything else (no header, immortal, DEAD, uncounted kind,
    /// threads, the last release) calls the runtime as before, so behaviour is unchanged.
    /// The runtime's `__mdh_rc_fast` is 1 while every object kind is counted and no thread
    /// has been spawned, else 0. Runs last, after the line-note pass.
    pub(super) fn rc_emit_fast_paths(&mut self) {
        if !self.rc_enabled {
            return;
        }
        let slow = [
            self.libc.rc_retain_obj,
            self.libc.rc_release_obj,
            self.libc.rc_retain,
            self.libc.rc_release,
        ];
        let obj_type = self.libc.rc_retain_obj.get_type();
        let value_type = self.libc.rc_retain.get_type();
        let attr = |name: &str| {
            self.context
                .create_enum_attribute(Attribute::get_named_enum_kind_id(name), 0)
        };
        let (always_inline, nounwind) = (attr("alwaysinline"), attr("nounwind"));
        let mut fast = Vec::new();
        for (i, name) in [
            "__mdh_rc_retain_obj_fast",
            "__mdh_rc_release_obj_fast",
            "__mdh_retain_fast",
            "__mdh_release_fast",
        ]
        .iter()
        .enumerate()
        {
            let ty = if i < 2 { obj_type } else { value_type };
            let f = self.module.add_function(name, ty, Some(Linkage::Internal));
            f.add_attribute(AttributeLoc::Function, always_inline);
            f.add_attribute(AttributeLoc::Function, nounwind);
            fast.push(f);
        }
        // Redirect first, so the fast paths' own calls to the runtime stay direct.
        for (s, f) in slow.iter().zip(&fast) {
            // SAFETY: both are functions of this module with the same type.
            unsafe {
                inkwell::llvm_sys::core::LLVMReplaceAllUsesWith(s.as_value_ref(), f.as_value_ref())
            };
        }
        self.rc_fold_never_threaded(fast[2], fast[3]);
        self.rc_build_obj_fast(fast[0], self.libc.rc_retain_obj, true);
        self.rc_build_obj_fast(fast[1], self.libc.rc_release_obj, false);
        self.rc_build_value_fast(fast[2], fast[0]);
        self.rc_build_value_fast(fast[3], fast[1]);
        // `__mdh_retain`/`__mdh_release` stay declared (unused): the ownership table
        // checks every declared runtime symbol against the module.
    }

    /// A program that never calls `__mdh_thread_spawn` never has a second thread: that call
    /// is the only thing that sets `__mdh_line_threaded`, and compiled code reaches it only
    /// through this module's references to it (a thread_spawn call, or a function value of
    /// it). With no reference left, every read of the flag is 0, so fold them: the
    /// shared-slot and line-note branches for threads go, and loops over globals no
    /// longer call the runtime on a path the optimiser has to keep. The remaining calls
    /// to `__mdh_shared_load`/`__mdh_shared_store` become what those do single-threaded:
    /// load and retain; load the old value, store, release the old value.
    fn rc_fold_never_threaded(
        &mut self,
        retain_fast: FunctionValue<'ctx>,
        release_fast: FunctionValue<'ctx>,
    ) {
        use inkwell::llvm_sys::core::{
            LLVMGetFirstUse, LLVMGetNextUse, LLVMGetUser, LLVMInstructionEraseFromParent,
            LLVMIsACallInst, LLVMIsALoadInst, LLVMReplaceAllUsesWith,
        };
        let spawned = self
            .module
            .get_function("__mdh_thread_spawn")
            .is_some_and(|f| {
                f.as_global_value()
                    .as_pointer_value()
                    .get_first_use()
                    .is_some()
            });
        if spawned {
            return;
        }
        let Some(flag) = self.module.get_global("__mdh_line_threaded") else {
            return;
        };
        let zero = self.types.i32_type.const_zero();
        let mut loads = Vec::new();
        // SAFETY: walking the use list of a live global, then replacing and erasing the
        // loads found (none of them is used by another of them).
        unsafe {
            let mut u = LLVMGetFirstUse(flag.as_value_ref());
            while !u.is_null() {
                let user = LLVMGetUser(u);
                if !LLVMIsALoadInst(user).is_null() {
                    loads.push(user);
                }
                u = LLVMGetNextUse(u);
            }
            for load in loads {
                LLVMReplaceAllUsesWith(load, zero.as_value_ref());
                LLVMInstructionEraseFromParent(load);
            }
        }
        let b = self.context.create_builder();
        for (f, is_load) in [
            (self.libc.shared_load, true),
            (self.libc.shared_store, false),
        ] {
            let mut calls = Vec::new();
            // SAFETY: as above; each call found is replaced and erased once.
            unsafe {
                let mut u = LLVMGetFirstUse(f.as_value_ref());
                while !u.is_null() {
                    let user = LLVMGetUser(u);
                    if !LLVMIsACallInst(user).is_null() {
                        calls.push(user);
                    }
                    u = LLVMGetNextUse(u);
                }
            }
            for call in calls {
                // SAFETY: `call` is a live call instruction of this module.
                let inst = unsafe { InstructionValue::new(call) };
                let Some(BasicValueEnum::PointerValue(slot)) = Self::rc_operand(inst, 0) else {
                    continue;
                };
                b.position_before(&inst);
                let old = b
                    .build_load(self.types.value_type, slot, "shared_old")
                    .unwrap();
                if is_load {
                    b.build_call(retain_fast, &[old.into()], "").unwrap();
                    // SAFETY: the call's result is replaced by an equal value, then erased.
                    unsafe {
                        LLVMReplaceAllUsesWith(call, old.as_value_ref());
                        LLVMInstructionEraseFromParent(call);
                    }
                } else {
                    let Some(v) = Self::rc_operand(inst, 1) else {
                        continue;
                    };
                    b.build_store(slot, v).unwrap();
                    b.build_call(release_fast, &[old.into()], "").unwrap();
                    // SAFETY: a void call with no uses.
                    unsafe { LLVMInstructionEraseFromParent(call) };
                }
            }
        }
    }

    /// `fast(data)`: the header test and count update inline, `slow(data)` otherwise.
    fn rc_build_obj_fast(
        &self,
        fast: FunctionValue<'ctx>,
        slow: FunctionValue<'ctx>,
        retain: bool,
    ) {
        let (i64t, i32t) = (self.types.i64_type, self.types.i32_type);
        let fast_flag = match self.module.get_global("__mdh_rc_fast") {
            Some(g) => g,
            None => {
                let g = self.module.add_global(i32t, None, "__mdh_rc_fast");
                g.set_linkage(Linkage::External);
                g
            }
        }
        .as_pointer_value();
        let b = self.context.create_builder();
        let entry = self.context.append_basic_block(fast, "entry");
        let check = self.context.append_basic_block(fast, "check");
        let quick = self.context.append_basic_block(fast, "quick");
        let call = self.context.append_basic_block(fast, "slow");
        let done = self.context.append_basic_block(fast, "done");
        let data = fast.get_nth_param(0).unwrap().into_int_value();

        b.position_at_end(entry);
        let is_null = b
            .build_int_compare(IntPredicate::EQ, data, i64t.const_zero(), "is_null")
            .unwrap();
        b.build_conditional_branch(is_null, call, check).unwrap();

        // The header is the 16 bytes before the payload: { i64 refcount, u32 kind, u32 flags }.
        b.position_at_end(check);
        let header_addr = b
            .build_int_sub(data, i64t.const_int(OBJ_HEADER_SIZE, false), "header_addr")
            .unwrap();
        let header = b
            .build_int_to_ptr(header_addr, self.types.str_ptr_type, "header")
            .unwrap();
        let kind_flags_ptr = unsafe {
            b.build_gep(
                self.context.i8_type(),
                header,
                &[i64t.const_int(8, false)],
                "kind_flags_ptr",
            )
        }
        .unwrap();
        let kind_flags = b
            .build_load(i64t, kind_flags_ptr, "kind_flags")
            .unwrap()
            .into_int_value();
        let flags = b
            .build_right_shift(kind_flags, i64t.const_int(32, false), false, "flags")
            .unwrap();
        // Exactly the magic: a header, not immortal, not DEAD, of a known kind
        // (MDH_OBJ_SLOW marks any other).
        let live = b
            .build_int_compare(
                IntPredicate::EQ,
                flags,
                i64t.const_int(OBJ_FLAG_MAGIC, false),
                "live",
            )
            .unwrap();
        let enabled = b
            .build_load(i32t, fast_flag, "rc_fast")
            .unwrap()
            .into_int_value();
        let enabled = b
            .build_int_compare(IntPredicate::NE, enabled, i32t.const_zero(), "rc_fast_on")
            .unwrap();
        let mut ok = b.build_and(live, enabled, "ok").unwrap();
        let count = b
            .build_load(i64t, header, "count")
            .unwrap()
            .into_int_value();
        if !retain {
            // The last release frees: the runtime does that.
            let shared = b
                .build_int_compare(IntPredicate::SGT, count, i64t.const_int(1, false), "shared")
                .unwrap();
            ok = b.build_and(ok, shared, "ok").unwrap();
        }
        b.build_conditional_branch(ok, quick, call).unwrap();

        b.position_at_end(quick);
        let one = i64t.const_int(1, false);
        let updated = if retain {
            b.build_int_add(count, one, "count_up")
        } else {
            b.build_int_sub(count, one, "count_down")
        }
        .unwrap();
        b.build_store(header, updated).unwrap();
        b.build_unconditional_branch(done).unwrap();

        b.position_at_end(call);
        b.build_call(slow, &[data.into()], "").unwrap();
        b.build_unconditional_branch(done).unwrap();

        b.position_at_end(done);
        b.build_return(None).unwrap();
    }

    /// `fast(value)`: the heap-tag test, then `obj(data)` (see `rc_emit_inline`).
    fn rc_build_value_fast(&self, fast: FunctionValue<'ctx>, obj: FunctionValue<'ctx>) {
        let i64t = self.types.i64_type;
        let b = self.context.create_builder();
        let entry = self.context.append_basic_block(fast, "entry");
        let call = self.context.append_basic_block(fast, "heap");
        let done = self.context.append_basic_block(fast, "done");
        let v = fast.get_nth_param(0).unwrap().into_struct_value();
        b.position_at_end(entry);
        let tag = b.build_extract_value(v, 0, "tag").unwrap().into_int_value();
        let data = b
            .build_extract_value(v, 1, "data")
            .unwrap()
            .into_int_value();
        let is_heap = self.rc_build_heap_test(&b, tag, "");
        b.build_conditional_branch(is_heap, call, done).unwrap();
        b.position_at_end(call);
        b.build_call(obj, &[data.into()], "").unwrap();
        b.build_unconditional_branch(done).unwrap();
        b.position_at_end(done);
        b.build_return(None).unwrap();
    }

    /// Before `ret v` in a function that returns an owned value: give the caller its +1,
    /// then release every open temp (the epilogue releases only the variable slots).
    ///
    /// A value an open temp holds (the `gie` expression's own result, say) moves to the
    /// caller (`rc_move_from_temp`), instead of a retain here and the temp's release
    /// just after it.
    pub(super) fn rc_return_value(&mut self, v: BasicValueEnum<'ctx>) {
        if self.rc_active() {
            if !self.rc_move_from_temp(v, true) {
                self.rc_take_inline(v);
            }
            self.rc_release_scopes_from(0);
        }
    }

    /// Every temp slot and variable slot registered for `function` (for try pinning).
    pub(super) fn rc_slots_of(&self, function: FunctionValue<'ctx>) -> Vec<PointerValue<'ctx>> {
        let mut slots: Vec<PointerValue<'ctx>> = self
            .rc_fn_index
            .get(&function)
            .map(|&i| self.rc_fn_slots[i].1.clone())
            .unwrap_or_default();
        if let Some(temps) = self.rc_temp_pins.get(&function) {
            slots.extend(temps.iter().copied());
        }
        slots
    }

    /// Every user global (top-level and imported-module `ken`s): main's epilogue releases
    /// them so the harness reports what really leaked.
    pub(super) fn rc_user_globals(&self) -> Vec<PointerValue<'ctx>> {
        self.module
            .get_globals()
            .filter(|g| {
                let name = g.get_name().to_string_lossy();
                (name.starts_with("__mdh_global_") || name.starts_with("__mdh_modvar_"))
                    && g.get_value_type() == self.types.value_type.into()
            })
            .map(|g| g.as_pointer_value())
            .collect()
    }

    /// The function epilogues: route every `ret` of each function with owned slots through
    /// one epilogue block that releases them (and, in `main`, every user global) first.
    pub(super) fn rc_emit_epilogues(&mut self, globals: &[PointerValue<'ctx>]) {
        if !self.rc_enabled {
            return;
        }
        let saved_block = self.builder.get_insert_block();
        let saved_function = self.current_function;
        let entries: Vec<(FunctionValue<'ctx>, Vec<PointerValue<'ctx>>)> = self.rc_fn_slots.clone();
        let main = self.module.get_function("main");
        let mut done: HashSet<FunctionValue<'ctx>> = HashSet::new();
        let mut work: Vec<(FunctionValue<'ctx>, Vec<PointerValue<'ctx>>)> = entries;
        if let Some(main) = main {
            if !self.rc_fn_index.contains_key(&main) {
                work.push((main, Vec::new()));
            }
        }
        for (function, slots) in work {
            if !done.insert(function) {
                continue;
            }
            let is_main = Some(function) == main;
            let rets: Vec<InstructionValue<'ctx>> = function
                .get_basic_blocks()
                .into_iter()
                .filter_map(|b| b.get_terminator())
                .filter(|t| t.get_opcode() == InstructionOpcode::Return)
                .collect();
            if rets.is_empty() || (slots.is_empty() && !is_main) {
                continue;
            }
            self.current_function = Some(function);
            let ret_type = function.get_type().get_return_type();
            let ret_slot = ret_type.map(|t| {
                let entry = function.get_first_basic_block().expect("entry");
                let b = self.context.create_builder();
                match entry.get_first_instruction() {
                    Some(i) => b.position_before(&i),
                    None => b.position_at_end(entry),
                }
                b.build_alloca(t, "rc_ret").unwrap()
            });
            let epilogue = self.context.append_basic_block(function, "rc_epilogue");
            for ret in rets {
                let value = match ret.get_operand(0) {
                    Some(inkwell::values::Operand::Value(v)) => Some(v),
                    _ => None,
                };
                let block = ret.get_parent().expect("ret block");
                ret.erase_from_basic_block();
                self.builder.position_at_end(block);
                if let (Some(slot), Some(v)) = (ret_slot, value) {
                    self.builder.build_store(slot, v).unwrap();
                }
                self.builder.build_unconditional_branch(epilogue).unwrap();
            }
            self.builder.position_at_end(epilogue);
            for slot in &slots {
                let v = self
                    .builder
                    .build_load(self.types.value_type, *slot, "rc_slot_done")
                    .unwrap();
                self.rc_release_unknown(v);
            }
            if is_main {
                // A thread still running may read the globals: swap each to nil (under
                // the slot lock once threads exist) and release what it held.
                let nil = self.make_nil();
                for g in globals {
                    self.builder
                        .build_call(self.libc.shared_store, &[(*g).into(), nil.into()], "")
                        .unwrap();
                }
            }
            match (ret_slot, ret_type) {
                (Some(slot), Some(t)) => {
                    let v = self.builder.build_load(t, slot, "rc_ret_val").unwrap();
                    self.builder.build_return(Some(&v)).unwrap();
                }
                _ => {
                    self.builder.build_return(None).unwrap();
                }
            }
        }
        self.current_function = saved_function;
        if let Some(block) = saved_block {
            self.builder.position_at_end(block);
        }
    }
}
