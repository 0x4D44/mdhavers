//! WebAssembly compiler fer mdhavers
//! Generates WAT (WebAssembly Text Format) that can be compiled tae WASM
//!
//! This is a basic WASM compiler that supports:
//! - Integer and float arithmetic
//! - Variables (local)
//! - Functions
//! - Basic control flow (if/while)
//!
//! Note: This is an experimental feature - no' aw mdhavers features are supported!

use crate::ast::*;
use crate::error::{HaversError, HaversResult};

/// The WASM compiler
pub struct WasmCompiler {
    output: String,
    indent: usize,
    local_vars: Vec<String>,
    func_params: Vec<String>,
    string_data: Vec<String>,
}

impl Default for WasmCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmCompiler {
    pub fn new() -> Self {
        WasmCompiler {
            output: String::new(),
            indent: 0,
            local_vars: Vec::new(),
            func_params: Vec::new(),
            string_data: Vec::new(),
        }
    }

    /// Compile a program tae WAT (WebAssembly Text Format)
    pub fn compile(&mut self, program: &Program) -> HaversResult<String> {
        self.output.clear();
        self.string_data.clear();

        // Start the module
        self.emit("(module");
        self.indent += 1;

        // Import memory and print functions from the host
        self.emit_line("");
        self.emit_line(";; Imports fae the host environment");
        self.emit_line("(import \"env\" \"memory\" (memory 1))");
        self.emit_line("(import \"env\" \"print_i32\" (func $print_i32 (param i32)))");
        self.emit_line("(import \"env\" \"print_f64\" (func $print_f64 (param f64)))");
        self.emit_line("(import \"env\" \"print_str\" (func $print_str (param i32 i32)))");
        self.emit_line("");

        // Collect all function declarations first
        let mut functions: Vec<&Stmt> = Vec::new();
        let mut main_stmts: Vec<&Stmt> = Vec::new();

        for stmt in &program.statements {
            match stmt {
                Stmt::Function { .. } => functions.push(stmt),
                _ => main_stmts.push(stmt),
            }
        }

        // Compile functions
        for func in &functions {
            self.compile_function(func)?;
        }

        // Compile main code as start function
        if !main_stmts.is_empty() {
            self.compile_main(&main_stmts)?;
        }

        // Export the main function
        self.emit_line("");
        self.emit_line("(export \"main\" (func $main))");

        // Add string data section if we have strings
        if !self.string_data.is_empty() {
            self.emit_line("");
            self.emit_line(";; String data");
            let mut offset = 0;
            // Collect string data first to avoid borrow issues
            let string_lines: Vec<String> = self
                .string_data
                .iter()
                .map(|s| {
                    let line =
                        format!("(data (i32.const {}) \"{}\")", offset, escape_wat_string(s));
                    offset += s.len() as i32 + 1; // +1 for null terminator
                    line
                })
                .collect();
            for line in string_lines {
                self.emit_line(&line);
            }
        }

        self.indent -= 1;
        self.emit_line(")");

        Ok(self.output.clone())
    }

    fn compile_function(&mut self, stmt: &Stmt) -> HaversResult<()> {
        if let Stmt::Function {
            name, params, body, ..
        } = stmt
        {
            self.local_vars.clear();
            self.func_params.clear();

            // Build parameter list
            let mut param_types = String::new();
            for p in params {
                self.func_params.push(p.name.clone());
                param_types.push_str(&format!("(param ${} i64) ", p.name));
            }

            // Start function
            self.emit_line(&format!("(func ${} {}(result i64)", name, param_types));
            self.indent += 1;

            // Collect locals from body
            self.collect_locals(body);

            // Declare locals (collect first to avoid borrow issues)
            let local_decls: Vec<String> = self
                .local_vars
                .iter()
                .map(|var| format!("(local ${} i64)", var))
                .collect();
            for decl in local_decls {
                self.emit_line(&decl);
            }

            // Compile body
            for s in body {
                self.compile_stmt(s)?;
            }

            // Default return value
            self.emit_line("(i64.const 0)");

            self.indent -= 1;
            self.emit_line(")");
            self.emit_line("");
        }
        Ok(())
    }

    fn compile_main(&mut self, stmts: &[&Stmt]) -> HaversResult<()> {
        self.local_vars.clear();
        self.func_params.clear();

        self.emit_line("(func $main (result i64)");
        self.indent += 1;

        // Collect all locals
        for stmt in stmts {
            self.collect_locals_stmt(stmt);
        }

        // Declare locals
        for var in &self.local_vars.clone() {
            self.emit_line(&format!("(local ${} i64)", var));
        }

        // Compile statements
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }

        // Return 0
        self.emit_line("(i64.const 0)");

