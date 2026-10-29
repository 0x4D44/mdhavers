//! Handwritten IR isolates C barriers that source programs often exercise together.
use super::*;
use inkwell::memory_buffer::MemoryBuffer;

fn fixture<'ctx>(context: &'ctx Context, extra: &str, body: &str) -> CodeGen<'ctx> {
    let mut cg = CodeGen::new(context, "slot_rule");
    let ir = format!(
        r#"
declare {{i8,i64}} @__mdh_list_clear({{i8,i64}})
{extra}
define {{i8,i64}} @subject({{i8,i64}} %input, ptr %items) {{
entry:
  %root = alloca {{i8,i64}}
  %park = alloca {{i8,i64}}
  %other = alloca {{i8,i64}}
  store {{i8,i64}} %input, ptr %root
  %origin = load {{i8,i64}}, ptr %root
  store {{i8,i64}} %origin, ptr %park
  {body}
  %borrow = load {{i8,i64}}, ptr %park
  ret {{i8,i64}} %borrow
}}
"#
    );
    cg.module = context
        .create_module_from_ir(MemoryBuffer::create_from_memory_range_copy(
            ir.as_bytes(),
            "slot_rule",
        ))
        .expect("parse C rule fixture");
    cg.module.verify().expect("C rule fixture verifies");
    let function = cg.module.get_function("subject").unwrap();
    let slots: [PointerValue<'ctx>; 3] =
        ["root", "park", "other"].map(|name| instruction(function, name).try_into().unwrap());
    cg.rc_fn_slots.push((function, slots.to_vec()));
    cg
}

fn instruction<'ctx>(function: FunctionValue<'ctx>, name: &str) -> InstructionValue<'ctx> {
    for bb in function.get_basic_blocks() {
        let mut next = bb.get_first_instruction();
        while let Some(inst) = next {
            next = inst.get_next_instruction();
            if inst
                .get_name()
                .is_some_and(|n| n.to_bytes() == name.as_bytes())
            {
                return inst;
            }
        }
    }
    panic!("missing fixture instruction {name}");
}

fn slot<'ctx>(cg: &CodeGen<'ctx>, name: &str) -> PointerValue<'ctx> {
    instruction(cg.module.get_function("subject").unwrap(), name)
        .try_into()
        .unwrap()
}

fn region_safe(cg: &CodeGen<'_>) -> bool {
    let function = cg.module.get_function("subject").unwrap();
    let park = slot(cg, "park");
    let (uses, escapes, blocks) = cg.rc_slot_uses(park).expect("ordinary slot uses");
    let holders = cg.rc_slot_owners();
    let effects = rc_c_effects(&cg.module, &holders, &cg.rc_owned_return_fns, &[]);
    cg.rc_slot_region_safe(
        instruction(function, "origin"),
        slot(cg, "root"),
        park,
        (&uses, &escapes, &blocks),
        &holders,
        &effects,
    )
}

#[test]
fn c_slot_temp_registered_as_local_cannot_supply_an_owner_root() {
    let context = Context::create();
    let mut cg = fixture(&context, "", "");
    let function = cg.module.get_function("subject").unwrap();
    let origin_store = instruction(function, "origin")
        .get_next_instruction()
        .unwrap();
    assert_eq!(origin_store.get_opcode(), InstructionOpcode::Store);
    let origin = CodeGen::rc_operand(origin_store, 0).unwrap();
    let root = slot(&cg, "root");
    assert_eq!(
        cg.rc_slot_roots(origin, &cg.rc_slot_owners()).unwrap(),
        vec![(root, instruction(function, "origin"))]
    );
    // Deliberately do not exclude the slot: this tests the independent temp filter.
    cg.rc_slots.temps.insert(value_id(root));
    assert!(!cg.rc_slots.excluded.contains(&value_id(root)));
    assert!(cg.rc_slot_roots(origin, &cg.rc_slot_owners()).is_none());
}

#[test]
fn c_slot_candidate_store_escapes_even_into_a_registered_owning_local() {
    let context = Context::create();
    let mut cg = fixture(
        &context,
        "",
        "%copied = load {i8,i64}, ptr %park\n store {i8,i64} %copied, ptr %other",
    );
    let park = slot(&cg, "park");
    let other = slot(&cg, "other");
    assert!(cg.rc_is_own_store_destination(other.into()));
    assert!(cg.rc_slot_uses(park).unwrap().1.is_empty());
    let copied = instruction(cg.module.get_function("subject").unwrap(), "copied");
    let store = copied.get_next_instruction().unwrap();
    assert_eq!(store.get_opcode(), InstructionOpcode::Store);
    cg.rc_slots.candidates.insert(value_id(other));
    assert_eq!(
        cg.rc_slot_uses(park).unwrap().1,
        HashSet::from([value_id(store)])
    );
}

