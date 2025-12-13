//! Comprehensive LLVM backend tests
//!
//! Tests for all major features of the LLVM code generator to ensure
//! correctness and catch regressions.

#![cfg(feature = "llvm")]

use std::process::Command;

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

/// Helper to compile source code and run the resulting executable
fn compile_and_run(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;

    let dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let exe_path = dir.path().join("test_exe");

    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_native(&program, &exe_path, 2)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Executable failed with exit code: {:?}, stderr: {}",
            output.status.code(),
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Helper that expects compilation/execution to succeed
fn run(source: &str) -> String {
    compile_and_run(source).expect("Should compile and run successfully")
}

// ============================================================================
// ARITHMETIC OPERATIONS
// ============================================================================

mod arithmetic {
    use super::*;

    #[test]
    fn test_addition() {
        assert_eq!(run("blether 1 + 2").trim(), "3");
        assert_eq!(run("blether 100 + 200").trim(), "300");
        assert_eq!(run("blether -5 + 10").trim(), "5");
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(run("blether 10 - 3").trim(), "7");
        assert_eq!(run("blether 5 - 10").trim(), "-5");
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(run("blether 6 * 7").trim(), "42");
        assert_eq!(run("blether -3 * 4").trim(), "-12");
    }

    #[test]
    fn test_division() {
        assert_eq!(run("blether 20 / 4").trim(), "5");
        assert_eq!(run("blether 7 / 2").trim(), "3");
    }

    #[test]
    fn test_modulo() {
        assert_eq!(run("blether 17 % 5").trim(), "2");
        assert_eq!(run("blether 10 % 3").trim(), "1");
    }

    #[test]
    fn test_float_arithmetic() {
        assert_eq!(run("blether 3.14 + 2.86").trim(), "6");
        assert_eq!(run("blether 10.5 - 0.5").trim(), "10");
        assert_eq!(run("blether 2.5 * 4.0").trim(), "10");
        assert_eq!(run("blether 7.5 / 2.5").trim(), "3");
    }

    #[test]
    fn test_mixed_int_float() {
        assert_eq!(run("blether 5 + 2.5").trim(), "7.5");
        assert_eq!(run("blether 10.0 - 3").trim(), "7");
    }

    #[test]
    fn test_complex_expression() {
        assert_eq!(run("blether (2 + 3) * 4").trim(), "20");
        assert_eq!(run("blether 2 + 3 * 4").trim(), "14");
        assert_eq!(run("blether (10 - 2) / (4 - 2)").trim(), "4");
    }

    #[test]
    fn test_negation() {
        assert_eq!(run("ken x = 5\nblether -x").trim(), "-5");
        assert_eq!(run("blether -(3 + 4)").trim(), "-7");
    }
}

// ============================================================================
// COMPARISON OPERATIONS
// ============================================================================

mod comparison {
    use super::*;

    #[test]
    fn test_equal() {
        assert_eq!(run("blether 5 == 5").trim(), "aye");
        assert_eq!(run("blether 5 == 6").trim(), "nae");
        assert_eq!(run(r#"blether "hello" == "hello""#).trim(), "aye");
        assert_eq!(run(r#"blether "hello" == "world""#).trim(), "nae");
    }

    #[test]
    fn test_not_equal() {
        assert_eq!(run("blether 5 != 6").trim(), "aye");
        assert_eq!(run("blether 5 != 5").trim(), "nae");
    }

    #[test]
    fn test_less_than() {
        assert_eq!(run("blether 3 < 5").trim(), "aye");
        assert_eq!(run("blether 5 < 3").trim(), "nae");
        assert_eq!(run("blether 5 < 5").trim(), "nae");
    }

    #[test]
    fn test_less_equal() {
        assert_eq!(run("blether 3 <= 5").trim(), "aye");
        assert_eq!(run("blether 5 <= 5").trim(), "aye");
        assert_eq!(run("blether 6 <= 5").trim(), "nae");
    }

    #[test]
    fn test_greater_than() {
        assert_eq!(run("blether 5 > 3").trim(), "aye");
        assert_eq!(run("blether 3 > 5").trim(), "nae");
        assert_eq!(run("blether 5 > 5").trim(), "nae");
    }

    #[test]
    fn test_greater_equal() {
        assert_eq!(run("blether 5 >= 3").trim(), "aye");
        assert_eq!(run("blether 5 >= 5").trim(), "aye");
        assert_eq!(run("blether 3 >= 5").trim(), "nae");
    }
}

// ============================================================================
// LOGICAL OPERATIONS
// ============================================================================

mod logical {
    use super::*;

    #[test]
    fn test_and() {
        assert_eq!(run("blether aye an aye").trim(), "aye");
        assert_eq!(run("blether aye an nae").trim(), "nae");
        assert_eq!(run("blether nae an aye").trim(), "nae");
        assert_eq!(run("blether nae an nae").trim(), "nae");
    }

    #[test]
    fn test_or() {
        assert_eq!(run("blether aye or aye").trim(), "aye");
        assert_eq!(run("blether aye or nae").trim(), "aye");
        assert_eq!(run("blether nae or aye").trim(), "aye");
        assert_eq!(run("blether nae or nae").trim(), "nae");
    }

    #[test]
    fn test_not() {
        // nae (not) requires parentheses for the operand
        assert_eq!(run("blether nae(aye)").trim(), "nae");
        assert_eq!(run("blether nae(nae)").trim(), "aye");
    }

    #[test]
    fn test_short_circuit_and() {
        // If first is false, second shouldn't be evaluated
        let code = r#"
            ken called = nae
            dae side_effect() {
                called = aye
                gie aye
            }
            ken result = nae an side_effect()
            blether called
        "#;
        assert_eq!(run(code).trim(), "nae");
    }

    #[test]
    fn test_short_circuit_or() {
        // If first is true, second shouldn't be evaluated
        let code = r#"
            ken called = nae
            dae side_effect() {
                called = aye
                gie aye
            }
            ken result = aye or side_effect()
            blether called
        "#;
        assert_eq!(run(code).trim(), "nae");
    }
}

// ============================================================================
// VARIABLES
// ============================================================================

mod variables {
    use super::*;

    #[test]
    fn test_variable_declaration() {
        assert_eq!(run("ken x = 42\nblether x").trim(), "42");
    }

    #[test]
    fn test_variable_assignment() {
        let code = r#"
            ken x = 10
            x = 20
            blether x
        "#;
        assert_eq!(run(code).trim(), "20");
    }

    #[test]
    fn test_multiple_variables() {
        let code = r#"
            ken a = 1
            ken b = 2
            ken c = 3
            blether a + b + c
        "#;
        assert_eq!(run(code).trim(), "6");
    }

    #[test]
    fn test_variable_shadowing() {
        let code = r#"
            ken x = 10
            {
                ken x = 20
                blether x
            }
        "#;
        assert_eq!(run(code).trim(), "20");
    }
}

// ============================================================================
// STRINGS
// ============================================================================

mod strings {
    use super::*;

    #[test]
    fn test_string_literal() {
        assert_eq!(run(r#"blether "Hello, World!""#).trim(), "Hello, World!");
    }

    #[test]
    fn test_string_concatenation() {
        assert_eq!(run(r#"blether "Hello, " + "World!""#).trim(), "Hello, World!");
    }

    #[test]
    fn test_string_escape_sequences() {
        assert_eq!(run(r#"blether "line1\nline2""#).trim(), "line1\nline2");
        assert_eq!(run(r#"blether "tab\there""#).trim(), "tab\there");
    }

    #[test]
    fn test_string_length() {
        assert_eq!(run(r#"blether len("hello")"#).trim(), "5");
        assert_eq!(run(r#"blether len("")"#).trim(), "0");
    }

    #[test]
    fn test_string_upper_lower() {
        assert_eq!(run(r#"blether upper("hello")"#).trim(), "HELLO");
        assert_eq!(run(r#"blether lower("HELLO")"#).trim(), "hello");
    }

    #[test]
    fn test_fstring() {
        let code = r#"
            ken name = "World"
            ken num = 42
            blether f"Hello, {name}! The answer is {num}."
        "#;
        assert_eq!(run(code).trim(), "Hello, World! The answer is 42.");
    }

    #[test]
    fn test_fstring_expression() {
        assert_eq!(run(r#"blether f"2 + 2 = {2 + 2}""#).trim(), "2 + 2 = 4");
    }

    #[test]
    fn test_string_contains() {
        assert_eq!(run(r#"blether contains("hello world", "world")"#).trim(), "aye");
        assert_eq!(run(r#"blether contains("hello world", "foo")"#).trim(), "nae");
    }

    #[test]
    fn test_string_starts_ends() {
        assert_eq!(run(r#"blether starts_wi("hello", "hel")"#).trim(), "aye");
        assert_eq!(run(r#"blether ends_wi("hello", "llo")"#).trim(), "aye");
    }

    #[test]
    fn test_string_split() {
        let code = r#"
            ken parts = split("a,b,c", ",")
            blether len(parts)
        "#;
        assert_eq!(run(code).trim(), "3");
    }

    #[test]
    fn test_string_join() {
        let code = r#"
            ken parts = ["a", "b", "c"]
            blether join(parts, "-")
        "#;
        assert_eq!(run(code).trim(), "a-b-c");
    }

    #[test]
    fn test_string_replace() {
        assert_eq!(run(r#"blether replace("hello", "l", "L")"#).trim(), "heLLo");
    }

    #[test]
    fn test_string_chars() {
        let code = r#"
            ken c = chars("abc")
            blether len(c)
        "#;
        assert_eq!(run(code).trim(), "3");
    }

    #[test]
    fn test_chr_ord() {
        assert_eq!(run("blether ord(\"A\")").trim(), "65");
        assert_eq!(run("blether chr(65)").trim(), "A");
    }
}

// ============================================================================
// LISTS
// ============================================================================

mod lists {
    use super::*;

    #[test]
    fn test_list_literal() {
        assert_eq!(run("blether [1, 2, 3]").trim(), "[1, 2, 3]");
    }

    #[test]
    fn test_empty_list() {
        assert_eq!(run("blether []").trim(), "[]");
    }

    #[test]
    fn test_list_index() {
        assert_eq!(run("blether [10, 20, 30][1]").trim(), "20");
    }

    #[test]
    fn test_list_negative_index() {
        assert_eq!(run("blether [10, 20, 30][-1]").trim(), "30");
    }

    #[test]
    fn test_list_length() {
        assert_eq!(run("blether len([1, 2, 3, 4, 5])").trim(), "5");
    }

    #[test]
    fn test_list_push() {
        let code = r#"
            ken list = [1, 2]
            shove(list, 3)
            blether list
        "#;
        assert_eq!(run(code).trim(), "[1, 2, 3]");
    }

    #[test]
    fn test_list_pop() {
        let code = r#"
            ken list = [1, 2, 3]
            ken last = yank(list)
            blether last
        "#;
        assert_eq!(run(code).trim(), "3");
    }

    #[test]
    fn test_list_assignment() {
        let code = r#"
            ken list = [1, 2, 3]
            list[1] = 20
            blether list
        "#;
        assert_eq!(run(code).trim(), "[1, 20, 3]");
    }

    #[test]
    fn test_list_heid_tail() {
        assert_eq!(run("blether heid([1, 2, 3])").trim(), "1");
        assert_eq!(run("blether tail([1, 2, 3])").trim(), "[2, 3]");
    }

    #[test]
    fn test_list_reverse() {
        assert_eq!(run("blether reverse([1, 2, 3])").trim(), "[3, 2, 1]");
    }

    #[test]
    fn test_list_sort() {
        assert_eq!(run("blether sort([3, 1, 2])").trim(), "[1, 2, 3]");
    }

    #[test]
    fn test_list_contains() {
        assert_eq!(run("blether contains([1, 2, 3], 2)").trim(), "aye");
        assert_eq!(run("blether contains([1, 2, 3], 5)").trim(), "nae");
    }

    #[test]
    fn test_list_index_of() {
        assert_eq!(run("blether index_of([10, 20, 30], 20)").trim(), "1");
        assert_eq!(run("blether index_of([10, 20, 30], 99)").trim(), "-1");
    }

    #[test]
    fn test_list_slice() {
        assert_eq!(run("blether [1, 2, 3, 4, 5][1:4]").trim(), "[2, 3, 4]");
        assert_eq!(run("blether [1, 2, 3, 4, 5][:3]").trim(), "[1, 2, 3]");
        assert_eq!(run("blether [1, 2, 3, 4, 5][2:]").trim(), "[3, 4, 5]");
    }

    #[test]
    fn test_list_uniq() {
        assert_eq!(run("blether uniq([1, 2, 2, 3, 3, 3])").trim(), "[1, 2, 3]");
    }

    #[test]
    fn test_list_sum() {
        assert_eq!(run("blether sumaw([1, 2, 3, 4])").trim(), "10");
    }

    #[test]
    fn test_list_min_max() {
        assert_eq!(run("blether min([3, 1, 4, 1, 5])").trim(), "1");
        assert_eq!(run("blether max([3, 1, 4, 1, 5])").trim(), "5");
    }
}

// ============================================================================
// DICTIONARIES
// ============================================================================

mod dicts {
    use super::*;

    #[test]
    fn test_dict_literal() {
        let code = r#"
            ken d = {"a": 1, "b": 2}
            blether d["a"]
        "#;
        assert_eq!(run(code).trim(), "1");
    }

    #[test]
    fn test_empty_dict() {
        assert_eq!(run("blether {}").trim(), "{}");
    }

    #[test]
    fn test_dict_assignment() {
        let code = r#"
            ken d = {"a": 1}
            d["b"] = 2
            blether d["b"]
        "#;
        assert_eq!(run(code).trim(), "2");
    }

    #[test]
    fn test_dict_keys() {
        let code = r#"
            ken d = {"x": 1, "y": 2}
            blether len(keys(d))
        "#;
        assert_eq!(run(code).trim(), "2");
    }

    #[test]
    fn test_dict_values() {
        let code = r#"
            ken d = {"x": 10, "y": 20}
            ken v = values(d)
            blether len(v)
        "#;
        assert_eq!(run(code).trim(), "2");
    }

    #[test]
    fn test_dict_contains() {
        let code = r#"
            ken d = {"a": 1, "b": 2}
            blether contains(d, "a")
        "#;
        assert_eq!(run(code).trim(), "aye");
    }
}

// ============================================================================
// CONTROL FLOW
// ============================================================================

mod control_flow {
    use super::*;

    #[test]
    fn test_if_true() {
        let code = r#"
            gin aye {
                blether "yes"
            }
        "#;
        assert_eq!(run(code).trim(), "yes");
    }

    #[test]
    fn test_if_false() {
        // Test if with false condition using boolean literal
        let code = r#"
            gin 1 == 2 {
                blether "yes"
            } ither {
                blether "no"
            }
        "#;
        assert_eq!(run(code).trim(), "no");
    }

    #[test]
    fn test_if_elif() {
        let code = r#"
            ken x = 2
            gin x == 1 {
                blether "one"
            } ither gin x == 2 {
                blether "two"
            } ither {
                blether "other"
            }
        "#;
        assert_eq!(run(code).trim(), "two");
    }

    #[test]
    fn test_while_loop() {
        let code = r#"
            ken i = 0
            ken sum = 0
            whiles i < 5 {
                sum = sum + i
                i = i + 1
            }
            blether sum
        "#;
        assert_eq!(run(code).trim(), "10");
    }

    #[test]
    fn test_while_break() {
        // Use brak (not brek) for break
        let code = r#"
            ken i = 0
            whiles aye {
                gin i >= 3 {
                    brak
                }
                i = i + 1
            }
            blether i
        "#;
        assert_eq!(run(code).trim(), "3");
    }

    #[test]
    fn test_while_continue() {
        // Use haud (hold on) for continue
        let code = r#"
            ken i = 0
            ken sum = 0
            whiles i < 5 {
                i = i + 1
                gin i == 3 {
                    haud
                }
                sum = sum + i
            }
            blether sum
        "#;
        // 1 + 2 + 4 + 5 = 12 (skips 3)
        assert_eq!(run(code).trim(), "12");
    }

    #[test]
    fn test_for_range() {
        let code = r#"
            ken sum = 0
            fer i in 0..5 {
                sum = sum + i
            }
            blether sum
        "#;
        assert_eq!(run(code).trim(), "10");
    }

    #[test]
    fn test_for_list() {
        let code = r#"
            ken sum = 0
            fer x in [1, 2, 3] {
                sum = sum + x
            }
            blether sum
        "#;
        assert_eq!(run(code).trim(), "6");
    }

    #[test]
    fn test_ternary() {
        // Use gin/than/ither for ternary expressions in Scots
        assert_eq!(run("blether gin 5 > 3 than \"yes\" ither \"no\"").trim(), "yes");
        assert_eq!(run("blether gin 2 > 3 than \"yes\" ither \"no\"").trim(), "no");
    }

    #[test]
    fn test_match_literal() {
        // Use keek/whan (peek/when) for pattern matching with -> arrows
        let code = r#"
            ken x = 2
            keek x {
                whan 1 -> { blether "one" }
                whan 2 -> { blether "two" }
                whan _ -> { blether "other" }
            }
        "#;
        assert_eq!(run(code).trim(), "two");
    }

    #[test]
    fn test_match_wildcard() {
        let code = r#"
            ken x = 99
            keek x {
                whan 1 -> { blether "one" }
                whan _ -> { blether "other" }
            }
        "#;
        assert_eq!(run(code).trim(), "other");
    }
}

// ============================================================================
// FUNCTIONS
// ============================================================================

mod functions {
    use super::*;

    #[test]
    fn test_function_no_params() {
        let code = r#"
            dae greet() {
                gie "Hello"
            }
            blether greet()
        "#;
        assert_eq!(run(code).trim(), "Hello");
    }

    #[test]
    fn test_function_with_params() {
        let code = r#"
            dae add(a, b) {
                gie a + b
            }
            blether add(3, 4)
        "#;
        assert_eq!(run(code).trim(), "7");
    }

    #[test]
    fn test_function_recursion() {
        let code = r#"
            dae factorial(n) {
                gin n <= 1 {
                    gie 1
                }
                gie n * factorial(n - 1)
            }
            blether factorial(5)
        "#;
        assert_eq!(run(code).trim(), "120");
    }

    #[test]
    fn test_function_fibonacci() {
        let code = r#"
            dae fib(n) {
                gin n <= 1 {
                    gie n
                }
                gie fib(n - 1) + fib(n - 2)
            }
            blether fib(10)
        "#;
        assert_eq!(run(code).trim(), "55");
    }

    #[test]
    fn test_function_multiple_calls() {
        let code = r#"
            dae double(x) {
                gie x * 2
            }
            blether double(double(double(2)))
        "#;
        assert_eq!(run(code).trim(), "16");
    }

    #[test]
    fn test_function_early_return() {
        let code = r#"
            dae check(x) {
                gin x < 0 {
                    gie "negative"
                }
                gie "non-negative"
            }
            blether check(-5)
        "#;
        assert_eq!(run(code).trim(), "negative");
    }

    #[test]
    fn test_lambda() {
        let code = r#"
            ken double = |x| x * 2
            blether double(5)
        "#;
        assert_eq!(run(code).trim(), "10");
    }

    #[test]
    fn test_lambda_multiline() {
        let code = r#"
            ken calc = |a, b| {
                ken sum = a + b
                gie sum * 2
            }
            blether calc(3, 4)
        "#;
        assert_eq!(run(code).trim(), "14");
    }

    #[test]
    fn test_higher_order_function() {
        let code = r#"
            dae apply(f, x) {
                gie f(x)
            }
            ken triple = |x| x * 3
            blether apply(triple, 7)
        "#;
        assert_eq!(run(code).trim(), "21");
    }
}

// ============================================================================
// CLASSES
// ============================================================================

mod classes {
    use super::*;

    #[test]
    fn test_class_basic() {
        let code = r#"
            kin Point {
                dae init(x, y) {
                    masel.x = x
                    masel.y = y
                }
            }
            ken p = Point(3, 4)
            blether p.x
        "#;
        assert_eq!(run(code).trim(), "3");
    }

    #[test]
    fn test_class_method() {
        let code = r#"
            kin Counter {
                dae init() {
                    masel.count = 0
                }
                dae increment() {
                    masel.count = masel.count + 1
                }
                dae get() {
                    gie masel.count
                }
            }
            ken c = Counter()
            c.increment()
            c.increment()
            c.increment()
            blether c.get()
        "#;
        assert_eq!(run(code).trim(), "3");
    }

    #[test]
    fn test_class_method_with_params() {
        let code = r#"
            kin Calculator {
                dae init(value) {
                    masel.value = value
                }
                dae add(n) {
                    masel.value = masel.value + n
                    gie masel
                }
                dae result() {
                    gie masel.value
                }
            }
            ken calc = Calculator(10)
            calc.add(5)
            calc.add(3)
            blether calc.result()
        "#;
        assert_eq!(run(code).trim(), "18");
    }
}

// ============================================================================
// MATH FUNCTIONS
// ============================================================================

mod math {
    use super::*;

    #[test]
    fn test_abs() {
        assert_eq!(run("blether abs(-5)").trim(), "5");
        assert_eq!(run("blether abs(5)").trim(), "5");
    }

    #[test]
    fn test_floor_ceil_round() {
        assert_eq!(run("blether floor(3.7)").trim(), "3");
        assert_eq!(run("blether ceil(3.2)").trim(), "4");
        assert_eq!(run("blether round(3.5)").trim(), "4");
        assert_eq!(run("blether round(3.4)").trim(), "3");
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(run("blether sqrt(16)").trim(), "4");
        // Float precision may vary, check prefix
        let result = run("blether sqrt(2)");
        assert!(
            result.trim().starts_with("1.41421"),
            "Expected sqrt(2) to start with 1.41421, got {}",
            result
        );
    }

    #[test]
    fn test_pow() {
        assert_eq!(run("blether pow(2, 10)").trim(), "1024");
        assert_eq!(run("blether pow(3, 3)").trim(), "27");
    }

    #[test]
    fn test_min_max_numbers() {
        assert_eq!(run("blether min(3, 7)").trim(), "3");
        assert_eq!(run("blether max(3, 7)").trim(), "7");
    }

    #[test]
    fn test_clamp() {
        assert_eq!(run("blether clamp(5, 0, 10)").trim(), "5");
        assert_eq!(run("blether clamp(-5, 0, 10)").trim(), "0");
        assert_eq!(run("blether clamp(15, 0, 10)").trim(), "10");
    }
}

// ============================================================================
// TYPE CONVERSIONS
// ============================================================================

mod conversions {
    use super::*;

    #[test]
    fn test_to_string() {
        assert_eq!(run("blether tae_string(42)").trim(), "42");
        assert_eq!(run("blether tae_string(3.14)").trim(), "3.14");
        assert_eq!(run("blether tae_string(aye)").trim(), "aye");
    }

    #[test]
    fn test_to_int() {
        assert_eq!(run(r#"blether tae_int("42")"#).trim(), "42");
        assert_eq!(run("blether tae_int(3.9)").trim(), "3");
    }

    #[test]
    fn test_to_float() {
        assert_eq!(run(r#"blether tae_float("3.14")"#).trim(), "3.14");
        assert_eq!(run("blether tae_float(42)").trim(), "42");
    }

    #[test]
    fn test_type_of() {
        assert_eq!(run("blether whit_kind(42)").trim(), "int");
        assert_eq!(run("blether whit_kind(3.14)").trim(), "float");
        assert_eq!(run(r#"blether whit_kind("hello")"#).trim(), "string");
        assert_eq!(run("blether whit_kind(aye)").trim(), "bool");
        assert_eq!(run("blether whit_kind([1,2,3])").trim(), "list");
        assert_eq!(run("blether whit_kind({})").trim(), "dict");
        assert_eq!(run("blether whit_kind(naething)").trim(), "nil");
    }
}

// ============================================================================
// FUNCTIONAL OPERATIONS
// ============================================================================

mod functional {
    use super::*;

    #[test]
    fn test_ilk_map() {
        let code = r#"
            ken nums = [1, 2, 3]
            ken doubled = ilk(nums, |x| x * 2)
            blether doubled
        "#;
        assert_eq!(run(code).trim(), "[2, 4, 6]");
    }

    #[test]
    fn test_sieve_filter() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5, 6]
            ken evens = sieve(nums, |x| x % 2 == 0)
            blether evens
        "#;
        assert_eq!(run(code).trim(), "[2, 4, 6]");
    }

    #[test]
    fn test_tumble_reduce() {
        let code = r#"
            ken nums = [1, 2, 3, 4]
            ken sum = tumble(nums, 0, |acc, x| acc + x)
            blether sum
        "#;
        assert_eq!(run(code).trim(), "10");
    }

    #[test]
    fn test_ony_any() {
        let code = r#"
            ken nums = [1, 2, 3, 4]
            blether ony(nums, |x| x > 3)
        "#;
        assert_eq!(run(code).trim(), "aye");
    }

    #[test]
    fn test_aw_all() {
        let code = r#"
            ken nums = [2, 4, 6, 8]
            blether aw(nums, |x| x % 2 == 0)
        "#;
        assert_eq!(run(code).trim(), "aye");
    }

    #[test]
    fn test_hunt_find() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5]
            ken found = hunt(nums, |x| x > 3)
            blether found
        "#;
        assert_eq!(run(code).trim(), "4");
    }
}

// ============================================================================
// PIPE OPERATOR
// ============================================================================

mod pipe {
    use super::*;

    #[test]
    fn test_pipe_basic() {
        let code = r#"
            ken result = 5 |> tae_string
            blether result
        "#;
        assert_eq!(run(code).trim(), "5");
    }

    #[test]
    fn test_pipe_chain() {
        let code = r#"
            ken nums = [3, 1, 4, 1, 5]
            ken result = nums |> sort |> reverse |> heid
            blether result
        "#;
        assert_eq!(run(code).trim(), "5");
    }
}

// ============================================================================
// RANGE
// ============================================================================

mod range {
    use super::*;

    #[test]
    fn test_range_basic() {
        let code = r#"
            ken r = range(0, 5)
            blether r
        "#;
        assert_eq!(run(code).trim(), "[0, 1, 2, 3, 4]");
    }

    #[test]
    fn test_range_with_step() {
        let code = r#"
            ken r = range(0, 10, 2)
            blether r
        "#;
        assert_eq!(run(code).trim(), "[0, 2, 4, 6, 8]");
    }
}

// ============================================================================
// SPECIAL VALUES
// ============================================================================

mod special_values {
    use super::*;

    #[test]
    fn test_nil() {
        assert_eq!(run("blether naething").trim(), "naething");
    }

    #[test]
    fn test_nil_comparison() {
        assert_eq!(run("blether naething == naething").trim(), "aye");
    }

    #[test]
    fn test_bool_true() {
        assert_eq!(run("blether aye").trim(), "aye");
    }

    #[test]
    fn test_bool_false() {
        assert_eq!(run("blether nae").trim(), "nae");
    }
}

// ============================================================================
// TRY-CATCH EXCEPTION HANDLING
// ============================================================================

mod try_catch {
    use super::*;

    #[test]
    fn test_try_catch_no_error() {
        // Try block executes successfully, catch is not executed
        let code = r#"
            hae_a_bash {
                blether "Success"
            } gin_it_gangs_wrang e {
                blether "Error: " + e
            }
        "#;
        assert_eq!(run(code).trim(), "Success");
    }

    #[test]
    fn test_try_catch_with_variable_access() {
        // Variables from before try block should be accessible
        let code = r#"
            ken x = 42
            hae_a_bash {
                blether x
            } gin_it_gangs_wrang e {
                blether "Error"
            }
        "#;
        assert_eq!(run(code).trim(), "42");
    }

    #[test]
    fn test_try_catch_multiple_statements() {
        // Multiple statements in try block
        let code = r#"
            hae_a_bash {
                ken a = 10
                ken b = 20
                blether a + b
            } gin_it_gangs_wrang e {
                blether "Error"
            }
        "#;
        assert_eq!(run(code).trim(), "30");
    }

    #[test]
    fn test_try_catch_in_function() {
        // Try-catch inside a function
        let code = r#"
            dae safe_operation() {
                hae_a_bash {
                    gie "Success"
                } gin_it_gangs_wrang e {
                    gie "Failed"
                }
            }
            blether safe_operation()
        "#;
        assert_eq!(run(code).trim(), "Success");
    }

    #[test]
    fn test_nested_try_catch() {
        // Nested try-catch blocks
        let code = r#"
            hae_a_bash {
                hae_a_bash {
                    blether "Inner"
                } gin_it_gangs_wrang e {
                    blether "Inner catch"
                }
                blether "Outer"
            } gin_it_gangs_wrang e {
                blether "Outer catch"
            }
        "#;
        assert_eq!(run(code).trim(), "Inner\nOuter");
    }
}

// ============================================================================
// ASSERTIONS
// ============================================================================

mod assertions {
    use super::*;

    #[test]
    fn test_assert_true() {
        let code = r#"
            mak_siccar aye
            blether "Passed"
        "#;
        assert_eq!(run(code).trim(), "Passed");
    }

    #[test]
    fn test_assert_expression() {
        let code = r#"
            ken x = 5
            mak_siccar x > 0
            blether "Positive"
        "#;
        assert_eq!(run(code).trim(), "Positive");
    }

    #[test]
    fn test_assert_with_comparison() {
        let code = r#"
            ken a = 10
            ken b = 10
            mak_siccar a == b
            blether "Equal"
        "#;
        assert_eq!(run(code).trim(), "Equal");
    }

    #[test]
    fn test_multiple_asserts() {
        let code = r#"
            mak_siccar 1 == 1
            mak_siccar 2 > 1
            mak_siccar 3 >= 3
            blether "All passed"
        "#;
        assert_eq!(run(code).trim(), "All passed");
    }
}

// ============================================================================
// CLOSURE AND LAMBDA CAPTURE
// ============================================================================

mod closures {
    use super::*;

    // Note: Full closure capture isn't supported yet in LLVM backend
    // These tests cover what's currently working

    #[test]
    fn test_lambda_as_argument() {
        let code = r#"
            dae apply_twice(f, x) {
                gie f(f(x))
            }
            ken inc = |n| n + 1
            blether apply_twice(inc, 5)
        "#;
        assert_eq!(run(code).trim(), "7");
    }

    #[test]
    fn test_lambda_as_argument_multiply() {
        let code = r#"
            dae apply(f, x) {
                gie f(x)
            }
            ken triple = |x| x * 3
            blether apply(triple, 7)
        "#;
        assert_eq!(run(code).trim(), "21");
    }

    #[test]
    fn test_lambda_expression_body() {
        let code = r#"
            ken square = |x| x * x
            blether square(5)
        "#;
        assert_eq!(run(code).trim(), "25");
    }

    #[test]
    fn test_lambda_multiple_params() {
        let code = r#"
            ken add = |a, b| a + b
            blether add(3, 4)
        "#;
        assert_eq!(run(code).trim(), "7");
    }

    #[test]
    fn test_lambda_returning_string() {
        let code = r#"
            ken greet = |name| "Hello, " + name
            blether greet("World")
        "#;
        assert_eq!(run(code).trim(), "Hello, World");
    }
}

// ============================================================================
// ADVANCED ARITHMETIC AND NUMERIC EDGE CASES
// ============================================================================

mod numeric_edge_cases {
    use super::*;

    #[test]
    fn test_large_integers() {
        assert_eq!(run("blether 999999999").trim(), "999999999");
        assert_eq!(run("blether 1000000 * 1000").trim(), "1000000000");
    }

    #[test]
    fn test_negative_operations() {
        assert_eq!(run("blether -10 + 5").trim(), "-5");
        assert_eq!(run("blether -10 - 5").trim(), "-15");
        assert_eq!(run("blether -10 * -2").trim(), "20");
    }

    #[test]
    fn test_float_precision() {
        // LLVM backend rounds some float output differently
        assert_eq!(run("blether 0.1 + 0.2").trim(), "0.3");
        assert_eq!(run("blether 1.0 / 3.0 * 3.0").trim(), "1");
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(run("blether 2 + 3 * 4").trim(), "14");
        assert_eq!(run("blether (2 + 3) * 4").trim(), "20");
        assert_eq!(run("blether 20 / 4 / 2").trim(), "2");
        assert_eq!(run("blether 2 * 3 + 4 * 5").trim(), "26");
    }

    #[test]
    fn test_chained_comparisons() {
        let code = r#"
            ken x = 5
            blether x > 0 an x < 10
        "#;
        assert_eq!(run(code).trim(), "aye");
    }

    #[test]
    fn test_complex_arithmetic() {
        assert_eq!(run("blether ((10 + 5) * 2 - 10) / 4").trim(), "5");
        assert_eq!(run("blether 100 - 50 + 25 - 10").trim(), "65");
    }
}

// ============================================================================
// ADVANCED STRING OPERATIONS
// ============================================================================

mod string_advanced {
    use super::*;

    #[test]
    fn test_string_comparison() {
        assert_eq!(run(r#"blether "abc" == "abc""#).trim(), "aye");
        assert_eq!(run(r#"blether "abc" != "def""#).trim(), "aye");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(run(r#"blether len("")"#).trim(), "0");
        assert_eq!(run(r#"blether "" + "hello""#).trim(), "hello");
    }

    #[test]
    fn test_string_with_numbers() {
        assert_eq!(run(r#"blether "value: " + tae_string(42)"#).trim(), "value: 42");
    }

    #[test]
    fn test_multiline_fstring() {
        let code = r#"
            ken name = "World"
            ken count = 3
            blether f"Hello {name}, count is {count}"
        "#;
        assert_eq!(run(code).trim(), "Hello World, count is 3");
    }

    #[test]
    fn test_fstring_with_expression() {
        let code = r#"
            ken x = 10
            ken y = 20
            blether f"Sum: {x + y}, Product: {x * y}"
        "#;
        assert_eq!(run(code).trim(), "Sum: 30, Product: 200");
    }

    #[test]
    fn test_string_upper_lower_chain() {
        assert_eq!(run(r#"blether upper(lower("HeLLo"))"#).trim(), "HELLO");
    }

    #[test]
    fn test_string_escape_newline() {
        // Test that \n is properly handled
        let code = r#"blether "line1\nline2""#;
        assert_eq!(run(code).trim(), "line1\nline2");
    }

    #[test]
    fn test_string_escape_tab() {
        let code = r#"blether "col1\tcol2""#;
        assert_eq!(run(code).trim(), "col1\tcol2");
    }

    #[test]
    fn test_contains_edge_cases() {
        assert_eq!(run(r#"blether contains("", "")"#).trim(), "aye");
        assert_eq!(run(r#"blether contains("hello", "")"#).trim(), "aye");
        assert_eq!(run(r#"blether contains("", "x")"#).trim(), "nae");
    }

    #[test]
    fn test_starts_ends_edge_cases() {
        assert_eq!(run(r#"blether starts_wi("hello", "")"#).trim(), "aye");
        assert_eq!(run(r#"blether ends_wi("hello", "")"#).trim(), "aye");
        assert_eq!(run(r#"blether starts_wi("", "")"#).trim(), "aye");
    }
}

// ============================================================================
// CONTROL FLOW EDGE CASES
// ============================================================================

mod control_flow_advanced {
    use super::*;

    #[test]
    fn test_nested_if() {
        let code = r#"
            ken x = 5
            ken y = 10
            gin x > 0 {
                gin y > 5 {
                    blether "both"
                }
            }
        "#;
        assert_eq!(run(code).trim(), "both");
    }

    #[test]
    fn test_if_with_complex_condition() {
        let code = r#"
            ken a = 5
            ken b = 10
            gin a > 0 an b > 0 {
                blether "positive"
            }
        "#;
        assert_eq!(run(code).trim(), "positive");
    }

    #[test]
    fn test_deeply_nested_loops() {
        let code = r#"
            ken sum = 0
            ken i = 0
            whiles i < 3 {
                ken j = 0
                whiles j < 3 {
                    sum = sum + 1
                    j = j + 1
                }
                i = i + 1
            }
            blether sum
        "#;
        assert_eq!(run(code).trim(), "9");
    }

    #[test]
    fn test_loop_with_conditional() {
        let code = r#"
            ken sum = 0
            ken i = 0
            whiles i < 10 {
                gin i % 2 == 0 {
                    sum = sum + i
                }
                i = i + 1
            }
            blether sum
        "#;
        // 0 + 2 + 4 + 6 + 8 = 20
        assert_eq!(run(code).trim(), "20");
    }

    #[test]
    fn test_while_with_decreasing_counter() {
        let code = r#"
            ken n = 5
            ken product = 1
            whiles n > 0 {
                product = product * n
                n = n - 1
            }
            blether product
        "#;
        // 5! = 120
        assert_eq!(run(code).trim(), "120");
    }
}

// ============================================================================
// FUNCTION EDGE CASES
// ============================================================================

mod function_advanced {
    use super::*;

    #[test]
    fn test_function_with_many_params() {
        let code = r#"
            dae sum5(a, b, c, d, e) {
                gie a + b + c + d + e
            }
            blether sum5(1, 2, 3, 4, 5)
        "#;
        assert_eq!(run(code).trim(), "15");
    }

    #[test]
    fn test_function_no_return() {
        let code = r#"
            dae print_double(x) {
                blether x * 2
            }
            print_double(21)
        "#;
        assert_eq!(run(code).trim(), "42");
    }

    #[test]
    fn test_function_returning_bool() {
        let code = r#"
            dae is_even(n) {
                gie n % 2 == 0
            }
            blether is_even(4)
        "#;
        assert_eq!(run(code).trim(), "aye");
    }

    #[test]
    fn test_function_with_string_param() {
        let code = r#"
            dae greet(name) {
                gie "Hello, " + name + "!"
            }
            blether greet("Alice")
        "#;
        assert_eq!(run(code).trim(), "Hello, Alice!");
    }

    #[test]
    fn test_mutual_recursion() {
        let code = r#"
            dae is_even(n) {
                gin n == 0 {
                    gie aye
                }
                gie is_odd(n - 1)
            }
            dae is_odd(n) {
                gin n == 0 {
                    gie nae
                }
                gie is_even(n - 1)
            }
            blether is_even(10)
        "#;
        assert_eq!(run(code).trim(), "aye");
    }

    #[test]
    fn test_tail_recursion_style() {
        let code = r#"
            dae sum_to(n, acc) {
                gin n <= 0 {
                    gie acc
                }
                gie sum_to(n - 1, acc + n)
            }
            blether sum_to(10, 0)
        "#;
        // 1+2+3+4+5+6+7+8+9+10 = 55
        assert_eq!(run(code).trim(), "55");
    }
}

// ============================================================================
// CLASS ADVANCED TESTS
// ============================================================================

mod class_advanced {
    use super::*;

    #[test]
    fn test_class_multiple_fields() {
        let code = r#"
            kin Rectangle {
                dae init(width, height) {
                    masel.width = width
                    masel.height = height
                }
                dae area() {
                    gie masel.width * masel.height
                }
            }
            ken r = Rectangle(5, 3)
            blether r.area()
        "#;
        assert_eq!(run(code).trim(), "15");
    }

    #[test]
    fn test_class_field_update() {
        let code = r#"
            kin Counter {
                dae init() {
                    masel.value = 0
                }
                dae inc() {
                    masel.value = masel.value + 1
                }
            }
            ken c = Counter()
            c.inc()
            c.inc()
            c.inc()
            blether c.value
        "#;
        assert_eq!(run(code).trim(), "3");
    }

    #[test]
    fn test_class_method_with_return_self() {
        let code = r#"
            kin Fluent {
                dae init() {
                    masel.val = 0
                }
                dae set(x) {
                    masel.val = x
                    gie masel
                }
                dae add(x) {
                    masel.val = masel.val + x
                    gie masel
                }
            }
            ken f = Fluent()
            blether f.set(5).add(3).val
        "#;
        assert_eq!(run(code).trim(), "8");
    }

    #[test]
    fn test_multiple_instances() {
        let code = r#"
            kin Point {
                dae init(x, y) {
                    masel.x = x
                    masel.y = y
                }
            }
            ken p1 = Point(1, 2)
            ken p2 = Point(3, 4)
            blether p1.x + p2.x
        "#;
        assert_eq!(run(code).trim(), "4");
    }

    #[test]
    fn test_class_method_calling_another() {
        let code = r#"
            kin Calculator {
                dae init(v) {
                    masel.value = v
                }
                dae double() {
                    gie masel.value * 2
                }
                dae quadruple() {
                    gie masel.double() * 2
                }
            }
            ken c = Calculator(5)
            blether c.quadruple()
        "#;
        assert_eq!(run(code).trim(), "20");
    }
}

// ============================================================================
// MATH FUNCTION EDGE CASES
// ============================================================================

mod math_advanced {
    use super::*;

    // Note: Some math functions have precision/implementation differences in LLVM backend

    #[test]
    fn test_abs_zero() {
        assert_eq!(run("blether abs(0)").trim(), "0");
    }

    #[test]
    fn test_abs_negative_int() {
        assert_eq!(run("blether abs(-42)").trim(), "42");
    }

    #[test]
    fn test_floor_negative() {
        assert_eq!(run("blether floor(-2.3)").trim(), "-3");
    }

    #[test]
    fn test_ceil_negative() {
        assert_eq!(run("blether ceil(-2.3)").trim(), "-2");
    }

    #[test]
    fn test_round_half() {
        assert_eq!(run("blether round(2.5)").trim(), "3");
        assert_eq!(run("blether round(2.4)").trim(), "2");
    }

    // Note: pow function has issues in LLVM backend - skipping tests for it

    #[test]
    fn test_clamp_within_range() {
        assert_eq!(run("blether clamp(5, 0, 10)").trim(), "5");
    }

    #[test]
    fn test_clamp_below_min() {
        assert_eq!(run("blether clamp(-5, 0, 10)").trim(), "0");
    }

    #[test]
    fn test_clamp_above_max() {
        assert_eq!(run("blether clamp(15, 0, 10)").trim(), "10");
    }

    #[test]
    fn test_min_function() {
        assert_eq!(run("blether min(10, 5)").trim(), "5");
        assert_eq!(run("blether min(-3, 2)").trim(), "-3");
    }

    #[test]
    fn test_max_function() {
        assert_eq!(run("blether max(10, 5)").trim(), "10");
        assert_eq!(run("blether max(-3, 2)").trim(), "2");
    }
}

// ============================================================================
// TYPE CONVERSION EDGE CASES
// ============================================================================

mod conversion_advanced {
    use super::*;

    #[test]
    fn test_to_string_negative() {
        assert_eq!(run("blether tae_string(-42)").trim(), "-42");
    }

    #[test]
    fn test_to_string_float_whole() {
        // Float that's a whole number
        assert_eq!(run("blether tae_string(5.0)").trim(), "5");
    }

    #[test]
    fn test_to_int_negative_float() {
        assert_eq!(run("blether tae_int(-3.9)").trim(), "-3");
    }

    #[test]
    fn test_to_float_integer() {
        assert_eq!(run("blether tae_float(42)").trim(), "42");
    }

    #[test]
    fn test_tae_string_then_concat() {
        // Test string conversion and then concatenation
        let code = r#"
            ken n = 42
            ken s = tae_string(n)
            blether "Value: " + s
        "#;
        assert_eq!(run(code).trim(), "Value: 42");
    }
}

// ============================================================================
// VARIABLE SCOPE TESTS
// ============================================================================

mod scope {
    use super::*;

    #[test]
    fn test_block_scope() {
        let code = r#"
            ken x = 10
            {
                ken y = 20
                blether x + y
            }
        "#;
        assert_eq!(run(code).trim(), "30");
    }

    #[test]
    fn test_function_scope() {
        let code = r#"
            ken x = 5
            dae f() {
                ken x = 10
                gie x
            }
            blether f()
        "#;
        assert_eq!(run(code).trim(), "10");
    }

    #[test]
    fn test_outer_variable_modification() {
        let code = r#"
            ken x = 5
            dae modify() {
                x = 10
            }
            modify()
            blether x
        "#;
        assert_eq!(run(code).trim(), "10");
    }

    #[test]
    fn test_multiple_nested_scopes() {
        let code = r#"
            ken a = 1
            {
                ken b = 2
                {
                    ken c = 3
                    blether a + b + c
                }
            }
        "#;
        assert_eq!(run(code).trim(), "6");
    }
}

// ============================================================================
// LOGICAL OPERATION EDGE CASES
// ============================================================================

mod logical_advanced {
    use super::*;

    #[test]
    fn test_chained_and() {
        assert_eq!(run("blether aye an aye an aye").trim(), "aye");
        assert_eq!(run("blether aye an nae an aye").trim(), "nae");
    }

    #[test]
    fn test_chained_or() {
        assert_eq!(run("blether nae or nae or aye").trim(), "aye");
        assert_eq!(run("blether nae or nae or nae").trim(), "nae");
    }

    #[test]
    fn test_mixed_and_or() {
        // AND has higher precedence than OR in most languages
        assert_eq!(run("blether aye or nae an nae").trim(), "aye");
    }

    #[test]
    fn test_not_with_comparison() {
        assert_eq!(run("blether nae (5 > 10)").trim(), "aye");
        assert_eq!(run("blether nae (5 < 10)").trim(), "nae");
    }

    #[test]
    fn test_complex_boolean_expression() {
        let code = r#"
            ken x = 5
            ken y = 10
            blether (x > 0 an y > 0) or (x < 0 an y < 0)
        "#;
        assert_eq!(run(code).trim(), "aye");
    }
}

// ============================================================================
// INTEGRATION / COMPLEX SCENARIOS
// ============================================================================

mod integration {
    use super::*;

    #[test]
    fn test_bubble_sort() {
        let code = r#"
            dae bubble_sort(arr) {
                ken n = len(arr)
                fer i in 0..n {
                    fer j in 0..(n - i - 1) {
                        gin arr[j] > arr[j + 1] {
                            ken temp = arr[j]
                            arr[j] = arr[j + 1]
                            arr[j + 1] = temp
                        }
                    }
                }
                gie arr
            }
            blether bubble_sort([64, 34, 25, 12, 22, 11, 90])
        "#;
        assert_eq!(run(code).trim(), "[11, 12, 22, 25, 34, 64, 90]");
    }

    #[test]
    fn test_binary_search() {
        let code = r#"
            dae binary_search(arr, target) {
                ken low = 0
                ken high = len(arr) - 1
                whiles low <= high {
                    ken mid = (low + high) / 2
                    gin arr[mid] == target {
                        gie mid
                    } ither gin arr[mid] < target {
                        low = mid + 1
                    } ither {
                        high = mid - 1
                    }
                }
                gie -1
            }
            blether binary_search([1, 3, 5, 7, 9, 11, 13], 7)
        "#;
        assert_eq!(run(code).trim(), "3");
    }

    #[test]
    fn test_nested_functions() {
        let code = r#"
            dae outer(x) {
                dae inner(y) {
                    gie x + y
                }
                gie inner(10)
            }
            blether outer(5)
        "#;
        assert_eq!(run(code).trim(), "15");
    }

    #[test]
    fn test_list_of_objects() {
        let code = r#"
            kin Item {
                dae init(name, value) {
                    masel.name = name
                    masel.value = value
                }
            }
            ken items = [Item("a", 1), Item("b", 2), Item("c", 3)]
            ken total = 0
            fer item in items {
                total = total + item.value
            }
            blether total
        "#;
        assert_eq!(run(code).trim(), "6");
    }

    #[test]
    fn test_method_chaining() {
        let code = r#"
            kin Builder {
                dae init() {
                    masel.value = 0
                }
                dae add(n) {
                    masel.value = masel.value + n
                    gie masel
                }
                dae multiply(n) {
                    masel.value = masel.value * n
                    gie masel
                }
                dae result() {
                    gie masel.value
                }
            }
            ken b = Builder()
            blether b.add(5).multiply(2).add(3).result()
        "#;
        assert_eq!(run(code).trim(), "13");
    }
}

// ============================================================================
// ADDITIONAL BASIC TESTS
// ============================================================================

mod additional_basics {
    use super::*;

    #[test]
    fn test_multiple_function_calls() {
        let code = r#"
            dae add(a, b) {
                gie a + b
            }
            dae mul(a, b) {
                gie a * b
            }
            blether add(mul(2, 3), mul(4, 5))
        "#;
        // 2*3 + 4*5 = 6 + 20 = 26
        assert_eq!(run(code).trim(), "26");
    }

    #[test]
    fn test_string_in_condition() {
        let code = r#"
            ken name = "Alice"
            gin len(name) > 3 {
                blether "long name"
            } ither {
                blether "short name"
            }
        "#;
        assert_eq!(run(code).trim(), "long name");
    }

    #[test]
    fn test_list_in_function() {
        let code = r#"
            dae sum_list(lst) {
                ken total = 0
                fer x in lst {
                    total = total + x
                }
                gie total
            }
            blether sum_list([1, 2, 3, 4, 5])
        "#;
        assert_eq!(run(code).trim(), "15");
    }

    #[test]
    fn test_nested_dict_access() {
        let code = r#"
            ken config = {"server": {"host": "localhost", "port": 8080}}
            blether config["server"]["host"]
        "#;
        assert_eq!(run(code).trim(), "localhost");
    }
}

// ============================================================================
// ERROR HANDLING (try-catch without hurl)
// ============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_try_catch_success() {
        // Try block executes successfully, catch is not executed
        let code = r#"
            hae_a_bash {
                blether "Success"
            } gin_it_gangs_wrang e {
                blether "Error"
            }
        "#;
        assert_eq!(run(code).trim(), "Success");
    }

    #[test]
    fn test_try_catch_variable_access() {
        // Variables from before try block should be accessible
        let code = r#"
            ken x = 42
            hae_a_bash {
                blether x
            } gin_it_gangs_wrang e {
                blether "Error"
            }
        "#;
        assert_eq!(run(code).trim(), "42");
    }

    #[test]
    fn test_try_catch_multiple_statements() {
        // Multiple statements in try block
        let code = r#"
            hae_a_bash {
                ken a = 10
                ken b = 20
                blether a + b
            } gin_it_gangs_wrang e {
                blether "Error"
            }
        "#;
        assert_eq!(run(code).trim(), "30");
    }
}

// ============================================================================
// ADVANCED STRING OPERATIONS
// ============================================================================

mod advanced_strings {
    use super::*;

    #[test]
    fn test_string_wheesht_trim() {
        let code = r#"
            ken s = "  hello world  "
            blether wheesht(s)
        "#;
        assert_eq!(run(code).trim(), "hello world");
    }

    #[test]
    fn test_string_slice_operations() {
        let code = r#"
            ken s = "hello world"
            blether s[0:5]
        "#;
        assert_eq!(run(code).trim(), "hello");
    }

    #[test]
    fn test_string_negative_slice() {
        let code = r#"
            ken s = "hello"
            blether s[-2:]
        "#;
        assert_eq!(run(code).trim(), "lo");
    }

    #[test]
    fn test_string_step_slice() {
        let code = r#"
            ken s = "abcdefgh"
            blether s[::2]
        "#;
        assert_eq!(run(code).trim(), "aceg");
    }

    #[test]
    fn test_string_reverse_slice() {
        let code = r#"
            ken s = "hello"
            blether s[::-1]
        "#;
        assert_eq!(run(code).trim(), "olleh");
    }

    #[test]
    fn test_fstring_with_expressions() {
        let code = r#"
            ken x = 5
            ken y = 10
            blether f"Sum: {x + y}, Product: {x * y}"
        "#;
        assert_eq!(run(code).trim(), "Sum: 15, Product: 50");
    }

    #[test]
    fn test_fstring_nested() {
        let code = r#"
            ken name = "world"
            ken greeting = f"Hello, {name}!"
            blether f"Message: {greeting}"
        "#;
        assert_eq!(run(code).trim(), "Message: Hello, world!");
    }
}

// ============================================================================
// ADVANCED LIST OPERATIONS
// ============================================================================

mod advanced_lists {
    use super::*;

    #[test]
    fn test_list_scran_take() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5]
            blether scran(nums, 0, 3)
        "#;
        assert_eq!(run(code).trim(), "[1, 2, 3]");
    }

    #[test]
    fn test_list_slap_concat() {
        let code = r#"
            ken a = [1, 2]
            ken b = [3, 4]
            blether slap(a, b)
        "#;
        assert_eq!(run(code).trim(), "[1, 2, 3, 4]");
    }

    #[test]
    fn test_list_shuffle() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5]
            ken shuffled = shuffle(nums)
            blether len(shuffled)
        "#;
        // Shuffle should preserve length
        assert_eq!(run(code).trim(), "5");
    }

    #[test]
    fn test_list_bum_last() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5]
            blether bum(nums)
        "#;
        assert_eq!(run(code).trim(), "5");
    }

    #[test]
    fn test_list_sumaw() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5]
            blether sumaw(nums)
        "#;
        assert_eq!(run(code).trim(), "15");
    }

    #[test]
    fn test_list_slice_read() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5]
            blether nums[1:4]
        "#;
        assert_eq!(run(code).trim(), "[2, 3, 4]");
    }
}

// ============================================================================
// ADVANCED CLOSURES
// ============================================================================

mod advanced_closures {
    use super::*;

    #[test]
    fn test_closure_captures_multiple() {
        let code = r#"
            dae make_adder(x, y) {
                gie |z| x + y + z
            }
            ken add = make_adder(10, 20)
            blether add(5)
        "#;
        assert_eq!(run(code).trim(), "35");
    }

    #[test]
    fn test_simple_closure() {
        let code = r#"
            ken x = 10
            ken f = || x * 2
            blether f()
        "#;
        assert_eq!(run(code).trim(), "20");
    }
}

// ============================================================================
// FOR LOOP VARIATIONS
// ============================================================================

mod for_loop_variations {
    use super::*;

    #[test]
    fn test_for_with_index() {
        let code = r#"
            ken items = ["a", "b", "c"]
            ken i = 0
            fer item in items {
                blether f"{i}: {item}"
                i = i + 1
            }
        "#;
        let output = run(code);
        assert!(output.contains("0: a"));
        assert!(output.contains("1: b"));
        assert!(output.contains("2: c"));
    }

    #[test]
    fn test_for_over_dict_keys() {
        let code = r#"
            ken d = {"x": 1, "y": 2}
            fer k in keys(d) {
                blether k
            }
        "#;
        let output = run(code);
        assert!(output.contains("x"));
        assert!(output.contains("y"));
    }

    #[test]
    fn test_nested_for_loops() {
        let code = r#"
            ken sum = 0
            fer i in range(0, 3) {
                fer j in range(0, 3) {
                    sum = sum + i * j
                }
            }
            blether sum
        "#;
        // (0*0 + 0*1 + 0*2) + (1*0 + 1*1 + 1*2) + (2*0 + 2*1 + 2*2) = 0 + 3 + 6 = 9
        assert_eq!(run(code).trim(), "9");
    }

    #[test]
    fn test_for_with_break() {
        let code = r#"
            ken result = 0
            fer i in range(0, 10) {
                gin i == 5 {
                    brak
                }
                result = result + i
            }
            blether result
        "#;
        // 0 + 1 + 2 + 3 + 4 = 10
        assert_eq!(run(code).trim(), "10");
    }

    #[test]
    fn test_for_with_continue() {
        let code = r#"
            ken result = 0
            fer i in range(0, 5) {
                gin i == 2 {
                    haud
                }
                result = result + i
            }
            blether result
        "#;
        // 0 + 1 + 3 + 4 = 8 (skips 2)
        assert_eq!(run(code).trim(), "8");
    }
}

// ============================================================================
// HIGHER ORDER FUNCTIONS
// ============================================================================

mod higher_order {
    use super::*;

    #[test]
    fn test_ilk_map_extended() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5]
            ken doubled = ilk(nums, |x| x * 2)
            blether doubled
        "#;
        assert_eq!(run(code).trim(), "[2, 4, 6, 8, 10]");
    }

    #[test]
    fn test_sieve_filter_extended() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5, 6]
            ken evens = sieve(nums, |x| x % 2 == 0)
            blether evens
        "#;
        assert_eq!(run(code).trim(), "[2, 4, 6]");
    }

    #[test]
    fn test_chained_higher_order() {
        let code = r#"
            ken nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            ken result = ilk(sieve(nums, |x| x % 2 == 0), |x| x * x)
            blether result
        "#;
        assert_eq!(run(code).trim(), "[4, 16, 36, 64, 100]");
    }
}

// ============================================================================
// TIMING AND PERFORMANCE
// ============================================================================

mod timing {
    use super::*;

    #[test]
    fn test_noo_timestamp() {
        let code = r#"
            ken t1 = noo()
            ken t2 = noo()
            blether t2 >= t1
        "#;
        assert_eq!(run(code).trim(), "aye");
    }

    #[test]
    fn test_tick_nanoseconds() {
        let code = r#"
            ken t1 = tick()
            ken t2 = tick()
            blether t2 >= t1
        "#;
        assert_eq!(run(code).trim(), "aye");
    }
}

// ============================================================================
// RECURSION
// ============================================================================

mod recursion {
    use super::*;

    #[test]
    fn test_mutual_recursion() {
        let code = r#"
            dae is_even(n) {
                gin n == 0 {
                    gie aye
                }
                gie is_odd(n - 1)
            }

            dae is_odd(n) {
                gin n == 0 {
                    gie nae
                }
                gie is_even(n - 1)
            }

            blether is_even(10)
        "#;
        assert_eq!(run(code).trim(), "aye");
    }

    #[test]
    fn test_tail_recursion_like() {
        let code = r#"
            dae sum_to(n, acc) {
                gin n == 0 {
                    gie acc
                }
                gie sum_to(n - 1, acc + n)
            }
            blether sum_to(100, 0)
        "#;
        assert_eq!(run(code).trim(), "5050");
    }
}

// ============================================================================
// EDGE CASES
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_function() {
        let code = r#"
            dae empty() {
            }
            ken result = empty()
            blether result
        "#;
        assert_eq!(run(code).trim(), "naething");
    }

    #[test]
    fn test_single_element_list() {
        let code = r#"
            ken single = [42]
            blether heid(single)
            blether bum(single)
            blether len(single)
        "#;
        let output = run(code);
        assert!(output.contains("42"));
        assert!(output.contains("1"));
    }

    #[test]
    fn test_zero_division_check() {
        let code = r#"
            ken x = 10
            ken y = 0
            gin y != 0 {
                blether x / y
            } ither {
                blether "cannot divide by zero"
            }
        "#;
        assert!(run(code).contains("cannot divide by zero"));
    }

    #[test]
    fn test_boolean_short_circuit() {
        let code = r#"
            ken called = nae
            dae side_effect() {
                called = aye
                gie aye
            }
            ken result = nae an side_effect()
            blether called
        "#;
        // Short-circuit: side_effect should not be called
        assert_eq!(run(code).trim(), "nae");
    }

    #[test]
    fn test_deeply_nested_access() {
        let code = r#"
            ken data = {"a": {"b": {"c": {"d": 42}}}}
            blether data["a"]["b"]["c"]["d"]
        "#;
        assert_eq!(run(code).trim(), "42");
    }

    #[test]
    fn test_very_long_string() {
        let code = r#"
            ken s = "a"
            fer i in range(0, 10) {
                s = s + s
            }
            blether len(s)
        "#;
        // 2^10 = 1024
        assert_eq!(run(code).trim(), "1024");
    }

    #[test]
    fn test_unicode_strings() {
        let code = r#"
            ken s = "héllo wörld 🌍"
            blether s
        "#;
        assert!(run(code).contains("héllo wörld 🌍"));
    }

    #[test]
    fn test_negative_modulo() {
        let code = r#"
            blether -7 % 3
        "#;
        // Behavior may vary, just ensure it doesn't crash
        let output = run(code);
        let result = output.trim();
        assert!(result == "-1" || result == "2");
    }

    #[test]
    fn test_float_precision() {
        let code = r#"
            ken x = 0.1 + 0.2
            blether x
        "#;
        // Should be close to 0.3
        let output = run(code);
        let result = output.trim();
        let val: f64 = result.parse().unwrap();
        assert!((val - 0.3).abs() < 0.0001);
    }
}

// ============================================================================
// PIPE OPERATIONS
// ============================================================================

mod pipes {
    use super::*;

    #[test]
    fn test_pipe_with_lambda() {
        let code = r#"
            ken result = 5 |> |x| x * 2 |> |x| x + 1
            blether result
        "#;
        assert_eq!(run(code).trim(), "11");
    }

    #[test]
    fn test_pipe_with_function() {
        let code = r#"
            dae double(x) {
                gie x * 2
            }
            dae add_one(x) {
                gie x + 1
            }
            ken result = 5 |> double |> add_one
            blether result
        "#;
        assert_eq!(run(code).trim(), "11");
    }

    #[test]
    fn test_pipe_with_list_operations() {
        let code = r#"
            ken result = [1, 2, 3, 4, 5] |> |arr| sieve(arr, |x| x > 2) |> |arr| ilk(arr, |x| x * 10)
            blether result
        "#;
        assert_eq!(run(code).trim(), "[30, 40, 50]");
    }
}
