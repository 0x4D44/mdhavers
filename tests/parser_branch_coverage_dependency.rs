#![cfg(coverage)]

use mdhavers::parser::{
    advance_at_end_for_coverage, destructure_eof_for_coverage, match_eof_for_coverage, parse,
    process_escapes_for_coverage,
};

#[test]
fn parser_dependency_branch_matrix_for_coverage() {
    let _ = parse("dae f() { gie }\n");
    let _ = parse("dae f() { gie 1 }\n");
    let _ = parse("gie");
    let _ = parse("ken [a] = [1]\n");
    let _ = parse("ken [a");
    let _ = parse("keek x { }\n");
    let _ = parse("keek x { 1 -> blether 1\n");
    let _ = parse("ken x = { ken y = 1\n");

    let _ = process_escapes_for_coverage("\\x41");
    let _ = process_escapes_for_coverage("\\xZZ");
    let _ = process_escapes_for_coverage("\\x");

    advance_at_end_for_coverage();
    destructure_eof_for_coverage();
    match_eof_for_coverage();
}
