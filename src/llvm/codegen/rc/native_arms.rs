//! Remove impossible generated native-tag arms before either effect query.
use super::*;
use inkwell::values::PhiValue;

/// Remap CodeGen ownership records, erase old phis, then delete these blocks.
/// No IR verification may run between this transform and that finalisation.
pub(super) struct NativeArmEdits<'ctx> {
    pub(super) replacements: Vec<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>)>,
    pub(super) blocks_to_delete: Vec<BasicBlock<'ctx>>,
}

fn successors(block: BasicBlock<'_>) -> Vec<BasicBlock<'_>> {
    block.get_terminator().map_or_else(Vec::new, |term| {
        (0..term.get_num_operands())
            .filter_map(|i| match term.get_operand(i) {
                Some(inkwell::values::Operand::Block(bb)) => Some(bb),
                _ => None,
            })
            .collect()
    })
}

/// Match a tagged-value test, not an unrelated integer equal to fourteen.
fn native_tag_test(inst: InstructionValue<'_>) -> bool {
    use inkwell::llvm_sys::core::{LLVMGetIndices, LLVMGetNumIndices};
    if inst.get_icmp_predicate() != Some(IntPredicate::EQ) {
        return false;
    }
    (0..2).any(|i| {
        let Some(BasicValueEnum::IntValue(constant)) = CodeGen::rc_operand(inst, i) else {
            return false;
        };
        if constant.get_zero_extended_constant() != Some(ValueTag::NativeObject.as_u8() as u64)
            || constant.get_type().get_bit_width() != 8
        {
            return false;
        }
        let Some(tag) = CodeGen::rc_operand(inst, 1 - i).and_then(|v| v.as_instruction_value())
        else {
            return false;
        };
        if tag.get_opcode() != InstructionOpcode::ExtractValue {
            return false;
        }
        // SAFETY: tag is a live extractvalue; its single index is the tag field.
        let tag_field = unsafe {
            LLVMGetNumIndices(tag.as_value_ref()) == 1 && *LLVMGetIndices(tag.as_value_ref()) == 0
        };
        let Some(BasicValueEnum::StructValue(value)) = CodeGen::rc_operand(tag, 0) else {
            return false;
        };
        let fields = value.get_type().get_field_types();
        tag_field
            && fields.len() == 2
            && fields[0].is_int_type()
            && fields[0].into_int_type().get_bit_width() == 8
            && fields[1].is_int_type()
            && fields[1].into_int_type().get_bit_width() == 64
    })
}

pub(super) fn rc_elide_native_arms<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> NativeArmEdits<'ctx> {
    use inkwell::llvm_sys::core::{LLVMGetSuccessor, LLVMReplaceAllUsesWith};
    let mut edits = NativeArmEdits {
        replacements: Vec::new(),
        blocks_to_delete: Vec::new(),
    };
    // Uses include address-taking and unreachable calls. Declarations alone do
    // not count. Run after all imports/functions/late dispatchers are emitted.
    if rc_natives_possible(module) {
        return edits;
    }
    let builder = context.create_builder();
    for function in module.get_functions() {
        let blocks = function.get_basic_blocks();
        let mut preds: HashMap<BasicBlock<'ctx>, Vec<BasicBlock<'ctx>>> = HashMap::new();
        for &bb in &blocks {
            for successor in successors(bb) {
                preds.entry(successor).or_default().push(bb);
            }
        }
        let mut removed = HashSet::new();
        let mut bypasses = Vec::new();
        for &bb in &blocks {
            let Some(term) = bb.get_terminator() else {
                continue;
            };
            if term.get_opcode() != InstructionOpcode::Br || term.get_num_operands() != 3 {
                continue;
            }
            let Some(test) = CodeGen::rc_operand(term, 0).and_then(|v| v.as_instruction_value())
            else {
                continue;
            };
            if !native_tag_test(test) {
                continue;
            }
            // SAFETY: a live conditional branch has true and false successors.
            let (yes, no) = unsafe {
                (
                    BasicBlock::new(LLVMGetSuccessor(term.as_value_ref(), 0)).unwrap(),
                    BasicBlock::new(LLVMGetSuccessor(term.as_value_ref(), 1)).unwrap(),
                )
            };
            if !preds
                .get(&yes)
                .is_some_and(|ps| ps.iter().all(|p| *p == bb))
            {
                continue;
            }
            let mut region = HashSet::from([yes]);
            loop {
                let mut added = false;
                for &candidate in &blocks {
                    if candidate == bb || candidate == no || region.contains(&candidate) {
                        continue;
                    }
                    if preds
                        .get(&candidate)
                        .is_some_and(|ps| !ps.is_empty() && ps.iter().all(|p| region.contains(p)))
                    {
                        added |= region.insert(candidate);
                    }
                }
                if !added {
                    break;
                }
            }
            // Restrict this transform to generated runtime dispatch arms.
            let has_native_call = region.iter().any(|block| {
                let mut next = block.get_first_instruction();
                while let Some(inst) = next {
                    next = inst.get_next_instruction();
                    if CodeGen::rc_callee(inst).is_some_and(|callee| {
                        callee.get_name().to_bytes().starts_with(b"__mdh_native_")
                    }) {
                        return true;
                    }
                }
                false
            });
            if has_native_call {
                removed.extend(region);
                bypasses.push((term, no));
            }
        }
        if removed.is_empty() {
            continue;
        }
        // Rebuild each merge once. IR uses update now; the caller remaps ownership
        // side tables before erasing old phi instructions and native arm blocks.
        for &bb in &blocks {
            if removed.contains(&bb) {
                continue;
            }
            let mut next = bb.get_first_instruction();
            while let Some(inst) = next {
                next = inst.get_next_instruction();
                if inst.get_opcode() != InstructionOpcode::Phi {
                    continue;
                }
                let phi = PhiValue::try_from(inst).expect("phi opcode");
                let incoming: Vec<_> = phi.get_incomings().collect();
                if !incoming.iter().any(|(_, from)| removed.contains(from)) {
                    continue;
                }
                let kept: Vec<_> = incoming
                    .into_iter()
                    .filter(|(_, from)| !removed.contains(from))
                    .collect();
                assert!(!kept.is_empty(), "native merge has a live incoming edge");
                builder.position_before(&inst);
                let replacement = builder
                    .build_phi(phi.as_basic_value().get_type(), "native_free_merge")
                    .unwrap();
                for (value, from) in kept {
                    replacement.add_incoming(&[(&value, from)]);
                }
                // SAFETY: both phis have exactly the same LLVM type.
                unsafe {
                    LLVMReplaceAllUsesWith(phi.as_value_ref(), replacement.as_value_ref());
                }
                edits
                    .replacements
                    .push((phi.as_basic_value(), replacement.as_basic_value()));
            }
        }
        for (term, no) in bypasses {
            builder.position_before(&term);
            builder.build_unconditional_branch(no).unwrap();
            term.erase_from_basic_block();
        }
        edits
            .blocks_to_delete
            .extend(blocks.into_iter().filter(|bb| removed.contains(bb)));
    }
    edits
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::memory_buffer::MemoryBuffer;

    fn parse<'ctx>(context: &'ctx Context, ir: &str) -> Module<'ctx> {
        let module = context
            .create_module_from_ir(MemoryBuffer::create_from_memory_range_copy(
                ir.as_bytes(),
                "rc7-c-effects",
            ))
            .expect("parse test IR");
        module.verify().expect("input IR verifies");
        module
    }

    #[test]
    fn terminal_exception_is_c_only_exact_and_transitive() {
        let context = Context::create();
        let module = parse(
            &context,
            r#"
declare void @__mdh_hurl_text_at_line(ptr)
declare void @__mdh_hurl({i8,i64})
declare void @__mdh_hurl_surprise()
declare void @callback()
define void @terminal() {
  call void @__mdh_hurl_text_at_line(ptr null)
  unreachable
}
define void @alias_error({i8,i64} %borrowed) {
  call void @__mdh_hurl({i8,i64} %borrowed)
  unreachable
}
define void @caller() {
  call void @terminal()
  ret void
}
define void @not_terminal({i8,i64} %borrowed) {
  call void @__mdh_hurl({i8,i64} %borrowed)
  ret void
}
define void @intervening_instruction({i8,i64} %borrowed) {
  call void @__mdh_hurl({i8,i64} %borrowed)
  %tag = extractvalue {i8,i64} %borrowed, 0
  unreachable
}
define void @unknown_raiser() {
  call void @__mdh_hurl_surprise()
  unreachable
}
define void @callback_then_error() {
  call void @callback()
  call void @__mdh_hurl_text_at_line(ptr null)
  unreachable
}
"#,
        );
        let effects = rc_c_effects(&module, &HashSet::new(), &HashSet::new(), &[]);
        let ordinary = rc_drops_heap(&module, &HashSet::new(), &HashSet::new());
        for name in ["terminal", "alias_error", "caller"] {
            let f = module.get_function(name).unwrap();
            assert!(
                ordinary.contains(&f),
                "ordinary A summary must still count {name}"
            );
            assert!(
                !effects.c_drops_heap.contains(&f),
                "C terminal error is clean: {name}"
            );
            assert!(!effects.c_writes_globals.contains(&f));
        }
        for name in [
            "not_terminal",
            "intervening_instruction",
            "unknown_raiser",
            "callback_then_error",
        ] {
            assert!(
                effects
                    .c_drops_heap
                    .contains(&module.get_function(name).unwrap()),
                "C must preserve other effects: {name}"
            );
        }
        assert!(effects
            .c_writes_globals
            .contains(&module.get_function("callback_then_error").unwrap()));
    }

    #[test]
    fn c_effects_keep_global_stores_candidate_releases_and_destructive_callees() {
        let context = Context::create();
        let module = parse(
            &context,
            r#"
@user = global {i8,i64} zeroinitializer
declare void @__mdh_hurl_text_at_line(ptr)
declare void @__mdh_rc_borrow_release({i8,i64})
define void @writer() {
  store {i8,i64} zeroinitializer, ptr @user
  call void @__mdh_hurl_text_at_line(ptr null)
  unreachable
}
define void @candidate() {
  %slot = alloca {i8,i64}
  store {i8,i64} zeroinitializer, ptr %slot
  %v = load {i8,i64}, ptr %slot
  call void @__mdh_rc_borrow_release({i8,i64} %v)
  ret void
}
define void @calls_candidate() {
  call void @candidate()
  ret void
}
define void @calls_writer() {
  call void @writer()
  ret void
}
"#,
        );
        let globals = [module.get_global("user").unwrap().as_pointer_value()];
        let effects = rc_c_effects(&module, &HashSet::new(), &HashSet::new(), &globals);
        let ordinary = rc_drops_heap(&module, &HashSet::new(), &HashSet::new());
        for name in ["candidate", "calls_candidate"] {
            let f = module.get_function(name).unwrap();
            assert!(effects.c_drops_heap.contains(&f));
            assert!(ordinary.contains(&f));
        }
        for name in ["writer", "calls_writer"] {
            assert!(effects
                .c_writes_globals
                .contains(&module.get_function(name).unwrap()));
        }
    }

    const NATIVE_DISPATCH: &str = r#"
