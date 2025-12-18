#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, LLVMCompiler};

fn compile_to_ir_ok(source: &str) {
    let program = parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let ir = LLVMCompiler::new()
        .compile_to_ir(&program)
        .unwrap_or_else(|e| panic!("compile failed for:\n{source}\nerr={e:?}"));
    assert!(!ir.is_empty());
}

#[test]
fn llvm_codegen_lambda_freevar_analysis_exercises_more_stmt_and_expr_kinds() {
    // This is intentionally compile-only. The goal is to force the LLVM codegen's
    // lambda/free-variable analysis to walk a wide variety of AST node kinds.
    let src = r#"
dae sum2(a, b) { gie a + b }

kin Box {
    dae init(v = 0) { masel.v = v }
}

ken outer = 100
ken xs = [1, 2, 3]
ken box = Box()

ken f = |p| {
    ken local = p + outer

    # Expr::Get / Expr::Set
    box.v = local
    ken got = box.v

    # Expr::Dict / Expr::Index / Expr::IndexSet
    ken d = {"k": got}
    d["k"] = got + 1
    ken read = d["k"]

    # Expr::Ternary / Expr::Grouping
    ken pick = gin read > 0 than (read) ither (0)

    # Expr::Slice / Expr::Range / Expr::Spread / Expr::Pipe
    ken s1 = xs[1:3]
    ken s2 = xs[:2:1]
    ken s3 = xs[1:]
    ken s4 = xs[:]
    ken s5 = xs[::2]
    ken s6 = xs[::-1]
    ken idx = xs[0]
    ken r = 1..4
    ken g = |a, b| { gie a + b + pick }
    ken out = sum2(...s1) |> tae_int
    blether g(out, pick)

    # Expr::Input / Expr::FString
    ken prompt = speir(f"ignored {out}")

    # Stmt::TryCatch / Stmt::Hurl / Stmt::Log
    hae_a_bash {
        log_whisper f"boom {prompt}"
        hurl "boom"
    } gin_it_gangs_wrang e {
        log_roar e
    }

    # Stmt::Assert (mak_siccar) + message
    mak_siccar out > 0, f"ok {out}"

    # Stmt::If with else branch
    gin out > 0 {
        outer = outer + 1
    } ither {
        outer = outer + 2
    }

    # Stmt::Match (keek) including range pattern
    keek out {
        whan 0..10 -> { blether "small" }
        whan _ -> { blether "nonzero" }
    }

    # Stmt::Destructure (with ignore + rest)
    ken [a, ...rest] = [1, 2, 3, 4]
    ken [_, first, ...rest2] = [0, 1, 2, 3]
    blether a
    blether first
    blether len(rest)

    # Stmt::While + Stmt::Continue
    ken i = 0
    whiles i < 2 {
        i = i + 1
        haud
    }

    # Stmt::For + Stmt::Break
    fer x in [1, 2, 3] {
        gin x == 2 { brak }
    }

    # Nested function statement (doesn't recurse into body in free-var scan)
    dae inner() { gie out + outer }

    # Expr::Assign (assignment to captured)
    outer = outer + 1

    gie out
}

blether f(5)
"#;

    compile_to_ir_ok(src);
}
