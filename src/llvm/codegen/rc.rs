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
//! an `__mdh_obj_alloc` (or `__mdh_obj_alloc_checked`) result, and phis of those.

use super::*;

mod slots;
pub(super) use slots::RcSlotState;
mod constant_arms;
mod native_arms;
#[cfg(test)]
mod r55_tests;
use native_arms::rc_elide_native_arms;

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

/// A retain of a borrowed value into a temp that may turn out to be unneeded (see
/// `rc_elide_borrows`): emitted as a call to the marker `__mdh_rc_borrow_retain`, which the
/// pass either deletes (with the park store, so the temp stays nil) or turns into
/// `__mdh_retain`.
pub(super) struct RcBorrow<'ctx> {
    /// The marker calls: one, or one per borrowed incoming edge of a phi
    /// (`rc_take_phi_by_edges`).
    markers: Vec<InstructionValue<'ctx>>,
    /// The value the retained reference covers: every use of it (or of anything pointing
    /// into its object) must come before anything that could drop the holder's reference.
    value: BasicValueEnum<'ctx>,
    /// The store that parked `value` in its temp slot, and the slot.
    park: Option<(InstructionValue<'ctx>, PointerValue<'ctx>)>,
    /// The temp's reference was moved to a holder (`rc_move_from_temp`): keep the retain.
    keep: bool,
    /// `value` is a phi that also took fresh +1s on other edges: on elision the temp parks
    /// a phi of just those (nil on the marked edges) instead of nothing.
    owned_edges: bool,
    /// A shared-slot read: the threaded branch's `__mdh_shared_load` call.
    shared_load: Option<InstructionValue<'ctx>>,
    /// C proved the named slot borrowed; only this authority grants C's terminal query.
    downgraded_by_c: bool,
    /// Eager materialisation at scope close, only needed if C borrows the slot.
    /// A note_write record still keeps the ordinary R5 safety decision.
    materialized_for_c: bool,
    /// The value was read from this variable slot (a global or a local), which holds a
    /// reference until it is written: only a write of it, or compiled code, can drop it.
    holder_slot: Option<PointerValue<'ctx>>,
}

/// Runtime symbol families that can call back into compiled code, or that only matter with
/// threads (see `rc_call_effect`).
const RC_CODE_FAMILIES: &[&str] = &[
    "log_", "span", "thread", "timer", "event", "chan_", "mutex", "condvar", "atomic", "native",
    "tri_", "midi", "muisic", "soond", "call",
];

/// Runtime symbol families that can drop references, or free or move objects.
const RC_DROP_FAMILIES: &[&str] = &[
    "release",
    "free",
    "realloc",
    "shared_store",
    "hurl",
    "socket",
    "tls",
    "srtp",
    "udp",
    "tcp",
    "clear",
    "heave",
    "toss",
];

/// External (non-runtime) functions the codegen declares that neither run compiled code
/// nor touch an object's count: libc readers, formatters into the caller's buffer, maths and
/// clocks. Every other external is `RunsCode`: `qsort` calls back into compiled code, and
/// `setjmp` returns a second time after the protected code ran, past calls the region scan
/// cannot see on the CFG. Keep sorted.
const RC_HARMLESS_EXTERNALS: &[&str] = &[
    "acos",
    "asin",
    "atan",
    "atan2",
    "ceil",
    "clock_gettime",
    "cos",
    "cosh",
    "exit",
    "exp",
    "fgets",
    "floor",
    "getenv",
    "isspace",
    "log",
    "log10",
    "log2",
    "memcpy",
    "nanosleep",
    "pow",
    "printf",
    "rand",
    "sin",
    "sinh",
    "sleep",
    "snprintf",
    "sqrt",
    "srand",
    "strcat",
    "strcmp",
    "strcpy",
    "strlen",
    "strncmp",
    "strstr",
    "strtod",
    "tan",
    "tanh",
    "time",
    "tolower",
    "toupper",
];

/// What a call can do to reference counts (see `rc_call_effect`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RcCallEffect {
    /// Nothing that lowers a count: a retain, an allocation, a reader.
    None,
    /// May drop references other code holds (a release, an overwrite or removal in a
    /// container, a free), but runs no compiled code.
    Drops,
    /// May run compiled code (which can do anything), or is unknown.
    RunsCode,
}

/// Per function, each block's predecessors (built once per function by `rc_elide_borrows`).
type RcPredMaps<'ctx> =
    HashMap<FunctionValue<'ctx>, HashMap<BasicBlock<'ctx>, Vec<BasicBlock<'ctx>>>>;

type LLVMValueRefAlias = inkwell::llvm_sys::prelude::LLVMValueRef;

/// The marker function a borrowed retain is emitted as until `rc_elide_borrows` decides.
const RC_BORROW_MARKER: &str = "__mdh_rc_borrow_retain";
const RC_BORROW_RELEASE_MARKER: &str = "__mdh_rc_borrow_release";

fn value_id(v: impl AsValueRef) -> usize {
    v.as_value_ref() as usize
}

fn rc_call_effect<'ctx>(module: &Module<'ctx>, inst: InstructionValue<'ctx>) -> RcCallEffect {
    use inkwell::llvm_sys::core::{LLVMGetCalledValue, LLVMIsAFunction, LLVMIsAInlineAsm};
    match inst.get_opcode() {
        InstructionOpcode::Call => {}
        InstructionOpcode::Invoke | InstructionOpcode::CallBr => return RcCallEffect::RunsCode,
        _ => return RcCallEffect::None,
    }
    // SAFETY: type queries on the called operand of a live call instruction.
    let callee = unsafe { LLVMGetCalledValue(inst.as_value_ref()) };
    if !unsafe { LLVMIsAInlineAsm(callee) }.is_null() {
        return RcCallEffect::None;
    }
    if unsafe { LLVMIsAFunction(callee) }.is_null() {
        return RcCallEffect::RunsCode;
    }
    // SAFETY: checked to be a function.
    let Some(f) = (unsafe { FunctionValue::new(callee) }) else {
        return RcCallEffect::RunsCode;
    };
    if f.count_basic_blocks() > 0 {
        return RcCallEffect::RunsCode;
    }
    let name = f.get_name().to_string_lossy();
    if name.starts_with("llvm.") {
        return RcCallEffect::None;
    }
    if !name.starts_with("__mdh_") {
        return match name.as_ref() {
            "free" | "realloc" => RcCallEffect::Drops,
            n if RC_HARMLESS_EXTERNALS.contains(&n) => RcCallEffect::None,
            _ => RcCallEffect::RunsCode,
        };
    }
    // `__mdh_list_push` drops no count, but it may move the list's items buffer: a
    // pointer into `items` loaded before it is stale after it (values read out are not).
    if matches!(
        name.as_ref(),
        RC_BORROW_MARKER
            | "__mdh_retain"
            | "__mdh_rc_retain_obj"
            | "__mdh_rc_check"
            | "__mdh_list_push"
    ) {
        return RcCallEffect::None;
    }
    // Without a native object (none of the functions that make one is used), a
    // native call only raises a type error.
    if name.starts_with("__mdh_native_") && !rc_natives_possible(module) {
        return RcCallEffect::Drops;
    }
    if RC_CODE_FAMILIES.iter().any(|fam| name.contains(fam)) {
        return RcCallEffect::RunsCode;
    }
    if RC_DROP_FAMILIES.iter().any(|fam| name.contains(fam)) {
        return RcCallEffect::Drops;
    }
    match ownership_of(&name) {
        Some((args, _)) if args.contains(['m', 's']) => RcCallEffect::Drops,
        Some(_) => RcCallEffect::None,
        None => RcCallEffect::RunsCode,
    }
}

/// The program can make a native object (a 3D object or a log span): only the
/// runtime functions named here return one that compiled code can get hold of.
fn rc_natives_possible(module: &Module<'_>) -> bool {
    [
        "__mdh_tri_module",
        "__mdh_log_span_begin",
        "__mdh_log_span_current",
    ]
    .iter()
    .any(|n| {
        module.get_function(n).is_some_and(|f| {
            f.as_global_value()
                .as_pointer_value()
                .get_first_use()
                .is_some()
        })
    })
}

/// The explicit +1 return contract shared with `rc_call_roots`. A body alone does
/// not establish ownership (an IR helper may simply return an argument).
fn rc_compiled_return_is_owned<'ctx>(
    callee: FunctionValue<'ctx>,
    owned_return_fns: &HashSet<FunctionValue<'ctx>>,
) -> bool {
    let name = callee.get_name().to_string_lossy();
    ["__mdh_fn_", "__lambda_", "__mdh_late_call_"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
        || owned_return_fns.contains(&callee)
}

/// Whether a release drops only a reference owned by this frame (RC7 I-own).
/// C candidates must be excluded from `owning_allocas` before computing this fact.
fn rc_release_is_own<'ctx>(
    value: BasicValueEnum<'ctx>,
    function: FunctionValue<'ctx>,
    owning_allocas: &HashSet<usize>,
    owned_return_fns: &HashSet<FunctionValue<'ctx>>,
) -> bool {
    let mut work = vec![value];
    let mut seen = HashSet::new();
    let mut rooted = false;
    while let Some(value) = work.pop() {
        if !seen.insert(value_id(value)) {
            continue;
        }
        if seen.len() > 4096 {
            return false;
        }
        let Some(inst) = value.as_instruction_value() else {
            return false;
        };
        match inst.get_opcode() {
            InstructionOpcode::ExtractValue => {
                let Some(aggregate) = CodeGen::rc_operand(inst, 0) else {
                    return false;
                };
                work.push(aggregate);
            }
            InstructionOpcode::Phi => {
                for i in 0..inst.get_num_operands() {
                    let Some(incoming) = CodeGen::rc_operand(inst, i) else {
                        return false;
                    };
                    work.push(incoming);
                }
            }
            InstructionOpcode::Load => {
                let Some(slot) = CodeGen::rc_operand(inst, 0) else {
                    return false;
                };
                let own = owning_allocas.contains(&value_id(slot))
                    && slot.as_instruction_value().is_some_and(|alloca| {
                        alloca.get_opcode() == InstructionOpcode::Alloca
                            && alloca.get_parent().and_then(|bb| bb.get_parent()) == Some(function)
                    });
                if !own {
                    return false;
                }
                rooted = true;
            }
            InstructionOpcode::Call => {
                let fresh = CodeGen::rc_callee(inst).is_some_and(|callee| {
                    let name = callee.get_name().to_string_lossy();
                    ownership_of(&name).map_or_else(
                        || rc_compiled_return_is_owned(callee, owned_return_fns),
                        |(_, ret)| ret == Ret::Fresh,
                    ) || matches!(
                        name.as_ref(),
                        "__mdh_obj_alloc"
                            | "__mdh_obj_alloc_checked"
                            | "__mdh_obj_strdup"
                            | "__mdh_obj_strdup_checked"
                    )
                });
                if !fresh {
                    return false;
                }
                rooted = true;
            }
            _ => return false,
        }
    }
    rooted
}

