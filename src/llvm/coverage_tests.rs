#![cfg(coverage)]

use super::LLVMCompiler;
use crate::llvm::codegen::CodeGen;
use crate::parser::parse;
use inkwell::context::Context;
use std::path::Path;
use tempfile::tempdir;

fn compile_to_ir_for_unit_coverage(source: &str) {
    let program = parse(source).unwrap();
    let _ = LLVMCompiler::new().compile_to_ir(&program).unwrap();
}

fn compile_with_source_path_for_unit_coverage(source: &str, source_path: &Path) -> crate::error::HaversResult<()> {
    let program = parse(source).unwrap();
    let context = Context::create();
    let mut codegen = CodeGen::new(&context, "coverage_codegen_source_path");
    codegen.set_source_path(source_path);
    codegen.compile(&program)
}

#[test]
fn llvm_codegen_boxed_var_decl_paths_are_exercised_for_unit_coverage() {
    // Drives the boxed-variable declaration path via a nested function that mutates an outer local.
    compile_to_ir_for_unit_coverage(
        r#"
dae outer() {
    ken x = 0
    dae inc() { x = x + 1 }
    inc()
    gie x
}
outer()
"#,
    );
}

#[test]
fn llvm_codegen_closure_value_boxes_and_captures_outer_locals_for_unit_coverage() {
    // Forces a nested function with captures to be referenced as a first-class value, exercising
    // closure construction and capture boxing logic.
    compile_to_ir_for_unit_coverage(
        r#"
dae outer() {
    ken x = 0
    dae inc() { x = x + 1; gie x }
    ken f = inc
    f()
    gie x
}
outer()
"#,
    );
}

#[test]
fn llvm_codegen_globals_lookup_fallback_is_exercised_for_unit_coverage() {
    // Drives variable resolution from function scope into the global slot.
    compile_to_ir_for_unit_coverage(
        r#"
ken g = 1
dae f() { gie g }
f()
"#,
    );
}

#[test]
fn llvm_codegen_llvm_compile_error_builder_error_is_exercised_for_unit_coverage() {
    let context = Context::create();
    let codegen = crate::llvm::codegen::CodeGen::new(&context, "coverage_builder_error");
    let _ = codegen.coverage_llvm_compile_error_builder_error();
}

#[test]
fn llvm_codegen_condition_direct_error_branches_are_exercised_for_unit_coverage() {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context, "coverage_condition_direct_error_branches");
    codegen.coverage_compile_condition_direct_error_branches();
}

#[test]
fn llvm_codegen_condition_direct_shadow_fallbacks_are_exercised_for_unit_coverage() {
    // Uses global int variables in comparisons so `compile_int_expr` returns `None` and the
    // `compile_expr`+`extract_data` fallback path is taken in `compile_condition_direct`.
    compile_to_ir_for_unit_coverage(
        r#"
ken g = 1
dae f() {
    gin g == 1 { blether 1 }
    gin g < 2 { blether 2 }
}
f()
"#,
    );
}

#[test]
fn llvm_codegen_condition_direct_bool_var_path_is_exercised_for_unit_coverage() {
    compile_to_ir_for_unit_coverage(
        r#"
ken b = aye
gin b { blether 1 }
"#,
    );
}

#[test]
fn llvm_codegen_condition_direct_index_fast_paths_are_exercised_for_unit_coverage() {
    // Forces the index fast path to take both `compile_expr` fallbacks (global list and global int
    // index without an alloca shadow).
    compile_to_ir_for_unit_coverage(
        r#"
ken xs = [1, 2, 3]
ken i = 0
gin xs[i] { blether 1 }
gin [1, 2][0] { blether 2 }
"#,
    );
}

#[test]
fn llvm_codegen_dict_callable_get_path_is_exercised_for_unit_coverage() {
    // Drives the `compile_call` dict-object callable path: d.f(args...)
    compile_to_ir_for_unit_coverage(
        r#"
ken d = {"f": |x| x + 1}
blether d.f(1)
"#,
    );
}

