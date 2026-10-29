//! Conservative, syntax-only eligibility for local slots in `main`.
//!
//! A name mentioned anywhere in a callable or class (including defaults and nested
//! declarations) cannot become a main-local. Shadowing deliberately does not matter.
//! Any import disables the transformation: imported code shares the global namespace.

use std::collections::HashSet;

use crate::ast::{DestructPattern, Expr, FStringPart, Pattern, Program, Stmt};

pub(super) fn main_local_names(program: &Program) -> HashSet<String> {
    let mut scan = Scan::default();
    for stmt in &program.statements {
        scan.stmt(stmt, false);
    }
    if scan.fetch {
        return HashSet::new();
    }
    scan.candidates
        .difference(&scan.nonlocal)
        .cloned()
        .collect()
}

#[derive(Default)]
struct Scan {
    fetch: bool,
    nonlocal: HashSet<String>,
    candidates: HashSet<String>,
}

impl Scan {
    fn name(&mut self, name: &str, nonlocal: bool) {
        if nonlocal {
            self.nonlocal.insert(name.to_string());
        }
    }

    fn stmt(&mut self, stmt: &Stmt, nonlocal: bool) {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                if !nonlocal {
                    self.candidates.insert(name.clone());
                }
                self.name(name, nonlocal);
                if let Some(expr) = initializer {
                    self.expr(expr, nonlocal);
                }
            }
            Stmt::Expression { expr, .. }
            | Stmt::Print { value: expr, .. }
            | Stmt::Hurl { message: expr, .. } => self.expr(expr, nonlocal),
            Stmt::Block { statements, .. } => {
                for stmt in statements {
                    self.stmt(stmt, nonlocal);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.expr(condition, nonlocal);
                self.stmt(then_branch, nonlocal);
                if let Some(stmt) = else_branch {
                    self.stmt(stmt, nonlocal);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.expr(condition, nonlocal);
                self.stmt(body, nonlocal);
            }
            Stmt::For {
                variable,
                iterable,
                body,
                ..
            } => {
                self.name(variable, nonlocal);
                self.expr(iterable, nonlocal);
                self.stmt(body, nonlocal);
            }
            Stmt::Function {
                name, params, body, ..
            } => {
                self.name(name, nonlocal);
                for param in params {
                    self.name(&param.name, true);
                    if let Some(default) = &param.default {
                        self.expr(default, true);
                    }
                }
                for stmt in body {
                    self.stmt(stmt, true);
                }
            }
            Stmt::Class {
                name,
                superclass,
                methods,
                ..
            } => {
                self.name(name, nonlocal);
                if let Some(name) = superclass {
                    self.name(name, true);
                }
                for stmt in methods {
                    self.stmt(stmt, true);
                }
            }
            Stmt::Struct { name, fields, .. } => {
                self.name(name, nonlocal);
                for field in fields {
                    self.name(field, true);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.expr(expr, nonlocal);
                }
            }
            Stmt::Import { .. } => self.fetch = true,
            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                self.stmt(try_block, nonlocal);
                self.name(error_name, nonlocal);
                self.stmt(catch_block, nonlocal);
            }
            Stmt::Match { value, arms, .. } => {
                self.expr(value, nonlocal);
                for arm in arms {
                    match &arm.pattern {
                        Pattern::Identifier(name) => self.name(name, nonlocal),
                        Pattern::Range { start, end, .. } => {
                            self.expr(start, nonlocal);
                            self.expr(end, nonlocal);
                        }
                        Pattern::Literal(_) | Pattern::Wildcard => {}
                    }
                    self.stmt(&arm.body, nonlocal);
                }
            }
            Stmt::Assert {
                condition, message, ..
            } => {
                self.expr(condition, nonlocal);
                if let Some(expr) = message {
                    self.expr(expr, nonlocal);
                }
            }
            Stmt::Destructure {
                patterns, value, ..
            } => {
                for pattern in patterns {
                    match pattern {
                        DestructPattern::Variable(name) | DestructPattern::Rest(name) => {
                            self.name(name, nonlocal);
                        }
                        DestructPattern::Ignore => {}
                    }
                }
                self.expr(value, nonlocal);
            }
            Stmt::Log {
                message, extras, ..
            } => {
                self.expr(message, nonlocal);
                for expr in extras {
                    self.expr(expr, nonlocal);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }

    fn expr(&mut self, expr: &Expr, nonlocal: bool) {
        match expr {
            Expr::Variable { name, .. } => self.name(name, nonlocal),
            Expr::Assign { name, value, .. } => {
                self.name(name, nonlocal);
                self.expr(value, nonlocal);
            }
            Expr::Binary { left, right, .. }
            | Expr::Logical { left, right, .. }
            | Expr::Range {
                start: left,
                end: right,
                ..
            }
            | Expr::Pipe { left, right, .. } => {
                self.expr(left, nonlocal);
                self.expr(right, nonlocal);
            }
            Expr::Unary { operand, .. }
            | Expr::Grouping { expr: operand, .. }
            | Expr::Spread { expr: operand, .. }
            | Expr::Input {
                prompt: operand, ..
            } => self.expr(operand, nonlocal),
            Expr::Call {
                callee, arguments, ..
            } => {
                self.expr(callee, nonlocal);
                for arg in arguments {
                    self.expr(arg, nonlocal);
                }
            }
            Expr::Get { object, .. } => self.expr(object, nonlocal),
            Expr::Set { object, value, .. } => {
                self.expr(object, nonlocal);
                self.expr(value, nonlocal);
            }
            Expr::Index { object, index, .. } => {
                self.expr(object, nonlocal);
                self.expr(index, nonlocal);
            }
            Expr::IndexSet {
                object,
                index,
                value,
                ..
            } => {
                self.expr(object, nonlocal);
                self.expr(index, nonlocal);
                self.expr(value, nonlocal);
            }
            Expr::Slice {
                object,
                start,
                end,
                step,
                ..
            } => {
                self.expr(object, nonlocal);
                for expr in [start, end, step].into_iter().flatten() {
                    self.expr(expr, nonlocal);
                }
            }
            Expr::List { elements, .. } => {
                for expr in elements {
                    self.expr(expr, nonlocal);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.expr(key, nonlocal);
                    self.expr(value, nonlocal);
                }
            }
            Expr::Lambda { params, body, .. } => {
                for param in params {
                    self.name(param, true);
                }
                self.expr(body, true);
            }
            Expr::BlockExpr { statements, .. } => {
                for stmt in statements {
                    self.stmt(stmt, nonlocal);
                }
            }
            Expr::FString { parts, .. } => {
                for part in parts {
                    if let FStringPart::Expr(expr) = part {
                        self.expr(expr, nonlocal);
                    }
                }
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.expr(condition, nonlocal);
                self.expr(then_expr, nonlocal);
                self.expr(else_expr, nonlocal);
            }
            Expr::Literal { .. } | Expr::Masel { .. } => {}
        }
    }
}
