/// Pretty printer fer mdhavers code
/// Makes yer code look braw and proper!

use crate::ast::*;

/// Configuration fer the formatter
#[allow(dead_code)]
pub struct FormatterConfig {
    /// Number o' spaces fer indentation
    pub indent_size: usize,
    /// Maximum line width before we wrap
    pub max_line_width: usize,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        FormatterConfig {
            indent_size: 4,
            max_line_width: 100,
        }
    }
}

/// The formatter itself
pub struct Formatter {
    config: FormatterConfig,
    output: String,
    indent_level: usize,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter::with_config(FormatterConfig::default())
    }

    pub fn with_config(config: FormatterConfig) -> Self {
        Formatter {
            config,
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Format a whole program
    pub fn format(&mut self, program: &Program) -> String {
        self.output.clear();
        self.indent_level = 0;

        for (i, stmt) in program.statements.iter().enumerate() {
            self.format_stmt(stmt);

            // Add blank line between top-level declarations
            if i < program.statements.len() - 1 {
                // Add extra newline after functions and classes
                match stmt {
                    Stmt::Function { .. } | Stmt::Class { .. } => {
                        self.output.push('\n');
                    }
                    _ => {}
                }
            }
        }

        // Ensure file ends with newline
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.output.clone()
    }

    fn indent(&self) -> String {
        " ".repeat(self.config.indent_size * self.indent_level)
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.output.push_str(&self.indent());
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, initializer, .. } => {
                let init = if let Some(expr) = initializer {
                    format!(" = {}", self.format_expr(expr))
                } else {
                    String::new()
                };
                self.writeln(&format!("ken {}{}", name, init));
            }

            Stmt::Expression { expr, .. } => {
                self.writeln(&self.format_expr(expr));
            }

            Stmt::Block { statements, .. } => {
                self.writeln("{");
                self.indent_level += 1;
                for s in statements {
                    self.format_stmt(s);
                }
                self.indent_level -= 1;
                self.writeln("}");
            }

            Stmt::If { condition, then_branch, else_branch, .. } => {
                let cond = self.format_expr(condition);
                self.write(&self.indent());
                self.write(&format!("gin {} ", cond));
                self.format_stmt_inline(then_branch);

                if let Some(else_stmt) = else_branch {
                    self.write(" ither ");
                    self.format_stmt_inline(else_stmt);
                }
                self.output.push('\n');
            }

            Stmt::While { condition, body, .. } => {
                let cond = self.format_expr(condition);
                self.write(&self.indent());
                self.write(&format!("whiles {} ", cond));
                self.format_stmt_inline(body);
                self.output.push('\n');
            }

            Stmt::For { variable, iterable, body, .. } => {
                let iter = self.format_expr(iterable);
                self.write(&self.indent());
                self.write(&format!("fer {} in {} ", variable, iter));
                self.format_stmt_inline(body);
                self.output.push('\n');
            }

            Stmt::Function { name, params, body, .. } => {
                let params_str = self.format_params(params);
                self.writeln(&format!("dae {}({}) {{", name, params_str));
                self.indent_level += 1;
                for s in body {
                    self.format_stmt(s);
                }
                self.indent_level -= 1;
                self.writeln("}");
            }

            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.writeln(&format!("gie {}", self.format_expr(expr)));
                } else {
                    self.writeln("gie");
                }
            }

            Stmt::Print { value, .. } => {
                self.writeln(&format!("blether {}", self.format_expr(value)));
            }

            Stmt::Break { .. } => {
                self.writeln("brak");
            }

            Stmt::Continue { .. } => {
                self.writeln("haud");
            }

            Stmt::Class { name, superclass, methods, .. } => {
                let inheritance = if let Some(parent) = superclass {
                    format!(" fae {}", parent)
                } else {
                    String::new()
                };
                self.writeln(&format!("kin {}{} {{", name, inheritance));
                self.indent_level += 1;
                for (i, method) in methods.iter().enumerate() {
                    self.format_stmt(method);
                    // Add blank line between methods
                    if i < methods.len() - 1 {
                        self.output.push('\n');
                    }
                }
                self.indent_level -= 1;
                self.writeln("}");
            }

            Stmt::Struct { name, fields, .. } => {
                let fields_str = fields.join(", ");
                self.writeln(&format!("thing {} {{ {} }}", name, fields_str));
            }

            Stmt::Import { path, alias, .. } => {
                if let Some(a) = alias {
                    self.writeln(&format!("fetch \"{}\" as {}", path, a));
                } else {
                    self.writeln(&format!("fetch \"{}\"", path));
                }
            }

            Stmt::TryCatch { try_block, error_name, catch_block, .. } => {
                self.write(&self.indent());
                self.write("hae_a_bash ");
                self.format_stmt_inline(try_block);
                self.write(&format!(" gin_it_gangs_wrang {} ", error_name));
                self.format_stmt_inline(catch_block);
                self.output.push('\n');
            }

            Stmt::Match { value, arms, .. } => {
                let val = self.format_expr(value);
                self.writeln(&format!("keek {} {{", val));
                self.indent_level += 1;
                for arm in arms {
                    self.format_match_arm(arm);
                }
                self.indent_level -= 1;
                self.writeln("}");
            }

            Stmt::Assert {
                condition,
                message,
                ..
            } => {
                let cond = self.format_expr(condition);
                if let Some(msg) = message {
                    let msg_str = self.format_expr(msg);
                    self.writeln(&format!("mak_siccar {}, {}", cond, msg_str));
                } else {
                    self.writeln(&format!("mak_siccar {}", cond));
                }
            }

            Stmt::Destructure {
                patterns,
                value,
                ..
            } => {
                let patterns_str = self.format_destruct_patterns(patterns);
                let val_str = self.format_expr(value);
                self.writeln(&format!("ken [{}] = {}", patterns_str, val_str));
            }
        }
    }

    /// Format destructuring patterns
    fn format_destruct_patterns(&self, patterns: &[DestructPattern]) -> String {
        patterns
            .iter()
            .map(|p| match p {
                DestructPattern::Variable(name) => name.clone(),
                DestructPattern::Rest(name) => format!("...{}", name),
                DestructPattern::Ignore => "_".to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format a statement inline (without adding its own newline)
    fn format_stmt_inline(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { statements, .. } => {
                self.write("{\n");
                self.indent_level += 1;
                for s in statements {
                    self.format_stmt(s);
                }
                self.indent_level -= 1;
                self.write(&self.indent());
                self.write("}");
            }
            _ => {
                // For single statements, wrap in block
                self.write("{ ");
                let formatted = self.format_stmt_single(stmt);
                self.write(&formatted);
                self.write(" }");
            }
        }
    }

    /// Format a single statement without newlines
    fn format_stmt_single(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::VarDecl { name, initializer, .. } => {
                if let Some(expr) = initializer {
                    format!("ken {} = {}", name, self.format_expr(expr))
                } else {
                    format!("ken {}", name)
                }
            }
            Stmt::Expression { expr, .. } => self.format_expr(expr),
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    format!("gie {}", self.format_expr(expr))
                } else {
                    "gie".to_string()
                }
            }
            Stmt::Print { value, .. } => format!("blether {}", self.format_expr(value)),
            Stmt::Break { .. } => "brak".to_string(),
            Stmt::Continue { .. } => "haud".to_string(),
            _ => "...".to_string(), // Complex statements should use blocks
        }
    }

    fn format_match_arm(&mut self, arm: &MatchArm) {
        let pattern = self.format_pattern(&arm.pattern);
        self.write(&self.indent());
        self.write(&format!("whan {} -> ", pattern));
        self.format_stmt_inline(&arm.body);
        self.output.push('\n');
    }

    fn format_pattern(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Literal(lit) => format!("{}", lit),
            Pattern::Identifier(name) => name.clone(),
            Pattern::Wildcard => "_".to_string(),
            Pattern::Range { start, end } => {
                format!("{}..{}", self.format_expr(start), self.format_expr(end))
            }
        }
    }

    /// Format function parameters, handling default values
    fn format_params(&self, params: &[Param]) -> String {
        params
            .iter()
            .map(|p| {
                if let Some(default) = &p.default {
                    format!("{} = {}", p.name, self.format_expr(default))
                } else {
                    p.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal { value, .. } => format!("{}", value),

            Expr::Variable { name, .. } => name.clone(),

            Expr::Assign { name, value, .. } => {
                format!("{} = {}", name, self.format_expr(value))
            }

            Expr::Binary { left, operator, right, .. } => {
                format!(
                    "{} {} {}",
                    self.format_expr(left),
                    operator,
                    self.format_expr(right)
                )
            }

            Expr::Unary { operator, operand, .. } => {
                match operator {
                    UnaryOp::Not => format!("nae {}", self.format_expr(operand)),
                    UnaryOp::Negate => format!("-{}", self.format_expr(operand)),
                }
            }

            Expr::Logical { left, operator, right, .. } => {
                format!(
                    "{} {} {}",
                    self.format_expr(left),
                    operator,
                    self.format_expr(right)
                )
            }

            Expr::Call { callee, arguments, .. } => {
                let args: Vec<String> = arguments.iter().map(|a| self.format_expr(a)).collect();
                format!("{}({})", self.format_expr(callee), args.join(", "))
            }

            Expr::Get { object, property, .. } => {
                format!("{}.{}", self.format_expr(object), property)
            }

            Expr::Set { object, property, value, .. } => {
                format!(
                    "{}.{} = {}",
                    self.format_expr(object),
                    property,
                    self.format_expr(value)
                )
            }

            Expr::Index { object, index, .. } => {
                format!("{}[{}]", self.format_expr(object), self.format_expr(index))
            }

            Expr::IndexSet { object, index, value, .. } => {
                format!(
                    "{}[{}] = {}",
                    self.format_expr(object),
                    self.format_expr(index),
                    self.format_expr(value)
                )
            }

            Expr::Slice { object, start, end, step, .. } => {
                let start_str = start.as_ref().map(|s| self.format_expr(s)).unwrap_or_default();
                let end_str = end.as_ref().map(|e| self.format_expr(e)).unwrap_or_default();
                if let Some(st) = step {
                    format!("{}[{}:{}:{}]", self.format_expr(object), start_str, end_str, self.format_expr(st))
                } else {
                    format!("{}[{}:{}]", self.format_expr(object), start_str, end_str)
                }
            }

            Expr::List { elements, .. } => {
                let elems: Vec<String> = elements.iter().map(|e| self.format_expr(e)).collect();
                format!("[{}]", elems.join(", "))
            }

            Expr::Dict { pairs, .. } => {
                let kvs: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", self.format_expr(k), self.format_expr(v)))
                    .collect();
                format!("{{{}}}", kvs.join(", "))
            }

            Expr::Range { start, end, inclusive, .. } => {
                let op = if *inclusive { "..=" } else { ".." };
                format!("{}{}{}", self.format_expr(start), op, self.format_expr(end))
            }

            Expr::Grouping { expr, .. } => {
                format!("({})", self.format_expr(expr))
            }

            Expr::Lambda { params, body, .. } => {
                let params_str = params.join(", ");
                format!("|{}| {}", params_str, self.format_expr(body))
            }

            Expr::Masel { .. } => "masel".to_string(),

            Expr::Input { prompt, .. } => {
                format!("speir {}", self.format_expr(prompt))
            }

            Expr::FString { parts, .. } => {
                let mut result = String::from("f\"");
                for part in parts {
                    match part {
                        FStringPart::Text(s) => result.push_str(s),
                        FStringPart::Expr(e) => {
                            result.push('{');
                            result.push_str(&self.format_expr(e));
                            result.push('}');
                        }
                    }
                }
                result.push('"');
                result
            }

            Expr::Spread { expr, .. } => {
                format!("...{}", self.format_expr(expr))
            }

            Expr::Pipe { left, right, .. } => {
                format!("{} |> {}", self.format_expr(left), self.format_expr(right))
            }

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                format!(
                    "gin {} than {} ither {}",
                    self.format_expr(condition),
                    self.format_expr(then_expr),
                    self.format_expr(else_expr)
                )
            }
        }
    }
}

/// Format source code (convenience function)
pub fn format_source(source: &str) -> Result<String, crate::error::HaversError> {
    let program = crate::parser::parse(source)?;
    let mut formatter = Formatter::new();
    Ok(formatter.format(&program))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_format_variable() {
        let source = "ken   x   =    42";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert_eq!(result.trim(), "ken x = 42");
    }

    #[test]
    fn test_format_function() {
        let source = "dae greet(name){blether name}";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("dae greet(name) {"));
        assert!(result.contains("    blether name"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_if_else() {
        let source = "gin x > 5 {blether \"big\"} ither {blether \"wee\"}";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("gin x > 5 {"));
        assert!(result.contains("} ither {"));
    }

    #[test]
    fn test_format_class() {
        let source = "kin Animal {dae init(name){masel.name = name}}";
        let program = parse(source).unwrap();
        let mut formatter = Formatter::new();
        let result = formatter.format(&program);
        assert!(result.contains("kin Animal {"));
        assert!(result.contains("dae init(name) {"));
    }
}