#[test]
fn llvm_codegen_append_and_shove_builtins_are_exercised_for_unit_coverage() {
    compile_to_ir_for_unit_coverage(
        r#"
dae f() {
    ken xs = [1]
    shove(xs, aye)
    shove(xs, 2)
    gie xs
}
f()

ken a = append([1], 2)
ken b = append([1], [2])

shove([1], 2)
ken not_a_list = 1
shove(not_a_list, 2)
"#,
    );
}

#[test]
fn llvm_codegen_list_add_string_repeat_and_not_equal_are_exercised_for_unit_coverage() {
    compile_to_ir_for_unit_coverage(
        r#"
blether [1] + [2]
blether "ha" * 3
blether 1 != 2
"#,
    );
}

#[test]
fn llvm_codegen_range_expr_and_speir_expr_are_exercised_for_unit_coverage() {
    compile_to_ir_for_unit_coverage(
        r#"
ken r1 = 1..3
ken r2 = 1..=3
ken s = speir "aye?"
blether r1
blether r2
blether s
"#,
    );
}

#[test]
fn llvm_codegen_set_on_global_dict_updates_global_slot_for_unit_coverage() {
    // Covers the `globals` store-back path in `compile_set` when mutating a global dict.
    compile_to_ir_for_unit_coverage(
        r#"
ken d = {}
dae f() { d.x = 1 }
f()
"#,
    );
}

#[test]
fn llvm_codegen_more_builtin_inline_helpers_are_exercised_for_unit_coverage() {
    // Target additional inline_* helpers that are otherwise only reached via the large integration
    // sweep (dependency-crate instance). Keeping a smaller curated set here improves instantiation
    // coverage in the unit-crate instance without adding flake.
    compile_to_ir_for_unit_coverage(
        r#"
blether abs(-1)
blether min(1, 2)
blether max(1, 2)
blether ceil(1.2)
blether floor(1.2)
blether round(1.2)
blether sqrt(4.0)
blether pow(2.0, 3.0)

blether len([1, 2, 3])
blether slice([1, 2, 3, 4, 5], 1, 3)
blether reverse("abc")
blether join(["a", "b"], ",")
blether split("a b", " ")
blether contains([1, 2, 3], 2)
blether index_of("hello", "ll")
blether starts_wi("hello", "he")
blether ends_wi("hello", "lo")
blether keys({"a": 1, "b": 2})
blether values({"a": 1, "b": 2})
blether dict_merge({"a": 1}, {"b": 2})
blether upper("a")
blether lower("A")
"#,
    );
}

#[test]
fn llvm_codegen_more_list_string_and_timing_builtins_are_exercised_for_unit_coverage() {
    // Cover additional inline_* helpers (list/string/timing/math) in the unit-crate instance to
    // improve instantiation/region coverage without executing any runtime behavior.
    compile_to_ir_for_unit_coverage(
        r#"
ken xs = [1, 2, 3]
blether heid(xs)
blether bum(xs)
blether tail(xs)
blether yank(xs)

blether scran([1, 2, 3, 4], 1, 3)
blether scran("hello", 1, 4)
blether scran("hello")

blether slap([1, 2], [3, 4])
blether slap("a", "b")

blether zipwith(|a, b| a + b, [1, 2], [3, 4])

blether sumaw([1, 2, 3])
blether product([2, 3, 4])

blether wheesht([0, 1, "", "a", naething])
blether wheesht("  hello  ")

blether coont([1, 2, 2, 3, 2], 2)
blether coont("banana", "na")
blether coont([1, 2, 3])

blether ord("A")
blether chr(65)
blether char_at("hello", 1)
blether substr("hello", 1, 4)
blether chars("hi")
blether repeat("ha", 3)

blether pad("x", 3)
blether pad_left("x", 3, "0")
blether pad_right("x", 3, ".")

blether radians(180.0)
blether degrees(3.14159)
blether atan2(1.0, 2.0)

blether noo()
blether tick()
blether time_now()
blether timestamp_millis()
blether timestamp()

blether bide(1)
blether bide(1.0)
"#,
    );
}

