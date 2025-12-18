#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{llvm::LLVMCompiler, parse};

fn compile_to_ir_ok(source: &str) -> String {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let ir = LLVMCompiler::new()
        .compile_to_ir(&program)
        .unwrap_or_else(|e| panic!("compile failed for:\n{source}\nerr={e:?}"));
    assert!(!ir.is_empty());
    ir
}

#[test]
fn llvm_codegen_exercises_string_list_helpers_slices_and_defaults_for_coverage() {
    // This is a coverage-driving “kitchen sink” program meant to hit large helper blocks in
    // `src/llvm/codegen.rs` that are otherwise rarely exercised by the test suite.
    let ir = compile_to_ir_ok(
        r#"
// Default-arg filling in compile_call
dae add(a, b = 2) { gie a + b }
blether add(40)

// Missing args without defaults (covers nil-fill logic at call sites)
dae pick_first(a, b) { gie a }
blether pick_first(123)

// Defaults present, but missing a non-default param (fills it with nil)
dae only_uses_default(a, b = 9) { gie b }
blether only_uses_default()

// Exercise sumaw inline helper
ken nums = [1, 2, 3, 4]
blether sumaw(nums)

// Exercise split/join helpers
ken s = "a,b,c"
ken parts = split(s, ",")
blether parts[0]
blether join(parts, "-")
blether jyne(parts, ":")

// Slice expressions (start/end/step variants)
blether nums[1:3]
blether nums[:2]
blether nums[2:]
blether nums[::2]
blether nums[0:4:2]

// to-string helper on multiple value kinds
blether tae_string(123)
blether tae_string(1.5)
blether tae_string(aye)
blether tae_string(naething)
blether tae_string(nums)
blether tae_string({"a": 1, "b": 2})

// Set/creel operations (hits update-in-place branches)
ken creel = make_creel(["a", "b"])
blether is_in_creel(creel, "a")
toss_in(creel, "c")
heave_oot(creel, "a")
blether creel_tae_list(creel)
blether empty_creel()

// Log level plumbing
blether get_log_level()
blether set_log_level(3)

// String predicate helpers
blether starts_with("hello", "he")
blether ends_with("hello", "lo")

// Bit shifting helpers
blether bit_shove_right(8, 1)

// Top-level boxed variable path: top-level var captured + mutated by nested function
ken g = 0
dae make_counter() {
    dae inc() {
        g = g + 1
        gie g
    }
    gie inc
}
ken c = make_counter()
blether c()
blether c()

// Instance field set/get paths
kin C {
    dae init() { masel.x = 1 }
    dae setx(v) { masel.x = v }
}
ken inst = C()
inst.setx(3)
blether inst.x

// Nested function capturing `masel` (covers capture_name == "masel" call-site logic)
kin MaselCap {
    dae init() { masel.x = 7 }
    dae go() {
        dae inner() { gie masel.x }
        gie inner()
    }
}
ken mc = MaselCap()
blether mc.go()
"#,
    );
    assert!(
        ir.contains("masel_cap"),
        "expected masel capture path to appear in IR"
    );
}
