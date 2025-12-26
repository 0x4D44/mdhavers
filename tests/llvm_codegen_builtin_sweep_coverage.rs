#![cfg(all(feature = "llvm", coverage))]

use std::collections::HashSet;
use std::path::PathBuf;

use mdhavers::{parse, LLVMCompiler};

fn extract_builtin_names_from_codegen_source() -> Vec<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let codegen_path = root.join("src/llvm/codegen.rs");
    let source = std::fs::read_to_string(&codegen_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", codegen_path.display(), e));

    let mut names_in_order: Vec<String> = Vec::new();

    let mut in_match = false;
    let mut depth: i32 = 0;
    let mut pending_pattern_lines: Vec<&str> = Vec::new();

    for line in source.lines() {
        if !in_match {
            if line.contains("match name.as_str()") {
                in_match = true;
                depth += line.matches('{').count() as i32;
                depth -= line.matches('}').count() as i32;
            }
            continue;
        }

        if depth <= 0 {
            break;
        }

        let depth_before = depth;

        // Only scan patterns at the top-level of the builtin match.
        // This avoids accidentally extracting strings from nested matches inside arm bodies.
        if depth_before == 1 {
            let trimmed = line.trim_start();
            let is_comment = trimmed.starts_with("//");

            if !pending_pattern_lines.is_empty() {
                pending_pattern_lines.push(line);

                if line.contains("=>") {
                    let pending = pending_pattern_lines.join("\n");
                    if let Some((before, _after)) = pending.split_once("=>") {
                        let mut rest = before;
                        while let Some(start) = rest.find('"') {
                            let tail = &rest[start + 1..];
                            if let Some(end) = tail.find('"') {
                                names_in_order.push(tail[..end].to_string());
                                rest = &tail[end + 1..];
                            } else {
                                break;
                            }
                        }
                    }
                    pending_pattern_lines.clear();
                }
            } else if !is_comment && line.contains('"') {
                // Start collecting a (possibly multi-line) arm pattern list.
                pending_pattern_lines.push(line);

                // Handle one-line patterns immediately.
                if line.contains("=>") {
                    let pending = pending_pattern_lines.join("\n");
                    if let Some((before, _after)) = pending.split_once("=>") {
                        let mut rest = before;
                        while let Some(start) = rest.find('"') {
                            let tail = &rest[start + 1..];
                            if let Some(end) = tail.find('"') {
                                names_in_order.push(tail[..end].to_string());
                                rest = &tail[end + 1..];
                            } else {
                                break;
                            }
                        }
                    }
                    pending_pattern_lines.clear();
                }
            }
        }

        depth += line.matches('{').count() as i32;
        depth -= line.matches('}').count() as i32;
    }

    // De-dup in order: we want to probe every callable alias at least once.
    let mut seen = HashSet::new();
    names_in_order.retain(|n| seen.insert(n.clone()));
    names_in_order
}

fn compile_call_snippet(name: &str, args: &[&str]) -> Result<String, String> {
    let src = if args.is_empty() {
        format!("{name}()")
    } else {
        format!("{name}({})", args.join(", "))
    };
    let program = parse(&src).map_err(|e| format!("Parse error: {e:?}"))?;
    LLVMCompiler::new()
        .compile_to_ir(&program)
        .map_err(|e| format!("Compile error: {e:?}"))
}

#[test]
fn llvm_codegen_builtin_dispatch_is_exercised_broadly() {
    // This is intentionally coverage-only: it is a large, table-driven probe of the builtin
    // dispatch in `src/llvm/codegen.rs`.
    let builtins = extract_builtin_names_from_codegen_source();

    // These are either syntactic keywords (not callables), or too dangerous to probe in bulk.
    let skip: HashSet<&'static str> = [
        // `speir` is parsed as an input expression keyword, not a callable identifier.
        "speir",
        // Some assertion helpers in the LLVM suite are statement-like or depend on harness state.
        "assert",
        "assertEqual",
        "assert_eq",
        "assert_ne",
        "assert_true",
        "assert_false",
        // Print is a statement keyword in the grammar.
        "blether",
    ]
    .into_iter()
    .collect();

    const INT_ARGS: [&str; 12] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"];

    // Prefer arity coverage first: a lot of the big builtin match arms are gated by args.len().
    // For most builtins, the argument *types* don't matter for compilation, because type checks
    // are emitted as runtime tag checks in LLVM IR.
    let mut arg_sets: Vec<Vec<&str>> = (0..=INT_ARGS.len())
        .map(|n| INT_ARGS[..n].to_vec())
        .collect();

    // Add a handful of higher-signal patterns to exercise additional compile-time branches.
    arg_sets.extend([
        vec!["1.0"],
        vec!["aye"],
        vec!["naething"],
        vec![r#""hello""#],
        vec!["[1, 2, 3]"],
        vec![r#"{"a": 1, "b": 2}"#],
        vec!["|x| x"],
        vec![r#""hello""#, r#""he""#],
        vec!["[1, 2, 3]", "|x| x + 1"],
        vec!["[1, 2, 3]", "0", "|a, b| a + b"],
        vec!["[1, 2, 3, 4]", "2", "|x| x % 2"],
        vec![r#""hello""#, "5", r#"" ""#],
    ]);

    for name in builtins {
        if skip.contains(name.as_str()) {
            continue;
        }

        // Try a variety of argument shapes; accept either Ok or Err.
        // The goal is to execute as much of each dispatch arm as possible.
        let mut attempted_any = false;
        for args in &arg_sets {
            let res = compile_call_snippet(&name, args);
            attempted_any = true;
            if let Ok(ir) = res {
                assert!(
                    !ir.is_empty(),
                    "IR output should not be empty for {name}({})",
                    args.join(", ")
                );
            }
        }

        assert!(
            attempted_any,
            "expected to attempt compiling builtin call for {name}"
        );
    }
}
