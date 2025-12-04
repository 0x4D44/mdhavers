use crate::ast::*;
use crate::error::HaversResult;

/// Compiler - transpiles mdhavers tae JavaScript
pub struct Compiler {
    indent: usize,
    output: String,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            indent: 0,
            output: String::new(),
        }
    }

    /// Compile a program tae JavaScript
    pub fn compile(&mut self, program: &Program) -> HaversResult<String> {
        self.output.clear();

        // Add runtime helpers
        self.emit_runtime();

        // Compile all statements
        for stmt in &program.statements {
            self.compile_stmt(stmt)?;
        }

        Ok(self.output.clone())
    }

    fn emit_runtime(&mut self) {
        self.emit_line("// mdhavers runtime - pure havers, but working havers!");
        self.emit_line("const __havers = {");
        self.indent += 1;

        // len function
        self.emit_line("len: (x) => {");
        self.indent += 1;
        self.emit_line("if (typeof x === 'string' || Array.isArray(x)) return x.length;");
        self.emit_line("if (x && typeof x === 'object') return Object.keys(x).length;");
        self.emit_line("throw new Error('Och! Cannae get length o\\' that!');");
        self.indent -= 1;
        self.emit_line("},");

        // whit_kind (type) function
        self.emit_line("whit_kind: (x) => {");
        self.indent += 1;
        self.emit_line("if (x === null || x === undefined) return 'naething';");
        self.emit_line("if (Array.isArray(x)) return 'list';");
        self.emit_line("if (typeof x === 'object') return 'dict';");
        self.emit_line("return typeof x;");
        self.indent -= 1;
        self.emit_line("},");

        // tae_string function
        self.emit_line("tae_string: (x) => String(x),");

        // tae_int function
        self.emit_line("tae_int: (x) => {");
        self.indent += 1;
        self.emit_line("const n = parseInt(x, 10);");
        self.emit_line("if (isNaN(n)) throw new Error(`Cannae turn '${x}' intae an integer`);");
        self.emit_line("return n;");
        self.indent -= 1;
        self.emit_line("},");

        // tae_float function
        self.emit_line("tae_float: (x) => {");
        self.indent += 1;
        self.emit_line("const n = parseFloat(x);");
        self.emit_line("if (isNaN(n)) throw new Error(`Cannae turn '${x}' intae a float`);");
        self.emit_line("return n;");
        self.indent -= 1;
        self.emit_line("},");

        // shove (push) function
        self.emit_line("shove: (arr, val) => { arr.push(val); },");

        // yank (pop) function
        self.emit_line("yank: (arr) => {");
        self.indent += 1;
        self.emit_line("if (arr.length === 0) throw new Error('Cannae yank fae an empty list!');");
        self.emit_line("return arr.pop();");
        self.indent -= 1;
        self.emit_line("},");

        // keys function
        self.emit_line("keys: (obj) => Object.keys(obj),");

        // values function
        self.emit_line("values: (obj) => Object.values(obj),");

        // range function
        self.emit_line("range: (start, end) => {");
        self.indent += 1;
        self.emit_line("const result = [];");
        self.emit_line("for (let i = start; i < end; i++) result.push(i);");
        self.emit_line("return result;");
        self.indent -= 1;
        self.emit_line("},");

        // abs function
        self.emit_line("abs: Math.abs,");

        // min function
        self.emit_line("min: Math.min,");

        // max function
        self.emit_line("max: Math.max,");

        // floor function
        self.emit_line("floor: Math.floor,");

        // ceil function
        self.emit_line("ceil: Math.ceil,");

        // round function
        self.emit_line("round: Math.round,");

        // sqrt function
        self.emit_line("sqrt: Math.sqrt,");

        // split function
        self.emit_line("split: (str, delim) => str.split(delim),");

        // join function
        self.emit_line("join: (arr, delim) => arr.join(delim),");

        // contains function
        self.emit_line("contains: (container, item) => {");
        self.indent += 1;
        self.emit_line("if (typeof container === 'string') return container.includes(item);");
        self.emit_line("if (Array.isArray(container)) return container.includes(item);");
        self.emit_line("if (typeof container === 'object') return item in container;");
        self.emit_line("return false;");
        self.indent -= 1;
        self.emit_line("},");

        // reverse function
        self.emit_line("reverse: (x) => {");
        self.indent += 1;
        self.emit_line("if (typeof x === 'string') return x.split('').reverse().join('');");
        self.emit_line("if (Array.isArray(x)) return [...x].reverse();");
        self.emit_line("throw new Error('reverse() expects a list or string');");
        self.indent -= 1;
        self.emit_line("},");

        // sort function
        self.emit_line("sort: (arr) => [...arr].sort((a, b) => {");
        self.indent += 1;
        self.emit_line("if (typeof a === 'number' && typeof b === 'number') return a - b;");
        self.emit_line("return String(a).localeCompare(String(b));");
        self.indent -= 1;
        self.emit_line("}),");

        // blether (print) function
        self.emit_line("blether: console.log,");

        // speir (input) - for Node.js
        self.emit_line("speir: (prompt) => {");
        self.indent += 1;
        self.emit_line("const fs = require('fs');");
        self.emit_line("process.stdout.write(String(prompt));");
        self.emit_line("const buf = Buffer.alloc(1024);");
        self.emit_line("const n = fs.readSync(0, buf);");
        self.emit_line("return buf.toString('utf8', 0, n).trim();");
        self.indent -= 1;
        self.emit_line("},");

        self.indent -= 1;
        self.emit_line("};");
        self.emit_line("");

        // Import runtime functions to global scope
        self.emit_line("const { len, whit_kind, tae_string, tae_int, tae_float, shove, yank, keys, values, range, abs, min, max, floor, ceil, round, sqrt, split, join, contains, reverse, sort, blether, speir } = __havers;");
        self.emit_line("");
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> HaversResult<()> {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                self.emit_indent();
                self.output.push_str(&format!("let {} = ", name));
                if let Some(init) = initializer {
                    self.compile_expr(init)?;
                } else {
                    self.output.push_str("null");
                }
                self.output.push_str(";\n");
            }

            Stmt::Expression { expr, .. } => {
                self.emit_indent();
                self.compile_expr(expr)?;
                self.output.push_str(";\n");
            }

            Stmt::Block { statements, .. } => {
                self.emit_line("{");
                self.indent += 1;
                for stmt in statements {
                    self.compile_stmt(stmt)?;
                }
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.emit_indent();
                self.output.push_str("if (");
                self.compile_expr(condition)?;
                self.output.push_str(") ");
                self.compile_stmt_inline(then_branch)?;
                if let Some(else_br) = else_branch {
                    self.output.push_str(" else ");
                    self.compile_stmt_inline(else_br)?;
                }
                self.output.push('\n');
            }

            Stmt::While {
                condition, body, ..
            } => {
                self.emit_indent();
                self.output.push_str("while (");
                self.compile_expr(condition)?;
                self.output.push_str(") ");
                self.compile_stmt_inline(body)?;
                self.output.push('\n');
            }

            Stmt::For {
                variable,
                iterable,
                body,
                ..
            } => {
                self.emit_indent();
                self.output.push_str(&format!("for (const {} of ", variable));
                self.compile_expr(iterable)?;
                self.output.push_str(") ");
                self.compile_stmt_inline(body)?;
                self.output.push('\n');
            }

            Stmt::Function {
                name,
                params,
                body,
                ..
            } => {
                self.emit_indent();
                self.output
                    .push_str(&format!("function {}({}) {{\n", name, params.join(", ")));
                self.indent += 1;
                for stmt in body {
                    self.compile_stmt(stmt)?;
                }
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::Return { value, .. } => {
                self.emit_indent();
                self.output.push_str("return");
                if let Some(expr) = value {
                    self.output.push(' ');
                    self.compile_expr(expr)?;
                }
                self.output.push_str(";\n");
            }

            Stmt::Print { value, .. } => {
                self.emit_indent();
                self.output.push_str("blether(");
                self.compile_expr(value)?;
                self.output.push_str(");\n");
            }

            Stmt::Break { .. } => {
                self.emit_line("break;");
            }

            Stmt::Continue { .. } => {
                self.emit_line("continue;");
            }

            Stmt::Class {
                name,
                superclass,
                methods,
                ..
            } => {
                self.emit_indent();
                self.output.push_str(&format!("class {}", name));
                if let Some(super_name) = superclass {
                    self.output.push_str(&format!(" extends {}", super_name));
                }
                self.output.push_str(" {\n");
                self.indent += 1;

                for method in methods {
                    if let Stmt::Function {
                        name: method_name,
                        params,
                        body,
                        ..
                    } = method
                    {
                        self.emit_indent();
                        let js_name = if method_name == "init" {
                            "constructor"
                        } else {
                            method_name
                        };
                        self.output
                            .push_str(&format!("{}({}) {{\n", js_name, params.join(", ")));
                        self.indent += 1;
                        for stmt in body {
                            self.compile_stmt(stmt)?;
                        }
                        self.indent -= 1;
                        self.emit_line("}");
                    }
                }

                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::Struct { name, fields, .. } => {
                // Compile struct as a class with a constructor
                self.emit_indent();
                self.output.push_str(&format!("class {} {{\n", name));
                self.indent += 1;
                self.emit_indent();
                self.output
                    .push_str(&format!("constructor({}) {{\n", fields.join(", ")));
                self.indent += 1;
                for field in fields {
                    self.emit_line(&format!("this.{} = {};", field, field));
                }
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::Import { path, alias, .. } => {
                let module_name = alias.clone().unwrap_or_else(|| {
                    // Extract filename from path
                    path.rsplit('/')
                        .next()
                        .unwrap_or(path)
                        .replace(".braw", "")
                        .replace(".js", "")
                });
                self.emit_line(&format!(
                    "const {} = require('{}');",
                    module_name, path
                ));
            }

            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                self.emit_indent();
                self.output.push_str("try ");
                self.compile_stmt_inline(try_block)?;
                self.output.push_str(&format!(" catch ({}) ", error_name));
                self.compile_stmt_inline(catch_block)?;
                self.output.push('\n');
            }

            Stmt::Match { value, arms, .. } => {
                // Compile match as switch or if-else chain
                self.emit_indent();
                self.output.push_str("const __match_val = ");
                self.compile_expr(value)?;
                self.output.push_str(";\n");

                for (i, arm) in arms.iter().enumerate() {
                    self.emit_indent();
                    if i == 0 {
                        self.output.push_str("if (");
                    } else {
                        self.output.push_str("} else if (");
                    }
                    self.compile_pattern(&arm.pattern, "__match_val")?;
                    self.output.push_str(") ");

                    // Bind pattern variable if identifier
                    if let Pattern::Identifier(name) = &arm.pattern {
                        self.output.push_str("{\n");
                        self.indent += 1;
                        self.emit_line(&format!("const {} = __match_val;", name));
                        self.compile_stmt(&arm.body)?;
                        self.indent -= 1;
                        self.emit_indent();
                    } else {
                        self.compile_stmt_inline(&arm.body)?;
                    }
                }

                if !arms.is_empty() {
                    self.output.push_str(" else {\n");
                    self.indent += 1;
                    self.emit_line("throw new Error('Nae match found!');");
                    self.indent -= 1;
                    self.emit_line("}");
                }
            }
        }

        Ok(())
    }

    fn compile_stmt_inline(&mut self, stmt: &Stmt) -> HaversResult<()> {
        match stmt {
            Stmt::Block { statements, .. } => {
                self.output.push_str("{\n");
                self.indent += 1;
                for s in statements {
                    self.compile_stmt(s)?;
                }
                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');
            }
            _ => {
                self.compile_stmt(stmt)?;
            }
        }
        Ok(())
    }

    fn compile_pattern(&mut self, pattern: &Pattern, match_var: &str) -> HaversResult<()> {
        match pattern {
            Pattern::Literal(lit) => {
                self.output.push_str(&format!("{} === ", match_var));
                match lit {
                    Literal::Integer(n) => self.output.push_str(&n.to_string()),
                    Literal::Float(f) => self.output.push_str(&f.to_string()),
                    Literal::String(s) => self.output.push_str(&format!("\"{}\"", s)),
                    Literal::Bool(b) => self.output.push_str(if *b { "true" } else { "false" }),
                    Literal::Nil => self.output.push_str("null"),
                }
            }
            Pattern::Identifier(_) => {
                self.output.push_str("true"); // Always matches
            }
            Pattern::Wildcard => {
                self.output.push_str("true"); // Always matches
            }
            Pattern::Range { start, end } => {
                self.output.push_str(&format!("({} >= ", match_var));
                self.compile_expr(start)?;
                self.output.push_str(&format!(" && {} < ", match_var));
                self.compile_expr(end)?;
                self.output.push(')');
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> HaversResult<()> {
        match expr {
            Expr::Literal { value, .. } => {
                match value {
                    Literal::Integer(n) => self.output.push_str(&n.to_string()),
                    Literal::Float(f) => self.output.push_str(&f.to_string()),
                    Literal::String(s) => {
                        // Escape the string properly
                        let escaped = s
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('\n', "\\n")
                            .replace('\r', "\\r")
                            .replace('\t', "\\t");
                        self.output.push_str(&format!("\"{}\"", escaped));
                    }
                    Literal::Bool(true) => self.output.push_str("true"),
                    Literal::Bool(false) => self.output.push_str("false"),
                    Literal::Nil => self.output.push_str("null"),
                }
            }

            Expr::Variable { name, .. } => {
                self.output.push_str(name);
            }

            Expr::Assign { name, value, .. } => {
                self.output.push_str(&format!("({} = ", name));
                self.compile_expr(value)?;
                self.output.push(')');
            }

            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                self.output.push('(');
                self.compile_expr(left)?;
                let op_str = match operator {
                    BinaryOp::Add => " + ",
                    BinaryOp::Subtract => " - ",
                    BinaryOp::Multiply => " * ",
                    BinaryOp::Divide => " / ",
                    BinaryOp::Modulo => " % ",
                    BinaryOp::Equal => " === ",
                    BinaryOp::NotEqual => " !== ",
                    BinaryOp::Less => " < ",
                    BinaryOp::LessEqual => " <= ",
                    BinaryOp::Greater => " > ",
                    BinaryOp::GreaterEqual => " >= ",
                };
                self.output.push_str(op_str);
                self.compile_expr(right)?;
                self.output.push(')');
            }

            Expr::Unary {
                operator, operand, ..
            } => {
                match operator {
                    UnaryOp::Negate => {
                        self.output.push_str("(-");
                        self.compile_expr(operand)?;
                        self.output.push(')');
                    }
                    UnaryOp::Not => {
                        self.output.push_str("(!");
                        self.compile_expr(operand)?;
                        self.output.push(')');
                    }
                }
            }

            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => {
                self.output.push('(');
                self.compile_expr(left)?;
                let op_str = match operator {
                    LogicalOp::And => " && ",
                    LogicalOp::Or => " || ",
                };
                self.output.push_str(op_str);
                self.compile_expr(right)?;
                self.output.push(')');
            }

            Expr::Call {
                callee, arguments, ..
            } => {
                self.compile_expr(callee)?;
                self.output.push('(');
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.compile_expr(arg)?;
                }
                self.output.push(')');
            }

            Expr::Get {
                object, property, ..
            } => {
                self.compile_expr(object)?;
                self.output.push('.');
                self.output.push_str(property);
            }

            Expr::Set {
                object,
                property,
                value,
                ..
            } => {
                self.output.push('(');
                self.compile_expr(object)?;
                self.output.push('.');
                self.output.push_str(property);
                self.output.push_str(" = ");
                self.compile_expr(value)?;
                self.output.push(')');
            }

            Expr::Index {
                object, index, ..
            } => {
                self.compile_expr(object)?;
                self.output.push('[');
                self.compile_expr(index)?;
                self.output.push(']');
            }

            Expr::List { elements, .. } => {
                self.output.push('[');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.compile_expr(elem)?;
                }
                self.output.push(']');
            }

            Expr::Dict { pairs, .. } => {
                self.output.push('{');
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.compile_expr(key)?;
                    self.output.push_str(": ");
                    self.compile_expr(value)?;
                }
                self.output.push('}');
            }

            Expr::Range { start, end, .. } => {
                self.output.push_str("__havers.range(");
                self.compile_expr(start)?;
                self.output.push_str(", ");
                self.compile_expr(end)?;
                self.output.push(')');
            }

            Expr::Grouping { expr, .. } => {
                self.output.push('(');
                self.compile_expr(expr)?;
                self.output.push(')');
            }

            Expr::Lambda { params, body, .. } => {
                self.output.push('(');
                self.output.push_str(&params.join(", "));
                self.output.push_str(") => ");
                self.compile_expr(body)?;
            }

            Expr::Masel { .. } => {
                self.output.push_str("this");
            }

            Expr::Input { prompt, .. } => {
                self.output.push_str("speir(");
                self.compile_expr(prompt)?;
                self.output.push(')');
            }
        }

        Ok(())
    }

    fn emit_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("  ");
        }
    }

    fn emit_line(&mut self, line: &str) {
        self.emit_indent();
        self.output.push_str(line);
        self.output.push('\n');
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile mdhavers source tae JavaScript
pub fn compile(source: &str) -> HaversResult<String> {
    let program = crate::parser::parse(source)?;
    let mut compiler = Compiler::new();
    compiler.compile(&program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_compile() {
        let result = compile("ken x = 5").unwrap();
        assert!(result.contains("let x = 5;"));
    }

    #[test]
    fn test_function_compile() {
        let result = compile(
            r#"
dae add(a, b) {
    gie a + b
}
"#,
        )
        .unwrap();
        assert!(result.contains("function add(a, b)"));
        assert!(result.contains("return"));
    }

    #[test]
    fn test_if_compile() {
        let result = compile(
            r#"
gin x > 5 {
    blether "big"
}
"#,
        )
        .unwrap();
        assert!(result.contains("if ("));
        assert!(result.contains("blether("));
    }

    #[test]
    fn test_class_compile() {
        let result = compile(
            r#"
kin Animal {
    dae init(name) {
        masel.name = name
    }
    dae speak() {
        blether masel.name
    }
}
"#,
        )
        .unwrap();
        assert!(result.contains("class Animal"));
        assert!(result.contains("constructor(name)"));
        assert!(result.contains("this.name"));
    }
}
