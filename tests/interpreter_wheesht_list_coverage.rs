#![cfg(coverage)]

#[test]
fn wheesht_list_branch_is_covered_for_instantiation_coverage() {
    let (_val, output) = mdhavers::run_with_output(
        r#"
        blether wheesht([0, 1, "", "x", [], [2], naething, aye, nae])
        "#,
    )
    .unwrap();

    assert_eq!(output, vec!["[1, x, [2], aye]"]);
}

