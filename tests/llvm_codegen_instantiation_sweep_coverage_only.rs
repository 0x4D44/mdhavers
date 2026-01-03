#![cfg(all(coverage, feature = "llvm"))]

use mdhavers::{parse, LLVMCompiler};

#[test]
fn llvm_codegen_instantiations_are_exercised_in_dependency_crate_instance() {
    let source = r#"
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

blether abs(-42)
blether floor(3.2)
blether ceil(3.2)
blether round(3.7)
blether sqrt(16)

blether radians(180)
blether degrees(3.14159)

blether sumaw([1, 2, 3])
blether sumaw([1.0, 2.0, 3.5])

blether sin(0)
blether cos(0)
blether tan(0)
blether log(1)
blether log10(100)
blether exp(0)
blether pow(2, 3)
blether pooer(2, 3)
blether atan2(0, 1)

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
"#;

    let program = parse(source).unwrap();
    let _ = LLVMCompiler::new().compile_to_ir(&program).unwrap();
}
