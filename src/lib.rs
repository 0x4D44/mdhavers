//! mdhavers - A Scots Programming Language
//!
//! Pure havers, but working havers!
//!
//! This crate provides the core functionality for the mdhavers programming language,
//! including lexing, parsing, interpretation, and compilation.

pub mod ast;
pub mod audio;
pub mod compiler;
pub mod error;
pub mod formatter;
pub mod graphics;
pub mod interpreter;
pub mod lexer;
pub mod logging;
pub mod parser;
pub mod token;
pub mod tri;
pub mod value;
pub mod wasm_compiler;

#[cfg(feature = "wasm_runner")]
pub mod wasm_runner;

// LLVM backend (optional, requires llvm feature)
#[cfg(feature = "llvm")]
pub mod llvm;

// Re-export commonly used types
pub use error::{HaversError, HaversResult};
pub use interpreter::Interpreter;
pub use parser::parse;
pub use value::Value;

// LLVM compiler re-export
#[cfg(feature = "llvm")]
pub use llvm::LLVMCompiler;

/// Run mdhavers source code and return the result
///
/// This is a convenience function that handles the full pipeline:
/// lexing, parsing, and interpretation.
///
/// # Example
/// ```
/// use mdhavers::run;
///
/// let code = r#"
///     ken x = 42
///     x * 2
/// "#;
/// let result = run(code);
/// ```
pub fn run(source: &str) -> HaversResult<Value> {
    let program = parse(source)?;
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&program)
}

/// Run mdhavers source code and capture output
///
/// Returns a tuple of (result, output_lines) where output_lines
/// contains all lines printed with `blether`.
pub fn run_with_output(source: &str) -> HaversResult<(Value, Vec<String>)> {
    let program = parse(source)?;
    let mut interpreter = Interpreter::new();
    let result = interpreter.interpret(&program)?;
    let output = interpreter.get_output().to_vec();
    Ok((result, output))
}

/// Compile mdhavers source code to JavaScript
pub fn compile_to_js(source: &str) -> HaversResult<String> {
    compiler::compile(source)
}

/// Compile mdhavers source code to WebAssembly Text format
pub fn compile_to_wat(source: &str) -> HaversResult<String> {
    wasm_compiler::compile_to_wat(source)
}

/// Compile mdhavers source code to LLVM IR
#[cfg(feature = "llvm")]
pub fn compile_to_llvm_ir(source: &str) -> HaversResult<String> {
    let program = parse(source)?;
    let compiler = llvm::LLVMCompiler::new();
    compiler.compile_to_ir(&program)
}

/// Format mdhavers source code
pub fn format_source(source: &str) -> HaversResult<String> {
    formatter::format_source(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_simple() {
        let result = run("ken x = 42\nx").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_run_arithmetic() {
        let result = run("10 + 5 * 2").unwrap();
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_run_function() {
        let result = run(r#"
            dae add(a, b) {
                gie a + b
            }
            add(3, 4)
        "#)
        .unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_run_error() {
        let result = run("undefined_var");
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_output_simple() {
        let (result, output) = run_with_output(
            r#"
            blether "Hello"
            blether "World"
            42
        "#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(42));
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], "Hello");
        assert_eq!(output[1], "World");
    }

    #[test]
    fn test_run_with_output_no_output() {
        let (result, output) = run_with_output("5 + 5").unwrap();
        assert_eq!(result, Value::Integer(10));
        assert!(output.is_empty());
    }

    #[test]
    fn test_compile_to_js_simple() {
        let js = compile_to_js("ken x = 42").unwrap();
        assert!(js.contains("let x"));
        assert!(js.contains("42"));
    }

    #[test]
    fn test_compile_to_js_function() {
        let js = compile_to_js(
            r#"
            dae greet(name) {
                gie "Hello " + name
            }
        "#,
        )
        .unwrap();
        assert!(js.contains("function greet"));
    }

    #[test]
    fn test_compile_to_js_error() {
        // Invalid syntax
        let result = compile_to_js("ken = ");
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_to_wat_simple() {
        let wat = compile_to_wat("ken x = 42").unwrap();
        assert!(wat.contains("(module"));
    }

    #[test]
    fn test_compile_to_wat_function() {
        let wat = compile_to_wat(
            r#"
            dae add(a, b) {
                gie a + b
            }
        "#,
        )
        .unwrap();
        assert!(wat.contains("(func"));
    }

    #[test]
    fn test_format_source_simple() {
        let formatted = format_source("ken x=42").unwrap();
        assert!(formatted.contains("ken x = 42"));
    }

    #[test]
    fn test_format_source_function() {
        let formatted = format_source("dae foo(){gie 1}").unwrap();
        assert!(formatted.contains("dae foo()"));
    }

    #[test]
    fn test_format_source_error() {
        let result = format_source("ken = invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_run_boolean_operations() {
        assert_eq!(run("aye").unwrap(), Value::Bool(true));
        assert_eq!(run("nae").unwrap(), Value::Bool(false));
        assert_eq!(run("aye an aye").unwrap(), Value::Bool(true));
        assert_eq!(run("aye an nae").unwrap(), Value::Bool(false));
        assert_eq!(run("aye or nae").unwrap(), Value::Bool(true));
        assert_eq!(run("nae or nae").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_run_comparison_operations() {
        assert_eq!(run("5 > 3").unwrap(), Value::Bool(true));
        assert_eq!(run("5 < 3").unwrap(), Value::Bool(false));
        assert_eq!(run("5 == 5").unwrap(), Value::Bool(true));
        assert_eq!(run("5 != 3").unwrap(), Value::Bool(true));
        assert_eq!(run("5 >= 5").unwrap(), Value::Bool(true));
        assert_eq!(run("5 <= 5").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_run_list_operations() {
        let result = run("[1, 2, 3]").unwrap();
        assert!(matches!(result, Value::List(items) if items.borrow().len() == 3));
    }

    #[test]
    fn test_run_string_operations() {
        let result = run(r#""Hello" + " " + "World""#).unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[cfg(feature = "llvm")]
    #[test]
    fn test_compile_to_llvm_ir_smoke() {
        let ir = compile_to_llvm_ir("ken x = 1\nx").unwrap();
        assert!(!ir.is_empty());
    }
}
