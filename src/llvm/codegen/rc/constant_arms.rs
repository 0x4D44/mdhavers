//! R5-6: remove only edges whose condition has one of the explicitly allowed forms.
use super::*;
use native_arms::NativeArmEdits;

fn constant_condition(condition: BasicValueEnum<'_>) -> Option<bool> {
    let BasicValueEnum::IntValue(value) = condition else {
        return None;
    };
    if value.get_type().get_bit_width() != 1 {
        return None;
    }
    if let Some(value) = value.get_zero_extended_constant() {
        return Some(value != 0);
    }
    let inst = value.as_instruction_value()?;
    let literal = |i| match CodeGen::rc_operand(inst, i) {
        Some(BasicValueEnum::IntValue(v)) => v.get_zero_extended_constant(),
        _ => None,
    };
    match inst.get_opcode() {
        InstructionOpcode::And if literal(0) == Some(0) || literal(1) == Some(0) => Some(false),
        InstructionOpcode::Or if literal(0) == Some(1) || literal(1) == Some(1) => Some(true),
        InstructionOpcode::ICmp => {
            let left = CodeGen::rc_operand(inst, 0)?;
            let right = CodeGen::rc_operand(inst, 1)?;
            // LLVM handles signedness and wide integer constants. Symbolic pointer
            // comparisons, undef and poison qualify only if LLVM returns a literal i1.
            unsafe {
                use inkwell::llvm_sys::core::{
                    LLVMBuildICmp, LLVMConstIntGetZExtValue, LLVMCreateBuilderInContext,
                    LLVMDisposeBuilder, LLVMGetICmpPredicate, LLVMGetTypeContext,
                    LLVMInstructionEraseFromParent, LLVMIsAConstantInt, LLVMIsAInstruction,
                    LLVMIsConstant, LLVMPositionBuilderBefore, LLVMTypeOf,
                };
                if LLVMIsConstant(left.as_value_ref()) == 0
                    || LLVMIsConstant(right.as_value_ref()) == 0
                {
                    return None;
                }
                let builder =
                    LLVMCreateBuilderInContext(LLVMGetTypeContext(LLVMTypeOf(inst.as_value_ref())));
                LLVMPositionBuilderBefore(builder, inst.as_value_ref());
                let folded = LLVMBuildICmp(
                    builder,
                    LLVMGetICmpPredicate(inst.as_value_ref()),
                    left.as_value_ref(),
                    right.as_value_ref(),
                    c"constant_compare".as_ptr(),
                );
                let result = (!LLVMIsAConstantInt(folded).is_null())
                    .then(|| LLVMConstIntGetZExtValue(folded) != 0);
                if !LLVMIsAInstruction(folded).is_null() {
                    LLVMInstructionEraseFromParent(folded);
                }
                LLVMDisposeBuilder(builder);
                result
            }
        }
        _ => None,
    }
}

