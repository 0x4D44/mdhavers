//! All-or-nothing borrowed local slots (RC7 C).
use super::*;

#[cfg(test)]
#[path = "slot_rule_tests.rs"]
mod rule_tests;

pub(super) struct RcSlotStore<'ctx> {
    pub(super) slot: PointerValue<'ctx>,
    pub(super) value: BasicValueEnum<'ctx>,
    pub(super) store: InstructionValue<'ctx>,
    pub(super) marker: Option<InstructionValue<'ctx>>,
    pub(super) moved_borrow: Option<usize>,
}

pub(super) struct RcSlotRelease<'ctx> {
    slot: PointerValue<'ctx>,
    marker: InstructionValue<'ctx>,
    inline: bool,
}

#[derive(Default)]
pub(crate) struct RcSlotState<'ctx> {
    pub(super) params: HashSet<usize>,
    pub(super) excluded: HashSet<usize>,
    pub(super) temps: HashSet<usize>,
    pub(super) stores: Vec<RcSlotStore<'ctx>>,
    pub(super) releases: Vec<RcSlotRelease<'ctx>>,
    pub(super) candidates: HashSet<usize>,
    pub(super) borrowed: HashSet<usize>,
    #[cfg(test)]
    container_store_rejections: std::cell::RefCell<HashSet<usize>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub(crate) fn rc_exclude_slot(&mut self, slot: PointerValue<'ctx>) {
        self.rc_slots.excluded.insert(value_id(slot));
    }

    pub(super) fn rc_exclude_temp(&mut self, slot: PointerValue<'ctx>) {
        self.rc_exclude_slot(slot);
        self.rc_slots.temps.insert(value_id(slot));
    }

    pub(super) fn rc_slot_release(
        &mut self,
        slot: PointerValue<'ctx>,
        value: BasicValueEnum<'ctx>,
        inline: bool,
    ) {
        if self.rc_slots.excluded.contains(&value_id(slot))
            || self.rc_slots.params.contains(&value_id(slot))
        {
            if inline {
                self.rc_release_unknown(value);
            } else {
                self.rc_release(value);
            }
            return;
        }
        let f = match self.module.get_function(RC_BORROW_RELEASE_MARKER) {
            Some(f) => f,
            None => self.module.add_function(
                RC_BORROW_RELEASE_MARKER,
                self.libc.rc_release.get_type(),
                Some(Linkage::External),
            ),
        };
        let call = self.builder.build_call(f, &[value.into()], "").unwrap();
        // SAFETY: this is the call just built in the current module.
        let inst = unsafe { InstructionValue::new(call.as_value_ref()) };
        self.rc_slots.releases.push(RcSlotRelease {
            slot,
            marker: inst,
            inline,
        });
    }

    /// Backward proof of a chain. Only data-bearing operands are traversed: index and
    /// condition operands never supply the owner. Fresh edges fail even if R1 has
    /// already consumed their +1. The result records each originating owner load.
    fn rc_slot_roots(
        &self,
        value: BasicValueEnum<'ctx>,
        owners: &HashSet<usize>,
    ) -> Option<Vec<(PointerValue<'ctx>, InstructionValue<'ctx>)>> {
        let mut roots = Vec::new();
        let mut work = vec![value];
        let mut seen = HashSet::new();
        while let Some(value) = work.pop() {
            if !seen.insert(value_id(value)) {
                continue;
            }
            if seen.len() > 4096 {
                return None;
            }
            if self.rc_class(value) == RcClass::Skip {
                continue;
            }
            let inst = value.as_instruction_value()?;
            match inst.get_opcode() {
                InstructionOpcode::Load => {
                    let ptr = Self::rc_operand(inst, 0)?;
                    if self.rc_is_value(value) && owners.contains(&value_id(ptr)) {
                        let BasicValueEnum::PointerValue(slot) = ptr else {
                            return None;
                        };
                        roots.push((slot, inst));
                    } else {
                        // A load through an element/field pointer continues the chain.
                        // A raw shadow alloca is deliberately not a root.
                        if ptr
                            .as_instruction_value()
                            .is_some_and(|i| i.get_opcode() == InstructionOpcode::Alloca)
                        {
                            return None;
                        }
                        work.push(ptr);
                    }
                }
                InstructionOpcode::ExtractValue
                | InstructionOpcode::IntToPtr
                | InstructionOpcode::PtrToInt
                | InstructionOpcode::BitCast
                | InstructionOpcode::AddrSpaceCast
                | InstructionOpcode::GetElementPtr
                | InstructionOpcode::Freeze => work.push(Self::rc_operand(inst, 0)?),
                InstructionOpcode::InsertValue => {
                    // MdhValue's payload is i64; tag inserts do not change provenance.
                    let inserted = Self::rc_operand(inst, 1)?;
                    if matches!(inserted, BasicValueEnum::IntValue(v) if v.get_type().get_bit_width() == 64)
                    {
                        work.push(inserted);
                    } else {
                        work.push(Self::rc_operand(inst, 0)?);
                    }
                }
                InstructionOpcode::Phi => {
                    let phi = inkwell::values::PhiValue::try_from(inst).ok()?;
                    for i in 0..phi.count_incoming() {
                        work.push(phi.get_incoming(i)?.0);
                    }
                }
                InstructionOpcode::Select => {
                    work.push(Self::rc_operand(inst, 1)?);
                    work.push(Self::rc_operand(inst, 2)?);
                }
                InstructionOpcode::Call => {
                    let callee = Self::rc_callee(inst)?;
                    let name = callee.get_name().to_string_lossy();
                    match ownership_of(&name) {
                        Some((_, Ret::Arg | Ret::Interior)) => {
                            work.push(Self::rc_operand(inst, 0)?)
                        }
                        Some((_, Ret::Stored)) if name == "__mdh_dict_get" => {
                            work.push(Self::rc_operand(inst, 0)?)
                        }
                        _ => return None,
                    }
                }
                _ => return None,
            }
        }
        (!roots.is_empty()).then_some(roots)
    }

    fn rc_slot_owners(&self) -> HashSet<usize> {
        self.rc_fn_slots
            .iter()
            .flat_map(|(_, slots)| slots.iter())
            .map(|s| value_id(*s))
            .chain(self.rc_slots.params.iter().copied())
            .chain(self.rc_user_globals().into_iter().map(value_id))
            .filter(|id| !self.rc_slots.temps.contains(id))
            .collect()
    }

    /// Discover the complete structural set before considering whether any root or
    /// interval is safe. A later failing candidate never becomes an eligible root.
    pub(super) fn rc_collect_slot_candidates(&mut self) {
        let owners = self.rc_slot_owners();
        self.rc_slots.candidates = self
            .rc_slots
            .stores
            .iter()
            .filter(|s| {
                !self.rc_slots.params.contains(&value_id(s.slot))
                    && !self.rc_slots.excluded.contains(&value_id(s.slot))
                    && self.rc_slot_roots(s.value, &owners).is_some()
            })
            .map(|s| value_id(s.slot))
            .collect();
    }

    /// Collect every use of slot loads and of their derivatives. Owning stores and
    /// returns take a reference themselves; candidate stores are explicit escapes.
    fn rc_slot_uses(
        &self,
        slot: PointerValue<'ctx>,
    ) -> Option<(HashSet<usize>, HashSet<usize>, HashSet<BasicBlock<'ctx>>)> {
        use inkwell::llvm_sys::core::{
            LLVMGetFirstUse, LLVMGetNextUse, LLVMGetUser, LLVMIsAInstruction,
        };
        let mut work = Vec::new();
        let mut seen = HashSet::new();
        let mut uses = HashSet::new();
        let mut escapes = HashSet::new();
        let mut blocks = HashSet::new();
        // SAFETY: slot and its use list belong to this live module.
        let mut u = unsafe { LLVMGetFirstUse(slot.as_value_ref()) };
        while !u.is_null() {
            let raw = unsafe { LLVMGetUser(u) };
            u = unsafe { LLVMGetNextUse(u) };
            if unsafe { LLVMIsAInstruction(raw) }.is_null() {
                return None;
            }
            let inst = unsafe { InstructionValue::new(raw) };
            match inst.get_opcode() {
                InstructionOpcode::Load => {
                    work.push(inst.as_value_ref());
                }
                InstructionOpcode::Store if Self::rc_operand(inst, 1) == Some(slot.into()) => {}
                _ => return None,
            }
        }
        while let Some(raw) = work.pop() {
            if !seen.insert(raw as usize) {
                continue;
            }
            if seen.len() > 4096 {
                return None;
            }
            let mut u = unsafe { LLVMGetFirstUse(raw) };
            while !u.is_null() {
                let user = unsafe { LLVMGetUser(u) };
                u = unsafe { LLVMGetNextUse(u) };
                if unsafe { LLVMIsAInstruction(user) }.is_null() {
                    return None;
                }
                let inst = unsafe { InstructionValue::new(user) };
                // Its own cleanup is removed together with the candidate's retain.
                if self
                    .rc_slots
                    .releases
                    .iter()
                    .any(|r| r.slot == slot && r.marker == inst)
                {
                    continue;
                }
                if Self::rc_callee(inst).is_some_and(|f| {
                    matches!(
                        f.get_name().to_bytes(),
                        b"__mdh_rc_borrow_retain" | b"__mdh_retain" | b"__mdh_rc_retain_obj"
                    )
                }) {
                    continue;
                }
                // Keep the definition too: a pre-origin load used after an
                // overwrite must not be mistaken for the next dynamic instance.
                let definition = unsafe { InstructionValue::new(raw) };
                uses.insert(value_id(definition));
                blocks.insert(definition.get_parent()?);
                uses.insert(value_id(inst));
                blocks.insert(inst.get_parent()?);
                let derived = match inst.get_opcode() {
                    InstructionOpcode::Store => {
                        if matches!(Self::rc_operand(inst, 0), Some(v) if v.as_value_ref() == raw) {
                            let dest = Self::rc_operand(inst, 1)?;
                            if self.rc_slots.candidates.contains(&value_id(dest))
                                && dest != BasicValueEnum::PointerValue(slot)
                            {
                                escapes.insert(value_id(inst));
                            } else if !self.rc_is_own_store_destination(dest) {
                                if let Some(loads) = Self::rc_scratch_loads(dest) {
                                    for load in loads {
                                        work.push(load.as_value_ref());
                                    }
                                } else if dest
                                    .as_instruction_value()
                                    .is_some_and(|i| i.get_opcode() == InstructionOpcode::Alloca)
                                {
                                    escapes.insert(value_id(inst));
                                }
                            }
                        }
                        false
                    }
                    InstructionOpcode::Return => false,
                    InstructionOpcode::Invoke
                    | InstructionOpcode::CallBr
                    | InstructionOpcode::AtomicRMW
                    | InstructionOpcode::AtomicCmpXchg => return None,
                    InstructionOpcode::Call => {
                        // An explicit +1 result has an independent owner. Even when
                        // it aliases an argument, that return reference protects it;
                        // C need not keep the argument's rooted chain alive for it.
                        // Ordinary R5 result provenance remains unchanged.
                        !Self::rc_callee(inst).is_some_and(|callee| {
                            rc_compiled_return_is_owned(callee, &self.rc_owned_return_fns)
                        }) && Self::rc_call_result_derived(inst, raw)
                    }
                    InstructionOpcode::ExtractValue
                    | InstructionOpcode::InsertValue
                    | InstructionOpcode::PtrToInt
                    | InstructionOpcode::IntToPtr
                    | InstructionOpcode::BitCast
                    | InstructionOpcode::AddrSpaceCast
                    | InstructionOpcode::GetElementPtr
                    | InstructionOpcode::Load
                    | InstructionOpcode::Phi
                    | InstructionOpcode::Select
                    | InstructionOpcode::Freeze => true,
                    _ => false,
                };
                if derived {
                    work.push(user);
                }
            }
        }
        Some((uses, escapes, blocks))
    }

    fn rc_is_own_store_destination(&self, dest: BasicValueEnum<'ctx>) -> bool {
        self.rc_fn_slots
            .iter()
            .flat_map(|(_, slots)| slots.iter())
            .any(|s| value_id(*s) == value_id(dest))
            || self
                .rc_temp_pins
                .values()
                .flatten()
                .any(|s| value_id(*s) == value_id(dest))
            || dest.as_instruction_value().is_some_and(|i| {
                i.get_opcode() == InstructionOpcode::Alloca
                    && i.get_name().is_some_and(|n| n.to_bytes() == b"rc_ret")
            })
    }

    fn rc_slot_region_safe(
        &self,
        start: InstructionValue<'ctx>,
        root: PointerValue<'ctx>,
        slot: PointerValue<'ctx>,
        region: (&HashSet<usize>, &HashSet<usize>, &HashSet<BasicBlock<'ctx>>),
        holders: &HashSet<usize>,
        effects: &RcCEffects<'ctx>,
    ) -> bool {
        let (uses, escapes, use_blocks) = region;
        let Some(start_bb) = start.get_parent() else {
            return false;
        };
        let Some(function) = start_bb.get_parent() else {
            return false;
        };
        let mut preds: HashMap<BasicBlock<'ctx>, Vec<BasicBlock<'ctx>>> = HashMap::new();
        for bb in function.get_basic_blocks() {
            for next in Self::rc_successors(bb) {
                preds.entry(next).or_default().push(bb);
            }
        }
        // Every live definition/use must belong to this evaluation of the root.
        // Reject paths entering one without passing its root block (and instructions
        // before the load in that block). This also rejects values carried across
        // the origin by a phi, while allowing ordinary repeated loop evaluations.
        let mut before = start_bb.get_first_instruction();
        while let Some(inst) = before {
            if inst == start {
                break;
            }
            if uses.contains(&value_id(inst)) {
                return false;
            }
            before = inst.get_next_instruction();
        }
        let Some(entry) = function.get_first_basic_block() else {
            return false;
        };
        let mut bypass = vec![entry];
        let mut bypassed = HashSet::new();
        while let Some(bb) = bypass.pop() {
            if bb == start_bb || !bypassed.insert(bb) {
                continue;
            }
            if use_blocks.contains(&bb) {
                return false;
            }
            bypass.extend(Self::rc_successors(bb));
        }
        let mut reaches_use = HashSet::new();
        let mut work: Vec<_> = use_blocks.iter().copied().collect();
        while let Some(bb) = work.pop() {
            if !reaches_use.insert(bb) {
                continue;
            }
            if reaches_use.len() > 8192 {
                return false;
            }
            if bb == start_bb {
                continue;
            }
            if let Some(ps) = preds.get(&bb) {
                work.extend(ps.iter().copied());
            }
        }
        let scan = |first: Option<InstructionValue<'ctx>>, bb: BasicBlock<'ctx>| {
            let through = Self::rc_successors(bb)
                .iter()
                .any(|s| reaches_use.contains(s));
            let mut last = None;
            if !through {
                let mut i = first;
                while let Some(inst) = i {
                    if uses.contains(&value_id(inst)) {
                        last = Some(value_id(inst));
                    }
                    i = inst.get_next_instruction();
                }
                if last.is_none() {
                    return true;
                }
            }
            let mut i = first;
            while let Some(inst) = i {
                i = inst.get_next_instruction();
                if self
                    .rc_slots
                    .releases
                    .iter()
                    .any(|r| r.slot == slot && r.marker == inst)
                {
                    continue;
                }
                let effect = rc_c_call_effect(&self.module, inst);
                if escapes.contains(&value_id(inst)) {
                    return false;
                }
                if inst.get_opcode() == InstructionOpcode::Store {
                    let Some(dest) = Self::rc_operand(inst, 1) else {
                        return false;
                    };
                    if dest == BasicValueEnum::PointerValue(root) {
                        return false;
                    }
                    // Only direct local/global variable stores can be non-destructive.
                    let variable = dest
                        .as_instruction_value()
                        .is_some_and(|p| p.get_opcode() == InstructionOpcode::Alloca)
                        || matches!(dest, BasicValueEnum::PointerValue(p) if self.rc_is_shared_slot(p));
                    if !variable {
                        #[cfg(test)]
                        self.rc_slots
                            .container_store_rejections
                            .borrow_mut()
                            .insert(value_id(slot));
                        return false;
                    }
                }
                match effect {
                    RcCallEffect::RunsCode => {
                        if !Self::rc_callee(inst).is_some_and(|f| {
                            f.count_basic_blocks() > 0
                                && !effects.c_drops_heap.contains(&f)
                                && !effects.c_writes_globals.contains(&f)
                        }) {
                            return false;
                        }
                    }
                    RcCallEffect::Drops => {
                        let release = Self::rc_callee(inst).is_some_and(|f| {
                            matches!(
                                f.get_name().to_bytes(),
                                b"__mdh_release"
                                    | b"__mdh_rc_release_obj"
                                    | b"__mdh_rc_borrow_release"
                            )
                        });
                        if !release
                            || !Self::rc_operand(inst, 0).is_some_and(|v| {
                                rc_release_is_own(v, function, holders, &self.rc_owned_return_fns)
                            })
                        {
                            return false;
                        }
                    }
                    RcCallEffect::None => {}
                }
                if !through && Some(value_id(inst)) == last {
                    break;
                }
            }
            true
        };
        if !scan(start.get_next_instruction(), start_bb) {
            return false;
        }
        let mut seen = HashSet::new();
        let mut work = Self::rc_successors(start_bb);
        while let Some(bb) = work.pop() {
            if bb == start_bb || !reaches_use.contains(&bb) || !seen.insert(bb) {
                continue;
            }
            if seen.len() > 8192 {
                return false;
            }
            if !scan(bb.get_first_instruction(), bb) {
                return false;
            }
            work.extend(Self::rc_successors(bb));
        }
        true
    }

    pub(super) fn rc_decide_slots(
        &mut self,
        threaded: bool,
        holders: &HashSet<usize>,
        effects: &RcCEffects<'ctx>,
    ) -> HashSet<usize> {
        let owners = self.rc_slot_owners();
        let mut borrowed = HashSet::new();
        for id in &self.rc_slots.candidates {
            if threaded {
                continue;
            }
            let stores: Vec<_> = self
                .rc_slots
                .stores
                .iter()
                .filter(|s| value_id(s.slot) == *id)
                .collect();
            let Some(first) = stores.first() else {
                continue;
            };
            let slot = first.slot;
            let Some(function) = slot
                .as_instruction()
                .and_then(|i| i.get_parent())
                .and_then(|bb| bb.get_parent())
            else {
                continue;
            };
            let forbidden = function.get_basic_blocks().iter().any(|bb| {
                let mut next = bb.get_first_instruction();
                while let Some(inst) = next {
                    next = inst.get_next_instruction();
                    if matches!(
                        inst.get_opcode(),
                        InstructionOpcode::Invoke | InstructionOpcode::CallBr
                    ) || Self::rc_callee(inst)
                        .is_some_and(|f| f.get_name().to_string_lossy().contains("setjmp"))
                    {
                        return true;
                    }
                }
                false
            });
            if forbidden {
                continue;
            }
            let mut root = None;
            let mut starts = Vec::new();
            let mut safe = true;
            for store in &stores {
                let Some(roots) = self.rc_slot_roots(store.value, &owners) else {
                    safe = false;
                    break;
                };
                for (r, load) in roots {
                    if r == slot
                        || self.rc_slots.candidates.contains(&value_id(r))
                        || root.is_some_and(|old| old != r)
                    {
                        safe = false;
                        break;
                    }
                    root = Some(r);
                    starts.push(load);
                }
                if !safe {
                    break;
                }
            }
            if !safe {
                continue;
            }
            // Every non-initialization store must be one of the ownership-managed
            // stores above. This rejects raw self-appends and other fresh rewrites.
            if !self.rc_slot_stores_complete(slot, &stores) {
                continue;
            }
            let Some(root) = root else {
                continue;
            };
            let Some((mut uses, escapes, mut blocks)) = self.rc_slot_uses(slot) else {
                continue;
            };
            for store in &stores {
                uses.insert(value_id(store.store));
                if let Some(bb) = store.store.get_parent() {
                    blocks.insert(bb);
                }
            }
            if starts.into_iter().all(|load| {
                self.rc_slot_region_safe(
                    load,
                    root,
                    slot,
                    (&uses, &escapes, &blocks),
                    holders,
                    effects,
                )
            }) {
                borrowed.insert(*id);
            }
        }
        self.rc_slots.borrowed = borrowed.clone();
        borrowed
    }

    fn rc_slot_stores_complete(
        &self,
        slot: PointerValue<'ctx>,
        stores: &[&RcSlotStore<'ctx>],
    ) -> bool {
        use inkwell::llvm_sys::core::{
            LLVMGetFirstUse, LLVMGetNextUse, LLVMGetUser, LLVMIsAInstruction,
        };
        let mut u = unsafe { LLVMGetFirstUse(slot.as_value_ref()) };
        while !u.is_null() {
            let user = unsafe { LLVMGetUser(u) };
            u = unsafe { LLVMGetNextUse(u) };
            if unsafe { LLVMIsAInstruction(user) }.is_null() {
                return false;
            }
            let inst = unsafe { InstructionValue::new(user) };
            if inst.get_opcode() == InstructionOpcode::Store
                && Self::rc_operand(inst, 1) == Some(slot.into())
                && !stores.iter().any(|s| s.store == inst)
                && !Self::rc_operand(inst, 0).is_some_and(|v| self.rc_class(v) == RcClass::Skip)
            {
                return false;
            }
        }
        true
    }

    pub(super) fn rc_finish_slots(&mut self, borrowed: &HashSet<usize>) -> HashSet<usize> {
        let b = self.context.create_builder();
        let mut moved = HashSet::new();
        for s in &self.rc_slots.stores {
            if let Some(marker) = s.marker {
                if borrowed.contains(&value_id(s.slot)) {
                    marker.erase_from_basic_block();
                } else {
                    self.rc_marker_to_retain(marker, &b);
                }
            }
            if borrowed.contains(&value_id(s.slot)) {
                if let Some(index) = s.moved_borrow {
                    moved.insert(index);
                }
            }
        }
        for release in std::mem::take(&mut self.rc_slots.releases) {
            if !borrowed.contains(&value_id(release.slot)) {
                if let Some(value) = Self::rc_operand(release.marker, 0) {
                    b.position_before(&release.marker);
                    let call = b
                        .build_call(self.libc.rc_release, &[value.into()], "")
                        .unwrap();
                    if release.inline {
                        // Expand the guard after R5 decisions, so splitting blocks
                        // cannot invalidate their predecessor maps or phi handles.
                        self.rc_slots.releases.push(RcSlotRelease {
                            // SAFETY: this is the call just built in this module.
                            marker: unsafe { InstructionValue::new(call.as_value_ref()) },
                            ..release
                        });
                    }
                }
            }
            release.marker.erase_from_basic_block();
        }
        if let Some(f) = self.module.get_function(RC_BORROW_RELEASE_MARKER) {
            // All marker uses are created and accounted for above.
            if f.as_global_value()
                .as_pointer_value()
                .get_first_use()
                .is_none()
            {
                unsafe { f.delete() };
            }
        }
        moved
    }

    /// Restore the original inline release at its exact position. All ownership and
    /// effect decisions are complete; remaining release records are owning inline sites.
    pub(super) fn rc_lower_inline_slot_releases(&mut self) {
        use inkwell::llvm_sys::core::LLVMReplaceAllUsesWith;
        use inkwell::values::PhiValue;

        let b = self.context.create_builder();
        for release in std::mem::take(&mut self.rc_slots.releases) {
            let marker = release.marker;
            let value = Self::rc_operand(marker, 0).expect("owning release value");
            let from = marker.get_parent().expect("owning release block");
            let function = from.get_parent().expect("owning release function");
            let call = self.context.append_basic_block(function, "rc_release");
            let done = self.context.append_basic_block(function, "rc_release_done");
            b.position_at_end(done);
            let mut next = marker.get_next_instruction();
            while let Some(inst) = next {
                next = inst.get_next_instruction();
                let name = inst
                    .get_name()
                    .map(|name| name.to_string_lossy().into_owned());
                inst.remove_from_basic_block();
                b.insert_instruction(&inst, name.as_deref());
            }
            // The old terminator moved to done. Preserve every successor phi's
            // values and edge multiplicity, changing only that predecessor's name.
            for successor in Self::rc_successors(done) {
                let mut next = successor.get_first_instruction();
                while let Some(inst) = next {
                    next = inst.get_next_instruction();
                    if inst.get_opcode() != InstructionOpcode::Phi {
                        break;
                    }
                    let phi = PhiValue::try_from(inst).expect("phi opcode");
                    let incoming: Vec<_> = phi.get_incomings().collect();
                    if !incoming.iter().any(|(_, block)| *block == from) {
                        continue;
                    }
                    let name = phi.get_name().to_string_lossy().into_owned();
                    phi.set_name("");
                    b.position_before(&inst);
                    let replacement = b.build_phi(phi.as_basic_value().get_type(), &name).unwrap();
                    for (value, block) in incoming {
                        replacement
                            .add_incoming(&[(&value, if block == from { done } else { block })]);
                    }
                    // SAFETY: same typed phi, with the same incoming values. No
                    // ownership records are queried after this final CFG lowering.
                    unsafe {
                        LLVMReplaceAllUsesWith(phi.as_value_ref(), replacement.as_value_ref())
                    };
                    inst.erase_from_basic_block();
                }
            }
            b.position_before(&marker);
            let value = value.into_struct_value();
            let tag = b
                .build_extract_value(value, 0, "rc_tag")
                .unwrap()
                .into_int_value();
            let data = b.build_extract_value(value, 1, "rc_data").unwrap();
            let is_heap = self.rc_build_heap_test(&b, tag, "rc_", true);
            marker.erase_from_basic_block();
            b.position_at_end(from);
            b.build_conditional_branch(is_heap, call, done).unwrap();
            b.position_at_end(call);
            b.build_call(self.libc.rc_release_obj, &[data.into()], "")
                .unwrap();
            b.build_unconditional_branch(done).unwrap();
        }
    }
}

impl<'ctx> CodeGen<'ctx> {
    /// Native-tag arms are removed before either summary is computed. Keep Rust-side
    /// handles synchronized before erasing old phis or dead blocks.
    pub(super) fn rc_remove_native_arms(&mut self) {
        let edits = rc_elide_native_arms(self.context, &self.module);
        self.rc_apply_arm_edits(edits);
    }

    pub(super) fn rc_apply_arm_edits(&mut self, edits: native_arms::NativeArmEdits<'ctx>) {
        let dead: HashSet<_> = edits.blocks_to_delete.iter().copied().collect();
        let mut index_map = HashMap::new();
        let borrows = std::mem::take(&mut self.rc_borrows);
        for (old_index, mut rec) in borrows.into_iter().enumerate() {
            if rec
                .park
                .is_some_and(|(p, _)| p.get_parent().is_some_and(|bb| dead.contains(&bb)))
                || rec
                    .value
                    .as_instruction_value()
                    .is_some_and(|v| v.get_parent().is_some_and(|bb| dead.contains(&bb)))
            {
                continue;
            }
            // One removed phi edge does not invalidate the other edges' borrows.
            // Preserve their live retain/park pairing and the remapped phi value.
            rec.markers
                .retain(|m| !m.get_parent().is_some_and(|bb| dead.contains(&bb)));
            if rec.markers.is_empty() {
                continue;
            }
            for (old, new) in &edits.replacements {
                if rec.value == *old {
                    rec.value = *new;
                }
            }
            index_map.insert(old_index, self.rc_borrows.len());
            self.rc_borrows.push(rec);
        }
        self.rc_slots
            .stores
            .retain(|s| !s.store.get_parent().is_some_and(|bb| dead.contains(&bb)));
        self.rc_slots
            .releases
            .retain(|r| !r.marker.get_parent().is_some_and(|bb| dead.contains(&bb)));
        for s in &mut self.rc_slots.stores {
            for (old, new) in &edits.replacements {
                if s.value == *old {
                    s.value = *new;
                }
            }
            s.moved_borrow = s.moved_borrow.and_then(|i| index_map.get(&i).copied());
        }
        for (old, new) in edits.replacements {
            if self.rc_owned_values.remove(&value_id(old)) {
                self.rc_owned_values.insert(value_id(new));
            }
            if self.rc_taken.remove(&value_id(old)) {
                self.rc_taken.insert(value_id(new));
            }
            if let Some(inst) = old.as_instruction_value() {
                inst.erase_from_basic_block();
            }
        }
        // No retained record refers to these instructions. Break dead-region SSA uses
        // before deletion, and remove terminators to sever its internal CFG edges.
        for bb in &edits.blocks_to_delete {
            if let Some(term) = bb.get_terminator() {
                term.erase_from_basic_block();
            }
        }
        for bb in &edits.blocks_to_delete {
            let mut next = bb.get_first_instruction();
            while let Some(inst) = next {
                next = inst.get_next_instruction();
                // SAFETY: only dead-region instructions are rewritten; void values
                // cannot have uses and have no undef representation.
                unsafe {
                    use inkwell::llvm_sys::core::{
                        LLVMGetTypeKind, LLVMGetUndef, LLVMReplaceAllUsesWith, LLVMTypeOf,
                    };
                    let ty = LLVMTypeOf(inst.as_value_ref());
                    if LLVMGetTypeKind(ty) != inkwell::llvm_sys::LLVMTypeKind::LLVMVoidTypeKind {
                        LLVMReplaceAllUsesWith(inst.as_value_ref(), LLVMGetUndef(ty));
                    }
                }
            }
        }
        for bb in edits.blocks_to_delete {
            // SAFETY: all external uses were replaced and region CFG edges removed.
            unsafe {
                bb.delete().expect("delete impossible native arm");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_and_check(source: &str, function: &str, slot_name: &str, expected: bool) {
        let context = Context::create();
        let mut cg = CodeGen::new_with_refcounting(&context, "rc7_c_decision", true);
        let program = crate::parser::parse(source).expect("parse C decision fixture");
        cg.compile(&program).expect("compile C decision fixture");
        cg.module.verify().expect("valid C decision IR");
        let function = cg.module.get_function(function).expect("fixture function");
        let slot = function
            .get_basic_blocks()
            .into_iter()
            .find_map(|bb| {
                let mut next = bb.get_first_instruction();
                while let Some(inst) = next {
                    next = inst.get_next_instruction();
                    if inst.get_opcode() == InstructionOpcode::Load
                        && inst.get_name().is_some_and(|name| {
                            let bytes = name.to_bytes();
                            bytes == slot_name.as_bytes()
                                || bytes
                                    .strip_prefix(slot_name.as_bytes())
                                    .is_some_and(|suffix| {
                                        !suffix.is_empty() && suffix.iter().all(u8::is_ascii_digit)
                                    })
                        })
                    {
                        if let Some(BasicValueEnum::PointerValue(slot)) =
                            CodeGen::rc_operand(inst, 0)
                        {
                            if slot
                                .as_instruction()
                                .is_some_and(|i| i.get_opcode() == InstructionOpcode::Alloca)
                            {
                                return Some(value_id(slot));
                            }
                        }
                    }
                }
                None
            })
            .expect("fixture slot");
        assert_eq!(
            cg.rc_slots.borrowed.contains(&slot),
            expected,
            "borrowed decision for {slot_name}"
        );
        if slot_name == "pj" {
            assert!(
                cg.rc_slots
                    .container_store_rejections
                    .borrow()
                    .contains(&slot),
                "R5-8: pj must reject for C(d), a live container-field store"
            );
        }
    }

    #[test]
    fn rc7_c_particle_pj_stays_owning_for_cd() {
        compile_and_check(
            include_str!("../../../../benchmarks/rc/particle_sim.braw"),
            "__mdh_fn_update_velocities",
            // R5-8: tag-set dataflow / audited list initialization is follow-up work.
            "pj",
            false,
        );
    }

    #[test]
    fn rc7_c_rejects_candidate_root_c9() {
        compile_and_check("dae run() {\n ken xs = [[[1]]]\n ken a = xs[0]\n ken p = a[0]\n xs = []\n blether p[0]\n}\nrun()\n", "__mdh_fn_run", "p", false);
    }

    #[test]
    fn rc7_c_rejects_same_function_setjmp() {
        compile_and_check("dae run() {\n ken xs = [[1]]\n ken p = xs[0]\n hae_a_bash { blether p[0] } gin_it_gangs_wrang e { blether e }\n}\nrun()\n", "__mdh_fn_run", "p", false);
    }
}
