//! Synchronization of shared slots without reference counting (MDH-BUG-FLU-00216).

use super::*;
use inkwell::values::{AsValueRef, BasicValue, InstructionOpcode};

impl CodeGen<'_> {
    /// Defer this until every function/import is compiled, so references to thread_spawn
    /// through function values count too. With no spawn reference, leave the IR byte for
    /// byte unchanged. The RC-on path already synchronizes its shared accesses.
    pub(super) fn synchronize_uncounted_shared_slots(&self) {
        if self.rc_enabled
            || !self
                .module
                .get_function("__mdh_thread_spawn")
                .is_some_and(|f| {
                    f.as_global_value()
                        .as_pointer_value()
                        .get_first_use()
                        .is_some()
                })
        {
            return;
        }

        let mut accesses = Vec::new();
        for function in self.module.get_functions() {
            for block in function.get_basic_blocks() {
                let mut next = block.get_first_instruction();
                while let Some(inst) = next {
                    next = inst.get_next_instruction();
                    let pointer_index = match inst.get_opcode() {
                        InstructionOpcode::Load
                            if inst.get_type() == self.types.value_type.into() =>
                        {
                            0
                        }
                        InstructionOpcode::Store => {
                            let Some(inkwell::values::Operand::Value(v)) = inst.get_operand(0)
                            else {
                                continue;
                            };
                            if !self.rc_is_value(v) {
                                continue;
                            }
                            1
                        }
                        _ => continue,
                    };
                    let Some(inkwell::values::Operand::Value(BasicValueEnum::PointerValue(ptr))) =
                        inst.get_operand(pointer_index)
                    else {
                        continue;
                    };
                    // Match RC-on's globals and box_get/box_set's captured elements.
                    // Ordinary container elements still require the caller's mutex.
                    if self.rc_is_shared_slot(ptr) || self.uncounted_cells.contains(&ptr) {
                        accesses.push((inst, pointer_index));
                    }
                }
            }
        }
        if accesses.is_empty() {
            return;
        }

        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let load = self.module.add_function(
            "__mdh_shared_load_uncounted",
            self.types.value_type.fn_type(&[ptr_type.into()], false),
            Some(Linkage::External),
        );
        let store = self.module.add_function(
            "__mdh_shared_store_uncounted",
            self.context
                .void_type()
                .fn_type(&[ptr_type.into(), self.types.value_type.into()], false),
            Some(Linkage::External),
        );
        let nounwind = Attribute::get_named_enum_kind_id("nounwind");
        for helper in [load, store] {
            helper.add_attribute(
                AttributeLoc::Function,
                self.context.create_enum_attribute(nounwind, 0),
            );
        }
        let builder = self.context.create_builder();
        for (inst, pointer_index) in accesses {
            // Earlier load replacements can also update this store's value. Read
            // operands now, rather than keeping handles to instructions since erased.
            let Some(inkwell::values::Operand::Value(ptr)) = inst.get_operand(pointer_index) else {
                unreachable!("shared slot pointer");
            };
            builder.position_before(&inst);
            if pointer_index == 1 {
                let Some(inkwell::values::Operand::Value(value)) = inst.get_operand(0) else {
                    unreachable!("shared slot value");
                };
                builder
                    .build_call(store, &[ptr.into(), value.into()], "")
                    .unwrap();
            } else {
                let value = builder
                    .build_call(load, &[ptr.into()], "shared_uncounted")
                    .unwrap()
                    .try_as_basic_value()
                    .basic()
                    .unwrap();
                // SAFETY: the call and replaced load have the same MdhValue type and
                // belong to this live module. Erase only after redirecting every use.
                unsafe {
                    inkwell::llvm_sys::core::LLVMReplaceAllUsesWith(
                        inst.as_value_ref(),
                        value.as_value_ref(),
                    );
                }
            }
            inst.erase_from_basic_block();
        }
    }
}