pub(super) fn rc_fold_constant_arms<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> NativeArmEdits<'ctx> {
    use inkwell::llvm_sys::core::{LLVMGetSuccessor, LLVMReplaceAllUsesWith};
    use inkwell::values::PhiValue;
    let mut edits = NativeArmEdits {
        replacements: Vec::new(),
        blocks_to_delete: Vec::new(),
    };
    let builder = context.create_builder();
    for function in module.get_functions() {
        let blocks = function.get_basic_blocks();
        let Some(entry) = blocks.first().copied() else {
            continue;
        };
        let mut changed = false;
        for bb in &blocks {
            let Some(term) = bb.get_terminator() else {
                continue;
            };
            if term.get_opcode() != InstructionOpcode::Br || term.get_num_operands() != 3 {
                continue;
            }
            let Some(taken) = CodeGen::rc_operand(term, 0).and_then(constant_condition) else {
                continue;
            };
            // SAFETY: a conditional branch has two basic-block successors.
            let live = unsafe {
                BasicBlock::new(LLVMGetSuccessor(term.as_value_ref(), u32::from(!taken)))
                    .expect("conditional branch successor")
            };
            builder.position_before(&term);
            builder.build_unconditional_branch(live).unwrap();
            term.erase_from_basic_block();
            changed = true;
        }
        if !changed {
            continue;
        }
        let mut live = HashSet::new();
        let mut work = vec![entry];
        while let Some(bb) = work.pop() {
            if live.insert(bb) {
                work.extend(CodeGen::rc_successors(bb));
            }
        }
        let mut predecessors: HashMap<BasicBlock<'ctx>, HashMap<BasicBlock<'ctx>, usize>> =
            HashMap::new();
        for bb in blocks.iter().copied().filter(|bb| live.contains(bb)) {
            for next in CodeGen::rc_successors(bb) {
                *predecessors.entry(next).or_default().entry(bb).or_default() += 1;
            }
        }
        for bb in blocks.iter().copied().filter(|bb| live.contains(bb)) {
            let mut next = bb.get_first_instruction();
            while let Some(inst) = next {
                next = inst.get_next_instruction();
                if inst.get_opcode() != InstructionOpcode::Phi {
                    continue;
                }
                let phi = PhiValue::try_from(inst).expect("phi opcode");
                let incoming: Vec<_> = phi.get_incomings().collect();
                let mut quota = predecessors.get(&bb).cloned().unwrap_or_default();
                let kept: Vec<_> = incoming
                    .iter()
                    .copied()
                    .filter(|(_, from)| {
                        let count = quota.entry(*from).or_default();
                        if *count == 0 {
                            false
                        } else {
                            *count -= 1;
                            true
                        }
                    })
                    .collect();
                if kept.len() == incoming.len() {
                    continue;
                }
                assert!(!kept.is_empty(), "reachable merge has a live incoming edge");
                builder.position_before(&inst);
                let replacement = builder
                    .build_phi(phi.as_basic_value().get_type(), "constant_live_merge")
                    .unwrap();
                for (value, from) in kept {
                    replacement.add_incoming(&[(&value, from)]);
                }
                // SAFETY: both phis have exactly the same LLVM type. The caller
                // remaps ownership metadata before deleting the old instructions.
                unsafe { LLVMReplaceAllUsesWith(phi.as_value_ref(), replacement.as_value_ref()) };
                edits
                    .replacements
                    .push((phi.as_basic_value(), replacement.as_basic_value()));
            }
        }
        edits
            .blocks_to_delete
            .extend(blocks.into_iter().filter(|bb| !live.contains(bb)));
    }
    edits
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::memory_buffer::MemoryBuffer;

    #[test]
    fn only_the_listed_constant_conditions_are_folded() {
        let context = Context::create();
        let cases = [
            ("true", "", Some(true)),
            ("false", "", Some(false)),
            ("%c", "%c = and i1 %x, false", Some(false)),
            ("%c", "%c = and i1 false, %x", Some(false)),
            ("%c", "%c = or i1 %x, true", Some(true)),
            ("%c", "%c = or i1 true, %x", Some(true)),
            ("%c", "%c = icmp slt i64 -2, 1", Some(true)),
            ("%c", "%c = icmp eq i64 1, 2", Some(false)),
            ("%c", "%c = and i1 %x, %y", None),
            ("%c", "%c = icmp eq i64 %n, 2", None),
        ];
        for (condition, setup, expected) in cases {
            let ir = format!(
                "define i32 @subject(i1 %x, i1 %y, i64 %n) {{\nentry:\n{setup}\n br i1 {condition}, label %yes, label %no\nyes:\n br label %merge\nno:\n br label %merge\nmerge:\n %p = phi i32 [1, %yes], [2, %no]\n ret i32 %p\n}}"
            );
            let module = context
                .create_module_from_ir(MemoryBuffer::create_from_memory_range_copy(
                    ir.as_bytes(),
                    "constant-branch-case",
                ))
                .unwrap();
            module.verify().unwrap();
            let before = module.print_to_string().to_string();
            let mut cg = CodeGen::new(&context, "constant-branch-owner");
            cg.module = module;
            let edits = rc_fold_constant_arms(&context, &cg.module);
            if expected.is_some() {
                assert_eq!(edits.blocks_to_delete.len(), 1, "{setup} / {condition}");
            } else {
                assert!(edits.blocks_to_delete.is_empty(), "nonconstant: {setup}");
                assert!(edits.replacements.is_empty(), "nonconstant: {setup}");
                assert_eq!(cg.module.print_to_string().to_string(), before);
            }
            cg.rc_apply_arm_edits(edits);
            cg.module
                .verify()
                .expect("folded CFG and merge phis verify");
            if let Some(taken) = expected {
                let subject = cg.module.get_function("subject").unwrap();
                let term = subject
                    .get_first_basic_block()
                    .unwrap()
                    .get_terminator()
                    .unwrap();
                assert_eq!(term.get_num_operands(), 1);
                let successor = CodeGen::rc_successors(subject.get_first_basic_block().unwrap())[0];
                assert_eq!(
                    successor.get_name().to_bytes(),
                    if taken {
                        b"yes".as_slice()
                    } else {
                        b"no".as_slice()
                    }
                );
            }
        }
    }

    #[test]
    fn rc_pipeline_folds_before_the_slot_and_summary_queries() {
        let context = Context::create();
        let mut cg = CodeGen::new_with_refcounting(&context, "constant_pipeline", true);
        cg.module = context.create_module_from_ir(MemoryBuffer::create_from_memory_range_copy(
            b"define i32 @subject() { entry: br i1 true, label %yes, label %no\nyes: ret i32 1\nno: ret i32 2\n}",
            "constant-pipeline")).unwrap();
        cg.rc_elide_borrows();
        cg.module.verify().expect("pipeline constant fold verifies");
        let subject = cg.module.get_function("subject").unwrap();
        assert_eq!(
            subject
                .get_first_basic_block()
                .unwrap()
                .get_terminator()
                .unwrap()
                .get_num_operands(),
            1,
            "RC pipeline must fold the literal condition before C"
        );
        assert!(
            !subject
                .get_basic_blocks()
                .iter()
                .any(|b| b.get_name().to_bytes() == b"no"),
            "RC pipeline must remove the impossible successor"
        );
    }
}