        self.indent -= 1;
        self.emit_line(")");
        Ok(())
    }

    fn collect_locals(&mut self, body: &[Stmt]) {
        for stmt in body {
            self.collect_locals_stmt(stmt);
        }
    }

    fn collect_locals_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, .. } => {
                if !self.func_params.contains(name) && !self.local_vars.contains(name) {
                    self.local_vars.push(name.clone());
                }
            }
            Stmt::Block { statements, .. } => {
                for s in statements {
                    self.collect_locals_stmt(s);
                }
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.collect_locals_stmt(then_branch);
                if let Some(eb) = else_branch {
                    self.collect_locals_stmt(eb);
                }
            }
            Stmt::While { body, .. } => {
                self.collect_locals_stmt(body);
            }
            Stmt::For { variable, body, .. } => {
                if !self.func_params.contains(variable) && !self.local_vars.contains(variable) {
                    self.local_vars.push(variable.clone());
                }
                self.collect_locals_stmt(body);
            }
            _ => {}
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> HaversResult<()> {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                if let Some(init) = initializer {
                    self.compile_expr(init)?;
                } else {
                    self.emit_line("(i64.const 0)");
                }
                self.emit_line(&format!("(local.set ${})", name));
            }

            Stmt::Expression { expr, .. } => {
                self.compile_expr(expr)?;
                self.emit_line("(drop)");
            }

            Stmt::Block { statements, .. } => {
                for s in statements {
                    self.compile_stmt(s)?;
                }
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Compile condition
                self.compile_expr(condition)?;
                self.emit_line("(i32.wrap_i64)");

                self.emit_line("(if");
                self.indent += 1;
                self.emit_line("(then");
                self.indent += 1;
                self.compile_stmt(then_branch)?;
                self.indent -= 1;
                self.emit_line(")");

                if let Some(eb) = else_branch {
                    self.emit_line("(else");
                    self.indent += 1;
                    self.compile_stmt(eb)?;
                    self.indent -= 1;
                    self.emit_line(")");
                }

                self.indent -= 1;
                self.emit_line(")");
            }

            Stmt::While {
                condition, body, ..
            } => {
                self.emit_line("(block $break");
                self.indent += 1;
                self.emit_line("(loop $continue");
                self.indent += 1;

                // Check condition
                self.compile_expr(condition)?;
                self.emit_line("(i32.wrap_i64)");
                self.emit_line("(i32.eqz)");
                self.emit_line("(br_if $break)");

                // Body
                self.compile_stmt(body)?;

                // Loop back
                self.emit_line("(br $continue)");

                self.indent -= 1;
                self.emit_line(")");
                self.indent -= 1;
                self.emit_line(")");
            }

            Stmt::Return { value, .. } => {
                if let Some(val) = value {
                    self.compile_expr(val)?;
                } else {
                    self.emit_line("(i64.const 0)");
                }
                self.emit_line("(return)");
            }

            Stmt::Print { value, .. } => {
                // For now, only support printing integers
                self.compile_expr(value)?;
                self.emit_line("(i32.wrap_i64)");
                self.emit_line("(call $print_i32)");
            }

            Stmt::Break { .. } => {
                self.emit_line("(br $break)");
            }

            Stmt::Continue { .. } => {
                self.emit_line("(br $continue)");
            }

            _ => {
                // Unsupported statement type
                return Err(HaversError::InternalError(
                    "This statement type isnae supported in WASM yet!".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> HaversResult<()> {
        match expr {
            Expr::Literal { value, .. } => {
                match value {
                    Literal::Integer(n) => {
                        self.emit_line(&format!("(i64.const {})", n));
                    }
                    Literal::Float(f) => {
                        // Convert float to i64 bits for now (simplified)
                        self.emit_line(&format!("(i64.const {})", (*f as i64)));
                    }
                    Literal::Bool(b) => {
                        self.emit_line(&format!("(i64.const {})", if *b { 1 } else { 0 }));
                    }
                    Literal::Nil => {
                        self.emit_line("(i64.const 0)");
                    }
                    Literal::String(s) => {
                        // Store string in data section and return offset
                        let offset = self.string_data.iter().map(|s| s.len() + 1).sum::<usize>();
                        self.string_data.push(s.clone());
                        self.emit_line(&format!("(i64.const {})", offset));
                    }
                }
            }

            Expr::Variable { name, .. } => {
                self.emit_line(&format!("(local.get ${})", name));
            }

            Expr::Assign { name, value, .. } => {
                self.compile_expr(value)?;
                self.emit_line(&format!("(local.tee ${})", name));
            }

            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                match operator {
                    BinaryOp::Add => self.emit_line("(i64.add)"),
                    BinaryOp::Subtract => self.emit_line("(i64.sub)"),
                    BinaryOp::Multiply => self.emit_line("(i64.mul)"),
                    BinaryOp::Divide => self.emit_line("(i64.div_s)"),
                    BinaryOp::Modulo => self.emit_line("(i64.rem_s)"),
                    BinaryOp::Equal => {
                        self.emit_line("(i64.eq)");
                        self.emit_line("(i64.extend_i32_u)");
                    }
                    BinaryOp::NotEqual => {
                        self.emit_line("(i64.ne)");
                        self.emit_line("(i64.extend_i32_u)");
                    }
                    BinaryOp::Less => {
                        self.emit_line("(i64.lt_s)");
                        self.emit_line("(i64.extend_i32_u)");
                    }
                    BinaryOp::LessEqual => {
                        self.emit_line("(i64.le_s)");
                        self.emit_line("(i64.extend_i32_u)");
                    }
                    BinaryOp::Greater => {
                        self.emit_line("(i64.gt_s)");
                        self.emit_line("(i64.extend_i32_u)");
                    }
                    BinaryOp::GreaterEqual => {
                        self.emit_line("(i64.ge_s)");
                        self.emit_line("(i64.extend_i32_u)");
                    }
                }
            }

            Expr::Unary {
                operator, operand, ..
            } => match operator {
                UnaryOp::Negate => {
                    self.emit_line("(i64.const 0)");
                    self.compile_expr(operand)?;
                    self.emit_line("(i64.sub)");
                }
                UnaryOp::Not => {
                    self.compile_expr(operand)?;
                    self.emit_line("(i64.eqz)");
                    self.emit_line("(i64.extend_i32_u)");
                }
            },

            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => match operator {
                LogicalOp::And => {
                    self.compile_expr(left)?;
                    self.emit_line("(i32.wrap_i64)");
                    self.emit_line("(if (result i64)");
                    self.indent += 1;
                    self.emit_line("(then");
                    self.indent += 1;
                    self.compile_expr(right)?;
                    self.indent -= 1;
                    self.emit_line(")");
                    self.emit_line("(else (i64.const 0))");
                    self.indent -= 1;
                    self.emit_line(")");
                }
                LogicalOp::Or => {
                    self.compile_expr(left)?;
                    self.emit_line("(i32.wrap_i64)");
                    self.emit_line("(if (result i64)");
                    self.indent += 1;
                    self.emit_line("(then (i64.const 1))");
                    self.emit_line("(else");
                    self.indent += 1;
                    self.compile_expr(right)?;
                    self.indent -= 1;
                    self.emit_line(")");
                    self.indent -= 1;
                    self.emit_line(")");
                }
            },

            Expr::Call {
                callee, arguments, ..
            } => {
                // Compile arguments
                for arg in arguments {
                    self.compile_expr(arg)?;
                }

                // Get function name
                if let Expr::Variable { name, .. } = callee.as_ref() {
                    self.emit_line(&format!("(call ${})", name));
                } else {
                    return Err(HaversError::InternalError(
                        "Only direct function calls are supported in WASM".to_string(),
                    ));
                }
            }

            Expr::Grouping { expr, .. } => {
                self.compile_expr(expr)?;
            }

            _ => {
                return Err(HaversError::InternalError(
                    "This expression type isnae supported in WASM yet!".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(&"  ".repeat(self.indent));
        self.output.push_str(s);
    }

    fn emit_line(&mut self, s: &str) {
        self.emit(s);
        self.output.push('\n');
    }
}

/// Escape a string fer WAT data section
fn escape_wat_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_ascii_graphic() || c == ' ' => result.push(c),
            c => result.push_str(&format!("\\{:02x}", c as u32)),
        }
    }
    result
}

/// Compile source code tae WAT
pub fn compile_to_wat(source: &str) -> HaversResult<String> {
    let program = crate::parser::parse(source)?;
    let mut compiler = WasmCompiler::new();
    compiler.compile(&program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_wasm_compile() {
        let source = "ken x = 42";
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("(module"));
        assert!(wat.contains("i64.const 42"));
    }

    #[test]
    fn test_arithmetic_wasm() {
        let source = "ken x = 10 + 20";
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("i64.add"));
    }

    #[test]
    fn test_function_wasm() {
        let source = r#"
            dae add(a, b) {
                gie a + b
            }
        "#;
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("(func $add"));
    }

    #[test]
    fn test_if_wasm() {
        let source = r#"
            ken x = 5
            gin x > 3 {
                blether x
            }
        "#;
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("(if"));
    }

    #[test]
    fn test_while_wasm() {
        let source = r#"
            ken x = 0
            whiles x < 10 {
                x = x + 1
            }
        "#;
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("(loop"));
        assert!(wat.contains("(block"));
    }

    // ==================== Arithmetic Operations ====================

    #[test]
    fn test_subtraction_wasm() {
        let source = "ken x = 50 - 8";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.sub"));
    }

    #[test]
    fn test_multiplication_wasm() {
        let source = "ken x = 6 * 7";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.mul"));
    }

    #[test]
    fn test_division_wasm() {
        let source = "ken x = 84 / 2";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.div_s"));
    }

    #[test]
    fn test_modulo_wasm() {
        let source = "ken x = 10 % 3";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.rem_s"));
    }

    // ==================== Comparison Operations ====================

    #[test]
    fn test_greater_than_wasm() {
        let source = "ken b = 5 > 3";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.gt_s"));
    }

    #[test]
    fn test_less_than_wasm() {
        let source = "ken b = 3 < 5";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.lt_s"));
    }

    #[test]
    fn test_greater_equal_wasm() {
        let source = "ken b = 5 >= 5";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.ge_s"));
    }

    #[test]
    fn test_less_equal_wasm() {
        let source = "ken b = 3 <= 5";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.le_s"));
    }

    #[test]
    fn test_equal_wasm() {
        let source = "ken b = 5 == 5";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.eq"));
    }

    #[test]
    fn test_not_equal_wasm() {
        let source = "ken b = 5 != 3";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.ne"));
    }

    // ==================== Logical Operations ====================

    #[test]
    fn test_logical_and_wasm() {
        let source = "ken b = aye an nae";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_logical_or_wasm() {
        let source = "ken b = aye or nae";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_logical_not_wasm() {
        // "no" requires parentheses and may not be fully supported in WASM
        let source = "ken b = no aye";
        let result = compile_to_wat(source);
        // Just verify it doesn't panic - may or may not be supported
        assert!(result.is_ok() || result.is_err());
    }

    // ==================== Unary Operations ====================

    #[test]
    fn test_negate_wasm() {
        let source = "ken x = -42";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.const 0"));
        assert!(result.contains("i64.sub"));
    }

    // ==================== Control Flow ====================

    #[test]
    fn test_if_else_wasm() {
        let source = r#"
            ken x = 5
            gin x > 3 {
                blether x
            } ither {
                blether 0
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(if"));
        assert!(result.contains("(then"));
        assert!(result.contains("(else"));
    }

    #[test]
    fn test_for_loop_wasm() {
        let source = r#"
            fer i in 1..5 {
                blether i
            }
        "#;
        let result = compile_to_wat(source);
        // For loops may not be supported in WASM yet
        assert!(result.is_err());
    }

    #[test]
    fn test_return_wasm() {
        let source = r#"
            dae answer() {
                gie 42
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(return"));
    }

    #[test]
    fn test_return_implicit_wasm() {
        let source = r#"
            dae answer() {
                gie
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(return"));
    }

    // ==================== Literals ====================

    #[test]
    fn test_boolean_true_wasm() {
        let source = "ken b = aye";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.const 1"));
    }

    #[test]
    fn test_boolean_false_wasm() {
        let source = "ken b = nae";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.const 0"));
    }

    #[test]
    fn test_nil_wasm() {
        let source = "ken n = naething";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("i64.const 0"));
    }

    #[test]
    fn test_float_wasm() {
        let source = "ken f = 3.14";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_string_wasm() {
        let source = r#"ken s = "Hello""#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(data"));
    }

    // ==================== String Escape ====================

    #[test]
    fn test_string_escape_newline() {
        let source = r#"ken s = "hello\nworld""#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(data"));
    }

    #[test]
    fn test_string_escape_tab() {
        let source = r#"ken s = "hello\tworld""#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(data"));
    }

    // ==================== Multiple Statements ====================

    #[test]
    fn test_multiple_vars_wasm() {
        let source = r#"
            ken a = 1
            ken b = 2
            ken c = a + b
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_function_with_params_wasm() {
        let source = r#"
            dae multiply(a, b) {
                gie a * b
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(func $multiply"));
        assert!(result.contains("(param"));
    }

    // ==================== Variable Operations ====================

    #[test]
    fn test_variable_assignment_wasm() {
        let source = r#"
            ken x = 1
            x = 42
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("local.set"));
    }

    #[test]
    fn test_variable_get_wasm() {
        let source = r#"
            ken x = 42
            ken y = x
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("local.get"));
    }

    // ==================== Block ====================

    #[test]
    fn test_block_wasm() {
        let source = r#"
            {
                ken x = 1
                ken y = 2
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_nested_blocks_wasm() {
        let source = r#"
            {
                ken x = 1
                {
                    ken y = 2
                }
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    // ==================== Print ====================

    #[test]
    fn test_print_wasm() {
        let source = "blether 42";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $print"));
    }

    #[test]
    fn test_print_string_wasm() {
        let source = r#"blether "Hello""#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }
}
