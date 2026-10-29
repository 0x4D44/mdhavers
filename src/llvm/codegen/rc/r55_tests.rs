//! Synthetic R5-5 records isolate the authority flag from C eligibility. A real
//! C candidate spanning a destructive root call must already fail C's interval proof.
use super::*;
use inkwell::memory_buffer::MemoryBuffer;

fn record_is_safe(terminal: bool, downgraded_by_c: bool) -> bool {
    let context = Context::create();
    let mut cg = CodeGen::new(&context, "r55_record");
    let body = if terminal {
        "call void @__mdh_hurl({i8,i64} %element)\n unreachable"
    } else {
        "%cleared = call {i8,i64} @__mdh_list_clear({i8,i64} %root)\n ret {i8,i64} %element"
    };
    let ir = format!(
        r#"
declare void @__mdh_rc_borrow_retain({{i8,i64}})
declare void @__mdh_hurl({{i8,i64}})
declare {{i8,i64}} @__mdh_list_clear({{i8,i64}})
define {{i8,i64}} @subject({{i8,i64}} %element, {{i8,i64}} %root) {{
entry:
  %park = alloca {{i8,i64}}
  call void @__mdh_rc_borrow_retain({{i8,i64}} %element)
  store {{i8,i64}} %element, ptr %park
  {body}
}}
"#
    );
    cg.module = context
        .create_module_from_ir(MemoryBuffer::create_from_memory_range_copy(
            ir.as_bytes(),
            "r55_record",
        ))
        .expect("parse synthetic R5-5 record");
    cg.module.verify().expect("synthetic R5-5 IR verifies");
    let function = cg.module.get_function("subject").unwrap();
    let mut marker = None;
    let mut park = None;
    let mut next = function
        .get_first_basic_block()
        .unwrap()
        .get_first_instruction();
    while let Some(inst) = next {
        next = inst.get_next_instruction();
        match inst.get_opcode() {
            InstructionOpcode::Store => park = Some(inst),
            InstructionOpcode::Call
                if CodeGen::rc_callee(inst)
                    .is_some_and(|f| f.get_name().to_bytes() == b"__mdh_rc_borrow_retain") =>
            {
                marker = Some(inst)
            }
            _ => {}
        }
    }
    let park = park.expect("fixture park store");
    let Some(BasicValueEnum::PointerValue(slot)) = CodeGen::rc_operand(park, 1) else {
        panic!("park destination is a pointer");
    };
    let holders = HashSet::from([value_id(slot)]);
    let owned_returns = HashSet::new();
    let effects = rc_c_effects(&cg.module, &holders, &owned_returns, &[]);
    let ordinary_drops = rc_drops_heap(&cg.module, &holders, &owned_returns);
    let ordinary_writers = HashSet::new();
    let rec = RcBorrow {
        markers: vec![marker.expect("fixture retain marker")],
        value: function.get_nth_param(0).unwrap(),
        park: Some((park, slot)),
        keep: false,
        owned_edges: false,
        shared_load: None,
        downgraded_by_c,
        materialized_for_c: false,
        holder_slot: None,
    };
    cg.rc_borrow_is_safe(
        &rec,
        &holders,
        &ordinary_writers,
        &ordinary_drops,
        &mut HashMap::new(),
        Some(&effects),
    )
}

#[test]
fn r55_flagged_record_still_rejects_a_genuine_nonterminal_drop() {
    assert!(
        !record_is_safe(false, true),
        "C downgrade authority must not hide a genuine nonterminal root mutation"
    );
}

#[test]
fn r55_ordinary_holder_none_keeps_terminal_error_conservative() {
    assert!(
        !record_is_safe(true, false),
        "an ordinary holder-None record has no C rooted-chain authority"
    );
}

#[test]
fn r55_c_downgraded_record_alone_gets_terminal_error_exemption() {
    assert!(
        record_is_safe(true, true),
        "a C-downgraded record must use C effects for its audited terminal arm"
    );
}
