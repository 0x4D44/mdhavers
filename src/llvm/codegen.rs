//! LLVM Code Generation
//!
//! Compiles mdhavers AST to LLVM IR.

use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};

use crate::ast::{BinaryOp, Expr, Literal, LogicalOp, Program, Stmt, UnaryOp};
use crate::error::HaversError;

use super::builtins;
use super::runtime::RuntimeFunctions;
use super::types::{MdhTypes, ValueTag};

/// Loop context for break/continue
struct LoopContext<'ctx> {
    /// Block to jump to on break
    break_block: BasicBlock<'ctx>,
    /// Block to jump to on continue
    continue_block: BasicBlock<'ctx>,
}

/// Main code generator
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    types: MdhTypes<'ctx>,
    runtime: RuntimeFunctions<'ctx>,

    /// Current function being compiled
    current_function: Option<FunctionValue<'ctx>>,

    /// Variable storage (name -> alloca pointer)
    variables: HashMap<String, PointerValue<'ctx>>,

    /// User-defined functions
    functions: HashMap<String, FunctionValue<'ctx>>,

    /// Loop context stack for break/continue
    loop_stack: Vec<LoopContext<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let types = MdhTypes::new(context);
        let runtime = RuntimeFunctions::declare(&module, &types);

        CodeGen {
            context,
            module,
            builder,
            types,
            runtime,
            current_function: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }

    /// Get the compiled module
    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Compile a complete program
    pub fn compile(&mut self, program: &Program) -> Result<(), HaversError> {
        // First pass: declare all functions
        for stmt in &program.statements {
            if let Stmt::Function { name, params, .. } = stmt {
                self.declare_function(name, params.len())?;
            }
        }

        // Create main function
        let main_fn_type = self.types.i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);
        self.current_function = Some(main_fn);

        // Compile all statements
        for stmt in &program.statements {
            self.compile_stmt(stmt)?;
        }

        // Return 0 from main
        self.builder
            .build_return(Some(&self.types.i32_type.const_int(0, false)))
            .map_err(|e| HaversError::CompileError(format!("Failed to build return: {}", e)))?;

        Ok(())
    }

    /// Declare a function (first pass)
    fn declare_function(&mut self, name: &str, param_count: usize) -> Result<(), HaversError> {
        let param_types: Vec<_> = (0..param_count)
            .map(|_| self.types.value_type.into())
            .collect();

        let fn_type = self.types.value_type.fn_type(&param_types, false);
        let function = self.module.add_function(name, fn_type, None);
        self.functions.insert(name.to_string(), function);
        Ok(())
    }

    /// Compile a statement
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), HaversError> {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                let value = if let Some(init) = initializer {
                    self.compile_expr(init)?
                } else {
                    self.make_nil()
                };

                // Allocate space for the variable
                let alloca = self.create_entry_block_alloca(name);
                self.builder
                    .build_store(alloca, value)
                    .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;
                self.variables.insert(name.clone(), alloca);
                Ok(())
            }

            Stmt::Expression { expr, .. } => {
                self.compile_expr(expr)?;
                Ok(())
            }

            Stmt::Block { statements, .. } => {
                for s in statements {
                    self.compile_stmt(s)?;
                }
                Ok(())
            }

            Stmt::Print { value, .. } => {
                let val = self.compile_expr(value)?;
                self.builder
                    .build_call(self.runtime.blether, &[val.into()], "")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to call blether: {}", e))
                    })?;
                Ok(())
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.compile_if(condition, then_branch, else_branch.as_deref()),

            Stmt::While {
                condition, body, ..
            } => self.compile_while(condition, body),

            Stmt::For {
                variable,
                iterable,
                body,
                ..
            } => self.compile_for(variable, iterable, body),

            Stmt::Function {
                name, params, body, ..
            } => self.compile_function(name, params, body),

            Stmt::Return { value, .. } => {
                let ret_val = if let Some(v) = value {
                    self.compile_expr(v)?
                } else {
                    self.make_nil()
                };
                self.builder.build_return(Some(&ret_val)).map_err(|e| {
                    HaversError::CompileError(format!("Failed to build return: {}", e))
                })?;
                Ok(())
            }

            Stmt::Break { .. } => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.builder
                        .build_unconditional_branch(loop_ctx.break_block)
                        .map_err(|e| {
                            HaversError::CompileError(format!("Failed to build break: {}", e))
                        })?;
                    Ok(())
                } else {
                    Err(HaversError::CompileError("Break outside loop".to_string()))
                }
            }

            Stmt::Continue { .. } => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.builder
                        .build_unconditional_branch(loop_ctx.continue_block)
                        .map_err(|e| {
                            HaversError::CompileError(format!("Failed to build continue: {}", e))
                        })?;
                    Ok(())
                } else {
                    Err(HaversError::CompileError(
                        "Continue outside loop".to_string(),
                    ))
                }
            }

            // Not yet implemented
            Stmt::Class { .. }
            | Stmt::Struct { .. }
            | Stmt::Import { .. }
            | Stmt::TryCatch { .. }
            | Stmt::Match { .. }
            | Stmt::Assert { .. }
            | Stmt::Destructure { .. } => Err(HaversError::CompileError(format!(
                "Statement not yet supported in LLVM backend: {:?}",
                stmt
            ))),
        }
    }

    /// Compile an expression, returning an MdhValue
    fn compile_expr(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>, HaversError> {
        match expr {
            Expr::Literal { value, .. } => self.compile_literal(value),

            Expr::Variable { name, .. } => {
                if let Some(&alloca) = self.variables.get(name) {
                    let val = self
                        .builder
                        .build_load(self.types.value_type, alloca, name)
                        .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;
                    Ok(val)
                } else {
                    Err(HaversError::CompileError(format!(
                        "Undefined variable: {}",
                        name
                    )))
                }
            }

            Expr::Assign { name, value, .. } => {
                let val = self.compile_expr(value)?;
                if let Some(&alloca) = self.variables.get(name) {
                    self.builder.build_store(alloca, val).map_err(|e| {
                        HaversError::CompileError(format!("Failed to store: {}", e))
                    })?;
                    Ok(val)
                } else {
                    Err(HaversError::CompileError(format!(
                        "Undefined variable: {}",
                        name
                    )))
                }
            }

            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => self.compile_binary(left, *operator, right),

            Expr::Unary {
                operator, operand, ..
            } => self.compile_unary(*operator, operand),

            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => self.compile_logical(left, *operator, right),

            Expr::Call {
                callee, arguments, ..
            } => self.compile_call(callee, arguments),

            Expr::List { elements, .. } => self.compile_list(elements),

            Expr::Index { object, index, .. } => self.compile_index(object, index),

            Expr::IndexSet {
                object,
                index,
                value,
                ..
            } => self.compile_index_set(object, index, value),

            Expr::Range { start, end, .. } => {
                // For now, just compile as a tuple-like structure
                // A proper implementation would create a range object
                let _start_val = self.compile_expr(start)?;
                let _end_val = self.compile_expr(end)?;
                // TODO: Create proper range object
                Ok(self.make_nil())
            }

            Expr::Grouping { expr, .. } => self.compile_expr(expr),

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => self.compile_ternary(condition, then_expr, else_expr),

            Expr::Input { prompt, .. } => {
                let prompt_val = self.compile_expr(prompt)?;
                let result = self
                    .builder
                    .build_call(self.runtime.speir, &[prompt_val.into()], "input")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to call speir: {}", e))
                    })?;
                Ok(result.try_as_basic_value().left().unwrap())
            }

            // Not yet implemented
            Expr::Get { .. }
            | Expr::Set { .. }
            | Expr::Slice { .. }
            | Expr::Dict { .. }
            | Expr::Lambda { .. }
            | Expr::Masel { .. }
            | Expr::FString { .. }
            | Expr::Spread { .. }
            | Expr::Pipe { .. } => Err(HaversError::CompileError(format!(
                "Expression not yet supported in LLVM backend: {:?}",
                expr
            ))),
        }
    }

    /// Compile a literal value
    fn compile_literal(&mut self, literal: &Literal) -> Result<BasicValueEnum<'ctx>, HaversError> {
        match literal {
            Literal::Nil => Ok(self.make_nil()),

            Literal::Bool(b) => {
                let bool_val = self.types.bool_type.const_int(*b as u64, false);
                let result = self
                    .builder
                    .build_call(self.runtime.make_bool, &[bool_val.into()], "bool")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to make bool: {}", e))
                    })?;
                Ok(result.try_as_basic_value().left().unwrap())
            }

            Literal::Integer(n) => {
                let int_val = self.types.i64_type.const_int(*n as u64, true);
                let result = self
                    .builder
                    .build_call(self.runtime.make_int, &[int_val.into()], "int")
                    .map_err(|e| HaversError::CompileError(format!("Failed to make int: {}", e)))?;
                Ok(result.try_as_basic_value().left().unwrap())
            }

            Literal::Float(f) => {
                let float_val = self.types.f64_type.const_float(*f);
                let result = self
                    .builder
                    .build_call(self.runtime.make_float, &[float_val.into()], "float")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to make float: {}", e))
                    })?;
                Ok(result.try_as_basic_value().left().unwrap())
            }

            Literal::String(s) => {
                let str_ptr = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to create string: {}", e))
                    })?;
                let result = self
                    .builder
                    .build_call(
                        self.runtime.make_string,
                        &[str_ptr.as_pointer_value().into()],
                        "string",
                    )
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to make string: {}", e))
                    })?;
                Ok(result.try_as_basic_value().left().unwrap())
            }
        }
    }

    /// Create a nil value
    fn make_nil(&self) -> BasicValueEnum<'ctx> {
        let tag = self
            .types
            .i8_type
            .const_int(ValueTag::Nil.as_u8() as u64, false);
        let data = self.types.i64_type.const_int(0, false);
        self.types
            .value_type
            .const_named_struct(&[tag.into(), data.into()])
            .into()
    }

    /// Compile a binary operation
    fn compile_binary(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_val = self.compile_expr(left)?;
        let right_val = self.compile_expr(right)?;

        let (func, name) = match op {
            BinaryOp::Add => (self.runtime.add, "add"),
            BinaryOp::Subtract => (self.runtime.sub, "sub"),
            BinaryOp::Multiply => (self.runtime.mul, "mul"),
            BinaryOp::Divide => (self.runtime.div, "div"),
            BinaryOp::Modulo => (self.runtime.modulo, "mod"),
            BinaryOp::Equal => {
                let result = self
                    .builder
                    .build_call(self.runtime.eq, &[left_val.into(), right_val.into()], "eq")
                    .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
                let bool_val = result.try_as_basic_value().left().unwrap();
                let mdh_bool = self
                    .builder
                    .build_call(self.runtime.make_bool, &[bool_val.into()], "bool")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to make bool: {}", e))
                    })?;
                return Ok(mdh_bool.try_as_basic_value().left().unwrap());
            }
            BinaryOp::NotEqual => {
                let result = self
                    .builder
                    .build_call(self.runtime.ne, &[left_val.into(), right_val.into()], "ne")
                    .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
                let bool_val = result.try_as_basic_value().left().unwrap();
                let mdh_bool = self
                    .builder
                    .build_call(self.runtime.make_bool, &[bool_val.into()], "bool")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to make bool: {}", e))
                    })?;
                return Ok(mdh_bool.try_as_basic_value().left().unwrap());
            }
            BinaryOp::Less => {
                let result = self
                    .builder
                    .build_call(self.runtime.lt, &[left_val.into(), right_val.into()], "lt")
                    .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
                let bool_val = result.try_as_basic_value().left().unwrap();
                let mdh_bool = self
                    .builder
                    .build_call(self.runtime.make_bool, &[bool_val.into()], "bool")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to make bool: {}", e))
                    })?;
                return Ok(mdh_bool.try_as_basic_value().left().unwrap());
            }
            BinaryOp::LessEqual => {
                let result = self
                    .builder
                    .build_call(self.runtime.le, &[left_val.into(), right_val.into()], "le")
                    .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
                let bool_val = result.try_as_basic_value().left().unwrap();
                let mdh_bool = self
                    .builder
                    .build_call(self.runtime.make_bool, &[bool_val.into()], "bool")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to make bool: {}", e))
                    })?;
                return Ok(mdh_bool.try_as_basic_value().left().unwrap());
            }
            BinaryOp::Greater => {
                let result = self
                    .builder
                    .build_call(self.runtime.gt, &[left_val.into(), right_val.into()], "gt")
                    .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
                let bool_val = result.try_as_basic_value().left().unwrap();
                let mdh_bool = self
                    .builder
                    .build_call(self.runtime.make_bool, &[bool_val.into()], "bool")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to make bool: {}", e))
                    })?;
                return Ok(mdh_bool.try_as_basic_value().left().unwrap());
            }
            BinaryOp::GreaterEqual => {
                let result = self
                    .builder
                    .build_call(self.runtime.ge, &[left_val.into(), right_val.into()], "ge")
                    .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
                let bool_val = result.try_as_basic_value().left().unwrap();
                let mdh_bool = self
                    .builder
                    .build_call(self.runtime.make_bool, &[bool_val.into()], "bool")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to make bool: {}", e))
                    })?;
                return Ok(mdh_bool.try_as_basic_value().left().unwrap());
            }
        };

        let result = self
            .builder
            .build_call(func, &[left_val.into(), right_val.into()], name)
            .map_err(|e| HaversError::CompileError(format!("Failed to build binary op: {}", e)))?;
        Ok(result.try_as_basic_value().left().unwrap())
    }

    /// Compile a unary operation
    fn compile_unary(
        &mut self,
        op: UnaryOp,
        operand: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let val = self.compile_expr(operand)?;

        match op {
            UnaryOp::Negate => {
                let result = self
                    .builder
                    .build_call(self.runtime.neg, &[val.into()], "neg")
                    .map_err(|e| HaversError::CompileError(format!("Failed to negate: {}", e)))?;
                Ok(result.try_as_basic_value().left().unwrap())
            }
            UnaryOp::Not => {
                let result = self
                    .builder
                    .build_call(self.runtime.not, &[val.into()], "not")
                    .map_err(|e| HaversError::CompileError(format!("Failed to not: {}", e)))?;
                Ok(result.try_as_basic_value().left().unwrap())
            }
        }
    }

    /// Compile a logical operation with short-circuit evaluation
    fn compile_logical(
        &mut self,
        left: &Expr,
        op: LogicalOp,
        right: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();
        let left_val = self.compile_expr(left)?;

        // Get truthiness of left
        let left_truthy = self
            .builder
            .build_call(self.runtime.truthy, &[left_val.into()], "truthy")
            .map_err(|e| HaversError::CompileError(format!("Failed to check truthy: {}", e)))?;
        let left_bool = left_truthy
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        // Create blocks for short-circuit
        let eval_right = self.context.append_basic_block(function, "eval_right");
        let merge = self.context.append_basic_block(function, "merge");

        match op {
            LogicalOp::And => {
                // If left is false, skip right
                self.builder
                    .build_conditional_branch(left_bool, eval_right, merge)
                    .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;
            }
            LogicalOp::Or => {
                // If left is true, skip right
                self.builder
                    .build_conditional_branch(left_bool, merge, eval_right)
                    .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;
            }
        }

        // Evaluate right side
        let left_block = self.builder.get_insert_block().unwrap();
        self.builder.position_at_end(eval_right);
        let right_val = self.compile_expr(right)?;
        let right_block = self.builder.get_insert_block().unwrap();
        self.builder
            .build_unconditional_branch(merge)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        // Merge
        self.builder.position_at_end(merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "logical")
            .map_err(|e| HaversError::CompileError(format!("Failed to build phi: {}", e)))?;
        phi.add_incoming(&[(&left_val, left_block), (&right_val, right_block)]);

        Ok(phi.as_basic_value())
    }

    /// Compile a function call
    fn compile_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Check if it's a builtin or user function
        if let Expr::Variable { name, .. } = callee {
            // Check builtins first
            if let Some(builtin) = builtins::get_builtin(name) {
                return self.compile_builtin_call(builtin.runtime_name, args);
            }

            // Check user-defined functions
            if let Some(&func) = self.functions.get(name) {
                let mut compiled_args = Vec::new();
                for arg in args {
                    compiled_args.push(self.compile_expr(arg)?.into());
                }

                let result = self
                    .builder
                    .build_call(func, &compiled_args, "call")
                    .map_err(|e| HaversError::CompileError(format!("Failed to call: {}", e)))?;
                return Ok(result.try_as_basic_value().left().unwrap());
            }
        }

        Err(HaversError::CompileError(format!(
            "Unknown function: {:?}",
            callee
        )))
    }

    /// Compile a builtin function call
    fn compile_builtin_call(
        &mut self,
        runtime_name: &str,
        args: &[Expr],
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let mut compiled_args: Vec<BasicValueEnum> = Vec::new();
        for arg in args {
            compiled_args.push(self.compile_expr(arg)?);
        }

        // Get the appropriate runtime function
        let func = match runtime_name {
            "__mdh_blether" => {
                self.builder
                    .build_call(self.runtime.blether, &[compiled_args[0].into()], "")
                    .map_err(|e| HaversError::CompileError(format!("Failed to call: {}", e)))?;
                return Ok(self.make_nil());
            }
            "__mdh_speir" => self.runtime.speir,
            "__mdh_to_string" => self.runtime.to_string,
            "__mdh_to_int" => self.runtime.to_int,
            "__mdh_to_float" => self.runtime.to_float,
            "__mdh_type_of" => self.runtime.type_of,
            "__mdh_len" => {
                let result = self
                    .builder
                    .build_call(self.runtime.len, &[compiled_args[0].into()], "len")
                    .map_err(|e| HaversError::CompileError(format!("Failed to call: {}", e)))?;
                let len_val = result.try_as_basic_value().left().unwrap();
                let mdh_int = self
                    .builder
                    .build_call(self.runtime.make_int, &[len_val.into()], "int")
                    .map_err(|e| HaversError::CompileError(format!("Failed to make int: {}", e)))?;
                return Ok(mdh_int.try_as_basic_value().left().unwrap());
            }
            "__mdh_list_push" => {
                self.builder
                    .build_call(
                        self.runtime.list_push,
                        &[compiled_args[0].into(), compiled_args[1].into()],
                        "",
                    )
                    .map_err(|e| HaversError::CompileError(format!("Failed to call: {}", e)))?;
                return Ok(self.make_nil());
            }
            "__mdh_list_pop" => self.runtime.list_pop,
            "__mdh_abs" => self.runtime.abs,
            "__mdh_floor" => self.runtime.floor,
            "__mdh_ceil" => self.runtime.ceil,
            "__mdh_round" => self.runtime.round,
            _ => {
                return Err(HaversError::CompileError(format!(
                    "Unknown builtin: {}",
                    runtime_name
                )))
            }
        };

        let arg_refs: Vec<_> = compiled_args.iter().map(|a| (*a).into()).collect();
        let result = self
            .builder
            .build_call(func, &arg_refs, "builtin")
            .map_err(|e| HaversError::CompileError(format!("Failed to call builtin: {}", e)))?;
        Ok(result.try_as_basic_value().left().unwrap())
    }

    /// Compile a list literal
    fn compile_list(&mut self, elements: &[Expr]) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let capacity = self.types.i32_type.const_int(elements.len() as u64, false);
        let list = self
            .builder
            .build_call(self.runtime.make_list, &[capacity.into()], "list")
            .map_err(|e| HaversError::CompileError(format!("Failed to make list: {}", e)))?;
        let list_val = list.try_as_basic_value().left().unwrap();

        for elem in elements {
            let elem_val = self.compile_expr(elem)?;
            self.builder
                .build_call(
                    self.runtime.list_push,
                    &[list_val.into(), elem_val.into()],
                    "",
                )
                .map_err(|e| HaversError::CompileError(format!("Failed to push to list: {}", e)))?;
        }

        Ok(list_val)
    }

    /// Compile index access
    fn compile_index(
        &mut self,
        object: &Expr,
        index: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let obj_val = self.compile_expr(object)?;
        let idx_val = self.compile_expr(index)?;

        // Extract index as i64
        let idx_struct = idx_val.into_struct_value();
        let idx_i64 = self
            .builder
            .build_extract_value(idx_struct, 1, "idx")
            .map_err(|e| HaversError::CompileError(format!("Failed to extract index: {}", e)))?;

        let result = self
            .builder
            .build_call(
                self.runtime.list_get,
                &[obj_val.into(), idx_i64.into()],
                "get",
            )
            .map_err(|e| HaversError::CompileError(format!("Failed to get index: {}", e)))?;
        Ok(result.try_as_basic_value().left().unwrap())
    }

    /// Compile index set
    fn compile_index_set(
        &mut self,
        object: &Expr,
        index: &Expr,
        value: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let obj_val = self.compile_expr(object)?;
        let idx_val = self.compile_expr(index)?;
        let new_val = self.compile_expr(value)?;

        // Extract index as i64
        let idx_struct = idx_val.into_struct_value();
        let idx_i64 = self
            .builder
            .build_extract_value(idx_struct, 1, "idx")
            .map_err(|e| HaversError::CompileError(format!("Failed to extract index: {}", e)))?;

        self.builder
            .build_call(
                self.runtime.list_set,
                &[obj_val.into(), idx_i64.into(), new_val.into()],
                "",
            )
            .map_err(|e| HaversError::CompileError(format!("Failed to set index: {}", e)))?;

        Ok(new_val)
    }

    /// Compile an if statement
    fn compile_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Result<(), HaversError> {
        let function = self.current_function.unwrap();

        let cond_val = self.compile_expr(condition)?;
        let cond_bool = self
            .builder
            .build_call(self.runtime.truthy, &[cond_val.into()], "cond")
            .map_err(|e| HaversError::CompileError(format!("Failed to check truthy: {}", e)))?;
        let cond_i1 = cond_bool
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        let then_block = self.context.append_basic_block(function, "then");
        let else_block = self.context.append_basic_block(function, "else");
        let merge_block = self.context.append_basic_block(function, "merge");

        self.builder
            .build_conditional_branch(cond_i1, then_block, else_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        // Then branch
        self.builder.position_at_end(then_block);
        self.compile_stmt(then_branch)?;
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;
        }

        // Else branch
        self.builder.position_at_end(else_block);
        if let Some(else_stmt) = else_branch {
            self.compile_stmt(else_stmt)?;
        }
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;
        }

        self.builder.position_at_end(merge_block);
        Ok(())
    }

    /// Compile a while loop
    fn compile_while(&mut self, condition: &Expr, body: &Stmt) -> Result<(), HaversError> {
        let function = self.current_function.unwrap();

        let loop_block = self.context.append_basic_block(function, "loop");
        let body_block = self.context.append_basic_block(function, "body");
        let after_block = self.context.append_basic_block(function, "after");

        // Push loop context
        self.loop_stack.push(LoopContext {
            break_block: after_block,
            continue_block: loop_block,
        });

        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        // Loop condition
        self.builder.position_at_end(loop_block);
        let cond_val = self.compile_expr(condition)?;
        let cond_bool = self
            .builder
            .build_call(self.runtime.truthy, &[cond_val.into()], "cond")
            .map_err(|e| HaversError::CompileError(format!("Failed to check truthy: {}", e)))?;
        let cond_i1 = cond_bool
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        self.builder
            .build_conditional_branch(cond_i1, body_block, after_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        // Body
        self.builder.position_at_end(body_block);
        self.compile_stmt(body)?;
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_block)
                .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;
        }

        self.loop_stack.pop();
        self.builder.position_at_end(after_block);
        Ok(())
    }

    /// Compile a for loop (currently only supports range-based)
    fn compile_for(
        &mut self,
        variable: &str,
        iterable: &Expr,
        body: &Stmt,
    ) -> Result<(), HaversError> {
        // For range-based: fer i in 0..10 { ... }
        if let Expr::Range {
            start,
            end,
            inclusive,
            ..
        } = iterable
        {
            return self.compile_for_range(variable, start, end, *inclusive, body);
        }

        // For list-based iteration, we'd need more runtime support
        Err(HaversError::CompileError(
            "For loop over non-range not yet supported in LLVM backend".to_string(),
        ))
    }

    /// Compile a range-based for loop
    fn compile_for_range(
        &mut self,
        variable: &str,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
        body: &Stmt,
    ) -> Result<(), HaversError> {
        let function = self.current_function.unwrap();

        // Compile start and end values
        let start_val = self.compile_expr(start)?;
        let end_val = self.compile_expr(end)?;

        // Extract integers from MdhValues
        let start_struct = start_val.into_struct_value();
        let start_i64 = self
            .builder
            .build_extract_value(start_struct, 1, "start")
            .map_err(|e| HaversError::CompileError(format!("Failed to extract: {}", e)))?
            .into_int_value();

        let end_struct = end_val.into_struct_value();
        let end_i64 = self
            .builder
            .build_extract_value(end_struct, 1, "end")
            .map_err(|e| HaversError::CompileError(format!("Failed to extract: {}", e)))?
            .into_int_value();

        // Create loop variable
        let var_alloca = self.create_entry_block_alloca(variable);
        let start_mdh = self
            .builder
            .build_call(self.runtime.make_int, &[start_i64.into()], "int")
            .map_err(|e| HaversError::CompileError(format!("Failed to make int: {}", e)))?;
        self.builder
            .build_store(var_alloca, start_mdh.try_as_basic_value().left().unwrap())
            .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;
        self.variables.insert(variable.to_string(), var_alloca);

        // Create counter
        let counter_alloca = self
            .builder
            .build_alloca(self.types.i64_type, "counter")
            .map_err(|e| HaversError::CompileError(format!("Failed to alloca: {}", e)))?;
        self.builder
            .build_store(counter_alloca, start_i64)
            .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;

        let loop_block = self.context.append_basic_block(function, "for_loop");
        let body_block = self.context.append_basic_block(function, "for_body");
        let incr_block = self.context.append_basic_block(function, "for_incr");
        let after_block = self.context.append_basic_block(function, "for_after");

        // Push loop context
        self.loop_stack.push(LoopContext {
            break_block: after_block,
            continue_block: incr_block,
        });

        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        // Loop condition
        self.builder.position_at_end(loop_block);
        let current = self
            .builder
            .build_load(self.types.i64_type, counter_alloca, "current")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();

        let cmp = if inclusive {
            self.builder
                .build_int_compare(inkwell::IntPredicate::SLE, current, end_i64, "cmp")
        } else {
            self.builder
                .build_int_compare(inkwell::IntPredicate::SLT, current, end_i64, "cmp")
        }
        .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;

        self.builder
            .build_conditional_branch(cmp, body_block, after_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        // Body
        self.builder.position_at_end(body_block);
        self.compile_stmt(body)?;
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(incr_block)
                .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;
        }

        // Increment
        self.builder.position_at_end(incr_block);
        let one = self.types.i64_type.const_int(1, false);
        let next = self
            .builder
            .build_int_add(current, one, "next")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        self.builder
            .build_store(counter_alloca, next)
            .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;

        // Update variable
        let next_mdh = self
            .builder
            .build_call(self.runtime.make_int, &[next.into()], "int")
            .map_err(|e| HaversError::CompileError(format!("Failed to make int: {}", e)))?;
        self.builder
            .build_store(var_alloca, next_mdh.try_as_basic_value().left().unwrap())
            .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;

        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        self.loop_stack.pop();
        self.builder.position_at_end(after_block);
        Ok(())
    }

    /// Compile a function definition
    fn compile_function(
        &mut self,
        name: &str,
        params: &[crate::ast::Param],
        body: &[Stmt],
    ) -> Result<(), HaversError> {
        let function =
            self.functions.get(name).copied().ok_or_else(|| {
                HaversError::CompileError(format!("Function not declared: {}", name))
            })?;

        let entry = self.context.append_basic_block(function, "entry");

        // Save current state
        let saved_function = self.current_function;
        let saved_variables = std::mem::take(&mut self.variables);

        self.builder.position_at_end(entry);
        self.current_function = Some(function);

        // Set up parameters
        for (i, param) in params.iter().enumerate() {
            let param_val = function
                .get_nth_param(i as u32)
                .ok_or_else(|| HaversError::CompileError("Missing parameter".to_string()))?;
            let alloca = self.create_entry_block_alloca(&param.name);
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| HaversError::CompileError(format!("Failed to store param: {}", e)))?;
            self.variables.insert(param.name.clone(), alloca);
        }

        // Compile body
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // Add implicit return if needed
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_return(Some(&self.make_nil()))
                .map_err(|e| HaversError::CompileError(format!("Failed to return: {}", e)))?;
        }

        // Restore state
        self.current_function = saved_function;
        self.variables = saved_variables;

        // Position builder back to where we were (main function entry)
        if let Some(func) = saved_function {
            if let Some(last_block) = func.get_last_basic_block() {
                self.builder.position_at_end(last_block);
            }
        }

        Ok(())
    }

    /// Compile a ternary expression
    fn compile_ternary(
        &mut self,
        condition: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        let cond_val = self.compile_expr(condition)?;
        let cond_bool = self
            .builder
            .build_call(self.runtime.truthy, &[cond_val.into()], "cond")
            .map_err(|e| HaversError::CompileError(format!("Failed to check truthy: {}", e)))?;
        let cond_i1 = cond_bool
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        let then_block = self.context.append_basic_block(function, "tern_then");
        let else_block = self.context.append_basic_block(function, "tern_else");
        let merge_block = self.context.append_basic_block(function, "tern_merge");

        self.builder
            .build_conditional_branch(cond_i1, then_block, else_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        // Then
        self.builder.position_at_end(then_block);
        let then_val = self.compile_expr(then_expr)?;
        let then_bb = self.builder.get_insert_block().unwrap();
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        // Else
        self.builder.position_at_end(else_block);
        let else_val = self.compile_expr(else_expr)?;
        let else_bb = self.builder.get_insert_block().unwrap();
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to branch: {}", e)))?;

        // Merge with phi
        self.builder.position_at_end(merge_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "tern")
            .map_err(|e| HaversError::CompileError(format!("Failed to build phi: {}", e)))?;
        phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);

        Ok(phi.as_basic_value())
    }

    /// Create an alloca in the entry block of the current function
    fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let function = self.current_function.unwrap();
        let entry = function.get_first_basic_block().unwrap();

        let builder = self.context.create_builder();
        match entry.get_first_instruction() {
            Some(instr) => builder.position_before(&instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.types.value_type, name).unwrap()
    }
}