/// Functions that may drop a reference held outside their frame, including through
/// direct callees. This is independent of CodeGen state so IR unit tests exercise the
/// same summary used by borrowing. All facts are computed before any marker is erased.
pub(crate) fn rc_drops_heap<'ctx>(
    module: &Module<'ctx>,
    owning_allocas: &HashSet<usize>,
    owned_return_fns: &HashSet<FunctionValue<'ctx>>,
) -> HashSet<FunctionValue<'ctx>> {
    rc_drops_heap_query(module, owning_allocas, owned_return_fns, false)
}

fn rc_drops_heap_query<'ctx>(
    module: &Module<'ctx>,
    owning_allocas: &HashSet<usize>,
    owned_return_fns: &HashSet<FunctionValue<'ctx>>,
    c_terminal_errors: bool,
) -> HashSet<FunctionValue<'ctx>> {
    let mut drops = HashSet::new();
    let mut callers: HashMap<FunctionValue<'ctx>, Vec<FunctionValue<'ctx>>> = HashMap::new();
    for function in module.get_functions() {
        for bb in function.get_basic_blocks() {
            let mut next = bb.get_first_instruction();
            while let Some(inst) = next {
                next = inst.get_next_instruction();
                if c_terminal_errors && rc_c_terminal_error_arm(inst) {
                    continue;
                }
                let effect = rc_call_effect(module, inst);
                if effect == RcCallEffect::None {
                    continue;
                }
                if let Some(callee) = CodeGen::rc_callee(inst) {
                    if callee.count_basic_blocks() > 0 {
                        callers.entry(callee).or_default().push(function);
                        continue;
                    }
                    if matches!(
                        callee.get_name().to_bytes(),
                        b"__mdh_release" | b"__mdh_rc_release_obj" | b"__mdh_rc_borrow_release"
                    ) {
                        if !CodeGen::rc_operand(inst, 0).is_some_and(|v| {
                            rc_release_is_own(v, function, owning_allocas, owned_return_fns)
                        }) {
                            drops.insert(function);
                        }
                        continue;
                    }
                }
                // Non-release Drops (m/s rows, free/realloc, shared_store), and
                // RunsCode without a visible compiled body, may drop heap holders.
                drops.insert(function);
            }
        }
    }
    // The same caller worklist fixpoint as rc_global_writers, including recursion
    // and late-call dispatchers. An ordering-dependent single pass is insufficient.
    let mut work: Vec<_> = drops.iter().copied().collect();
    while let Some(callee) = work.pop() {
        if let Some(cs) = callers.get(&callee) {
            for caller in cs {
                if drops.insert(*caller) {
                    work.push(*caller);
                }
            }
        }
    }
    drops
}

/// Audited terminal errors release only the thread's previous-error reference,
/// after storing the replacement, then longjmp or exit. This exception is exclusively
/// for C's rooted-chain proof; ordinary A/R5 summaries retain their original meaning.
/// Audit: runtime/mdh_runtime.c:504-520 (text), 550-563 (overflow),
/// 1076-1150 (type), 12199-12201 (raw hurl), 12250-12264 (line hurl),
/// and 11827-11846 (replacement before release); runtime/gc_stub.c:568-595
/// tears down children without running compiled code.
fn rc_c_terminal_error_arm(inst: InstructionValue<'_>) -> bool {
    let Some(callee) = CodeGen::rc_callee(inst) else {
        return false;
    };
    callee.count_basic_blocks() == 0
        && matches!(
            callee.get_name().to_bytes(),
            b"__mdh_hurl_text_at_line"
                | b"__mdh_hurl"
                | b"__mdh_hurl_at"
                | b"__mdh_type_error"
                | b"__mdh_int_overflow_error"
        )
        && inst
            .get_next_instruction()
            .is_some_and(|next| next.get_opcode() == InstructionOpcode::Unreachable)
}

/// C's instruction query; ordinary R5 callers must continue using `rc_call_effect`.
fn rc_c_call_effect<'ctx>(module: &Module<'ctx>, inst: InstructionValue<'ctx>) -> RcCallEffect {
    if rc_c_terminal_error_arm(inst) {
        RcCallEffect::None
    } else {
        rc_call_effect(module, inst)
    }
}

pub(super) struct RcCEffects<'ctx> {
    pub(super) c_drops_heap: HashSet<FunctionValue<'ctx>>,
    pub(super) c_writes_globals: HashSet<FunctionValue<'ctx>>,
}