#[test]
fn llvm_codegen_import_var_initializer_error_branch_is_exercised_for_unit_coverage() {
    let dir = tempdir().unwrap();
    let main_path = dir.path().join("main.braw");
    std::fs::write(
        &main_path,
        r#"
fetch "bad_var_init"
"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("bad_var_init.braw"),
        r#"
ken x = __missing__
"#,
    )
    .unwrap();

    let err = compile_with_source_path_for_unit_coverage(
        r#"
fetch "bad_var_init"
"#,
        &main_path,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Undefined variable"));
}

#[test]
fn llvm_codegen_import_function_body_error_branch_is_exercised_for_unit_coverage() {
    let dir = tempdir().unwrap();
    let main_path = dir.path().join("main.braw");
    std::fs::write(
        &main_path,
        r#"
fetch "bad_fn"
"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("bad_fn.braw"),
        r#"
dae bad() { gie __missing__ }
"#,
    )
    .unwrap();

    let err = compile_with_source_path_for_unit_coverage(
        r#"
fetch "bad_fn"
"#,
        &main_path,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Undefined variable"));
}

#[test]
fn llvm_codegen_import_class_body_error_branch_is_exercised_for_unit_coverage() {
    let dir = tempdir().unwrap();
    let main_path = dir.path().join("main.braw");
    std::fs::write(
        &main_path,
        r#"
fetch "bad_class"
"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("bad_class.braw"),
        r#"
kin Bad {
    dae init() { gie __missing__ }
}
"#,
    )
    .unwrap();

    let err = compile_with_source_path_for_unit_coverage(
        r#"
fetch "bad_class"
"#,
        &main_path,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Undefined variable"));
}

#[test]
fn llvm_codegen_import_parse_error_branch_is_exercised_for_unit_coverage() {
    let dir = tempdir().unwrap();
    let main_path = dir.path().join("main.braw");
    std::fs::write(
        &main_path,
        r#"
fetch "bad_parse"
"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("bad_parse.braw"), "ken a =\n").unwrap();

    let _ = compile_with_source_path_for_unit_coverage(
        r#"
fetch "bad_parse"
"#,
        &main_path,
    )
    .unwrap_err();
}

#[test]
fn llvm_codegen_import_duplicate_module_vars_reuse_existing_globals_for_unit_coverage() {
    let dir = tempdir().unwrap();
    let main_path = dir.path().join("main.braw");
    std::fs::write(
        &main_path,
        r#"
fetch "dup_vars"
"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("dup_vars.braw"),
        r#"
ken a = 1
ken a = 2
"#,
    )
    .unwrap();

    compile_with_source_path_for_unit_coverage(
        r#"
fetch "dup_vars"
"#,
        &main_path,
    )
    .unwrap();
}

#[test]
fn llvm_codegen_misc_error_propagation_branches_are_exercised_for_unit_coverage() {
    // Targets additional `?` error-propagation branches inside `compile_expr` without relying on
    // any runtime behavior.
    let cases = [
        // Range expression: start/end compile failures.
        r#"ken r = __missing__..3"#,
        r#"ken r = 1..__missing__"#,
        // Input expression: prompt compile failure.
        r#"ken s = speir __missing__"#,
        // Boxed assignment path: boxed var assignment RHS compile failure.
        r#"
dae outer() {
    ken x = 0
    dae inc() { x = __missing__ }
    inc()
}
outer()
"#,
        // Import resolution: exercise the `lib/*` stripped-path fallthrough.
        r#"fetch "lib/__coverage_missing_module""#,
    ];

    for src in cases {
        let program = parse(src).unwrap();
        let err = LLVMCompiler::new().compile_to_ir(&program).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Undefined variable") || msg.contains("Cannot find module to import"));
    }
}

#[test]
fn llvm_codegen_more_higher_order_builtins_are_exercised_for_unit_coverage() {
    // Exercise higher-order builtins and overload resolution in `compile_call` (aw/ony/hunt/tumble,
    // plus map/filter/reduce style helpers) to improve codegen instantiation depth.
    compile_to_ir_for_unit_coverage(
        r#"
ken xs = [1, 2, 3, 4]

blether gaun(xs, |x| x + 1)
blether sieve(xs, |x| x % 2 == 0)
blether reduce(xs, |a, b| a + b, 0)

blether tumble(|a, b| a + b, xs, 0)
blether tumble(xs, |a, b| a + b, 0)
blether tumble(xs, 0, |a, b| a + b)

blether aw(xs)
blether aw(xs, |x| x > 0)

blether ony([0, 0, 1])
blether ony(xs, |x| x == 2)

blether hunt("hello", "ll")
blether hunt([1, 2, 3], 2)
blether hunt([1, 2, 3], |x| x == 2)

blether find_index([10, 20, 30], |x| x == 20)

blether grup_up([1, 1, 2, 2], |x| x)
blether pairt_by(xs, |x| x % 2 == 0)

blether ilk([[1, 2], [3, 4]], |inner| sumaw(inner))

ken sum = 0
ilkane([10, 20, 30], |item, idx| {
    sum = sum + idx
})
ilkane({"a": 1, "b": 2}, |k, v| {
    sum = sum + v
})
blether sum
"#,
    );
}

#[test]
fn llvm_codegen_more_control_flow_and_object_constructs_are_exercised_for_unit_coverage() {
    // Exercise non-trivial constructs that are often only hit via integration sweeps
    // (dependency-crate instance), to improve instantiation depth in the unit-crate instance.
    compile_to_ir_for_unit_coverage(
        r#"
thing Pair { a, b }
ken p = Pair(1, 2)
blether p["a"]
blether p["b"]

kin Foo {
    dae init(a, b, c = 2) {
        masel.c = c
    }
    dae m(a, b, c = 2) {
        gie c
    }
}
ken f = Foo(1)
blether f.m(1)

hae_a_bash {
    hurl "boom"
} gin_it_gangs_wrang e {
    blether e
}

dae add3(a, b, c) { gie a + b + c }
ken xs = [1, 2]
blether add3(0, ...xs)

blether 5 |> |x| x + 1
blether 5 |> tae_string()
ken d = {"f": |x| x + 1}
blether 5 |> d["f"]

keek 1 {
    whan 1 -> { blether 1 }
}
blether 1
"#,
    );
}

#[test]
fn llvm_codegen_defaults_restore_boxed_callers_for_unit_coverage() {
    // Ensures default-evaluation temporarily binding param names does not lose boxed state from
    // the caller scope when names collide (restoring `boxed_vars` via the `was_boxed` path).
    compile_to_ir_for_unit_coverage(
        r#"
dae f(x = 1) { gie x }

kin C {
    dae init(x = 1) { masel.v = x }
    dae m(x = 1) { gie x }
}

kin D {
    dae init(x = 1) { masel.v = x }
    dae m(x = 1) { gie x }
}

dae choose(flag) {
    gin flag { gie C() } ither { gie D() }
}

dae outer() {
    ken x = 0
    dae inc() { x = x + 1 }
    inc()

    f()

    ken c = C()
    c.m()

    ken u = choose(aye)
    u.m()
}
outer()
"#,
    );
}

#[test]
fn llvm_codegen_for_ternary_slice_import_destructure_fstring_paths_are_exercised_for_unit_coverage() {
    // Covers a broad set of currently low-instantiation helpers in `codegen.rs`:
    // - fetch/import plumbing
    // - fer loops over list/string/range
    // - ternary expression
    // - list spread + index set + dynamic index + slice expr
    // - destructuring + f-strings + asserts
    compile_to_ir_for_unit_coverage(
        r#"
fetch "math" tae m

ken xs = [1, 2, 3, 4]
ken ys = [...xs, 5]
ken i = 1

xs[0] = 9
ken d = {"a": 1}
d["b"] = 2

blether xs[0]
blether xs[i]
blether xs[1:3]
blether "hello"[1:4]

ken flag = aye
blether gin flag than 1 ither 0

fer x in ys {
    blether x
}

fer c in "ab" {
    blether c
}

fer n in 1..=3 {
    blether n
}

mak_siccar 1 == 1

ken [a, b, ...rest] = xs
blether a
blether b
blether len(rest)

ken name = "Bob"
blether f"Hello {name}!"
"#,
    );
}

#[test]
fn llvm_codegen_import_tri_requires_alias_error_is_exercised_for_unit_coverage() {
    let program = parse(r#"fetch "tri""#).expect("parse program");
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("expected tri import to require an alias");
    assert!(err
        .to_string()
        .contains("tri import requires an alias"));
}

#[test]
fn llvm_codegen_import_tri_success_path_is_exercised_for_unit_coverage() {
    compile_to_ir_for_unit_coverage(
        r#"
fetch "tri" tae t
"#,
    );
}

#[test]
fn llvm_codegen_boxed_var_decl_initializer_error_is_exercised_for_unit_coverage() {
    let program = parse(
        r#"
dae outer() {
    ken x = missing
    dae inc() { x = x + 1 }
    inc()
}
outer()
"#,
    )
    .expect("parse program");
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("expected compile error from boxed var initializer");
    assert_eq!(
        std::mem::discriminant(&err),
        std::mem::discriminant(&crate::error::HaversError::CompileError(String::new()))
    );
}

#[test]
fn llvm_codegen_list_shadow_initializer_error_is_exercised_for_unit_coverage() {
    let program = parse(
        r#"
dae f() {
    ken xs = [missing]
}
f()
"#,
    )
    .expect("parse program");
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("expected compile error from list initializer element");
    assert_eq!(
        std::mem::discriminant(&err),
        std::mem::discriminant(&crate::error::HaversError::CompileError(String::new()))
    );
}

#[test]
fn llvm_codegen_block_stmt_error_propagates_for_unit_coverage() {
    let program = parse(
        r#"
dae f() {
    {
        blether missing
    }
}
f()
"#,
    )
    .expect("parse program");
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("expected compile error from invalid stmt in nested block");
    assert_eq!(
        std::mem::discriminant(&err),
        std::mem::discriminant(&crate::error::HaversError::CompileError(String::new()))
    );
}

#[test]
fn llvm_codegen_log_stmt_error_paths_are_exercised_for_unit_coverage() {
    for source in [
        r#"log_blether "msg", missing"#,
        r#"log_blether "msg", 1, missing"#,
        r#"log_blether missing"#,
    ] {
        let program = parse(source).expect("parse program");
        let err = LLVMCompiler::new()
            .compile_to_ir(&program)
            .expect_err("expected compile error from log stmt");
        assert_eq!(
            std::mem::discriminant(&err),
            std::mem::discriminant(&crate::error::HaversError::CompileError(String::new()))
        );
    }
}

#[test]
fn llvm_codegen_hurl_stmt_error_path_is_exercised_for_unit_coverage() {
    let program = parse(r#"hurl missing"#).expect("parse program");
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("expected compile error from hurl stmt");
    assert_eq!(
        std::mem::discriminant(&err),
        std::mem::discriminant(&crate::error::HaversError::CompileError(String::new()))
    );
}

#[test]
fn llvm_codegen_more_inline_builtins_and_class_paths_are_exercised_for_unit_coverage() {
    // Drives additional inline_* helpers (random/term helpers, list shuffles, predicate-based find),
    // plus class/method compilation paths that depend on nested captures.
    compile_to_ir_for_unit_coverage(
        r#"
ken bx = {
    ken y = 1
    gie y
}
blether bx

blether find([1, 2, 3], |x| x == 2)
blether sample([1, 2, 3], 2)
blether drap([1, 2, 3], 1)

blether ceilidh({"a": 1}, {"b": 2})
blether ceilidh([1, 2], [3, 4])
blether birl([1, 2, 3, 4], 1)
blether dram([1, 2, 3])

blether jammy(1, 10)
blether randfloat()
blether snooze(0)

blether term_width()
blether term_height()
blether get_key()

blether tae_int(1.2)
blether tae_float(1)
blether nae aye
blether 2 >= 1
blether 1 < 2

blether is_upper("A")
blether is_lower("a")
blether is_alpha("a")
blether is_digit("1")
blether is_alnum("1")

ken idx = 1
blether [10, 20, 30][idx]

kin Box {
    dae init(x) {
        masel.x = x
    }
    dae m(x) {
        dae inner() { gie masel.x + x }
        gie inner()
    }
}
ken b = Box(1)
blether b.m(2)

ken d = {"f": |x| x + 1}
blether d["f"](1)
"#,
    );
}

#[test]
fn llvm_codegen_current_function_none_branch_is_exercised_for_unit_coverage() {
    let context = Context::create();
    let codegen = crate::llvm::codegen::CodeGen::new(&context, "coverage_current_fn_none");
    let _ = codegen.coverage_current_function_none_branch();
}

#[test]
fn llvm_codegen_remaining_instantiation_hotspots_are_exercised_for_unit_coverage() {
    compile_to_ir_for_unit_coverage(
        r#"
blether tae_bool(1)
blether tae_float("1.2")

blether substr_between("a[b]c", "[", "]")

ilka([1, 2, 3], |x| {
    blether x
})

blether dict_has({"a": 1}, "a")
blether starts_with("hello", "he")
blether ends_with("hello", "lo")

ken set1 = make_creel([1, 2, 3])
blether is_in_creel(set1, 2)
toss_in(set1, 4)
heave_oot(set1, 2)
blether creel_tae_list(set1)

scrieve("instantiation.txt", "hi")
scrieve_append("instantiation.txt", "there")

blether regex_replace_first("aaaa", "a+", "b")
blether regex_split("a b c", "\\s+")

blether muckle([1, 2, 3])
blether muckle(1, 2)
blether dicht([1, 2, 3], 1)

keek 2 {
    whan 1..3 -> { blether 1 }
    whan x -> { blether x }
}

ken [a, ...mid, z] = [1, 2, 3, 4]
blether a
blether z
blether len(mid)

dae cond() {
    ken a = 1
    ken b = 2
    gin a == b { blether 0 }
    gin a != b { blether 1 }
}
cond()

ken gcap = 1
dae make_closure() {
    dae inner() { gie gcap }
    ken f = inner
    blether f()
}
make_closure()

ken b = bytes(4)
blether bytes_len(b)
blether bytes_get(b, 0)
blether bytes_slice(b, 0, 2)

kin C {
    dae init() {
        masel.cb = |x| x + 1
    }
}
ken c = C()
blether c.cb(1)
"#,
    );
}

#[cfg(unix)]
#[test]
fn llvm_codegen_import_read_to_string_error_is_mapped_for_unit_coverage() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.braw");
    std::fs::write(&source_path, "fetch \"secret\"").expect("write main");

    let import_path = dir.path().join("secret.braw");
    std::fs::write(&import_path, "ken x = 1").expect("write import");
    std::fs::set_permissions(&import_path, std::fs::Permissions::from_mode(0o000))
        .expect("chmod import");

    let source = std::fs::read_to_string(&source_path).expect("read main");
    let program = parse(&source).expect("parse main");

    let context = Context::create();
    let mut codegen = CodeGen::new(&context, "coverage_import_read_error");
    codegen.set_source_path(&source_path);

    let err = codegen
        .compile(&program)
        .expect_err("expected compile error from unreadable import");

    assert_eq!(
        std::mem::discriminant(&err),
        std::mem::discriminant(&crate::error::HaversError::CompileError(String::new()))
    );
}