declare {i8,i64} @__mdh_tri_module()
declare {i8,i64} @__mdh_log_span_begin({i8,i64},{i8,i64},{i8,i64},{i8,i64})
declare {i8,i64} @__mdh_log_span_current()
declare {i8,i64} @__mdh_native_call0({i8,i64},{i8,i64})
define {i8,i64} @dispatch({i8,i64} %value) {
entry:
  %tag = extractvalue {i8,i64} %value, 0
  %native = icmp eq i8 %tag, 14
  br i1 %native, label %native_arm, label %ordinary
native_arm:
  br label %native_call
native_call:
  %result = call {i8,i64} @__mdh_native_call0({i8,i64} %value, {i8,i64} zeroinitializer)
  br label %merge
ordinary:
  br label %merge
merge:
  %result_phi = phi {i8,i64} [%result, %native_call], [zeroinitializer, %ordinary]
  ret {i8,i64} %result_phi
}
"#;

    #[test]
    fn native_elision_removes_blocks_and_phi_inputs_and_verifies() {
        let context = Context::create();
        let module = parse(&context, NATIVE_DISPATCH);
        assert!(
            !rc_natives_possible(&module),
            "declarations alone are not native seeds"
        );
        let edits = rc_elide_native_arms(&context, &module);
        assert_eq!(edits.replacements.len(), 1);
        assert_eq!(edits.blocks_to_delete.len(), 2);
        for (old, _) in edits.replacements {
            old.as_instruction_value().unwrap().erase_from_basic_block();
        }
        for &bb in &edits.blocks_to_delete {
            bb.get_terminator().unwrap().erase_from_basic_block();
        }
        for bb in edits.blocks_to_delete {
            // SAFETY: no live predecessors or phi inputs name these dead blocks.
            unsafe {
                bb.delete().unwrap();
            }
        }
        module.verify().expect("elided native dispatch verifies");
        let text = module.print_to_string().to_string();
        assert!(!text.contains("call { i8, i64 } @__mdh_native_call0"));
        assert!(!text.contains("native_call:"));
        assert!(!rc_drops_heap(&module, &HashSet::new(), &HashSet::new())
            .contains(&module.get_function("dispatch").unwrap()));
    }

    #[test]
    fn every_native_seed_use_preserves_dispatch() {
        for seed in [
            "call {i8,i64} @__mdh_tri_module()",
            "call {i8,i64} @__mdh_log_span_current()",
            "call {i8,i64} @__mdh_log_span_begin({i8,i64} zeroinitializer,{i8,i64} zeroinitializer,{i8,i64} zeroinitializer,{i8,i64} zeroinitializer)",
        ] {
            let context = Context::create();
            let ir = format!("{NATIVE_DISPATCH}\ndefine void @seed() {{\n %v = {seed}\n ret void\n}}\n");
            let module = parse(&context, &ir);
            assert!(rc_natives_possible(&module));
            let before = module.print_to_string();
            let edits = rc_elide_native_arms(&context, &module);
            assert!(edits.replacements.is_empty());
            assert!(edits.blocks_to_delete.is_empty());
            assert_eq!(before, module.print_to_string());
            module.verify().expect("native-producing module stays valid");
        }
    }

    #[test]
    fn removing_one_edge_preserves_the_live_borrow_record() {
        let context = Context::create();
        let module = parse(
            &context,
            &format!("{NATIVE_DISPATCH}\ndeclare void @__mdh_rc_borrow_retain({{i8,i64}})\n"),
        );
        let mut cg = CodeGen::new_with_refcounting(&context, "live-edge-record", true);
        cg.module = module;
        let f = cg.module.get_function("dispatch").unwrap();
        let blocks = f.get_basic_blocks();
        let marker = cg.module.get_function(RC_BORROW_MARKER).unwrap();
        let b = context.create_builder();
        let native_bb = blocks
            .iter()
            .find(|bb| bb.get_name().to_bytes() == b"native_call")
            .unwrap();
        let ordinary_bb = blocks
            .iter()
            .find(|bb| bb.get_name().to_bytes() == b"ordinary")
            .unwrap();
        let merge = *blocks
            .iter()
            .find(|bb| bb.get_name().to_bytes() == b"merge")
            .unwrap();
        let phi = merge.get_first_instruction().unwrap();
        let value: BasicValueEnum<'_> = inkwell::values::PhiValue::try_from(phi)
            .unwrap()
            .as_basic_value();
        let mut markers = Vec::new();
        for bb in [native_bb, ordinary_bb] {
            b.position_before(&bb.get_terminator().unwrap());
            let call = b
                .build_call(marker, &[f.get_nth_param(0).unwrap().into()], "")
                .unwrap();
            markers.push(unsafe { InstructionValue::new(call.as_value_ref()) });
        }
        b.position_before(&blocks[0].get_first_instruction().unwrap());
        let slot = b.build_alloca(value.get_type(), "park").unwrap();
        b.position_before(&merge.get_terminator().unwrap());
        let park = b.build_store(slot, value).unwrap();
        cg.rc_borrows.push(RcBorrow {
            markers,
            value,
            park: Some((park, slot)),
            keep: false,
            owned_edges: false,
            shared_load: None,
            downgraded_by_c: false,
            materialized_for_c: false,
            holder_slot: None,
        });
        cg.rc_remove_native_arms();
        cg.module
            .verify()
            .expect("live-edge metadata fixture verifies");
        assert_eq!(
            cg.rc_borrows.len(),
            1,
            "removing a dead edge must preserve the live borrow record"
        );
        assert_eq!(
            cg.rc_borrows[0].markers.len(),
            1,
            "only the dead edge marker is discarded"
        );
    }
}
