use crate::ast::*;
use crate::error::HaversResult;

/// Compiler - transpiles mdhavers tae JavaScript
pub struct Compiler {
    indent: usize,
    output: String,
    match_counter: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            indent: 0,
            output: String::new(),
            match_counter: 0,
        }
    }

    /// Compile a program tae JavaScript
    pub fn compile(&mut self, program: &Program) -> HaversResult<String> {
        self.output.clear();

        // Add runtime helpers
        self.emit_runtime();

        // Compile all statements
        for stmt in &program.statements {
            self.compile_stmt(stmt);
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

        // Scots-flavored functions

        // heid - first element
        self.emit_line("heid: (x) => {");
        self.indent += 1;
        self.emit_line("if (typeof x === 'string' || Array.isArray(x)) {");
        self.indent += 1;
        self.emit_line(
            "if (x.length === 0) throw new Error('Cannae get heid o\\' an empty list!');",
        );
        self.emit_line("return x[0];");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("throw new Error('heid() expects a list or string');");
        self.indent -= 1;
        self.emit_line("},");

        // tail - all but first
        self.emit_line("tail: (x) => {");
        self.indent += 1;
        self.emit_line("if (typeof x === 'string') return x.slice(1);");
        self.emit_line("if (Array.isArray(x)) return x.slice(1);");
        self.emit_line("throw new Error('tail() expects a list or string');");
        self.indent -= 1;
        self.emit_line("},");

        // bum - last element
        self.emit_line("bum: (x) => {");
        self.indent += 1;
        self.emit_line("if (typeof x === 'string' || Array.isArray(x)) {");
        self.indent += 1;
        self.emit_line(
            "if (x.length === 0) throw new Error('Cannae get bum o\\' an empty list!');",
        );
        self.emit_line("return x[x.length - 1];");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("throw new Error('bum() expects a list or string');");
        self.indent -= 1;
        self.emit_line("},");

        // scran - slice
        self.emit_line("scran: (x, start, end) => {");
        self.indent += 1;
        self.emit_line(
            "if (typeof x === 'string' || Array.isArray(x)) return x.slice(start, end);",
        );
        self.emit_line("throw new Error('scran() expects a list or string');");
        self.indent -= 1;
        self.emit_line("},");

        // slap - concatenate
        self.emit_line("slap: (a, b) => {");
        self.indent += 1;
        self.emit_line("if (typeof a === 'string' && typeof b === 'string') return a + b;");
        self.emit_line("if (Array.isArray(a) && Array.isArray(b)) return [...a, ...b];");
        self.emit_line("throw new Error('slap() expects two lists or two strings');");
        self.indent -= 1;
        self.emit_line("},");

        // sumaw - sum all
        self.emit_line("sumaw: (arr) => {");
        self.indent += 1;
        self.emit_line("if (!Array.isArray(arr)) throw new Error('sumaw() expects a list');");
        self.emit_line("return arr.reduce((a, b) => a + b, 0);");
        self.indent -= 1;
        self.emit_line("},");

        // coont - count occurrences
        self.emit_line("coont: (x, item) => {");
        self.indent += 1;
        self.emit_line("if (typeof x === 'string') return x.split(item).length - 1;");
        self.emit_line("if (Array.isArray(x)) return x.filter(e => e === item).length;");
        self.emit_line("throw new Error('coont() expects a list or string');");
        self.indent -= 1;
        self.emit_line("},");

        // wheesht - trim whitespace
        self.emit_line("wheesht: (str) => String(str).trim(),");

        // upper - uppercase
        self.emit_line("upper: (str) => String(str).toUpperCase(),");

        // lower - lowercase
        self.emit_line("lower: (str) => String(str).toLowerCase(),");

        // shuffle - randomly shuffle
        self.emit_line("shuffle: (arr) => {");
        self.indent += 1;
        self.emit_line("if (!Array.isArray(arr)) throw new Error('shuffle() expects a list');");
        self.emit_line("const result = [...arr];");
        self.emit_line("for (let i = result.length - 1; i > 0; i--) {");
        self.indent += 1;
        self.emit_line("const j = Math.floor(Math.random() * (i + 1));");
        self.emit_line("[result[i], result[j]] = [result[j], result[i]];");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("return result;");
        self.indent -= 1;
        self.emit_line("},");

        // slice - slice with step (fer [start:end:step] syntax)
        self.emit_line("slice: (x, start, end, step) => {");
        self.indent += 1;
        self.emit_line("const len = x.length;");
        self.emit_line("const isStr = typeof x === 'string';");
        self.emit_line("const arr = isStr ? x.split('') : x;");
        self.emit_line("if (step === 0) throw new Error('Slice step cannae be zero, ya dafty!');");
        self.emit_line("// Handle defaults based on step direction");
        self.emit_line("const s = start !== null ? (start < 0 ? Math.max(len + start, 0) : Math.min(start, len)) : (step > 0 ? 0 : len - 1);");
        self.emit_line("const e = end !== null ? (end < 0 ? Math.max(len + end, step > 0 ? 0 : -1) : Math.min(end, len)) : (step > 0 ? len : -len - 1);");
        self.emit_line("const result = [];");
        self.emit_line("if (step > 0) {");
        self.indent += 1;
        self.emit_line("for (let i = s; i < e; i += step) result.push(arr[i]);");
        self.indent -= 1;
        self.emit_line("} else {");
        self.indent += 1;
        self.emit_line("for (let i = s; i > e; i += step) result.push(arr[i]);");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("return isStr ? result.join('') : result;");
        self.indent -= 1;
        self.emit_line("},");

        // Timing functions
        self.emit_line("// Timing functions");
        self.emit_line("noo: () => Date.now(),");
        self.emit_line("tick: () => {");
        self.indent += 1;
        self.emit_line("if (typeof process !== 'undefined' && process.hrtime) {");
        self.indent += 1;
        self.emit_line("const [s, ns] = process.hrtime();");
        self.emit_line("return s * 1e9 + ns;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("return Date.now() * 1e6; // Fallback for browser");
        self.indent -= 1;
        self.emit_line("},");
        self.emit_line("bide: (ms) => {");
        self.indent += 1;
        self.emit_line("const end = Date.now() + ms;");
        self.emit_line("while (Date.now() < end) {} // Busy wait (sync)");
        self.indent -= 1;
        self.emit_line("},");

        // Higher-order functions
        self.emit_line("// Higher-order functions");
        self.emit_line("gaun: (arr, fn) => arr.map(fn),");
        self.emit_line("sieve: (arr, fn) => arr.filter(fn),");
        self.emit_line("tumble: (arr, init, fn) => arr.reduce(fn, init),");
        self.emit_line("aw: (arr, fn) => arr.every(fn),");
        self.emit_line("ony: (arr, fn) => arr.some(fn),");
        self.emit_line("hunt: (arr, fn) => arr.find(fn),");

        self.indent -= 1;
        self.emit_line("};");
        self.emit_line("");

        // Import runtime functions to global scope
        self.emit_line("const { len, whit_kind, tae_string, tae_int, tae_float, shove, yank, keys, values, range, abs, min, max, floor, ceil, round, sqrt, split, join, contains, reverse, sort, blether, speir, heid, tail, bum, scran, slap, sumaw, coont, wheesht, upper, lower, shuffle, noo, tick, bide, gaun, sieve, tumble, aw, ony, hunt } = __havers;");
        self.emit_line("");
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                self.emit_indent();
                self.output.push_str(&format!("let {} = ", name));
                if let Some(init) = initializer {
                    self.compile_expr(init);
                } else {
                    self.output.push_str("null");
                }
                self.output.push_str(";\n");
            }

            Stmt::Expression { expr, .. } => {
                self.emit_indent();
                self.compile_expr(expr);
                self.output.push_str(";\n");
            }

            Stmt::Block { statements, .. } => {
                self.emit_line("{");
                self.indent += 1;
                for stmt in statements {
                    self.compile_stmt(stmt);
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
                self.compile_expr(condition);
                self.output.push_str(") ");
                self.compile_stmt_inline(then_branch);
                if let Some(else_br) = else_branch {
                    self.output.push_str(" else ");
                    self.compile_stmt_inline(else_br);
                }
                self.output.push('\n');
            }

            Stmt::While {
                condition, body, ..
            } => {
                self.emit_indent();
                self.output.push_str("while (");
                self.compile_expr(condition);
                self.output.push_str(") ");
                self.compile_stmt_inline(body);
                self.output.push('\n');
            }

            Stmt::For {
                variable,
                iterable,
                body,
                ..
            } => {
                self.emit_indent();
                self.output
                    .push_str(&format!("for (const {} of ", variable));
                self.compile_expr(iterable);
                self.output.push_str(") ");
                self.compile_stmt_inline(body);
                self.output.push('\n');
            }

            Stmt::Function {
                name, params, body, ..
            } => {
                self.emit_indent();
                let params_str = self.compile_params(params);
                self.output
                    .push_str(&format!("function {}({}) {{\n", name, params_str));
                self.indent += 1;
                for stmt in body {
                    self.compile_stmt(stmt);
                }
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::Return { value, .. } => {
                self.emit_indent();
                self.output.push_str("return");
                if let Some(expr) = value {
                    self.output.push(' ');
                    self.compile_expr(expr);
                }
                self.output.push_str(";\n");
            }

            Stmt::Print { value, .. } => {
                self.emit_indent();
                self.output.push_str("blether(");
                self.compile_expr(value);
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
                        let params_str = self.compile_params(params);
                        self.output
                            .push_str(&format!("{}({}) {{\n", js_name, params_str));
                        self.indent += 1;
                        for stmt in body {
                            self.compile_stmt(stmt);
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
                self.emit_line(&format!("const {} = require('{}');", module_name, path));
            }

            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                self.emit_indent();
                self.output.push_str("try ");
                self.compile_stmt_inline(try_block);
                self.output.push_str(&format!(" catch ({}) ", error_name));
                self.compile_stmt_inline(catch_block);
                self.output.push('\n');
            }

            Stmt::Match { value, arms, .. } => {
                // Compile match as switch or if-else chain
                // Use unique variable name for each match statement
                let match_var = format!("__match_val_{}", self.match_counter);
                self.match_counter += 1;

                self.emit_indent();
                self.output.push_str(&format!("const {} = ", match_var));
                self.compile_expr(value);
                self.output.push_str(";\n");

                for (i, arm) in arms.iter().enumerate() {
                    self.emit_indent();
                    if i == 0 {
                        self.output.push_str("if (");
                    } else {
                        self.output.push_str("else if (");
                    }
                    self.compile_pattern(&arm.pattern, &match_var);
                    self.output.push_str(") {\n");
                    self.indent += 1;

                    // Bind pattern variable if identifier
                    if let Pattern::Identifier(name) = &arm.pattern {
                        self.emit_line(&format!("const {} = {};", name, match_var));
                    }

                    // Compile the body - unwrap block to avoid double braces
                    match &arm.body {
                        Stmt::Block { statements, .. } => {
                            for s in statements {
                                self.compile_stmt(s);
                            }
                        }
                        other => {
                            self.compile_stmt(other);
                        }
                    }

                    self.indent -= 1;
                    self.emit_indent();
                    self.output.push_str("} ");
                }

                if !arms.is_empty() {
                    self.output.push_str("else {\n");
                    self.indent += 1;
                    self.emit_line("throw new Error('Nae match found!');");
                    self.indent -= 1;
                    self.emit_line("}");
                }
            }

            Stmt::Assert {
                condition, message, ..
            } => {
                self.emit_indent();
                self.output.push_str("if (!(");
                self.compile_expr(condition);
                self.output.push_str(")) {\n");
                self.indent += 1;
                self.emit_indent();
                if let Some(msg) = message {
                    self.output.push_str("throw new Error(");
                    self.compile_expr(msg);
                    self.output.push_str(");\n");
                } else {
                    self.emit_line("throw new Error('Assertion failed');");
                }
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::Destructure {
                patterns, value, ..
            } => {
                // JavaScript destructuring: const [a, b, ...rest] = value
                self.emit_indent();
                self.output.push_str("const [");

                for (i, pattern) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    match pattern {
                        DestructPattern::Variable(name) => self.output.push_str(name),
                        DestructPattern::Rest(name) => {
                            self.output.push_str("...");
                            self.output.push_str(name);
                        }
                        DestructPattern::Ignore => self.output.push('_'),
                    }
                }

                self.output.push_str("] = ");
                self.compile_expr(value);
                self.output.push_str(";\n");
            }

            Stmt::Log { level, message, .. } => {
                // Compile log to console.error with level prefix
                let level_name = match level {
                    crate::ast::LogLevel::Wheesht => return, // Silent - no output
                    crate::ast::LogLevel::Roar => "ROAR",
                    crate::ast::LogLevel::Holler => "HOLLER",
                    crate::ast::LogLevel::Blether => "BLETHER",
                    crate::ast::LogLevel::Mutter => "MUTTER",
                    crate::ast::LogLevel::Whisper => "WHISPER",
                };
                self.emit_indent();
                self.output.push_str(&format!(
                    "console.error(`[{}] ${{new Date().toISOString()}} | ` + ",
                    level_name
                ));
                self.compile_expr(message);
                self.output.push_str(");\n");
            }

            Stmt::Hurl { message, .. } => {
                self.emit_indent();
                self.output.push_str("throw new Error(");
                self.compile_expr(message);
                self.output.push_str(");\n");
            }
        }
    }

    fn compile_stmt_inline(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { statements, .. } => {
                self.output.push_str("{\n");
                self.indent += 1;
                for s in statements {
                    self.compile_stmt(s);
                }
                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');
            }
            _ => {
                self.compile_stmt(stmt);
            }
        }
    }

    fn compile_pattern(&mut self, pattern: &Pattern, match_var: &str) {
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
                self.compile_expr(start);
                self.output.push_str(&format!(" && {} < ", match_var));
                self.compile_expr(end);
                self.output.push(')');
            }
        }
    }

    /// Compile function parameters, handling default values
    fn compile_params(&mut self, params: &[Param]) -> String {
        let mut result = Vec::new();
        for param in params {
            if let Some(default_expr) = &param.default {
                // Compile the default value
                let old_output = std::mem::take(&mut self.output);
                self.compile_expr(default_expr);
                let default_js = std::mem::replace(&mut self.output, old_output);
                result.push(format!("{} = {}", param.name, default_js));
            } else {
                result.push(param.name.clone());
            }
        }
        result.join(", ")
    }

    fn compile_expr(&mut self, expr: &Expr) {
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
                self.compile_expr(value);
                self.output.push(')');
            }

            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                self.output.push('(');
                self.compile_expr(left);
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
                self.compile_expr(right);
                self.output.push(')');
            }

            Expr::Unary {
                operator, operand, ..
            } => match operator {
                UnaryOp::Negate => {
                    self.output.push_str("(-");
                    self.compile_expr(operand);
                    self.output.push(')');
                }
                UnaryOp::Not => {
                    self.output.push_str("(!");
                    self.compile_expr(operand);
                    self.output.push(')');
                }
            },

            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => {
                self.output.push('(');
                self.compile_expr(left);
                let op_str = match operator {
                    LogicalOp::And => " && ",
                    LogicalOp::Or => " || ",
                };
                self.output.push_str(op_str);
                self.compile_expr(right);
                self.output.push(')');
            }

            Expr::Call {
                callee, arguments, ..
            } => {
                // Heuristic: If calling a variable with a capitalized name, assume it's a class constructor
                if let Expr::Variable { name, .. } = &**callee {
                    if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        self.output.push_str("new ");
                    }
                }

                self.compile_expr(callee);
                self.output.push('(');
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.compile_expr(arg);
                }
                self.output.push(')');
            }

            Expr::Get {
                object, property, ..
            } => {
                self.compile_expr(object);
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
                self.compile_expr(object);
                self.output.push('.');
                self.output.push_str(property);
                self.output.push_str(" = ");
                self.compile_expr(value);
                self.output.push(')');
            }

            Expr::Index { object, index, .. } => {
                self.compile_expr(object);
                self.output.push('[');
                self.compile_expr(index);
                self.output.push(']');
            }

            Expr::IndexSet {
                object,
                index,
                value,
                ..
            } => {
                self.output.push('(');
                self.compile_expr(object);
                self.output.push('[');
                self.compile_expr(index);
                self.output.push_str("] = ");
                self.compile_expr(value);
                self.output.push(')');
            }

            Expr::Slice {
                object,
                start,
                end,
                step,
                ..
            } => {
                // JavaScript: Use helper function fer step slices, or .slice() fer simple ones
                if let Some(st) = step {
                    // Need to use a helper function fer step slices
                    self.output.push_str("__havers.slice(");
                    self.compile_expr(object);
                    self.output.push_str(", ");
                    if let Some(s) = start {
                        self.compile_expr(s);
                    } else {
                        self.output.push_str("null");
                    }
                    self.output.push_str(", ");
                    if let Some(e) = end {
                        self.compile_expr(e);
                    } else {
                        self.output.push_str("null");
                    }
                    self.output.push_str(", ");
                    self.compile_expr(st);
                    self.output.push(')');
                } else {
                    // Simple slice: obj.slice(start, end)
                    self.compile_expr(object);
                    self.output.push_str(".slice(");
                    if let Some(s) = start {
                        self.compile_expr(s);
                    } else {
                        self.output.push('0');
                    }
                    if let Some(e) = end {
                        self.output.push_str(", ");
                        self.compile_expr(e);
                    }
                    self.output.push(')');
                }
            }

            Expr::List { elements, .. } => {
                self.output.push('[');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.compile_expr(elem);
                }
                self.output.push(']');
            }

            Expr::Dict { pairs, .. } => {
                self.output.push('{');
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.compile_expr(key);
                    self.output.push_str(": ");
                    self.compile_expr(value);
                }
                self.output.push('}');
            }

            Expr::Range { start, end, .. } => {
                self.output.push_str("__havers.range(");
                self.compile_expr(start);
                self.output.push_str(", ");
                self.compile_expr(end);
                self.output.push(')');
            }

            Expr::Grouping { expr, .. } => {
                self.output.push('(');
                self.compile_expr(expr);
                self.output.push(')');
            }

            Expr::Lambda { params, body, .. } => {
                self.output.push('(');
                self.output.push_str(&params.join(", "));
                self.output.push_str(") => ");
                self.compile_expr(body);
            }

            Expr::Masel { .. } => {
                self.output.push_str("this");
            }

            Expr::Input { prompt, .. } => {
                self.output.push_str("speir(");
                self.compile_expr(prompt);
                self.output.push(')');
            }

            Expr::FString { parts, .. } => {
                // Compile to JavaScript template literal
                self.output.push('`');
                for part in parts {
                    match part {
                        FStringPart::Text(text) => {
                            // Escape backticks in the text
                            for c in text.chars() {
                                if c == '`' {
                                    self.output.push_str("\\`");
                                } else if c == '$' {
                                    self.output.push_str("\\$");
                                } else {
                                    self.output.push(c);
                                }
                            }
                        }
                        FStringPart::Expr(expr) => {
                            self.output.push_str("${");
                            self.compile_expr(expr);
                            self.output.push('}');
                        }
                    }
                }
                self.output.push('`');
            }

            Expr::Spread { expr, .. } => {
                self.output.push_str("...");
                self.compile_expr(expr);
            }

            Expr::Pipe { left, right, .. } => {
                // In JavaScript, we transform left |> right to right(left)
                self.compile_expr(right);
                self.output.push('(');
                self.compile_expr(left);
                self.output.push(')');
            }

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                // JavaScript ternary: condition ? then : else
                self.output.push('(');
                self.compile_expr(condition);
                self.output.push_str(" ? ");
                self.compile_expr(then_expr);
                self.output.push_str(" : ");
                self.compile_expr(else_expr);
                self.output.push(')');
            }
            Expr::BlockExpr { statements, .. } => {
                // Block expressions compile to an IIFE (immediately invoked function expression)
                self.output.push_str("(() => {\n");
                self.indent += 1;
                for stmt in statements {
                    self.compile_stmt(stmt);
                }
                self.indent -= 1;
                self.emit_indent();
                self.output.push_str("})()");
            }
        }
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

    // ==================== Basic Tests ====================

    #[test]
    fn test_simple_compile() {
        let result = compile("ken x = 5").unwrap();
        assert!(result.contains("let x = 5;"));
    }

    #[test]
    fn test_var_no_initializer() {
        let result = compile("ken x").unwrap();
        assert!(result.contains("let x = null;"));
    }

    #[test]
    fn test_expression_statement() {
        let result = compile("ken x = 1\nx + 2").unwrap();
        assert!(result.contains("(x + 2);"));
    }

    // ==================== Function Tests ====================

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
    fn test_function_with_defaults() {
        let result = compile(
            r#"dae greet(name, greeting = "Hello") {
    gie greeting + name
}"#,
        )
        .unwrap();
        assert!(result.contains("greeting = \"Hello\""));
    }

    #[test]
    fn test_return_no_value() {
        let result = compile("dae foo() {\n    gie\n}").unwrap();
        assert!(result.contains("return;"));
    }

    // ==================== Control Flow Tests ====================

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
    fn test_if_else_compile() {
        let result = compile(r#"gin x > 5 { blether "big" } ither { blether "small" }"#).unwrap();
        assert!(result.contains("if ("));
        assert!(result.contains("else"));
    }

    #[test]
    fn test_while_compile() {
        let result = compile("whiles x < 10 { ken x = x + 1 }").unwrap();
        assert!(result.contains("while ("));
    }

    #[test]
    fn test_for_compile() {
        let result = compile("fer i in 0..10 { blether i }").unwrap();
        assert!(result.contains("for (const i of"));
    }

    #[test]
    fn test_break_compile() {
        let result = compile("whiles aye { brak }").unwrap();
        assert!(result.contains("break;"));
    }

    #[test]
    fn test_continue_compile() {
        let result = compile("whiles aye { haud }").unwrap();
        assert!(result.contains("continue;"));
    }

    // ==================== Block Tests ====================

    #[test]
    fn test_block_compile() {
        let result = compile("{ ken x = 1\n ken y = 2 }").unwrap();
        assert!(result.contains("let x = 1;"));
        assert!(result.contains("let y = 2;"));
    }

    // ==================== Class Tests ====================

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

    #[test]
    fn test_class_inheritance() {
        let result = compile(
            r#"kin Dog fae Animal {
    dae bark() {
        blether "woof"
    }
}"#,
        )
        .unwrap();
        assert!(result.contains("class Dog extends Animal"));
    }

    // ==================== Struct Tests ====================

    #[test]
    fn test_struct_compile() {
        let result = compile("thing Point { x, y }").unwrap();
        assert!(result.contains("class Point"));
        assert!(result.contains("constructor(x, y)"));
        assert!(result.contains("this.x = x;"));
        assert!(result.contains("this.y = y;"));
    }

    // ==================== Import Tests ====================

    #[test]
    fn test_import_compile() {
        let result = compile("fetch \"math\"").unwrap();
        assert!(result.contains("require('math')"));
    }

    #[test]
    fn test_import_with_alias() {
        let result = compile("fetch \"math\" tae m").unwrap();
        assert!(result.contains("const m = require('math')"));
    }

    // ==================== Try-Catch Tests ====================

    #[test]
    fn test_try_catch_compile() {
        let result =
            compile("hae_a_bash { ken x = 1 } gin_it_gangs_wrang e { blether e }").unwrap();
        assert!(result.contains("try {"));
        assert!(result.contains("catch (e)"));
    }

    // ==================== Match Tests ====================

    #[test]
    fn test_match_compile() {
        let result = compile(
            r#"keek x {
    whan 1 -> blether "one"
    whan 2 -> blether "two"
    whan _ -> blether "other"
}"#,
        )
        .unwrap();
        assert!(result.contains("__match_val_"));
        assert!(result.contains("if ("));
        assert!(result.contains("else if ("));
    }

    #[test]
    fn test_match_literal_patterns() {
        let result = compile(
            r#"keek x {
    whan "hello" -> blether "hi"
    whan 3.14 -> blether "pi"
    whan aye -> blether "true"
    whan naething -> blether "nil"
}"#,
        )
        .unwrap();
        assert!(result.contains("=== \"hello\""));
        assert!(result.contains("=== 3.14"));
        assert!(result.contains("=== true"));
        assert!(result.contains("=== null"));
    }

    #[test]
    fn test_match_identifier_pattern() {
        let result = compile(
            r#"keek x {
    whan value -> blether value
}"#,
        )
        .unwrap();
        // Identifier patterns bind the value
        assert!(result.contains("const value ="));
    }

    #[test]
    fn test_match_range_pattern() {
        let result = compile(
            r#"keek x {
    whan 1..10 -> blether "in range"
    whan _ -> blether "out"
}"#,
        )
        .unwrap();
        assert!(result.contains(">= "));
        assert!(result.contains("< "));
    }

    // ==================== Assert Tests ====================

    #[test]
    fn test_assert_compile() {
        let result = compile("mak_siccar x > 0").unwrap();
        assert!(result.contains("if (!("));
        assert!(result.contains("throw new Error"));
    }

    #[test]
    fn test_assert_with_message() {
        let result = compile("mak_siccar x > 0, \"x must be positive\"").unwrap();
        assert!(result.contains("throw new Error("));
        assert!(result.contains("\"x must be positive\""));
    }

    // ==================== Destructuring Tests ====================

    #[test]
    fn test_destructure_compile() {
        let result = compile("ken [a, b] = [1, 2]").unwrap();
        assert!(result.contains("const [a, b] = "));
    }

    #[test]
    fn test_destructure_rest() {
        let result = compile("ken [first, ...rest] = [1, 2, 3]").unwrap();
        assert!(result.contains("const [first, ...rest] = "));
    }

    #[test]
    fn test_destructure_ignore() {
        let result = compile("ken [_, second, _] = [1, 2, 3]").unwrap();
        assert!(result.contains("const [_, second, _] = "));
    }

    // ==================== Expression Tests ====================

    #[test]
    fn test_assignment_compile() {
        let result = compile("ken x = 1\nx = 42").unwrap();
        assert!(result.contains("(x = 42)"));
    }

    #[test]
    fn test_unary_negate() {
        let result = compile("-42").unwrap();
        assert!(result.contains("(-42)"));
    }

    #[test]
    fn test_unary_not() {
        let result = compile("nae aye").unwrap();
        assert!(result.contains("(!true)"));
    }

    #[test]
    fn test_logical_and() {
        let result = compile("aye an nae").unwrap();
        assert!(result.contains("&&"));
    }

    #[test]
    fn test_logical_or() {
        let result = compile("aye or nae").unwrap();
        assert!(result.contains("||"));
    }

    #[test]
    fn test_call_compile() {
        let result = compile("foo(1, 2, 3)").unwrap();
        assert!(result.contains("foo(1, 2, 3)"));
    }

    #[test]
    fn test_get_property() {
        let result = compile("obj.prop").unwrap();
        assert!(result.contains("obj.prop"));
    }

    #[test]
    fn test_set_property() {
        let result = compile("ken obj = {}\nobj.prop = 42").unwrap();
        assert!(result.contains("obj.prop = 42"));
    }

    #[test]
    fn test_index_compile() {
        let result = compile("list[0]").unwrap();
        assert!(result.contains("list[0]"));
    }

    #[test]
    fn test_index_set_compile() {
        let result = compile("ken list = [1,2,3]\nlist[0] = 99").unwrap();
        assert!(result.contains("list[0] = 99"));
    }

    #[test]
    fn test_slice_simple() {
        let result = compile("list[1:3]").unwrap();
        assert!(result.contains(".slice("));
    }

    #[test]
    fn test_slice_with_step() {
        let result = compile("list[::2]").unwrap();
        assert!(result.contains("__havers.slice("));
    }

    #[test]
    fn test_slice_start_only() {
        let result = compile("list[1:]").unwrap();
        assert!(result.contains(".slice(1)"));
    }

    #[test]
    fn test_list_compile() {
        let result = compile("[1, 2, 3]").unwrap();
        assert!(result.contains("[1, 2, 3]"));
    }

    #[test]
    fn test_dict_compile() {
        let result = compile("ken d = {\"a\": 1, \"b\": 2}").unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_range_compile() {
        let result = compile("0..10").unwrap();
        assert!(result.contains("__havers.range(0, 10)"));
    }

    #[test]
    fn test_grouping_compile() {
        let result = compile("(1 + 2) * 3").unwrap();
        assert!(result.contains("((1 + 2))"));
    }

    #[test]
    fn test_lambda_compile() {
        let result = compile("|x, y| x + y").unwrap();
        assert!(result.contains("(x, y) =>"));
    }

    #[test]
    fn test_masel_compile() {
        let result = compile("kin Foo { dae test() { gie masel } }").unwrap();
        assert!(result.contains("return this"));
    }

    #[test]
    fn test_input_compile() {
        let result = compile("speir \"What? \"").unwrap();
        assert!(result.contains("speir(\"What? \")"));
    }

    #[test]
    fn test_fstring_compile() {
        let result = compile("ken name = \"world\"\nf\"Hello {name}!\"").unwrap();
        assert!(result.contains("`Hello ${name}!`"));
    }

    #[test]
    fn test_fstring_escapes() {
        let result = compile("f\"cost: $5\"").unwrap();
        assert!(result.contains("`cost: \\$5`"));
    }

    #[test]
    fn test_spread_compile() {
        let result = compile("[1, ...[2, 3]]").unwrap();
        assert!(result.contains("...[2, 3]"));
    }

    #[test]
    fn test_pipe_compile() {
        let result = compile("ken dbl = |x| x * 2\n5 |> dbl").unwrap();
        assert!(result.contains("dbl(5)"));
    }

    #[test]
    fn test_ternary_compile() {
        let result = compile("ken x = gin aye than 1 ither 0").unwrap();
        assert!(result.contains("true ? 1 : 0"));
    }

    // ==================== String Escaping Tests ====================

    #[test]
    fn test_string_escapes() {
        let result = compile("ken s = \"line1\\nline2\"").unwrap();
        assert!(result.contains("\\n"));
    }

    // ==================== Compiler Default Tests ====================

    #[test]
    fn test_compiler_default() {
        let compiler = Compiler::default();
        assert_eq!(compiler.indent, 0);
        assert!(compiler.output.is_empty());
    }

    // ==================== Runtime Tests ====================

    #[test]
    fn test_runtime_emitted() {
        let result = compile("ken x = 1").unwrap();
        assert!(result.contains("const __havers = {"));
        assert!(result.contains("len:"));
        assert!(result.contains("whit_kind:"));
        assert!(result.contains("blether:"));
    }
}