#[test]
fn c_slot_root_overwrite_blocks_without_any_release_call() {
    let context = Context::create();
    let unrelated = fixture(&context, "", "store {i8,i64} %input, ptr %other");
    assert!(
        region_safe(&unrelated),
        "an unrelated local store preserves the root"
    );
    let overwritten = fixture(&context, "", "store {i8,i64} %input, ptr %root");
    assert!(
        !region_safe(&overwritten),
        "overwriting the root ends its lifetime"
    );
}

#[test]
fn c_slot_candidate_root_is_rejected_even_if_its_own_decision_fails() {
    let context = Context::create();
    let mut cg = fixture(&context, "", "");
    let function = cg.module.get_function("subject").unwrap();
    let root = slot(&cg, "root");
    let park = slot(&cg, "park");
    let origin = instruction(function, "origin");
    cg.rc_slots.stores.push(RcSlotStore {
        slot: park,
        value: CodeGen::rc_operand(origin.get_next_instruction().unwrap(), 0).unwrap(),
        store: origin.get_next_instruction().unwrap(),
        marker: None,
        moved_borrow: None,
    });
    // Root deliberately has no qualifying stores, so it fails its own decision.
    // The complete candidate set still excludes it as a root for park.
    cg.rc_slots
        .candidates
        .extend([value_id(root), value_id(park)]);
    let effects = rc_c_effects(&cg.module, &HashSet::new(), &HashSet::new(), &[]);
    let borrowed = cg.rc_decide_slots(false, &HashSet::new(), &effects);
    assert!(
        !borrowed.contains(&value_id(park)),
        "a failed C candidate never becomes an owning root for another candidate"
    );
}

#[test]
fn c_slot_same_function_setjmp_before_the_origin_is_ineligible() {
    let context = Context::create();
    let mut cg = fixture(&context, "declare i32 @setjmp(ptr)", "");
    let function = cg.module.get_function("subject").unwrap();
    let park = slot(&cg, "park");
    let origin = instruction(function, "origin");
    let b = context.create_builder();
    b.position_before(&origin);
    let setjmp = cg.module.get_function("setjmp").unwrap();
    b.build_call(
        setjmp,
        &[context
            .ptr_type(inkwell::AddressSpace::default())
            .const_null()
            .into()],
        "status",
    )
    .unwrap();
    cg.rc_slots.stores.push(RcSlotStore {
        slot: park,
        value: CodeGen::rc_operand(origin.get_next_instruction().unwrap(), 0).unwrap(),
        store: origin.get_next_instruction().unwrap(),
        marker: None,
        moved_borrow: None,
    });
    cg.rc_slots.candidates.insert(value_id(park));
    let effects = rc_c_effects(&cg.module, &HashSet::new(), &HashSet::new(), &[]);
    let borrowed = cg.rc_decide_slots(false, &HashSet::new(), &effects);
    assert!(
        !borrowed.contains(&value_id(park)),
        "same-function setjmp is forbidden even before the protected interval"
    );
}

#[test]
fn c_slot_raw_element_store_blocks_without_a_runtime_mutator() {
    let context = Context::create();
    let cg = fixture(
        &context,
        "",
        "%element = getelementptr {i8,i64}, ptr %items, i64 1\n store {i8,i64} %input, ptr %element",
    );
    assert!(
        !region_safe(&cg),
        "a raw MdhValue element store may destroy an alias"
    );
}

#[test]
fn c_slot_runtime_drop_blocks_without_a_root_store() {
    let context = Context::create();
    let cg = fixture(
        &context,
        "",
        "%cleared = call {i8,i64} @__mdh_list_clear({i8,i64} %origin)",
    );
    let call = instruction(cg.module.get_function("subject").unwrap(), "cleared");
    assert_eq!(rc_c_call_effect(&cg.module, call), RcCallEffect::Drops);
    assert!(
        !region_safe(&cg),
        "runtime mutation can destroy a rooted child"
    );
}

#[test]
fn c_slot_defined_destructive_callee_uses_the_heap_drop_summary() {
    let context = Context::create();
    let harmless = fixture(
        &context,
        "define void @helper({i8,i64} %value) { ret void }",
        "call void @helper({i8,i64} %origin)",
    );
    assert!(
        region_safe(&harmless),
        "a defined harmless helper is admissible"
    );
    let destructive = fixture(
        &context,
        "define void @helper({i8,i64} %value) {\n %cleared = call {i8,i64} @__mdh_list_clear({i8,i64} %value)\n ret void\n }",
        "call void @helper({i8,i64} %origin)",
    );
    let effects = rc_c_effects(
        &destructive.module,
        &destructive.rc_slot_owners(),
        &HashSet::new(),
        &[],
    );
    let helper = destructive.module.get_function("helper").unwrap();
    assert!(effects.c_drops_heap.contains(&helper));
    assert!(!effects.c_writes_globals.contains(&helper));
    assert!(
        !region_safe(&destructive),
        "the defined callee can destroy a rooted child"
    );
}
