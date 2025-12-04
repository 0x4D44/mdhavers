use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;

use crate::ast::*;
use crate::error::{HaversError, HaversResult};
use crate::value::*;

/// Control flow signals
#[derive(Debug)]
enum ControlFlow {
    Return(Value),
    Break,
    Continue,
}

/// The interpreter - runs mdhavers programs
pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    output: Vec<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));

        // Define native functions
        Self::define_natives(&globals);

        Interpreter {
            globals: globals.clone(),
            environment: globals,
            output: Vec::new(),
        }
    }

    fn define_natives(globals: &Rc<RefCell<Environment>>) {
        // len - get length of list or string
        globals.borrow_mut().define(
            "len".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("len", 1, |args| {
                match &args[0] {
                    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                    Value::List(l) => Ok(Value::Integer(l.borrow().len() as i64)),
                    Value::Dict(d) => Ok(Value::Integer(d.borrow().len() as i64)),
                    _ => Err("len() expects a string, list, or dict".to_string()),
                }
            }))),
        );

        // type - get type of value (whit_kind in Scots!)
        globals.borrow_mut().define(
            "whit_kind".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("whit_kind", 1, |args| {
                Ok(Value::String(args[0].type_name().to_string()))
            }))),
        );

        // str - convert to string (tae_string in Scots!)
        globals.borrow_mut().define(
            "tae_string".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tae_string", 1, |args| {
                Ok(Value::String(format!("{}", args[0])))
            }))),
        );

        // int - convert to integer (tae_int in Scots!)
        globals.borrow_mut().define(
            "tae_int".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tae_int", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    Value::Float(f) => Ok(Value::Integer(*f as i64)),
                    Value::String(s) => s
                        .parse::<i64>()
                        .map(Value::Integer)
                        .map_err(|_| format!("Cannae turn '{}' intae an integer", s)),
                    Value::Bool(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
                    _ => Err(format!("Cannae turn {} intae an integer", args[0].type_name())),
                }
            }))),
        );

        // float - convert to float (tae_float in Scots!)
        globals.borrow_mut().define(
            "tae_float".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("tae_float", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Float(*n as f64)),
                    Value::Float(f) => Ok(Value::Float(*f)),
                    Value::String(s) => s
                        .parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| format!("Cannae turn '{}' intae a float", s)),
                    _ => Err(format!("Cannae turn {} intae a float", args[0].type_name())),
                }
            }))),
        );

        // push - add to list (shove in Scots!)
        globals.borrow_mut().define(
            "shove".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("shove", 2, |args| {
                if let Value::List(list) = &args[0] {
                    list.borrow_mut().push(args[1].clone());
                    Ok(Value::Nil)
                } else {
                    Err("shove() expects a list as first argument".to_string())
                }
            }))),
        );

        // pop - remove from list (yank in Scots!)
        globals.borrow_mut().define(
            "yank".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("yank", 1, |args| {
                if let Value::List(list) = &args[0] {
                    list.borrow_mut()
                        .pop()
                        .ok_or_else(|| "Cannae yank fae an empty list!".to_string())
                } else {
                    Err("yank() expects a list".to_string())
                }
            }))),
        );

        // keys - get dictionary keys
        globals.borrow_mut().define(
            "keys".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("keys", 1, |args| {
                if let Value::Dict(dict) = &args[0] {
                    let keys: Vec<Value> = dict
                        .borrow()
                        .keys()
                        .map(|k| Value::String(k.clone()))
                        .collect();
                    Ok(Value::List(Rc::new(RefCell::new(keys))))
                } else {
                    Err("keys() expects a dict".to_string())
                }
            }))),
        );

        // values - get dictionary values
        globals.borrow_mut().define(
            "values".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("values", 1, |args| {
                if let Value::Dict(dict) = &args[0] {
                    let vals: Vec<Value> = dict.borrow().values().cloned().collect();
                    Ok(Value::List(Rc::new(RefCell::new(vals))))
                } else {
                    Err("values() expects a dict".to_string())
                }
            }))),
        );

        // range - create a range
        globals.borrow_mut().define(
            "range".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("range", 2, |args| {
                let start = args[0]
                    .as_integer()
                    .ok_or("range() expects integers")?;
                let end = args[1]
                    .as_integer()
                    .ok_or("range() expects integers")?;
                Ok(Value::Range(RangeValue::new(start, end, false)))
            }))),
        );

        // abs - absolute value
        globals.borrow_mut().define(
            "abs".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("abs", 1, |args| {
                match &args[0] {
                    Value::Integer(n) => Ok(Value::Integer(n.abs())),
                    Value::Float(f) => Ok(Value::Float(f.abs())),
                    _ => Err("abs() expects a number".to_string()),
                }
            }))),
        );

        // min - minimum value
        globals.borrow_mut().define(
            "min".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("min", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        Ok(Value::Integer(std::cmp::min(*a, *b)))
                    }
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                    _ => Err("min() expects two numbers of the same type".to_string()),
                }
            }))),
        );

        // max - maximum value
        globals.borrow_mut().define(
            "max".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("max", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        Ok(Value::Integer(std::cmp::max(*a, *b)))
                    }
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                    _ => Err("max() expects two numbers of the same type".to_string()),
                }
            }))),
        );

        // floor
        globals.borrow_mut().define(
            "floor".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("floor", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.floor() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("floor() expects a number".to_string()),
                }
            }))),
        );

        // ceil
        globals.borrow_mut().define(
            "ceil".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("ceil", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.ceil() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("ceil() expects a number".to_string()),
                }
            }))),
        );

        // round
        globals.borrow_mut().define(
            "round".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("round", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Integer(f.round() as i64)),
                    Value::Integer(n) => Ok(Value::Integer(*n)),
                    _ => Err("round() expects a number".to_string()),
                }
            }))),
        );

        // sqrt
        globals.borrow_mut().define(
            "sqrt".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("sqrt", 1, |args| {
                match &args[0] {
                    Value::Float(f) => Ok(Value::Float(f.sqrt())),
                    Value::Integer(n) => Ok(Value::Float((*n as f64).sqrt())),
                    _ => Err("sqrt() expects a number".to_string()),
                }
            }))),
        );

        // split - split string
        globals.borrow_mut().define(
            "split".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("split", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(delim)) => {
                        let parts: Vec<Value> = s
                            .split(delim.as_str())
                            .map(|p| Value::String(p.to_string()))
                            .collect();
                        Ok(Value::List(Rc::new(RefCell::new(parts))))
                    }
                    _ => Err("split() expects two strings".to_string()),
                }
            }))),
        );

        // join - join list into string
        globals.borrow_mut().define(
            "join".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("join", 2, |args| {
                match (&args[0], &args[1]) {
                    (Value::List(list), Value::String(delim)) => {
                        let parts: Vec<String> = list
                            .borrow()
                            .iter()
                            .map(|v| format!("{}", v))
                            .collect();
                        Ok(Value::String(parts.join(delim)))
                    }
                    _ => Err("join() expects a list and a string".to_string()),
                }
            }))),
        );

        // contains - check if list/string contains value
        globals.borrow_mut().define(
            "contains".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("contains", 2, |args| {
                match &args[0] {
                    Value::List(list) => {
                        let found = list.borrow().iter().any(|v| v == &args[1]);
                        Ok(Value::Bool(found))
                    }
                    Value::String(s) => {
                        if let Value::String(needle) = &args[1] {
                            Ok(Value::Bool(s.contains(needle.as_str())))
                        } else {
                            Err("contains() on string expects a string needle".to_string())
                        }
                    }
                    Value::Dict(dict) => {
                        if let Value::String(key) = &args[1] {
                            Ok(Value::Bool(dict.borrow().contains_key(key)))
                        } else {
                            Err("contains() on dict expects a string key".to_string())
                        }
                    }
                    _ => Err("contains() expects a list, string, or dict".to_string()),
                }
            }))),
        );

        // reverse - reverse a list or string
        globals.borrow_mut().define(
            "reverse".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("reverse", 1, |args| {
                match &args[0] {
                    Value::List(list) => {
                        let mut reversed = list.borrow().clone();
                        reversed.reverse();
                        Ok(Value::List(Rc::new(RefCell::new(reversed))))
                    }
                    Value::String(s) => {
                        Ok(Value::String(s.chars().rev().collect()))
                    }
                    _ => Err("reverse() expects a list or string".to_string()),
                }
            }))),
        );

        // sort - sort a list
        globals.borrow_mut().define(
            "sort".to_string(),
            Value::NativeFunction(Rc::new(NativeFunction::new("sort", 1, |args| {
                if let Value::List(list) = &args[0] {
                    let mut sorted = list.borrow().clone();
                    sorted.sort_by(|a, b| {
                        match (a, b) {
                            (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                            (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                            (Value::String(x), Value::String(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        }
                    });
                    Ok(Value::List(Rc::new(RefCell::new(sorted))))
                } else {
                    Err("sort() expects a list".to_string())
                }
            }))),
        );
    }

    /// Run a program
    pub fn interpret(&mut self, program: &Program) -> HaversResult<Value> {
        let mut result = Value::Nil;
        for stmt in &program.statements {
            result = self.execute_stmt(stmt)?;
        }
        Ok(result)
    }

    /// Get captured output (for testing)
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    /// Clear captured output
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> HaversResult<Value> {
        match self.execute_stmt_with_control(stmt)? {
            Ok(value) => Ok(value),
            Err(ControlFlow::Return(value)) => Ok(value),
            Err(ControlFlow::Break) => Err(HaversError::BreakOutsideLoop {
                line: stmt.span().line,
            }),
            Err(ControlFlow::Continue) => Err(HaversError::ContinueOutsideLoop {
                line: stmt.span().line,
            }),
        }
    }

    fn execute_stmt_with_control(
        &mut self,
        stmt: &Stmt,
    ) -> HaversResult<Result<Value, ControlFlow>> {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                let value = if let Some(init) = initializer {
                    self.evaluate(init)?
                } else {
                    Value::Nil
                };
                self.environment.borrow_mut().define(name.clone(), value);
                Ok(Ok(Value::Nil))
            }

            Stmt::Expression { expr, .. } => {
                let value = self.evaluate(expr)?;
                Ok(Ok(value))
            }

            Stmt::Block { statements, .. } => {
                self.execute_block(statements, None)
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_value = self.evaluate(condition)?;
                if cond_value.is_truthy() {
                    self.execute_stmt_with_control(then_branch)
                } else if let Some(else_br) = else_branch {
                    self.execute_stmt_with_control(else_br)
                } else {
                    Ok(Ok(Value::Nil))
                }
            }

            Stmt::While {
                condition, body, ..
            } => {
                while self.evaluate(condition)?.is_truthy() {
                    match self.execute_stmt_with_control(body)? {
                        Ok(_) => {}
                        Err(ControlFlow::Break) => break,
                        Err(ControlFlow::Continue) => continue,
                        Err(ControlFlow::Return(v)) => return Ok(Err(ControlFlow::Return(v))),
                    }
                }
                Ok(Ok(Value::Nil))
            }

            Stmt::For {
                variable,
                iterable,
                body,
                span,
            } => {
                let iter_value = self.evaluate(iterable)?;

                let items: Vec<Value> = match iter_value {
                    Value::Range(range) => range.iter().map(Value::Integer).collect(),
                    Value::List(list) => list.borrow().clone(),
                    Value::String(s) => {
                        s.chars().map(|c| Value::String(c.to_string())).collect()
                    }
                    _ => {
                        return Err(HaversError::TypeError {
                            message: format!(
                                "Cannae iterate ower a {}",
                                iter_value.type_name()
                            ),
                            line: span.line,
                        });
                    }
                };

                for item in items {
                    self.environment
                        .borrow_mut()
                        .define(variable.clone(), item);
                    match self.execute_stmt_with_control(body)? {
                        Ok(_) => {}
                        Err(ControlFlow::Break) => break,
                        Err(ControlFlow::Continue) => continue,
                        Err(ControlFlow::Return(v)) => return Ok(Err(ControlFlow::Return(v))),
                    }
                }
                Ok(Ok(Value::Nil))
            }

            Stmt::Function {
                name,
                params,
                body,
                ..
            } => {
                let func = HaversFunction::new(
                    name.clone(),
                    params.clone(),
                    body.clone(),
                    Some(self.environment.clone()),
                );
                self.environment
                    .borrow_mut()
                    .define(name.clone(), Value::Function(Rc::new(func)));
                Ok(Ok(Value::Nil))
            }

            Stmt::Return { value, .. } => {
                let ret_val = if let Some(expr) = value {
                    self.evaluate(expr)?
                } else {
                    Value::Nil
                };
                Ok(Err(ControlFlow::Return(ret_val)))
            }

            Stmt::Print { value, .. } => {
                let val = self.evaluate(value)?;
                let output = format!("{}", val);
                println!("{}", output);
                self.output.push(output);
                Ok(Ok(Value::Nil))
            }

            Stmt::Break { .. } => Ok(Err(ControlFlow::Break)),

            Stmt::Continue { .. } => Ok(Err(ControlFlow::Continue)),

            Stmt::Class {
                name,
                superclass,
                methods,
                span,
            } => {
                let super_class = if let Some(super_name) = superclass {
                    let super_val = self
                        .environment
                        .borrow()
                        .get(super_name)
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: super_name.clone(),
                            line: span.line,
                        })?;
                    match super_val {
                        Value::Class(c) => Some(c),
                        _ => {
                            return Err(HaversError::TypeError {
                                message: format!("{} isnae a class", super_name),
                                line: span.line,
                            });
                        }
                    }
                } else {
                    None
                };

                let mut class = HaversClass::new(name.clone(), super_class);

                for method in methods {
                    if let Stmt::Function {
                        name: method_name,
                        params,
                        body,
                        ..
                    } = method
                    {
                        let func = HaversFunction::new(
                            method_name.clone(),
                            params.clone(),
                            body.clone(),
                            Some(self.environment.clone()),
                        );
                        class.methods.insert(method_name.clone(), Rc::new(func));
                    }
                }

                self.environment
                    .borrow_mut()
                    .define(name.clone(), Value::Class(Rc::new(class)));
                Ok(Ok(Value::Nil))
            }

            Stmt::Struct { name, fields, .. } => {
                let structure = HaversStruct::new(name.clone(), fields.clone());
                self.environment
                    .borrow_mut()
                    .define(name.clone(), Value::Struct(Rc::new(structure)));
                Ok(Ok(Value::Nil))
            }

            Stmt::Import { path, alias, span: _ } => {
                // For now, just a placeholder - would need file system access
                let _module_name = alias.clone().unwrap_or_else(|| path.clone());
                Err(HaversError::ModuleNotFound {
                    name: path.clone(),
                })
            }

            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                match self.execute_stmt_with_control(try_block) {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        // Bind the error to the catch variable
                        self.environment
                            .borrow_mut()
                            .define(error_name.clone(), Value::String(e.to_string()));
                        self.execute_stmt_with_control(catch_block)
                    }
                }
            }

            Stmt::Match { value, arms, span } => {
                let val = self.evaluate(value)?;

                for arm in arms {
                    if self.pattern_matches(&arm.pattern, &val)? {
                        // Bind pattern variables if needed
                        if let Pattern::Identifier(name) = &arm.pattern {
                            self.environment
                                .borrow_mut()
                                .define(name.clone(), val.clone());
                        }
                        return self.execute_stmt_with_control(&arm.body);
                    }
                }

                // No match found
                Err(HaversError::TypeError {
                    message: format!("Nae match found fer {}", val),
                    line: span.line,
                })
            }
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        env: Option<Rc<RefCell<Environment>>>,
    ) -> HaversResult<Result<Value, ControlFlow>> {
        let previous = self.environment.clone();
        let new_env = env.unwrap_or_else(|| {
            Rc::new(RefCell::new(Environment::with_enclosing(previous.clone())))
        });
        self.environment = new_env;

        let mut result = Ok(Value::Nil);
        for stmt in statements {
            match self.execute_stmt_with_control(stmt)? {
                Ok(v) => result = Ok(v),
                Err(cf) => {
                    self.environment = previous;
                    return Ok(Err(cf));
                }
            }
        }

        self.environment = previous;
        Ok(result)
    }

    fn pattern_matches(&mut self, pattern: &Pattern, value: &Value) -> HaversResult<bool> {
        match pattern {
            Pattern::Literal(lit) => {
                let lit_val = match lit {
                    Literal::Integer(n) => Value::Integer(*n),
                    Literal::Float(f) => Value::Float(*f),
                    Literal::String(s) => Value::String(s.clone()),
                    Literal::Bool(b) => Value::Bool(*b),
                    Literal::Nil => Value::Nil,
                };
                Ok(lit_val == *value)
            }
            Pattern::Identifier(_) => Ok(true), // Always matches, binds value
            Pattern::Wildcard => Ok(true),
            Pattern::Range { start, end } => {
                if let Value::Integer(n) = value {
                    let start_val = self.evaluate(start)?;
                    let end_val = self.evaluate(end)?;
                    if let (Some(s), Some(e)) = (start_val.as_integer(), end_val.as_integer()) {
                        Ok(*n >= s && *n < e)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> HaversResult<Value> {
        match expr {
            Expr::Literal { value, .. } => Ok(match value {
                Literal::Integer(n) => Value::Integer(*n),
                Literal::Float(f) => Value::Float(*f),
                Literal::String(s) => Value::String(s.clone()),
                Literal::Bool(b) => Value::Bool(*b),
                Literal::Nil => Value::Nil,
            }),

            Expr::Variable { name, span } => self
                .environment
                .borrow()
                .get(name)
                .ok_or_else(|| HaversError::UndefinedVariable {
                    name: name.clone(),
                    line: span.line,
                }),

            Expr::Assign { name, value, span } => {
                let val = self.evaluate(value)?;
                if !self.environment.borrow_mut().assign(name, val.clone()) {
                    return Err(HaversError::UndefinedVariable {
                        name: name.clone(),
                        line: span.line,
                    });
                }
                Ok(val)
            }

            Expr::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.binary_op(&left_val, operator, &right_val, span.line)
            }

            Expr::Unary {
                operator,
                operand,
                span,
            } => {
                let val = self.evaluate(operand)?;
                match operator {
                    UnaryOp::Negate => match val {
                        Value::Integer(n) => Ok(Value::Integer(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(HaversError::TypeError {
                            message: format!("Cannae negate a {}", val.type_name()),
                            line: span.line,
                        }),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
                }
            }

            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => {
                let left_val = self.evaluate(left)?;
                match operator {
                    LogicalOp::And => {
                        if !left_val.is_truthy() {
                            Ok(left_val)
                        } else {
                            self.evaluate(right)
                        }
                    }
                    LogicalOp::Or => {
                        if left_val.is_truthy() {
                            Ok(left_val)
                        } else {
                            self.evaluate(right)
                        }
                    }
                }
            }

            Expr::Call {
                callee,
                arguments,
                span,
            } => {
                // Check if this is a method call (callee is a Get expression)
                if let Expr::Get { object, property, .. } = callee.as_ref() {
                    let obj = self.evaluate(object)?;
                    if let Value::Instance(inst) = &obj {
                        // It's a method call - get the method and bind 'masel'
                        // Clone what we need to avoid holding the borrow
                        let method_opt = {
                            let borrowed = inst.borrow();
                            borrowed.class.find_method(property)
                        };
                        if let Some(method) = method_opt {
                            let mut args = Vec::new();
                            for arg in arguments {
                                args.push(self.evaluate(arg)?);
                            }
                            let env = Rc::new(RefCell::new(Environment::with_enclosing(
                                method.closure.clone().unwrap_or(self.globals.clone()),
                            )));
                            env.borrow_mut()
                                .define("masel".to_string(), Value::Instance(inst.clone()));
                            return self.call_function_with_env(&method, args, env, span.line);
                        }
                        // Check instance fields for callable values
                        let field_val_opt = {
                            let borrowed = inst.borrow();
                            borrowed.fields.get(property).cloned()
                        };
                        if let Some(field_val) = field_val_opt {
                            let mut args = Vec::new();
                            for arg in arguments {
                                args.push(self.evaluate(arg)?);
                            }
                            return self.call_value(field_val, args, span.line);
                        }
                        return Err(HaversError::UndefinedVariable {
                            name: property.clone(),
                            line: span.line,
                        });
                    }
                }

                let callee_val = self.evaluate(callee)?;
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.evaluate(arg)?);
                }
                self.call_value(callee_val, args, span.line)
            }

            Expr::Get {
                object,
                property,
                span,
            } => {
                let obj = self.evaluate(object)?;
                match obj {
                    Value::Instance(inst) => inst
                        .borrow()
                        .get(property)
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: property.clone(),
                            line: span.line,
                        }),
                    Value::Dict(dict) => dict
                        .borrow()
                        .get(property)
                        .cloned()
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: property.clone(),
                            line: span.line,
                        }),
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae access property '{}' on a {}",
                            property,
                            obj.type_name()
                        ),
                        line: span.line,
                    }),
                }
            }

            Expr::Set {
                object,
                property,
                value,
                span,
            } => {
                let obj = self.evaluate(object)?;
                let val = self.evaluate(value)?;
                match obj {
                    Value::Instance(inst) => {
                        inst.borrow_mut().set(property.clone(), val.clone());
                        Ok(val)
                    }
                    Value::Dict(dict) => {
                        dict.borrow_mut().insert(property.clone(), val.clone());
                        Ok(val)
                    }
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae set property '{}' on a {}",
                            property,
                            obj.type_name()
                        ),
                        line: span.line,
                    }),
                }
            }

            Expr::Index {
                object,
                index,
                span,
            } => {
                let obj = self.evaluate(object)?;
                let idx = self.evaluate(index)?;
                match (&obj, &idx) {
                    (Value::List(list), Value::Integer(i)) => {
                        let list = list.borrow();
                        let idx = if *i < 0 {
                            list.len() as i64 + *i
                        } else {
                            *i
                        };
                        list.get(idx as usize)
                            .cloned()
                            .ok_or_else(|| HaversError::IndexOutOfBounds {
                                index: *i,
                                size: list.len(),
                                line: span.line,
                            })
                    }
                    (Value::String(s), Value::Integer(i)) => {
                        let idx = if *i < 0 {
                            s.len() as i64 + *i
                        } else {
                            *i
                        };
                        s.chars()
                            .nth(idx as usize)
                            .map(|c| Value::String(c.to_string()))
                            .ok_or_else(|| HaversError::IndexOutOfBounds {
                                index: *i,
                                size: s.len(),
                                line: span.line,
                            })
                    }
                    (Value::Dict(dict), Value::String(key)) => dict
                        .borrow()
                        .get(key)
                        .cloned()
                        .ok_or_else(|| HaversError::UndefinedVariable {
                            name: key.clone(),
                            line: span.line,
                        }),
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae index a {} wi' a {}",
                            obj.type_name(),
                            idx.type_name()
                        ),
                        line: span.line,
                    }),
                }
            }

            Expr::IndexSet {
                object,
                index,
                value,
                span,
            } => {
                let obj = self.evaluate(object)?;
                let idx = self.evaluate(index)?;
                let val = self.evaluate(value)?;

                match (&obj, &idx) {
                    (Value::List(list), Value::Integer(i)) => {
                        let mut list_mut = list.borrow_mut();
                        let idx = if *i < 0 {
                            list_mut.len() as i64 + *i
                        } else {
                            *i
                        };
                        if idx < 0 || idx as usize >= list_mut.len() {
                            return Err(HaversError::IndexOutOfBounds {
                                index: *i,
                                size: list_mut.len(),
                                line: span.line,
                            });
                        }
                        list_mut[idx as usize] = val.clone();
                        Ok(val)
                    }
                    (Value::Dict(dict), Value::String(key)) => {
                        dict.borrow_mut().insert(key.clone(), val.clone());
                        Ok(val)
                    }
                    (Value::Dict(dict), key) => {
                        // Convert non-string key to string
                        let key_str = format!("{}", key);
                        dict.borrow_mut().insert(key_str, val.clone());
                        Ok(val)
                    }
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae set index on a {} wi' a {}",
                            obj.type_name(),
                            idx.type_name()
                        ),
                        line: span.line,
                    }),
                }
            }

            Expr::List { elements, .. } => {
                let mut items = Vec::new();
                for elem in elements {
                    items.push(self.evaluate(elem)?);
                }
                Ok(Value::List(Rc::new(RefCell::new(items))))
            }

            Expr::Dict { pairs, .. } => {
                let mut map = HashMap::new();
                for (key, value) in pairs {
                    let k = self.evaluate(key)?;
                    let v = self.evaluate(value)?;
                    let key_str = match k {
                        Value::String(s) => s,
                        _ => format!("{}", k),
                    };
                    map.insert(key_str, v);
                }
                Ok(Value::Dict(Rc::new(RefCell::new(map))))
            }

            Expr::Range {
                start,
                end,
                inclusive,
                ..
            } => {
                let start_val = self.evaluate(start)?;
                let end_val = self.evaluate(end)?;
                match (start_val.as_integer(), end_val.as_integer()) {
                    (Some(s), Some(e)) => Ok(Value::Range(RangeValue::new(s, e, *inclusive))),
                    _ => Err(HaversError::TypeError {
                        message: "Range bounds must be integers".to_string(),
                        line: expr.span().line,
                    }),
                }
            }

            Expr::Grouping { expr, .. } => self.evaluate(expr),

            Expr::Lambda {
                params,
                body,
                span,
            } => {
                // Create a function from the lambda
                let func = HaversFunction::new(
                    "<lambda>".to_string(),
                    params.clone(),
                    vec![Stmt::Return {
                        value: Some((**body).clone()),
                        span: *span,
                    }],
                    Some(self.environment.clone()),
                );
                Ok(Value::Function(Rc::new(func)))
            }

            Expr::Masel { span } => {
                self.environment
                    .borrow()
                    .get("masel")
                    .ok_or_else(|| HaversError::UndefinedVariable {
                        name: "masel".to_string(),
                        line: span.line,
                    })
            }

            Expr::Input { prompt, span: _ } => {
                let prompt_val = self.evaluate(prompt)?;
                print!("{}", prompt_val);
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| HaversError::InternalError(e.to_string()))?;

                Ok(Value::String(input.trim().to_string()))
            }
        }
    }

    fn binary_op(
        &self,
        left: &Value,
        op: &BinaryOp,
        right: &Value,
        line: usize,
    ) -> HaversResult<Value> {
        match op {
            BinaryOp::Add => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => {
                    Ok(Value::String(format!("{}{}", a, b)))
                }
                (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, b))),
                (a, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (Value::List(a), Value::List(b)) => {
                    let mut result = a.borrow().clone();
                    result.extend(b.borrow().clone());
                    Ok(Value::List(Rc::new(RefCell::new(result))))
                }
                _ => Err(HaversError::TypeError {
                    message: format!(
                        "Cannae add {} an' {}",
                        left.type_name(),
                        right.type_name()
                    ),
                    line,
                }),
            },

            BinaryOp::Subtract => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(HaversError::TypeError {
                    message: format!(
                        "Cannae subtract {} fae {}",
                        right.type_name(),
                        left.type_name()
                    ),
                    line,
                }),
            },

            BinaryOp::Multiply => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
                (Value::String(s), Value::Integer(n)) | (Value::Integer(n), Value::String(s)) => {
                    Ok(Value::String(s.repeat(*n as usize)))
                }
                _ => Err(HaversError::TypeError {
                    message: format!(
                        "Cannae multiply {} by {}",
                        left.type_name(),
                        right.type_name()
                    ),
                    line,
                }),
            },

            BinaryOp::Divide => {
                // Check for division by zero
                match right {
                    Value::Integer(0) => return Err(HaversError::DivisionByZero { line }),
                    Value::Float(f) if *f == 0.0 => {
                        return Err(HaversError::DivisionByZero { line })
                    }
                    _ => {}
                }
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                    (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                    (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae divide {} by {}",
                            left.type_name(),
                            right.type_name()
                        ),
                        line,
                    }),
                }
            }

            BinaryOp::Modulo => {
                match right {
                    Value::Integer(0) => return Err(HaversError::DivisionByZero { line }),
                    _ => {}
                }
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
                    _ => Err(HaversError::TypeError {
                        message: format!(
                            "Cannae get remainder o' {} by {}",
                            left.type_name(),
                            right.type_name()
                        ),
                        line,
                    }),
                }
            }

            BinaryOp::Equal => Ok(Value::Bool(left == right)),
            BinaryOp::NotEqual => Ok(Value::Bool(left != right)),

            BinaryOp::Less => self.compare(left, right, |a, b| a < b, line),
            BinaryOp::LessEqual => self.compare(left, right, |a, b| a <= b, line),
            BinaryOp::Greater => self.compare(left, right, |a, b| a > b, line),
            BinaryOp::GreaterEqual => self.compare(left, right, |a, b| a >= b, line),
        }
    }

    fn compare<F>(&self, left: &Value, right: &Value, cmp: F, line: usize) -> HaversResult<Value>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Bool(cmp(*a as f64, *b as f64)))
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(cmp(*a, *b))),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Bool(cmp(*a as f64, *b))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Bool(cmp(*a, *b as f64))),
            (Value::String(a), Value::String(b)) => Ok(Value::Bool(cmp(
                a.len() as f64,
                b.len() as f64,
            ))),
            _ => Err(HaversError::TypeError {
                message: format!(
                    "Cannae compare {} wi' {}",
                    left.type_name(),
                    right.type_name()
                ),
                line,
            }),
        }
    }

    fn call_value(&mut self, callee: Value, args: Vec<Value>, line: usize) -> HaversResult<Value> {
        match callee {
            Value::Function(func) => self.call_function(&func, args, line),
            Value::NativeFunction(native) => {
                if args.len() != native.arity {
                    return Err(HaversError::WrongArity {
                        name: native.name.clone(),
                        expected: native.arity,
                        got: args.len(),
                        line,
                    });
                }
                (native.func)(args).map_err(|e| HaversError::InternalError(e))
            }
            Value::Class(class) => {
                // Create new instance
                let instance = Rc::new(RefCell::new(HaversInstance::new(class.clone())));

                // Call init if it exists
                if let Some(init) = class.find_method("init") {
                    let env = Rc::new(RefCell::new(Environment::with_enclosing(
                        init.closure.clone().unwrap_or(self.globals.clone()),
                    )));
                    env.borrow_mut()
                        .define("masel".to_string(), Value::Instance(instance.clone()));
                    self.call_function_with_env(&init, args, env, line)?;
                }

                Ok(Value::Instance(instance))
            }
            Value::Struct(structure) => {
                // Create instance with fields
                if args.len() != structure.fields.len() {
                    return Err(HaversError::WrongArity {
                        name: structure.name.clone(),
                        expected: structure.fields.len(),
                        got: args.len(),
                        line,
                    });
                }

                let mut fields = HashMap::new();
                for (field, value) in structure.fields.iter().zip(args) {
                    fields.insert(field.clone(), value);
                }

                // Return as a dict for now
                Ok(Value::Dict(Rc::new(RefCell::new(fields))))
            }
            _ => Err(HaversError::NotCallable {
                name: format!("{}", callee),
                line,
            }),
        }
    }

    fn call_function(
        &mut self,
        func: &HaversFunction,
        args: Vec<Value>,
        line: usize,
    ) -> HaversResult<Value> {
        if args.len() != func.params.len() {
            return Err(HaversError::WrongArity {
                name: func.name.clone(),
                expected: func.params.len(),
                got: args.len(),
                line,
            });
        }

        let env = Rc::new(RefCell::new(Environment::with_enclosing(
            func.closure.clone().unwrap_or(self.globals.clone()),
        )));

        self.call_function_with_env(func, args, env, line)
    }

    fn call_function_with_env(
        &mut self,
        func: &HaversFunction,
        args: Vec<Value>,
        env: Rc<RefCell<Environment>>,
        _line: usize,
    ) -> HaversResult<Value> {
        // Bind parameters
        for (param, arg) in func.params.iter().zip(args) {
            env.borrow_mut().define(param.clone(), arg);
        }

        match self.execute_block(&func.body, Some(env))? {
            Ok(v) => Ok(v),
            Err(ControlFlow::Return(v)) => Ok(v),
            Err(ControlFlow::Break) => Ok(Value::Nil),
            Err(ControlFlow::Continue) => Ok(Value::Nil),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn run(source: &str) -> HaversResult<Value> {
        let program = parse(source)?;
        let mut interp = Interpreter::new();
        interp.interpret(&program)
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(run("5 + 3").unwrap(), Value::Integer(8));
        assert_eq!(run("10 - 4").unwrap(), Value::Integer(6));
        assert_eq!(run("3 * 4").unwrap(), Value::Integer(12));
        assert_eq!(run("15 / 3").unwrap(), Value::Integer(5));
        assert_eq!(run("17 % 5").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_variables() {
        assert_eq!(run("ken x = 5\nx").unwrap(), Value::Integer(5));
        assert_eq!(run("ken x = 5\nx = 10\nx").unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_strings() {
        assert_eq!(
            run(r#""Hello" + " " + "World""#).unwrap(),
            Value::String("Hello World".to_string())
        );
        assert_eq!(
            run(r#""ha" * 3"#).unwrap(),
            Value::String("hahaha".to_string())
        );
    }

    #[test]
    fn test_booleans() {
        assert_eq!(run("aye").unwrap(), Value::Bool(true));
        assert_eq!(run("nae").unwrap(), Value::Bool(false));
        assert_eq!(run("5 > 3").unwrap(), Value::Bool(true));
        assert_eq!(run("5 < 3").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_if_statement() {
        let result = run(
            r#"
ken x = 10
ken result = 0
gin x > 5 {
    result = 1
} ither {
    result = 2
}
result
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_while_loop() {
        let result = run(
            r#"
ken sum = 0
ken i = 1
whiles i <= 5 {
    sum = sum + i
    i = i + 1
}
sum
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_for_loop() {
        let result = run(
            r#"
ken sum = 0
fer i in 1..6 {
    sum = sum + i
}
sum
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_function() {
        let result = run(
            r#"
dae add(a, b) {
    gie a + b
}
add(3, 4)
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_recursion() {
        let result = run(
            r#"
dae factorial(n) {
    gin n <= 1 {
        gie 1
    }
    gie n * factorial(n - 1)
}
factorial(5)
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(120));
    }

    #[test]
    fn test_list() {
        let result = run(
            r#"
ken arr = [1, 2, 3]
arr[1]
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_dict() {
        let result = run(
            r#"
ken d = {"a": 1, "b": 2}
d["a"]
"#,
        )
        .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_native_functions() {
        assert_eq!(run("len([1, 2, 3])").unwrap(), Value::Integer(3));
        assert_eq!(
            run(r#"len("hello")"#).unwrap(),
            Value::Integer(5)
        );
    }

    #[test]
    fn test_division_by_zero() {
        assert!(run("5 / 0").is_err());
    }

    #[test]
    fn test_undefined_variable() {
        assert!(run("undefined_var").is_err());
    }
}
