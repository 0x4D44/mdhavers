#![cfg(coverage)]

use mdhavers::ast::{Expr, Literal, LogLevel, Program, Span, Stmt};
use mdhavers::compiler::{compile, Compiler};
use mdhavers::formatter::Formatter;
use mdhavers::parse;

fn lit_str(text: &str) -> Expr {
    Expr::Literal {
        value: Literal::String(text.to_string()),
        span: Span::new(1, 1),
    }
}

#[test]
fn compiler_exercises_additional_uncovered_constructs_for_coverage() {
    let source = r#"
kin Foo {
    dae init() {
        gie nil
    }
}

keek x {
    whan 1 -> {
        blether "one"
        blether "two"
    }
}

log_roar "roar"
log_holler "holler"
log_blether "blether"
log_mutter "mutter"
log_whisper "whisper"

hurl "boom"

ken a = 1.5
ken b = nil
ken c = 5 - 2
ken d = 6 / 3
ken e = 7 % 2
ken f = 1 == 1
ken g = 1 != 2
ken h = 1 <= 2
ken i = 2 >= 1

ken j = Foo()
obj.method()
ken k = arr[1:3]
ken l = arr[:3]
ken m = arr[1:3:2]

ken n = f"hi`there"

ken o = { ken x = 1
          gie x }
"#;

    let output = compile(source).unwrap();
    assert!(output.contains("new Foo("));
    assert!(output.contains("obj.method("));
    assert!(output.contains("arr.slice(1, 3)"));
    assert!(output.contains("arr.slice(0, 3)"));
    assert!(output.contains("__havers.slice(arr, 1, 3, 2)"));
    assert!(output.contains("`hi\\`there`"));
    assert!(output.contains("console.error(`[ROAR]"));
    assert!(output.contains("throw new Error(\"boom\")"));
}

#[test]
fn compiler_exercises_unreachable_or_hard_to_parse_paths_for_coverage() {
    let span = Span::new(1, 1);

    // Cover class-method "non-function" path (parser only produces function methods).
    let class_stmt = Stmt::Class {
        name: "Weird".to_string(),
        superclass: None,
        methods: vec![Stmt::VarDecl {
            name: "nope".to_string(),
            initializer: Some(Expr::Literal {
                value: Literal::Integer(1),
                span,
            }),
            span,
        }],
        span,
    };

    // Cover compile_stmt_inline non-block path via a TryCatch with non-block branches.
    // (Parser always produces blocks for try/catch bodies.)
    let try_catch_stmt = Stmt::TryCatch {
        try_block: Box::new(Stmt::Expression {
            expr: Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            span,
        }),
        error_name: "e".to_string(),
        catch_block: Box::new(Stmt::Expression {
            expr: Expr::Literal {
                value: Literal::Integer(2),
                span,
            },
            span,
        }),
        span,
    };

    // Cover log-level "Wheesht" (no AST produced from parsing source).
    let wheesht_log_stmt = Stmt::Log {
        level: LogLevel::Wheesht,
        message: lit_str("quiet"),
        extras: Vec::new(),
        span,
    };

    let program = Program::new(vec![class_stmt, try_catch_stmt, wheesht_log_stmt]);
    let mut compiler = Compiler::new();
    let output = compiler.compile(&program).unwrap();
    assert!(output.contains("class Weird"));
}

#[test]
fn formatter_exercises_uncovered_branches_for_coverage() {
    // Cover Formatter::default and the "empty file adds newline" path.
    let empty_program = Program::new(Vec::new());
    let mut formatter = Formatter::default();
    let out = formatter.format(&empty_program);
    assert_eq!(out, "\n");

    // Cover log keyword selection (including Wheesht) and hurl formatting.
    let log_program = Program::new(vec![
        Stmt::Log {
            level: LogLevel::Wheesht,
            message: lit_str("w"),
            extras: Vec::new(),
            span: Span::new(1, 1),
        },
        Stmt::Log {
            level: LogLevel::Roar,
            message: lit_str("r"),
            extras: Vec::new(),
            span: Span::new(1, 1),
        },
        Stmt::Log {
            level: LogLevel::Holler,
            message: lit_str("h"),
            extras: Vec::new(),
            span: Span::new(1, 1),
        },
        Stmt::Log {
            level: LogLevel::Blether,
            message: lit_str("b"),
            extras: Vec::new(),
            span: Span::new(1, 1),
        },
        Stmt::Log {
            level: LogLevel::Mutter,
            message: lit_str("m"),
            extras: Vec::new(),
            span: Span::new(1, 1),
        },
        Stmt::Log {
            level: LogLevel::Whisper,
            message: lit_str("t"),
            extras: Vec::new(),
            span: Span::new(1, 1),
        },
        Stmt::Hurl {
            message: lit_str("boom"),
            span: Span::new(1, 1),
        },
    ]);
    let mut formatter = Formatter::new();
    let out = formatter.format(&log_program);
    assert!(out.contains("log_wheesht"));
    assert!(out.contains("log_roar"));
    assert!(out.contains("log_holler"));
    assert!(out.contains("log_blether"));
    assert!(out.contains("log_mutter"));
    assert!(out.contains("log_whisper"));
    assert!(out.contains("hurl"));

    // Cover BlockExpr formatting + format_stmt_single branches (var/expr/return/break/continue/_).
    let block_expr_source = r#"
ken x = {
    ken a = 1
    ken b
    a + 2
    gie 3
    gie
    brak
    haud
    dae foo() { gie 1 }
}
"#;
    let program = parse(block_expr_source).unwrap();
    let mut formatter = Formatter::new();
    let out = formatter.format(&program);
    assert!(out.contains("ken a = 1"));
    assert!(out.contains("ken b"));
    assert!(out.contains("gie 3"));
    assert!(out.contains("\n    gie\n"));
    assert!(out.contains("brak"));
    assert!(out.contains("haud"));
    assert!(out.contains("\n    ...\n"));

    // Cover Pattern::Identifier formatting in match arms.
    let match_source = r#"
keek x {
    whan y -> blether "ok"
}
"#;
    let program = parse(match_source).unwrap();
    let mut formatter = Formatter::new();
    let out = formatter.format(&program);
    assert!(out.contains("whan y ->"));
}