/// Revision 5 effect query used only when deciding C slots. The ordinary summaries
/// deliberately still count terminal errors. All remaining local effects propagate
/// through the same direct-caller fixpoint, including recursion and late dispatchers.
pub(super) fn rc_c_effects<'ctx>(
    module: &Module<'ctx>,
    owning_allocas: &HashSet<usize>,
    owned_return_fns: &HashSet<FunctionValue<'ctx>>,
    user_globals: &[PointerValue<'ctx>],
) -> RcCEffects<'ctx> {
    let globals: HashSet<usize> = user_globals.iter().map(|p| value_id(*p)).collect();
    let is_global = |v: Option<BasicValueEnum<'ctx>>| matches!(v, Some(BasicValueEnum::PointerValue(p)) if globals.contains(&value_id(p)));
    let mut writers = HashSet::new();
    let mut callers: HashMap<FunctionValue<'ctx>, Vec<FunctionValue<'ctx>>> = HashMap::new();
    for function in module.get_functions() {
        for bb in function.get_basic_blocks() {
            let mut next = bb.get_first_instruction();
            while let Some(inst) = next {
                next = inst.get_next_instruction();
                if rc_c_terminal_error_arm(inst) {
                    continue;
                }
                match inst.get_opcode() {
                    InstructionOpcode::Store if is_global(CodeGen::rc_operand(inst, 1)) => {
                        writers.insert(function);
                    }
                    InstructionOpcode::Call
                    | InstructionOpcode::Invoke
                    | InstructionOpcode::CallBr => {
                        if let Some(callee) =
                            CodeGen::rc_callee(inst).filter(|f| f.count_basic_blocks() > 0)
                        {
                            callers.entry(callee).or_default().push(function);
                            continue;
                        }
                        let effect = rc_call_effect(module, inst);
                        let args = inst.get_num_operands().saturating_sub(1);
                        if effect == RcCallEffect::RunsCode
                            || effect == RcCallEffect::Drops
                                && (0..args).any(|i| is_global(CodeGen::rc_operand(inst, i)))
                        {
                            writers.insert(function);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let mut work: Vec<_> = writers.iter().copied().collect();
    while let Some(callee) = work.pop() {
        if let Some(cs) = callers.get(&callee) {
            for caller in cs {
                if writers.insert(*caller) {
                    work.push(*caller);
                }
            }
        }
    }
    RcCEffects {
        c_drops_heap: rc_drops_heap_query(module, owning_allocas, owned_return_fns, true),
        c_writes_globals: writers,
    }
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
                                b"__mdh_obj_alloc"
                                    | b"__mdh_obj_alloc_checked"
                                    | b"__mdh_obj_strdup"
                                    | b"__mdh_obj_strdup_checked"
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
        if rc_compiled_return_is_owned(callee, &self.rc_owned_return_fns) {
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
        let retain = match self.rc_edge_markers {
            Some(_) => self.rc_borrow_marker_fn(),
            None => self.libc.rc_retain,
        };
        for (iv, bb, class, roots) in plan {
            match class {
                RcClass::Skip => {}
                RcClass::Fresh => {
                    self.rc_taken.extend(roots);
                    self.rc_edge_fresh = true;
                }
                // A phi of fresh and borrowed values (a property read that is a field or a
                // new bound method): normalise it in turn, so its fresh +1s are taken.
                RcClass::Borrowed if self.rc_phi_has_fresh(iv) && self.rc_take_phi_by_edges(iv) => {
                }
                RcClass::Borrowed => {
                    let term = bb.get_terminator().expect("terminator");
                    let b = self.context.create_builder();
                    b.position_before(&term);
                    let call = b.build_call(retain, &[iv.into()], "").unwrap();
                    if let Some(markers) = self.rc_edge_markers.as_mut() {
                        // SAFETY: a call instruction just built.
                        markers.push(unsafe { InstructionValue::new(call.as_value_ref()) });
                    }
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
        let is_heap = self.rc_build_heap_test(&self.builder, tag, "rc_", true);
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
        threadable: bool,
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
        if !threadable {
            return b
                .build_int_truncate(bits, self.types.bool_type, &format!("{prefix}is_heap"))
                .unwrap();
        }
        let in_mask = b
            .build_int_truncate(bits, self.types.bool_type, &format!("{prefix}in_mask"))
            .unwrap();
        // Tags 0-3 (nil, bool, int, float) are never counted. Testing that first as a
        // compare with a constant lets jump threading see the answer on edges where the
        // tag is a known immediate (a phi of nil and a counted value, an arithmetic
        // result), which it cannot through a shift of a constant.
        let above = b
            .build_int_compare(
                IntPredicate::UGT,
                shift,
                i64t.const_int(3, false),
                &format!("{prefix}not_imm"),
            )
            .unwrap();
        b.build_and(above, in_mask, &format!("{prefix}is_heap"))
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
        for b in self.rc_borrows.iter_mut() {
            if b.park.is_some_and(|(_, s)| s == slot) {
                b.keep = true;
            }
        }
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

    /// `store_slot` into a container element or field: when a temp of the statement holds
    /// `v`, the element takes that reference (the temp becomes nil) instead of a retain
    /// now and the temp's release at the statement's end. The expression's own result is
    /// parked again afterwards with a borrow marker, so a later use of `v` in an enclosing
    /// expression keeps a reference whenever user code could overwrite the element first
    /// (`rc_elide_borrows`). Plain calls, no new blocks.
    pub(super) fn store_slot_move(&mut self, ptr: PointerValue<'ctx>, v: BasicValueEnum<'ctx>) {
        if !self.rc_enabled || !self.rc_is_value(v) || !self.rc_active() {
            self.builder.build_store(ptr, v).unwrap();
            return;
        }
        self.rc_note_write(ptr);
        if self.rc_is_shared_slot(ptr) {
            self.store_shared_slot(ptr, v, false);
            return;
        }
        let old = self
            .builder
            .build_load(self.types.value_type, ptr, "rc_old")
            .unwrap();
        if !self.rc_move_from_temp(v, false) {
            self.rc_take(v);
        }
        self.builder.build_store(ptr, v).unwrap();
        self.rc_release(old);
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
        let local = ptr
            .as_instruction()
            .is_some_and(|i| i.get_opcode() == InstructionOpcode::Alloca)
            && !self.rc_slots.excluded.contains(&value_id(ptr));
        if local {
            let before = self.rc_borrows.len();
            let moved = inline && self.rc_move_from_temp(v, false);
            let marker = if moved {
                None
            } else if self.rc_class(v) == RcClass::Borrowed {
                Some(self.rc_build_borrow_marker(v))
            } else {
                self.rc_take_mode(v, inline);
                None
            };
            let moved_borrow = moved
                .then(|| {
                    self.rc_borrows[..before]
                        .iter()
                        .rposition(|r| r.value == v && r.keep)
                })
                .flatten();
            let store = self.builder.build_store(ptr, v).unwrap();
            self.rc_slots.stores.push(slots::RcSlotStore {
                slot: ptr,
                value: v,
                store,
                marker,
                moved_borrow,
            });
            self.rc_slot_release(ptr, old, inline);
        } else {
            self.rc_take_or_move(v, inline);
            self.builder.build_store(ptr, v).unwrap();
            if inline {
                self.rc_release_inline(old);
            } else {
                self.rc_release(old);
            }
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
        let marker = self.rc_build_borrow_marker(plain);
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
        self.rc_borrows.push(RcBorrow {
            markers: vec![marker],
            value: v,
            park: None,
            keep: false,
            owned_edges: false,
            shared_load: locked.as_instruction_value(),
            downgraded_by_c: false,
            materialized_for_c: false,
            holder_slot: self.rc_is_shared_slot(ptr).then_some(ptr),
        });
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
        self.rc_slots.params.insert(value_id(ptr));
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
        self.rc_materialize_reads(Some(ptr));
    }

    // Materialise while the scope still owns its initialization and early-exit sites.
    // The C decision later downgrades successful borrowed-slot records to holder None.
    fn rc_materialize_reads(&mut self, only: Option<PointerValue<'ctx>>) {
        let function = match self.current_function {
            Some(f) => f,
            None => return,
        };
        let first = if only.is_none() {
            // Outer scopes can still be written later: keep their reads deferred
            // until their own close, so note_write retains its original authority.
            self.rc_state.scopes.len().saturating_sub(1)
        } else {
            0
        };
        for si in first..self.rc_state.scopes.len() {
            let reads = &mut self.rc_state.scopes[si].reads;
            if !reads.iter().any(|r| only.is_none_or(|ptr| r.slot == ptr)) {
                continue;
            }
            let (hit, kept): (Vec<RcRead<'ctx>>, Vec<RcRead<'ctx>>) = std::mem::take(reads)
                .into_iter()
                .partition(|r| only.is_none_or(|ptr| r.slot == ptr));
            self.rc_state.scopes[si].reads = kept;
            for r in hit {
                // Right after the read: retain and park (load the temp's old value,
                // store, release the old one), as `rc_own_temp` would have done there.
                let temp = self.create_entry_block_alloca_raw("rc_tmp");
                self.rc_exclude_temp(temp);
                self.rc_temp_pins.entry(function).or_default().push(temp);
                self.rc_state.scopes[si].slots.push(temp);
                let b = self.context.create_builder();
                match r.load.get_next_instruction() {
                    Some(next) => b.position_before(&next),
                    None => b.position_at_end(r.load.get_parent().unwrap()),
                }
                // A borrow marker: the read is often used up before the store (`x = x + 1`),
                // and then `rc_elide_borrows` drops the retain again.
                let marker_fn = self.rc_borrow_marker_fn();
                let marker = b.build_call(marker_fn, &[r.value.into()], "").unwrap();
                let old = b
                    .build_load(self.types.value_type, temp, "rc_tmp_old")
                    .unwrap();
                let park = b.build_store(temp, r.value).unwrap();
                b.build_call(self.libc.rc_release, &[old.into()], "")
                    .unwrap();
                self.rc_borrows.push(RcBorrow {
                    // SAFETY: a call instruction just built.
                    markers: vec![unsafe { InstructionValue::new(marker.as_value_ref()) }],
                    value: r.value,
                    park: Some((park, temp)),
                    keep: false,
                    owned_edges: false,
                    shared_load: None,
                    downgraded_by_c: false,
                    materialized_for_c: only.is_none(),
                    holder_slot: Some(r.slot),
                });
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
        let mut roots = Vec::new();
        // Retains `rc_elide_borrows` may find unneeded, and whether the temp also takes a
        // fresh +1 (then they stay: the temp's release is needed for it).
        let mut markers = None;
        match self.rc_roots(v, 0, &mut roots) {
            RcClass::Skip => return,
            RcClass::Fresh => self.rc_taken.extend(roots),
            RcClass::Borrowed => {
                self.rc_edge_markers = Some(Vec::new());
                self.rc_edge_fresh = false;
                let by_edges = self.rc_take_phi_by_edges(v);
                let edge_markers = self.rc_edge_markers.take().unwrap_or_default();
                markers = Some(if by_edges {
                    (edge_markers, self.rc_edge_fresh)
                } else {
                    (vec![self.rc_build_borrow_marker(v)], false)
                });
            }
        }
        let slot = self.rc_temp_slot();
        // A slot parked again before its scope ends (a loop condition) lets go of the
        // previous value.
        let old = self
            .builder
            .build_load(self.types.value_type, slot, "rc_tmp_old")
            .unwrap();
        let park = self.builder.build_store(slot, v).unwrap();
        match markers {
            Some((markers, owned_edges)) => self.rc_borrows.push(RcBorrow {
                markers,
                value: v,
                park: Some((park, slot)),
                keep: false,
                owned_edges,
                shared_load: None,
                downgraded_by_c: false,
                materialized_for_c: false,
                holder_slot: None,
            }),
            // A shared-slot read (`rc_shared_load`) parked here.
            None => {
                if let Some(b) = self
                    .rc_borrows
                    .iter_mut()
                    .rev()
                    .find(|b| b.park.is_none() && !b.keep && b.value == v)
                {
                    b.park = Some((park, slot));
                }
            }
        }
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
            self.rc_exclude_temp(slot);
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
        self.rc_exclude_temp(slot);
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
        if self.rc_enabled && self.current_function.is_some() {
            self.rc_materialize_reads(None);
        }
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
            self.rc_slot_release(*slot, v, true);
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
        // Declared with counting on in every build, so the module's declarations do not
        // depend on the compiler's profile (see `rc_debug_check_dead`).
        self.rc_check_fn();
    }

    /// `__mdh_rc_check(MdhValue, const char *op)` (runtime/gc_stub.c): aborts on a DEAD object.
    fn rc_check_fn(&mut self) -> FunctionValue<'ctx> {
        match self.module.get_function("__mdh_rc_check") {
            Some(f) => f,
            None => {
                let ptr = self.context.ptr_type(AddressSpace::default());
                let ty = self
                    .types
                    .void_type
                    .fn_type(&[self.types.value_type.into(), ptr.into()], false);
                let f = self
                    .module
                    .add_function("__mdh_rc_check", ty, Some(Linkage::External));
                let nounwind = self
                    .context
                    .create_enum_attribute(Attribute::get_named_enum_kind_id("nounwind"), 0);
                f.add_attribute(AttributeLoc::Function, nounwind);
                f
            }
        }
    }

    /// Debug builds of the compiler (which embed the debug runtime), with counting on:
    /// abort if `v` is a DEAD object when an operation reads it (`op` names it). With
    /// MDH_RC_NOFREE=1 an object that died early is kept and marked DEAD, but a read whose
    /// retain borrowing elided leaves nothing else to touch it, so the early free would go
    /// unseen. Release builds emit nothing.
    pub(super) fn rc_debug_check_dead(&mut self, v: BasicValueEnum<'ctx>, op: &str) {
        if !cfg!(debug_assertions) || !self.rc_active() || !self.rc_is_value(v) {
            return;
        }
        let f = self.rc_check_fn();
        let name = format!("__mdh_rc_check_op_{op}");
        let op_ptr = match self.module.get_global(&name) {
            Some(g) => g.as_pointer_value(),
            None => self
                .builder
                .build_global_string_ptr(op, &name)
                .unwrap()
                .as_pointer_value(),
        };
        self.builder
            .build_call(f, &[v.into(), op_ptr.into()], "")
            .unwrap();
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

    /// A call to the borrow marker (`RcBorrow`) for `v`, which the end of compilation turns
    /// into `__mdh_retain(v)` or deletes. A plain call: no new blocks.
    fn rc_build_borrow_marker(&mut self, v: BasicValueEnum<'ctx>) -> InstructionValue<'ctx> {
        let f = self.rc_borrow_marker_fn();
        let call = self.builder.build_call(f, &[v.into()], "").unwrap();
        // SAFETY: a call instruction just built.
        unsafe { InstructionValue::new(call.as_value_ref()) }
    }

    fn rc_borrow_marker_fn(&mut self) -> FunctionValue<'ctx> {
        match self.module.get_function(RC_BORROW_MARKER) {
            Some(f) => f,
            None => {
                let ty = self.libc.rc_retain.get_type();
                let f = self
                    .module
                    .add_function(RC_BORROW_MARKER, ty, Some(Linkage::External));
                let nounwind = self
                    .context
                    .create_enum_attribute(Attribute::get_named_enum_kind_id("nounwind"), 0);
                f.add_attribute(AttributeLoc::Function, nounwind);
                f
            }
        }
    }

    /// Borrowing (R5, `wrk_docs/2026.10.26 - DES - RC borrowing.md`): every function body
    /// is complete. A borrowed value parked in a temp needs its retain only if something
    /// that could drop its holder's reference runs before its last use. In a program that
    /// can never start a thread, a reference count only falls through a call, so when no
    /// such call (`rc_inst_is_safe`) lies between the retain and any use of the value or
    /// of anything pointing into its object, and the value never escapes to memory, delete
    /// the retain and the park: the temp stays nil and its releases fold away. Every other
    /// marker becomes the `__mdh_retain` it stood for.
    pub(super) fn rc_elide_borrows(&mut self) {
        use inkwell::llvm_sys::core::{
            LLVMGetFirstUse, LLVMGetUser, LLVMInstructionEraseFromParent, LLVMReplaceAllUsesWith,
        };
        if !self.rc_enabled {
            return;
        }
        let threaded = self
            .module
            .get_function("__mdh_thread_spawn")
            .is_some_and(|f| {
                f.as_global_value()
                    .as_pointer_value()
                    .get_first_use()
                    .is_some()
            });
        if !threaded {
            self.rc_remove_native_arms();
        }
        let edits = constant_arms::rc_fold_constant_arms(self.context, &self.module);
        self.rc_apply_arm_edits(edits);
        self.rc_collect_slot_candidates();
        let mut preds: RcPredMaps<'ctx> = HashMap::new();
        // Slots that hold a reference of their own to what is stored in them.
        let holders: HashSet<usize> = self
            .rc_fn_slots
            .iter()
            .flat_map(|(_, slots)| slots.iter())
            .chain(self.rc_temp_pins.values().flatten())
            .map(|s| value_id(*s))
            .filter(|id| !self.rc_slots.candidates.contains(id))
            .collect();
        let global_writers = if threaded {
            HashSet::new()
        } else {
            self.rc_global_writers()
        };
        let drops_heap = if threaded {
            HashSet::new()
        } else {
            rc_drops_heap(&self.module, &holders, &self.rc_owned_return_fns)
        };
        let c_effects = rc_c_effects(
            &self.module,
            &holders,
            &self.rc_owned_return_fns,
            &self.rc_user_globals(),
        );
        let borrowed = self.rc_decide_slots(threaded, &holders, &c_effects);
        let moved = self.rc_finish_slots(&borrowed);
        let borrows = std::mem::take(&mut self.rc_borrows);
        let marker_fn = self.module.get_function(RC_BORROW_MARKER);
        let b = self.context.create_builder();
        for (index, mut rec) in borrows.into_iter().enumerate() {
            if rec.materialized_for_c
                && rec
                    .holder_slot
                    .is_some_and(|slot| !borrowed.contains(&value_id(slot)))
            {
                // The owning slot never needed an extra read reference. Preserve
                // pre-C R5 behaviour, including catches that abandon temp cleanup.
                // These reads saw no write while their scope remained open.
                if let Some((park, _)) = rec.park {
                    park.erase_from_basic_block();
                }
                for marker in &rec.markers {
                    marker.erase_from_basic_block();
                }
                continue;
            }
            if moved.contains(&index) {
                for m in &rec.markers {
                    m.erase_from_basic_block();
                }
                continue;
            }
            if rec
                .holder_slot
                .is_some_and(|slot| borrowed.contains(&value_id(slot)))
            {
                rec.downgraded_by_c = true;
                rec.holder_slot = None;
            }
            let elide = !threaded
                && !rec.keep
                && rec.park.is_some()
                && self.rc_borrow_is_safe(
                    &rec,
                    &holders,
                    &global_writers,
                    &drops_heap,
                    &mut preds,
                    Some(&c_effects),
                );
            if elide && rec.owned_edges && !self.rc_park_fresh_edges(&rec) {
                for m in &rec.markers {
                    self.rc_marker_to_retain(*m, &b);
                }
            } else if elide {
                let (park, _) = rec.park.expect("park");
                if !rec.owned_edges {
                    park.erase_from_basic_block();
                }
                for m in &rec.markers {
                    m.erase_from_basic_block();
                }
                if let Some(call) = rec.shared_load {
                    // Never threaded: the locked read is dead code; make it a plain read
                    // without a reference, like the branch that is taken.
                    if let Some(BasicValueEnum::PointerValue(slot)) = Self::rc_operand(call, 0) {
                        b.position_before(&call);
                        let plain = b
                            .build_load(self.types.value_type, slot, "shared_borrow")
                            .unwrap();
                        // SAFETY: the call's result is replaced by a value of its type,
                        // then the call is erased.
                        unsafe {
                            LLVMReplaceAllUsesWith(call.as_value_ref(), plain.as_value_ref());
                            LLVMInstructionEraseFromParent(call.as_value_ref());
                        }
                    }
                }
            } else {
                for m in &rec.markers {
                    self.rc_marker_to_retain(*m, &b);
                }
            }
        }
        self.rc_lower_inline_slot_releases();
        // Defensive: any marker call not recorded keeps its retain.
        let Some(marker_fn) = marker_fn else {
            return;
        };
        loop {
            // SAFETY: walking the use list of a live function.
            let user = unsafe {
                let u = LLVMGetFirstUse(marker_fn.as_value_ref());
                if u.is_null() {
                    break;
                }
                LLVMGetUser(u)
            };
            // SAFETY: the user of the marker is a call instruction of this module.
            let inst = unsafe { InstructionValue::new(user) };
            self.rc_marker_to_retain(inst, &b);
        }
        // SAFETY: no uses remain.
        unsafe { marker_fn.delete() };
    }

    /// `rec.value` is a phi whose temp also holds fresh +1s from some edges: park a phi of
    /// those instead, nil on each edge a marker retained. False when the markers are not
    /// all on direct incoming edges of the phi (nothing changed).
    fn rc_park_fresh_edges(&self, rec: &RcBorrow<'ctx>) -> bool {
        let Some((park, _)) = rec.park else {
            return false;
        };
        let Some(inst) = rec.value.as_instruction_value() else {
            return false;
        };
        if inst.get_opcode() != InstructionOpcode::Phi {
            return false;
        }
        let phi = inkwell::values::PhiValue::try_from(inst).expect("phi");
        let incoming: Vec<(BasicValueEnum<'ctx>, BasicBlock<'ctx>)> = (0..phi.count_incoming())
            .map(|i| phi.get_incoming(i).expect("incoming"))
            .collect();
        let mut marked: HashSet<BasicBlock<'ctx>> = HashSet::new();
        for m in &rec.markers {
            let (Some(bb), Some(arg)) = (m.get_parent(), Self::rc_operand(*m, 0)) else {
                return false;
            };
            if !incoming.iter().any(|(iv, ib)| *ib == bb && *iv == arg) {
                return false;
            }
            marked.insert(bb);
        }
        let b = self.context.create_builder();
        b.position_before(&inst);
        let owned = b.build_phi(self.types.value_type, "rc_owned_part").unwrap();
        let nil = self.make_nil();
        for (iv, bb) in &incoming {
            let v = if marked.contains(bb) { nil } else { *iv };
            owned.add_incoming(&[(&v, *bb)]);
        }
        // SAFETY: the park store's value operand is replaced by a value of the same type.
        unsafe {
            inkwell::llvm_sys::core::LLVMSetOperand(
                park.as_value_ref(),
                0,
                owned.as_basic_value().as_value_ref(),
            )
        };
        true
    }

    fn rc_marker_to_retain(&self, marker: InstructionValue<'ctx>, b: &Builder<'ctx>) {
        if let Some(v) = Self::rc_operand(marker, 0) {
            b.position_before(&marker);
            b.build_call(self.libc.rc_retain, &[v.into()], "").unwrap();
        }
        marker.erase_from_basic_block();
    }

    /// No call that could drop a reference or run compiled code lies on a path from
    /// `rec`'s retain to a use of its value or of anything derived from it, and nothing
    /// derived from it is stored to memory (other than by the park) or returned.
    fn rc_borrow_is_safe(
        &self,
        rec: &RcBorrow<'ctx>,
        holders: &HashSet<usize>,
        global_writers: &HashSet<FunctionValue<'ctx>>,
        drops_heap: &HashSet<FunctionValue<'ctx>>,
        preds: &mut RcPredMaps<'ctx>,
        c_slot_effects: Option<&RcCEffects<'ctx>>,
    ) -> bool {
        use inkwell::llvm_sys::core::{
            LLVMGetFirstUse, LLVMGetNextUse, LLVMGetUser, LLVMIsAInstruction,
        };
        // R5-5: only a read downgraded from a successful C slot inherits C's
        // rooted-chain effect query. Other unrooted R5 records retain A's facts.
        let c_slot_effects = c_slot_effects.filter(|_| rec.downgraded_by_c);
        let (global_writers, drops_heap) = c_slot_effects
            .map_or((global_writers, drops_heap), |effects| {
                (&effects.c_writes_globals, &effects.c_drops_heap)
            });
        let Some((park, _)) = rec.park else {
            return false;
        };
        let marker_ids: HashSet<usize> = rec.markers.iter().map(|m| value_id(*m)).collect();
        let park_id = value_id(park);
        // The uses of the value and of everything derived from it.
        let mut uses: HashSet<usize> = HashSet::new();
        // Stores of it into scratch allocas (a call's `masel` or capture binding, which hold
        // no reference): after the retain they would let it outlive the reference. Every
        // other store of a counted value is made by a holder that takes its own reference
        // (variable and temp slots, `rc_ret`, container elements and fields, globals).
        let mut escapes: HashSet<usize> = HashSet::new();
        let mut use_blocks: HashSet<BasicBlock<'ctx>> = HashSet::new();
        let mut seen: HashSet<usize> = HashSet::new();
        // A path that enters the value's block again reaches a new instance of it.
        let def_block = rec
            .value
            .as_instruction_value()
            .and_then(|i| i.get_parent());
        let mut work = vec![rec.value.as_value_ref()];
        while let Some(x) = work.pop() {
            if !seen.insert(x as usize) {
                continue;
            }
            if seen.len() > 4096 {
                return false;
            }
            // SAFETY: walking the use list of a live value.
            let mut u = unsafe { LLVMGetFirstUse(x) };
            while !u.is_null() {
                // SAFETY: as above.
                let user = unsafe { LLVMGetUser(u) };
                u = unsafe { LLVMGetNextUse(u) };
                // SAFETY: a type query on a live value.
                if unsafe { LLVMIsAInstruction(user) }.is_null() {
                    return false;
                }
                let id = user as usize;
                if marker_ids.contains(&id) || id == park_id {
                    continue;
                }
                // SAFETY: an instruction of this module.
                let inst = unsafe { InstructionValue::new(user) };
                let derived = match inst.get_opcode() {
                    InstructionOpcode::Store => {
                        // Storing the value (or a pointer into it) lets it outlive the
                        // reference; storing into it is only a use.
                        if matches!(Self::rc_operand(inst, 1), Some(p) if self.rc_slots.candidates.contains(&value_id(p))
                            && rec.holder_slot.map(BasicValueEnum::PointerValue) != Some(p))
                            && matches!(Self::rc_operand(inst, 0), Some(v) if v.as_value_ref() == x)
                        {
                            escapes.insert(id);
                        }
                        let scratch = match Self::rc_operand(inst, 1) {
                            Some(BasicValueEnum::PointerValue(p)) => {
                                p.as_instruction().is_some_and(|a| {
                                    a.get_opcode() == InstructionOpcode::Alloca
                                        // A failed candidate still owns its own holder.
                                        // Successful C records have already cleared it.
                                        && rec.holder_slot != Some(p)
                                        && !holders.contains(&value_id(p))
                                        && a.get_name().is_none_or(|n| n.to_bytes() != b"rc_ret")
                                })
                            }
                            _ => false,
                        };
                        if scratch
                            && matches!(Self::rc_operand(inst, 0), Some(v) if v.as_value_ref() == x)
                        {
                            // A scratch alloca only loaded and stored: its loads are
                            // further uses of the value. Anything else lets it escape.
                            match Self::rc_operand(inst, 1).and_then(Self::rc_scratch_loads) {
                                Some(loads) => {
                                    for l in loads {
                                        uses.insert(value_id(l));
                                        if let Some(bb) = l.get_parent() {
                                            use_blocks.insert(bb);
                                        }
                                        work.push(l.as_value_ref());
                                    }
                                }
                                None => {
                                    escapes.insert(id);
                                }
                            }
                        }
                        false
                    }
                    // A returned value carries its own +1 (`rc_return_value`).
                    InstructionOpcode::Return => false,
                    InstructionOpcode::Invoke
                    | InstructionOpcode::CallBr
                    | InstructionOpcode::AtomicRMW
                    | InstructionOpcode::AtomicCmpXchg => return false,
                    InstructionOpcode::Call => Self::rc_call_result_derived(inst, x),
                    // What can still point into the object. Arithmetic, comparisons and
                    // float conversions hold no pointer (codegen forms interior pointers
                    // with inttoptr and getelementptr, never integer adds).
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
                uses.insert(id);
                match inst.get_parent() {
                    // A phi in the value's own block carries it round a loop.
                    Some(bb)
                        if inst.get_opcode() == InstructionOpcode::Phi && Some(bb) == def_block =>
                    {
                        return false
                    }
                    Some(bb) => {
                        use_blocks.insert(bb);
                    }
                    None => return false,
                }
                if derived {
                    work.push(user);
                }
            }
        }
        rec.markers.iter().all(|m| {
            self.rc_marker_region_safe(
                *m,
                def_block,
                rec.holder_slot,
                global_writers,
                drops_heap,
                &uses,
                &escapes,
                &use_blocks,
                holders,
                preds,
                c_slot_effects.is_some(),
            )
        })
    }

    /// No unsafe instruction (`rc_inst_is_safe`) lies on a path from `marker` to one of
    /// `uses` that does not pass `marker` again.
    #[allow(clippy::too_many_arguments)]
    fn rc_marker_region_safe(
        &self,
        marker: InstructionValue<'ctx>,
        def_block: Option<BasicBlock<'ctx>>,
        holder_slot: Option<PointerValue<'ctx>>,
        global_writers: &HashSet<FunctionValue<'ctx>>,
        drops_heap: &HashSet<FunctionValue<'ctx>>,
        uses: &HashSet<usize>,
        escapes: &HashSet<usize>,
        use_blocks: &HashSet<BasicBlock<'ctx>>,
        holders: &HashSet<usize>,
        preds: &mut RcPredMaps<'ctx>,
        c_slot_downgrade: bool,
    ) -> bool {
        let Some(m_block) = marker.get_parent() else {
            return false;
        };
        let Some(function) = m_block.get_parent() else {
            return false;
        };
        let pred_map = preds.entry(function).or_insert_with(|| {
            let mut map: HashMap<BasicBlock<'ctx>, Vec<BasicBlock<'ctx>>> = HashMap::new();
            for bb in function.get_basic_blocks() {
                for s in Self::rc_successors(bb) {
                    map.entry(s).or_default().push(bb);
                }
            }
            map
        });
        // Blocks from whose start a use is reachable without passing the marker again or
        // entering the value's block again (only the marker's way into it counts).
        let mut from_start: HashSet<BasicBlock<'ctx>> = HashSet::new();
        let mut stack: Vec<BasicBlock<'ctx>> = use_blocks
            .iter()
            .copied()
            .filter(|bb| *bb != m_block)
            .collect();
        while let Some(bb) = stack.pop() {
            if !from_start.insert(bb) {
                continue;
            }
            if from_start.len() > 8192 {
                return false;
            }
            if Some(bb) == def_block {
                continue;
            }
            if let Some(ps) = pred_map.get(&bb) {
                for p in ps {
                    if *p != m_block && !from_start.contains(p) {
                        stack.push(*p);
                    }
                }
            }
        }
        let reaches_end = |bb: BasicBlock<'ctx>| {
            Self::rc_successors(bb)
                .iter()
                .any(|s| from_start.contains(s))
        };
        // The instructions of a block from `first` that lie on a way to a use: all of them
        // when a use lies beyond the block, else up to its last use there.
        let scan = |first: Option<InstructionValue<'ctx>>, all: bool| -> bool {
            let mut last_use = None;
            if !all {
                let mut i = first;
                while let Some(inst) = i {
                    if uses.contains(&value_id(inst)) {
                        last_use = Some(value_id(inst));
                    }
                    i = inst.get_next_instruction();
                }
                if last_use.is_none() {
                    return true;
                }
            }
            let mut i = first;
            while let Some(inst) = i {
                let effect = if c_slot_downgrade {
                    rc_c_call_effect(&self.module, inst)
                } else {
                    self.rc_call_effect(inst)
                };
                let code_unsafe = effect == RcCallEffect::RunsCode
                    && match holder_slot {
                        // No other code can reach an unboxed local's slot.
                        Some(slot) if !self.rc_is_shared_slot(slot) => false,
                        // A global: only compiled code that may write a global can.
                        Some(_) => !Self::rc_callee(inst).is_some_and(|f| {
                            f.count_basic_blocks() > 0 && !global_writers.contains(&f)
                        }),
                        None => !Self::rc_callee(inst).is_some_and(|f| {
                            f.count_basic_blocks() > 0
                                && !drops_heap.contains(&f)
                                && !global_writers.contains(&f)
                        }),
                    };
                let unsafe_here = escapes.contains(&value_id(inst))
                    || code_unsafe
                    || match holder_slot {
                        // A variable slot holds the value: only writing the slot (or code)
                        // can drop its reference.
                        // A store into the slot drops nothing itself (the old value's
                        // release after it does, and that release is checked in turn), so
                        // when the store is the region's last use nothing is lost.
                        Some(slot) => {
                            let last_store = inst.get_opcode() == InstructionOpcode::Store
                                && !all
                                && Some(value_id(inst)) == last_use;
                            (inst.get_opcode() == InstructionOpcode::Store
                                || effect == RcCallEffect::Drops)
                                && !last_store
                                && Self::rc_touches_slot(inst, slot)
                        }
                        None => {
                            effect == RcCallEffect::Drops
                                && !Self::rc_releases_parked_old(inst, holders)
                        }
                    };
                if unsafe_here {
                    return false;
                }
                if !all && Some(value_id(inst)) == last_use {
                    break;
                }
                i = inst.get_next_instruction();
            }
            true
        };
        if !scan(marker.get_next_instruction(), reaches_end(m_block)) {
            return false;
        }
        // The blocks reachable from the marker on the way to a use.
        let mut visited: HashSet<BasicBlock<'ctx>> = HashSet::new();
        let mut stack: Vec<BasicBlock<'ctx>> = Self::rc_successors(m_block)
            .into_iter()
            .filter(|s| from_start.contains(s))
            .collect();
        while let Some(bb) = stack.pop() {
            if !visited.insert(bb) {
                continue;
            }
            if !scan(bb.get_first_instruction(), reaches_end(bb)) {
                return false;
            }
            for s in Self::rc_successors(bb) {
                if from_start.contains(&s) && !visited.contains(&s) {
                    stack.push(s);
                }
            }
        }
        true
    }

    /// `inst` releases what a temp held before it was parked again (`rc_tmp_old`, see
    /// `rc_own_temp`): the result of an earlier evaluation of that temp's expression, or nil.
    /// That cannot drop a holder of a value read now: nothing read in this evaluation goes
    /// through it, and every holder on the way to the value holds its own reference.
    fn rc_releases_parked_old(inst: InstructionValue<'ctx>, holders: &HashSet<usize>) -> bool {
        let is_release = Self::rc_callee(inst).is_some_and(|f| {
            matches!(
                f.get_name().to_bytes(),
                b"__mdh_release" | b"__mdh_rc_release_obj"
            )
        });
        if !is_release {
            return false;
        }
        let mut arg = Self::rc_operand(inst, 0).and_then(|v| v.as_instruction_value());
        if let Some(i) = arg {
            if i.get_opcode() == InstructionOpcode::ExtractValue {
                arg = Self::rc_operand(i, 0).and_then(|v| v.as_instruction_value());
            }
        }
        arg.is_some_and(|load| {
            load.get_opcode() == InstructionOpcode::Load
                && load
                    .get_name()
                    .is_some_and(|n| n.to_bytes().starts_with(b"rc_tmp_old"))
                && matches!(Self::rc_operand(load, 0),
                    Some(BasicValueEnum::PointerValue(p)) if holders.contains(&value_id(p)))
        })
    }

    /// The compiled functions that may write a user global: directly (a store, or
    /// `__mdh_shared_store`, into one), through code this pass cannot see (an indirect call,
    /// a runtime function that calls back), or by calling such a function.
    fn rc_global_writers(&self) -> HashSet<FunctionValue<'ctx>> {
        let globals: HashSet<usize> = self
            .rc_user_globals()
            .iter()
            .map(|g| value_id(*g))
            .collect();
        let is_global = |v: Option<BasicValueEnum<'ctx>>| matches!(v, Some(BasicValueEnum::PointerValue(p)) if globals.contains(&value_id(p)));
        let mut writers: HashSet<FunctionValue<'ctx>> = HashSet::new();
        let mut callers: HashMap<FunctionValue<'ctx>, Vec<FunctionValue<'ctx>>> = HashMap::new();
        for f in self.module.get_functions() {
            for bb in f.get_basic_blocks() {
                let mut i = bb.get_first_instruction();
                while let Some(inst) = i {
                    i = inst.get_next_instruction();
                    match inst.get_opcode() {
                        InstructionOpcode::Store if is_global(Self::rc_operand(inst, 1)) => {
                            writers.insert(f);
                        }
                        InstructionOpcode::Call
                        | InstructionOpcode::Invoke
                        | InstructionOpcode::CallBr => match Self::rc_callee(inst) {
                            Some(g) if g.count_basic_blocks() > 0 => {
                                callers.entry(g).or_default().push(f);
                            }
                            _ => {
                                let args = inst.get_num_operands().saturating_sub(1);
                                if self.rc_call_effect(inst) == RcCallEffect::RunsCode
                                    || (0..args).any(|a| is_global(Self::rc_operand(inst, a)))
                                        && self.rc_call_effect(inst) == RcCallEffect::Drops
                                {
                                    writers.insert(f);
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        let mut work: Vec<FunctionValue<'ctx>> = writers.iter().copied().collect();
        while let Some(f) = work.pop() {
            if let Some(cs) = callers.get(&f) {
                for c in cs {
                    if writers.insert(*c) {
                        work.push(*c);
                    }
                }
            }
        }
        writers
    }

    /// The blocks a block's terminator can branch to.
    fn rc_successors(bb: BasicBlock<'ctx>) -> Vec<BasicBlock<'ctx>> {
        let Some(term) = bb.get_terminator() else {
            return Vec::new();
        };
        (0..term.get_num_operands())
            .filter_map(|i| match term.get_operand(i) {
                Some(inkwell::values::Operand::Block(b)) => Some(b),
                _ => None,
            })
            .collect()
    }

    /// The result of call `inst`, one of whose arguments is `x`, may point into `x`'s
    /// object. Not for a runtime function whose result is an immediate or a new object
    /// (whose elements the runtime retained), nor for a libc function unless it returns a
    /// pointer and `x` is its first argument (`memcpy` and `strcpy` return their
    /// destination, `strstr` points into its first argument).
    fn rc_call_result_derived(inst: InstructionValue<'ctx>, x: LLVMValueRefAlias) -> bool {
        if matches!(inst.get_type(), inkwell::types::AnyTypeEnum::VoidType(_)) {
            return false;
        }
        let Some(f) = Self::rc_callee(inst) else {
            return true;
        };
        if f.count_basic_blocks() > 0 {
            return true;
        }
        let name = f.get_name().to_string_lossy();
        if let Some((_, ret)) = ownership_of(&name) {
            return !matches!(ret, Ret::None | Ret::Fresh);
        }
        if name.starts_with("__mdh_") {
            return true;
        }
        matches!(inst.get_type(), inkwell::types::AnyTypeEnum::PointerType(_))
            && matches!(Self::rc_operand(inst, 0), Some(v) if v.as_value_ref() == x)
    }

    /// What instruction `inst` can do to reference counts. Only calls lower one: a
    /// release, a runtime function that overwrites, removes, frees or moves an object, or
    /// any compiled code (a compiled function, an indirect call, a runtime function that
    /// calls back). Unknown callees count as running code.
    fn rc_call_effect(&self, inst: InstructionValue<'ctx>) -> RcCallEffect {
        rc_call_effect(&self.module, inst)
    }

    /// The loads of alloca `p` when it is only ever loaded from and stored to (a call's
    /// scratch binding), else `None`.
    fn rc_scratch_loads(p: BasicValueEnum<'ctx>) -> Option<Vec<InstructionValue<'ctx>>> {
        use inkwell::llvm_sys::core::{
            LLVMGetFirstUse, LLVMGetNextUse, LLVMGetUser, LLVMIsAInstruction,
        };
        let mut loads = Vec::new();
        // SAFETY: walking the use list of a live alloca.
        let mut u = unsafe { LLVMGetFirstUse(p.as_value_ref()) };
        while !u.is_null() {
            let user = unsafe { LLVMGetUser(u) };
            u = unsafe { LLVMGetNextUse(u) };
            if unsafe { LLVMIsAInstruction(user) }.is_null() {
                return None;
            }
            // SAFETY: an instruction of this module.
            let inst = unsafe { InstructionValue::new(user) };
            match inst.get_opcode() {
                InstructionOpcode::Load => loads.push(inst),
                InstructionOpcode::Store
                    if Self::rc_operand(inst, 1) == Some(p)
                        && Self::rc_operand(inst, 0) != Some(p) => {}
                _ => return None,
            }
        }
        Some(loads)
    }

    /// `inst` writes the variable slot `slot`, passes its address to a call, or passes a
    /// value loaded from it to a call that may release that value: the only ways, besides
    /// compiled code, to drop the slot's reference. A runtime function with an ownership
    /// row borrows its arguments (`b`, `m`, `s`: it reads them, mutates their contents or
    /// takes a reference of its own), so it cannot drop the caller's reference to the
    /// object itself; a release, free or realloc, or a function with no row, can.
    /// Invariant (`src/llvm/ownership.rs`, test `no_row_consumes_its_argument`): no
    /// function with a row consumes or frees its argument object; the release family is
    /// the exception and is matched by name here.
    fn rc_touches_slot(inst: InstructionValue<'ctx>, slot: PointerValue<'ctx>) -> bool {
        let may_release_arg = Self::rc_callee(inst).is_none_or(|f| {
            let name = f.get_name().to_string_lossy();
            ["release", "free", "realloc"]
                .iter()
                .any(|fam| name.contains(fam))
                || ownership_of(&name).is_none()
        });
        let is_slot = |v: Option<BasicValueEnum<'ctx>>| -> bool {
            let mut v = v;
            for _ in 0..2 {
                match v {
                    Some(BasicValueEnum::PointerValue(p)) if p == slot => return true,
                    Some(other) => match other.as_instruction_value() {
                        Some(i) if i.get_opcode() == InstructionOpcode::Load => {
                            return may_release_arg
                                && matches!(Self::rc_operand(i, 0),
                                    Some(BasicValueEnum::PointerValue(p)) if p == slot);
                        }
                        Some(i) if i.get_opcode() == InstructionOpcode::ExtractValue => {
                            v = Self::rc_operand(i, 0);
                        }
                        _ => return false,
                    },
                    None => return false,
                }
            }
            false
        };
        match inst.get_opcode() {
            InstructionOpcode::Store => is_slot(Self::rc_operand(inst, 1)),
            InstructionOpcode::Call => {
                (0..inst.get_num_operands()).any(|i| is_slot(Self::rc_operand(inst, i)))
            }
            _ => false,
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
        let is_heap = self.rc_build_heap_test(&b, tag, "", false);
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
                self.rc_slot_release(*slot, v, true);
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

/// After the optimiser (counting on, -O1 and up): give every tag read of an `MdhValue` phi
/// its own `i8` phi. InstCombine merges the values a numeric operator builds on each path
/// (`{ i8 2, x }`, `{ i8 3, y }`) into one phi of aggregates, and cannot see through a
/// multi-use phi of aggregates to the constant tags, so the heap tests on the result (a
/// temp's release, a slot store's retain and release) stay as run-time tests. With the
/// tag as a phi of constants, the passes run next (`RC_TAG_CLEANUP`) fold the tests or
/// thread them per path. A pure rewrite: each new phi takes, on each edge, the tag of the
/// value the old phi took there. Returns true if anything changed.
pub(crate) fn rc_split_tag_phis(module: &Module<'_>) -> bool {
    use inkwell::llvm_sys::core::{
        LLVMAddIncoming, LLVMBuildExtractValue, LLVMBuildPhi, LLVMCountIncoming,
        LLVMCountStructElementTypes, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
        LLVMGetAggregateElement, LLVMGetBasicBlockTerminator, LLVMGetIncomingBlock,
        LLVMGetIncomingValue, LLVMGetIndices, LLVMGetInstructionOpcode, LLVMGetModuleContext,
        LLVMGetNumIndices, LLVMGetOperand, LLVMGetTypeKind, LLVMInstructionEraseFromParent,
        LLVMIsAConstant, LLVMIsAInstruction, LLVMIsAPHINode, LLVMPositionBuilderBefore,
        LLVMReplaceAllUsesWith, LLVMStructGetTypeAtIndex, LLVMTypeOf,
    };
    use inkwell::llvm_sys::prelude::{LLVMBasicBlockRef, LLVMBuilderRef, LLVMValueRef};
    use inkwell::llvm_sys::{LLVMOpcode, LLVMTypeKind};

    /// The single index of an `extractvalue`/`insertvalue`, if it has one.
    unsafe fn single_index(inst: LLVMValueRef) -> Option<u32> {
        if LLVMGetNumIndices(inst) != 1 {
            return None;
        }
        Some(*LLVMGetIndices(inst))
    }

    /// A tagged-value aggregate: `{ i8, ... }`.
    unsafe fn is_tagged(v: LLVMValueRef) -> bool {
        let ty = LLVMTypeOf(v);
        LLVMGetTypeKind(ty) == LLVMTypeKind::LLVMStructTypeKind
            && LLVMCountStructElementTypes(ty) == 2
            && LLVMGetTypeKind(LLVMStructGetTypeAtIndex(ty, 0)) == LLVMTypeKind::LLVMIntegerTypeKind
    }

    struct Split {
        builder: LLVMBuilderRef,
        done: HashMap<usize, LLVMValueRef>,
        /// The extract made for a value at the end of an edge's block: a phi may list the
        /// same predecessor more than once (a switch with two cases to one block), and
        /// every entry for one block must carry the same value.
        extracted: HashMap<(usize, usize), LLVMValueRef>,
    }

    impl Split {
        /// The tag of `v`, as available at the end of `bb` (where `v` is available).
        /// `depth` bounds the walk down `insertvalue` chains: an unreachable block may hold
        /// one that feeds itself.
        unsafe fn tag_of(
            &mut self,
            v: LLVMValueRef,
            bb: LLVMBasicBlockRef,
            depth: u32,
        ) -> LLVMValueRef {
            if !LLVMIsAConstant(v).is_null() {
                let t = LLVMGetAggregateElement(v, 0);
                if !t.is_null() {
                    return t;
                }
            }
            if !LLVMIsAInstruction(v).is_null() && depth < 32 {
                match LLVMGetInstructionOpcode(v) {
                    LLVMOpcode::LLVMInsertValue => match single_index(v) {
                        Some(0) => return LLVMGetOperand(v, 1),
                        Some(_) => return self.tag_of(LLVMGetOperand(v, 0), bb, depth + 1),
                        None => {}
                    },
                    LLVMOpcode::LLVMPHI => return self.phi_tag(v),
                    _ => {}
                }
            }
            let key = (v as usize, bb as usize);
            if let Some(t) = self.extracted.get(&key) {
                return *t;
            }
            LLVMPositionBuilderBefore(self.builder, LLVMGetBasicBlockTerminator(bb));
            let t = LLVMBuildExtractValue(self.builder, v, 0, c"rc_tag_in".as_ptr());
            self.extracted.insert(key, t);
            t
        }

        /// The `i8` phi of `phi`'s tags (made once; loops reach it again while it fills).
        unsafe fn phi_tag(&mut self, phi: LLVMValueRef) -> LLVMValueRef {
            if let Some(t) = self.done.get(&(phi as usize)) {
                return *t;
            }
            let ty = LLVMStructGetTypeAtIndex(LLVMTypeOf(phi), 0);
            LLVMPositionBuilderBefore(self.builder, phi);
            let tag = LLVMBuildPhi(self.builder, ty, c"rc_tag_phi".as_ptr());
            self.done.insert(phi as usize, tag);
            for i in 0..LLVMCountIncoming(phi) {
                let mut bb = LLVMGetIncomingBlock(phi, i);
                let mut t = self.tag_of(LLVMGetIncomingValue(phi, i), bb, 0);
                LLVMAddIncoming(tag, &mut t, &mut bb, 1);
            }
            tag
        }
    }

    let mut extracts: Vec<LLVMValueRef> = Vec::new();
    for f in module.get_functions() {
        // An incoming value made by its edge's terminator (an invoke's result) has no
        // place for the extract; codegen emits none, but stay clear of them.
        let has_invoke = f.get_basic_blocks().iter().any(|bb| {
            bb.get_terminator().is_some_and(|t| {
                matches!(
                    t.get_opcode(),
                    InstructionOpcode::Invoke | InstructionOpcode::CallBr
                )
            })
        });
        if has_invoke {
            continue;
        }
        for bb in f.get_basic_blocks() {
            let mut i = bb.get_first_instruction();
            while let Some(inst) = i {
                i = inst.get_next_instruction();
                if inst.get_opcode() != InstructionOpcode::ExtractValue {
                    continue;
                }
                let raw = inst.as_value_ref();
                // SAFETY: queries on a live instruction of this module.
                unsafe {
                    let agg = LLVMGetOperand(raw, 0);
                    if single_index(raw) == Some(0)
                        && !LLVMIsAPHINode(agg).is_null()
                        && is_tagged(agg)
                    {
                        extracts.push(raw);
                    }
                }
            }
        }
    }
    if extracts.is_empty() {
        return false;
    }
    // SAFETY: every value handled is a live instruction or constant of this module; each
    // extract is replaced by a value of its own type, then erased once.
    unsafe {
        let ctx = LLVMGetModuleContext(module.as_mut_ptr());
        let mut split = Split {
            builder: LLVMCreateBuilderInContext(ctx),
            done: HashMap::new(),
            extracted: HashMap::new(),
        };
        for e in extracts {
            let tag = split.phi_tag(LLVMGetOperand(e, 0));
            LLVMReplaceAllUsesWith(e, tag);
            LLVMInstructionEraseFromParent(e);
        }
        LLVMDisposeBuilder(split.builder);
    }
    true
}

/// The passes run after `rc_split_tag_phis`: fold the heap tests the new tag phis decide
/// (value ranges, then per-predecessor threading), and tidy up.
pub(crate) const RC_TAG_CLEANUP: &str =
    "function(instcombine<no-verify-fixpoint>,correlated-propagation,jump-threading,simplifycfg,instcombine<no-verify-fixpoint>)";

#[cfg(test)]
mod split_tests {
    use super::rc_split_tag_phis;
    use inkwell::context::Context;
    use inkwell::memory_buffer::MemoryBuffer;

    fn split_and_verify(ir: &str) {
        let context = Context::create();
        let buf = MemoryBuffer::create_from_memory_range_copy(ir.as_bytes(), "t");
        let module = context.create_module_from_ir(buf).expect("parse IR");
        module.verify().expect("input verifies");
        assert!(rc_split_tag_phis(&module));
        if let Err(e) = module.verify() {
            panic!(
                "{}\n{}",
                e.to_string(),
                module.print_to_string().to_string()
            );
        }
    }

    /// A phi listing one predecessor twice (two switch cases to one block) must get the
    /// same tag value for both entries.
    #[test]
    fn a_predecessor_listed_twice_gets_one_tag() {
        split_and_verify(
            r#"
declare {i8, i64} @g()
define i8 @f(i32 %c) {
entry:
  %v = call {i8, i64} @g()
  switch i32 %c, label %o [ i32 1, label %m
                            i32 2, label %m ]
o:
  br label %m
m:
  %p = phi {i8, i64} [ %v, %entry ], [ %v, %entry ], [ { i8 2, i64 0 }, %o ]
  %t = extractvalue {i8, i64} %p, 0
  ret i8 %t
}
"#,
        );
    }

    /// An unreachable block may hold an `insertvalue` that feeds itself: the walk down
    /// the chain must stop.
    #[test]
    fn a_self_feeding_insertvalue_ends_the_walk() {
        split_and_verify(
            r#"
define i8 @h() {
entry:
  ret i8 0
dead:
  %a = insertvalue {i8, i64} %a, i64 1, 1
  br label %m
m:
  %p = phi {i8, i64} [ %a, %dead ]
  %t = extractvalue {i8, i64} %p, 0
  ret i8 %t
}
"#,
        );
    }
}

#[cfg(test)]
mod rc7_summary_tests {
    use super::{rc_drops_heap, value_id};
    use inkwell::{context::Context, memory_buffer::MemoryBuffer, values::InstructionOpcode};
    use std::collections::HashSet;

    fn summary(ir: &str) -> HashSet<String> {
        summary_with_owned_returns(ir, &[])
    }

    fn summary_with_owned_returns(ir: &str, owned_names: &[&str]) -> HashSet<String> {
        let context = Context::create();
        let module = context
            .create_module_from_ir(MemoryBuffer::create_from_memory_range_copy(
                ir.as_bytes(),
                "summary",
            ))
            .expect("parse hand-written IR");
        module.verify().expect("summary input verifies");
        let mut own = HashSet::new();
        for function in module.get_functions() {
            for bb in function.get_basic_blocks() {
                let mut next = bb.get_first_instruction();
                while let Some(inst) = next {
                    next = inst.get_next_instruction();
                    if inst.get_opcode() == InstructionOpcode::Alloca
                        && inst
                            .get_name()
                            .is_some_and(|n| n.to_bytes().starts_with(b"own"))
                    {
                        own.insert(value_id(inst));
                    }
                }
            }
        }
        let owned_returns = owned_names
            .iter()
            .map(|name| {
                module
                    .get_function(name)
                    .expect("registered return function")
            })
            .collect();
        rc_drops_heap(&module, &own, &owned_returns)
            .iter()
            .map(|f| f.get_name().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn only_the_explicit_return_contract_makes_call_results_owned() {
        let drops = summary_with_owned_returns(
            r#"
declare void @__mdh_release({i8, i64})
declare {i8, i64} @__mdh_fn_forward()
define {i8, i64} @alias({i8, i64} %arg) {
  ret {i8, i64} %arg
}
define {i8, i64} @registered() {
  ret {i8, i64} zeroinitializer
}
define void @releases_alias({i8, i64} %arg) {
  %v = call {i8, i64} @alias({i8, i64} %arg)
  call void @__mdh_release({i8, i64} %v)
  ret void
}
define void @releases_compiled() {
  %v = call {i8, i64} @__mdh_fn_forward()
  call void @__mdh_release({i8, i64} %v)
  ret void
}
define void @releases_registered() {
  %v = call {i8, i64} @registered()
  call void @__mdh_release({i8, i64} %v)
  ret void
}
"#,
            &["registered"],
        );
        assert!(
            drops.contains("releases_alias"),
            "a body is not a +1 contract"
        );
        // The declaration itself remains RunsCode; the return contract only makes
        // its result owned. A registered body can be proved clean end to end.
        assert!(!drops.contains("releases_registered"));
        assert!(drops.contains("releases_compiled"));
    }

    #[test]
    fn own_slot_releases_are_clean_but_element_releases_propagate() {
        let drops = summary(
            r#"
declare void @__mdh_release({i8, i64})
define void @own_only() {
  %own = alloca {i8, i64}
  store {i8, i64} zeroinitializer, ptr %own
  %v = load {i8, i64}, ptr %own
  call void @__mdh_release({i8, i64} %v)
  ret void
}
define void @element(ptr %slot) {
  %old = load {i8, i64}, ptr %slot
  store {i8, i64} zeroinitializer, ptr %slot
  call void @__mdh_release({i8, i64} %old)
  ret void
}
define void @calls_own() {
  call void @own_only()
  ret void
}
define void @calls_element(ptr %slot) {
  call void @element(ptr %slot)
  ret void
}
"#,
        );
        assert!(
            !drops.contains("own_only"),
            "an owning slot's release is local"
        );
        assert!(
            !drops.contains("calls_own"),
            "a clean direct call remains clean"
        );
        assert!(
            drops.contains("element"),
            "an element's old-value release drops a heap holder"
        );
        assert!(
            drops.contains("calls_element"),
            "heap drops must reach callers"
        );
    }

    #[test]
    fn fresh_results_extracts_and_phis_are_traced_but_scratch_is_not_owned() {
        let drops = summary(
            r#"
declare {i8, i64} @__mdh_empty_dict()
declare void @__mdh_rc_release_obj(i64)
declare void @__mdh_release({i8, i64})
define void @fresh_or_own(i1 %c) {
entry:
  %own = alloca {i8, i64}
  store {i8, i64} zeroinitializer, ptr %own
  br i1 %c, label %left, label %right
left:
  %fresh = call {i8, i64} @__mdh_empty_dict()
  br label %merge
right:
  %loaded = load {i8, i64}, ptr %own
  br label %merge
merge:
  %v = phi {i8, i64} [%fresh, %left], [%loaded, %right]
  %data = extractvalue {i8, i64} %v, 1
  call void @__mdh_rc_release_obj(i64 %data)
  ret void
}
define void @scratch({i8, i64} %arg) {
  %binding = alloca {i8, i64}
  store {i8, i64} %arg, ptr %binding
  %v = load {i8, i64}, ptr %binding
  call void @__mdh_release({i8, i64} %v)
  ret void
}
define void @mixed(i1 %c, {i8, i64} %heap) {
entry:
  %fresh = call {i8, i64} @__mdh_empty_dict()
  br i1 %c, label %left, label %right
left:
  br label %merge
right:
  br label %merge
merge:
  %v = phi {i8, i64} [%fresh, %left], [%heap, %right]
  call void @__mdh_release({i8, i64} %v)
  ret void
}
"#,
        );
        assert!(
            !drops.contains("fresh_or_own"),
            "all phi roots carry frame-owned references"
        );
        assert!(
            drops.contains("scratch"),
            "an unregistered scratch binding is not owning"
        );
        assert!(
            drops.contains("mixed"),
            "every phi incoming must be traceable"
        );
    }

    #[test]
    fn heap_drops_reach_the_whole_recursive_caller_fixpoint() {
        let drops = summary(
            r#"
declare void @__mdh_release({i8, i64})
define void @first(ptr %slot) {
  call void @bridge(ptr %slot)
  ret void
}
define void @bridge(ptr %slot) {
  call void @second(ptr %slot)
  ret void
}
define void @second(ptr %slot) {
  %old = load {i8, i64}, ptr %slot
  store {i8, i64} zeroinitializer, ptr %slot
  call void @__mdh_release({i8, i64} %old)
  call void @first(ptr %slot)
  ret void
}
define void @outer(ptr %slot) {
  call void @first(ptr %slot)
  ret void
}
"#,
        );
        for name in ["first", "bridge", "second", "outer"] {
            assert!(drops.contains(name), "fixpoint must reach {name}");
        }
    }

    #[test]
    fn runtime_drops_callbacks_and_unknown_calls_are_conservative() {
        let drops = summary(
            r#"
declare void @__mdh_shared_store(ptr, {i8, i64})
declare {i8, i64} @__mdh_dict_set({i8, i64}, {i8, i64}, {i8, i64})
declare void @__mdh_log()
declare void @free(ptr)
declare ptr @realloc(ptr, i64)
declare i32 @setjmp(ptr)
declare void @unknown_external()
define void @shared(ptr %slot) {
  call void @__mdh_shared_store(ptr %slot, {i8, i64} zeroinitializer)
  ret void
}
define void @mutating({i8, i64} %dict) {
  %v = call {i8, i64} @__mdh_dict_set({i8, i64} %dict, {i8, i64} zeroinitializer, {i8, i64} zeroinitializer)
  ret void
}
define void @callback() {
  call void @__mdh_log()
  ret void
}
define void @freeing(ptr %p) {
  call void @free(ptr %p)
  ret void
}
define void @moving(ptr %p) {
  %v = call ptr @realloc(ptr %p, i64 32)
  ret void
}
define void @nonlocal(ptr %p) {
  %v = call i32 @setjmp(ptr %p)
  ret void
}
define void @external() {
  call void @unknown_external()
  ret void
}
define void @indirect(ptr %f) {
  call void %f()
  ret void
}
"#,
        );
        for name in [
            "shared", "mutating", "callback", "freeing", "moving", "nonlocal", "external",
            "indirect",
        ] {
            assert!(
                drops.contains(name),
                "{name} can drop a heap holder or run unknown code"
            );
        }
    }
}
