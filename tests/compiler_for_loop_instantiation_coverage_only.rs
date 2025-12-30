#![cfg(coverage)]

#[test]
fn compiler_for_loop_is_covered_in_dependency_instance() {
    let js = mdhavers::compiler::compile("fer i in 1..3 { blether i }\n").expect("compile");
    assert!(!js.trim().is_empty());
}
