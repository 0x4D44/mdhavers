#![cfg(coverage)]

use mdhavers::interpreter;

#[test]
fn interpreter_misc_public_helpers_are_exercised_for_instantiation_coverage() {
    interpreter::set_crash_handling(true);
    let _ = interpreter::is_crash_handling_enabled();
    interpreter::set_crash_handling(false);

    interpreter::set_global_log_level_raw(3);

    interpreter::set_stack_file("coverage.braw");
    interpreter::push_stack_frame("f", 1);
    interpreter::print_stack_trace();
    let trace = interpreter::get_stack_trace();
    if let Some(frame) = trace.first() {
        let _ = frame.to_string();
    }
    interpreter::pop_stack_frame();
    interpreter::clear_stack_trace();

    // Coverage-only helpers.
    interpreter::poison_shadow_stack_for_coverage();
    interpreter::exercise_interpreter_dir_instantiations_for_coverage();
}
