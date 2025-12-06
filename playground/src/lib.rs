//! mdhavers Playground WASM Bindings
//!
//! This crate provides WebAssembly bindings for running mdhavers
//! code in the browser.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Initialize panic hook for better error messages in browser
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Result of running mdhavers code
#[derive(Serialize, Deserialize)]
pub struct RunResult {
    /// Whether execution was successful
    pub success: bool,
    /// The result value (as string)
    pub result: String,
    /// Output from blether statements
    pub output: Vec<String>,
    /// Error message if execution failed
    pub error: Option<String>,
}

/// Run mdhavers code and return the result as JSON
///
/// # Arguments
/// * `code` - The mdhavers source code to execute
///
/// # Returns
/// A JSON string containing the RunResult
#[wasm_bindgen]
pub fn run(code: &str) -> String {
    let result = run_internal(code);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(r#"{{"success":false,"error":"Serialization error: {}","result":"","output":[]}}"#, e)
    })
}

fn run_internal(code: &str) -> RunResult {
    // Parse the code
    let program = match mdhavers::parse(code) {
        Ok(p) => p,
        Err(e) => {
            return RunResult {
                success: false,
                result: String::new(),
                output: vec![],
                error: Some(format!("{}", e)),
            };
        }
    };

    // Create interpreter and run
    let mut interpreter = mdhavers::Interpreter::new();

    match interpreter.interpret(&program) {
        Ok(value) => {
            let output = interpreter.get_output().to_vec();
            RunResult {
                success: true,
                result: format!("{}", value),
                output,
                error: None,
            }
        }
        Err(e) => {
            let output = interpreter.get_output().to_vec();
            RunResult {
                success: false,
                result: String::new(),
                output,
                error: Some(format!("{}", e)),
            }
        }
    }
}

/// Check mdhavers code for syntax errors without running
///
/// # Arguments
/// * `code` - The mdhavers source code to check
///
/// # Returns
/// A JSON string with success status and any error message
#[wasm_bindgen]
pub fn check(code: &str) -> String {
    match mdhavers::parse(code) {
        Ok(_) => r#"{"success":true,"error":null}"#.to_string(),
        Err(e) => {
            let error = format!("{}", e).replace('"', "\\\"");
            format!(r#"{{"success":false,"error":"{}"}}"#, error)
        }
    }
}

/// Format mdhavers code
///
/// # Arguments
/// * `code` - The mdhavers source code to format
///
/// # Returns
/// The formatted code or original code if formatting fails
#[wasm_bindgen]
pub fn format(code: &str) -> String {
    mdhavers::format_source(code).unwrap_or_else(|_| code.to_string())
}

/// Compile mdhavers code to JavaScript
///
/// # Arguments
/// * `code` - The mdhavers source code to compile
///
/// # Returns
/// A JSON string with success status and compiled code or error
#[wasm_bindgen]
pub fn compile_to_js(code: &str) -> String {
    match mdhavers::compile_to_js(code) {
        Ok(js) => {
            let js_escaped = js.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
            format!(r#"{{"success":true,"code":"{}","error":null}}"#, js_escaped)
        }
        Err(e) => {
            let error = format!("{}", e).replace('"', "\\\"");
            format!(r#"{{"success":false,"code":null,"error":"{}"}}"#, error)
        }
    }
}

/// Get version information
#[wasm_bindgen]
pub fn version() -> String {
    "0.1.0".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_simple() {
        let result = run("blether 42");
        assert!(result.contains("success\":true"));
    }

    #[test]
    fn test_run_error() {
        let result = run("undefined_var");
        assert!(result.contains("success\":false"));
    }

    #[test]
    fn test_check_valid() {
        let result = check("ken x = 42");
        assert!(result.contains("success\":true"));
    }

    #[test]
    fn test_check_invalid() {
        let result = check("ken = ");
        assert!(result.contains("success\":false"));
    }
}
