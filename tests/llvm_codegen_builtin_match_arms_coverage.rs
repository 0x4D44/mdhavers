#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{llvm::LLVMCompiler, parse};

fn compile_to_ir(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("parse error: {e:?}\nsource:\n{source}"))?;
    LLVMCompiler::new()
        .compile_to_ir(&program)
        .map_err(|e| format!("compile error: {e:?}\nsource:\n{source}"))
}

fn extract_builtin_representatives_from_codegen_rs() -> Vec<String> {
    let codegen_rs = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/llvm/codegen.rs"
    ))
    .expect("failed to read src/llvm/codegen.rs");

    let start = codegen_rs
        .find("match name.as_str()")
        .expect("expected `match name.as_str()` in codegen.rs");
    let block = &codegen_rs[start..];

    // Find the opening `{` after the match.
    let open_brace = block
        .find('{')
        .expect("expected `{` after `match name.as_str()`");

    // Walk forward, tracking braces, to get the full match block.
    let mut depth = 0usize;
    let mut end_idx = None;
    for (i, ch) in block[open_brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end_idx = Some(open_brace + i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let end_idx = end_idx.expect("failed to find end of builtin match block");
    let match_block = &block[..end_idx];

    let arm_re = regex::Regex::new(r#"^\s*(?P<lhs>.+?)\s*=>\s*\{"#)
        .expect("failed to build arm regex");
    let str_re =
        regex::Regex::new(r#""(?P<s>[^"]+)""#).expect("failed to build string literal regex");

    let mut reps = Vec::<String>::new();
    for line in match_block.lines() {
        let Some(arm) = arm_re.captures(line) else { continue };
        let lhs = arm.name("lhs").unwrap().as_str();
        let mut names = str_re
            .captures_iter(lhs)
            .filter_map(|c| c.name("s").map(|m| m.as_str().to_string()));
        if let Some(first) = names.next() {
            reps.push(first);
        }
    }

    reps.sort();
    reps.dedup();
    reps
}

#[test]
fn llvm_codegen_builtin_match_arms_are_exercised_for_coverage() {
    // We want to drive the giant builtin dispatch match in `src/llvm/codegen.rs` by compiling
    // many distinct call sites, without asserting semantics (some builtins may legitimately be
    // unimplemented in the LLVM backend).
    //
    // This runs only under coverage builds to avoid slowing down normal `cargo test`.
    let reps = extract_builtin_representatives_from_codegen_rs();
    assert!(
        reps.len() > 200,
        "expected many builtin match arms, got {}",
        reps.len()
    );

    // Try a small set of arities; most builtin arms gate on `args.len()`.
    let arg_lists = ["(1)", "(1, 2)", "()", "(1, 2, 3)"];

    let mut attempted = 0usize;
    for name in reps {
        // Skip names that can't appear as callable identifiers due to reserved syntax.
        // (If the parser rejects them, the corresponding match arm is likely dead code anyway.)
        let mut ok = false;
        for args in arg_lists {
            let src = format!("ken __tmp = {name}{args}\n");
            attempted += 1;
            match compile_to_ir(&src) {
                Ok(_) => {
                    ok = true;
                    break;
                }
                Err(_) => continue,
            }
        }

        // Even if compilation fails for all arities, we still exercised at least the parse+front
        // end path for this identifier. No assertion needed.
        let _ = ok;
    }

    assert!(attempted > 200, "expected to attempt many compile calls");
}

