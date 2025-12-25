use mdhavers::run;

#[test]
fn interpreter_statement_error_paths_are_covered() {
    // If: error in condition evaluation.
    assert!(
        run(
            r#"
gin nope {
    blether "unreachable"
}
"#,
        )
        .is_err()
    );

    // While: error in condition evaluation.
    assert!(
        run(
            r#"
whiles nope {
    blether "unreachable"
}
"#,
        )
        .is_err()
    );

    // While: error inside body (propagates via `execute_stmt_with_control(body)?`).
    assert!(
        run(
            r#"
whiles aye {
    1 / 0
}
"#,
        )
        .is_err()
    );

    // For: error in iterable evaluation.
    assert!(
        run(
            r#"
fer i in nope {
    blether i
}
"#,
        )
        .is_err()
    );

    // For: error inside body (propagates via `execute_stmt_with_control(body)?`).
    assert!(
        run(
            r#"
fer i in 0..1 {
    1 / 0
}
"#,
        )
        .is_err()
    );

    // Return: error in return expression evaluation.
    assert!(
        run(
            r#"
dae foo() {
    gie nope
}
foo()
"#,
        )
        .is_err()
    );

    // Print: error in value evaluation.
    assert!(run("blether nope\n").is_err());

    // Match: error in match value evaluation.
    assert!(
        run(
            r#"
keek nope {
    whan 1 -> 1
}
"#,
        )
        .is_err()
    );

    // Assert: error in condition evaluation.
    assert!(run("mak_siccar nope\n").is_err());

    // Assert: error in optional message evaluation.
    assert!(run("mak_siccar nae, nope\n").is_err());

    // Hurl: error in message evaluation.
    assert!(run("hurl nope\n").is_err());
}

