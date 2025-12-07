//! LLVM Code Generation
//!
//! Compiles mdhavers AST to LLVM IR with fully inlined runtime.
//! Produces standalone executables that only depend on libc.

use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::IntPredicate;

use crate::ast::{BinaryOp, Expr, Literal, LogicalOp, Program, Stmt, UnaryOp};
use crate::error::HaversError;

use super::types::{MdhTypes, ValueTag};

/// Loop context for break/continue
struct LoopContext<'ctx> {
    break_block: BasicBlock<'ctx>,
    continue_block: BasicBlock<'ctx>,
}

/// Libc functions we use
struct LibcFunctions<'ctx> {
    printf: FunctionValue<'ctx>,
    malloc: FunctionValue<'ctx>,
    strlen: FunctionValue<'ctx>,
    strcpy: FunctionValue<'ctx>,
    strcat: FunctionValue<'ctx>,
    snprintf: FunctionValue<'ctx>,
    exit: FunctionValue<'ctx>,
}

/// Main code generator with inlined runtime
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    types: MdhTypes<'ctx>,
    libc: LibcFunctions<'ctx>,

    /// Current function being compiled
    current_function: Option<FunctionValue<'ctx>>,

    /// Variable storage (name -> alloca pointer)
    variables: HashMap<String, PointerValue<'ctx>>,

    /// User-defined functions
    functions: HashMap<String, FunctionValue<'ctx>>,

    /// Loop context stack for break/continue
    loop_stack: Vec<LoopContext<'ctx>>,

    /// Format strings for printf
    fmt_int: inkwell::values::GlobalValue<'ctx>,
    fmt_float: inkwell::values::GlobalValue<'ctx>,
    fmt_string: inkwell::values::GlobalValue<'ctx>,
    fmt_true: inkwell::values::GlobalValue<'ctx>,
    fmt_false: inkwell::values::GlobalValue<'ctx>,
    fmt_nil: inkwell::values::GlobalValue<'ctx>,
    fmt_newline: inkwell::values::GlobalValue<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let types = MdhTypes::new(context);

        // Declare libc functions
        let libc = Self::declare_libc_functions(&module, context);

        // Create format strings
        let fmt_int = Self::create_global_string(&module, context, "%lld", "fmt_int");
        let fmt_float = Self::create_global_string(&module, context, "%g", "fmt_float");
        let fmt_string = Self::create_global_string(&module, context, "%s", "fmt_string");
        let fmt_true = Self::create_global_string(&module, context, "aye", "fmt_true");
        let fmt_false = Self::create_global_string(&module, context, "nae", "fmt_false");
        let fmt_nil = Self::create_global_string(&module, context, "naething", "fmt_nil");
        let fmt_newline = Self::create_global_string(&module, context, "\n", "fmt_newline");

        CodeGen {
            context,
            module,
            builder,
            types,
            libc,
            current_function: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
            loop_stack: Vec::new(),
            fmt_int,
            fmt_float,
            fmt_string,
            fmt_true,
            fmt_false,
            fmt_nil,
            fmt_newline,
        }
    }

    fn declare_libc_functions(module: &Module<'ctx>, context: &'ctx Context) -> LibcFunctions<'ctx> {
        let i8_ptr = context.i8_type().ptr_type(AddressSpace::default());
        let i32_type = context.i32_type();
        let i64_type = context.i64_type();
        let void_type = context.void_type();

        // printf(const char* fmt, ...) -> int
        let printf_type = i32_type.fn_type(&[i8_ptr.into()], true);
        let printf = module.add_function("printf", printf_type, Some(Linkage::External));

        // malloc(size_t) -> void*
        let malloc_type = i8_ptr.fn_type(&[i64_type.into()], false);
        let malloc = module.add_function("malloc", malloc_type, Some(Linkage::External));

        // strlen(const char*) -> size_t
        let strlen_type = i64_type.fn_type(&[i8_ptr.into()], false);
        let strlen = module.add_function("strlen", strlen_type, Some(Linkage::External));

        // strcpy(char* dest, const char* src) -> char*
        let strcpy_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let strcpy = module.add_function("strcpy", strcpy_type, Some(Linkage::External));

        // strcat(char* dest, const char* src) -> char*
        let strcat_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let strcat = module.add_function("strcat", strcat_type, Some(Linkage::External));

        // snprintf(char* str, size_t size, const char* format, ...) -> int
        let snprintf_type = i32_type.fn_type(&[i8_ptr.into(), i64_type.into(), i8_ptr.into()], true);
        let snprintf = module.add_function("snprintf", snprintf_type, Some(Linkage::External));

        // exit(int) -> void
        let exit_type = void_type.fn_type(&[i32_type.into()], false);
        let exit = module.add_function("exit", exit_type, Some(Linkage::External));

        LibcFunctions {
            printf,
            malloc,
            strlen,
            strcpy,
            strcat,
            snprintf,
            exit,
        }
    }

    fn create_global_string(module: &Module<'ctx>, context: &'ctx Context, s: &str, name: &str) -> inkwell::values::GlobalValue<'ctx> {
        let bytes: Vec<u8> = s.bytes().chain(std::iter::once(0)).collect();
        let arr_type = context.i8_type().array_type(bytes.len() as u32);
        let global = module.add_global(arr_type, Some(AddressSpace::default()), name);
        global.set_linkage(Linkage::Private);
        global.set_constant(true);
        let values: Vec<_> = bytes.iter().map(|b| context.i8_type().const_int(*b as u64, false)).collect();
        global.set_initializer(&context.i8_type().const_array(&values));
        global
    }

    fn get_string_ptr(&self, global: inkwell::values::GlobalValue<'ctx>) -> PointerValue<'ctx> {
        self.builder.build_pointer_cast(
            global.as_pointer_value(),
            self.context.i8_type().ptr_type(AddressSpace::default()),
            "str_ptr"
        ).unwrap()
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
        let param_types: Vec<BasicMetadataTypeEnum> = (0..param_count)
            .map(|_| self.types.value_type.into())
            .collect();

        let fn_type = self.types.value_type.fn_type(&param_types, false);
        let function = self.module.add_function(name, fn_type, None);
        self.functions.insert(name.to_string(), function);
        Ok(())
    }

    // ========== Inline Value Creation ==========

    /// Create a nil value: {tag=0, data=0}
    fn make_nil(&self) -> BasicValueEnum<'ctx> {
        let tag = self.types.i8_type.const_int(ValueTag::Nil.as_u8() as u64, false);
        let data = self.types.i64_type.const_int(0, false);
        self.types.value_type.const_named_struct(&[tag.into(), data.into()]).into()
    }

    /// Create a bool value: {tag=1, data=0|1}
    fn make_bool(&self, val: IntValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.types.i8_type.const_int(ValueTag::Bool.as_u8() as u64, false);
        let data = self.builder.build_int_z_extend(val, self.types.i64_type, "bool_ext")
            .map_err(|e| HaversError::CompileError(format!("Failed to extend bool: {}", e)))?;

        let undef = self.types.value_type.get_undef();
        let v1 = self.builder.build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self.builder.build_insert_value(v1, data, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create an int value: {tag=2, data=i64}
    fn make_int(&self, val: IntValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.types.i8_type.const_int(ValueTag::Int.as_u8() as u64, false);

        let undef = self.types.value_type.get_undef();
        let v1 = self.builder.build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self.builder.build_insert_value(v1, val, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create a float value: {tag=3, data=bitcast(f64)}
    fn make_float(&self, val: inkwell::values::FloatValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.types.i8_type.const_int(ValueTag::Float.as_u8() as u64, false);
        let data = self.builder.build_bitcast(val, self.types.i64_type, "float_bits")
            .map_err(|e| HaversError::CompileError(format!("Failed to bitcast float: {}", e)))?;

        let undef = self.types.value_type.get_undef();
        let v1 = self.builder.build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self.builder.build_insert_value(v1, data, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create a string value: {tag=4, data=ptr as i64}
    fn make_string(&self, ptr: PointerValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.types.i8_type.const_int(ValueTag::String.as_u8() as u64, false);
        let data = self.builder.build_ptr_to_int(ptr, self.types.i64_type, "str_ptr_int")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert ptr: {}", e)))?;

        let undef = self.types.value_type.get_undef();
        let v1 = self.builder.build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self.builder.build_insert_value(v1, data, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Extract tag from value
    fn extract_tag(&self, val: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, HaversError> {
        let struct_val = val.into_struct_value();
        let tag = self.builder.build_extract_value(struct_val, 0, "tag")
            .map_err(|e| HaversError::CompileError(format!("Failed to extract tag: {}", e)))?;
        Ok(tag.into_int_value())
    }

    /// Extract data from value as i64
    fn extract_data(&self, val: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, HaversError> {
        let struct_val = val.into_struct_value();
        let data = self.builder.build_extract_value(struct_val, 1, "data")
            .map_err(|e| HaversError::CompileError(format!("Failed to extract data: {}", e)))?;
        Ok(data.into_int_value())
    }

    /// Extract data as f64 (for float values)
    fn extract_float(&self, val: BasicValueEnum<'ctx>) -> Result<inkwell::values::FloatValue<'ctx>, HaversError> {
        let data = self.extract_data(val)?;
        let float_val = self.builder.build_bitcast(data, self.types.f64_type, "as_float")
            .map_err(|e| HaversError::CompileError(format!("Failed to bitcast to float: {}", e)))?;
        Ok(float_val.into_float_value())
    }

    /// Extract data as string pointer
    fn extract_string_ptr(&self, val: BasicValueEnum<'ctx>) -> Result<PointerValue<'ctx>, HaversError> {
        let data = self.extract_data(val)?;
        let ptr = self.builder.build_int_to_ptr(data, self.context.i8_type().ptr_type(AddressSpace::default()), "as_str")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert to ptr: {}", e)))?;
        Ok(ptr)
    }

    /// Check if value is truthy
    fn is_truthy(&self, val: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, HaversError> {
        let tag = self.extract_tag(val)?;
        let data = self.extract_data(val)?;

        let function = self.current_function.unwrap();
        let is_nil = self.context.append_basic_block(function, "is_nil");
        let is_bool = self.context.append_basic_block(function, "is_bool");
        let is_int = self.context.append_basic_block(function, "is_int");
        let is_other = self.context.append_basic_block(function, "is_other");
        let merge = self.context.append_basic_block(function, "truthy_merge");

        // Switch on tag
        let nil_tag = self.types.i8_type.const_int(ValueTag::Nil.as_u8() as u64, false);
        let bool_tag = self.types.i8_type.const_int(ValueTag::Bool.as_u8() as u64, false);
        let int_tag = self.types.i8_type.const_int(ValueTag::Int.as_u8() as u64, false);

        self.builder.build_switch(tag, is_other, &[
            (nil_tag, is_nil),
            (bool_tag, is_bool),
            (int_tag, is_int),
        ]).map_err(|e| HaversError::CompileError(format!("Failed to build switch: {}", e)))?;

        // nil -> false
        self.builder.position_at_end(is_nil);
        let nil_result = self.types.bool_type.const_int(0, false);
        self.builder.build_unconditional_branch(merge).unwrap();
        let nil_block = self.builder.get_insert_block().unwrap();

        // bool -> value
        self.builder.position_at_end(is_bool);
        let bool_result = self.builder.build_int_truncate(data, self.types.bool_type, "bool_val").unwrap();
        self.builder.build_unconditional_branch(merge).unwrap();
        let bool_block = self.builder.get_insert_block().unwrap();

        // int -> value != 0
        self.builder.position_at_end(is_int);
        let zero = self.types.i64_type.const_int(0, false);
        let int_result = self.builder.build_int_compare(IntPredicate::NE, data, zero, "int_truthy").unwrap();
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // other -> true
        self.builder.position_at_end(is_other);
        let other_result = self.types.bool_type.const_int(1, false);
        self.builder.build_unconditional_branch(merge).unwrap();
        let other_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self.builder.build_phi(self.types.bool_type, "truthy").unwrap();
        phi.add_incoming(&[
            (&nil_result, nil_block),
            (&bool_result, bool_block),
            (&int_result, int_block),
            (&other_result, other_block),
        ]);

        Ok(phi.as_basic_value().into_int_value())
    }

    // ========== Inline Arithmetic ==========

    /// Add two values with type checking
    fn inline_add(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        let function = self.current_function.unwrap();
        let int_int = self.context.append_basic_block(function, "add_int_int");
        let float_case = self.context.append_basic_block(function, "add_float");
        let string_case = self.context.append_basic_block(function, "add_string");
        let error_case = self.context.append_basic_block(function, "add_error");
        let merge = self.context.append_basic_block(function, "add_merge");

        let int_tag = self.types.i8_type.const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self.types.i8_type.const_int(ValueTag::Float.as_u8() as u64, false);
        let string_tag = self.types.i8_type.const_int(ValueTag::String.as_u8() as u64, false);

        // Check if both are ints
        let left_is_int = self.builder.build_int_compare(IntPredicate::EQ, left_tag, int_tag, "l_int").unwrap();
        let right_is_int = self.builder.build_int_compare(IntPredicate::EQ, right_tag, int_tag, "r_int").unwrap();
        let both_int = self.builder.build_and(left_is_int, right_is_int, "both_int").unwrap();

        // Check if either is float
        let left_is_float = self.builder.build_int_compare(IntPredicate::EQ, left_tag, float_tag, "l_float").unwrap();
        let right_is_float = self.builder.build_int_compare(IntPredicate::EQ, right_tag, float_tag, "r_float").unwrap();
        let either_float = self.builder.build_or(left_is_float, right_is_float, "either_float").unwrap();

        // Check if both are strings
        let left_is_string = self.builder.build_int_compare(IntPredicate::EQ, left_tag, string_tag, "l_str").unwrap();
        let right_is_string = self.builder.build_int_compare(IntPredicate::EQ, right_tag, string_tag, "r_str").unwrap();
        let both_string = self.builder.build_and(left_is_string, right_is_string, "both_str").unwrap();

        // Branch based on types
        let check_float = self.context.append_basic_block(function, "check_float");
        let check_string = self.context.append_basic_block(function, "check_string");

        self.builder.build_conditional_branch(both_int, int_int, check_float).unwrap();

        self.builder.position_at_end(check_float);
        self.builder.build_conditional_branch(either_float, float_case, check_string).unwrap();

        self.builder.position_at_end(check_string);
        self.builder.build_conditional_branch(both_string, string_case, error_case).unwrap();

        // int + int
        self.builder.position_at_end(int_int);
        let int_sum = self.builder.build_int_add(left_data, right_data, "sum").unwrap();
        let int_result = self.make_int(int_sum)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float + float (or int+float)
        self.builder.position_at_end(float_case);
        // Convert both to float
        let left_f = self.builder.build_select(
            left_is_float,
            BasicValueEnum::FloatValue(self.builder.build_bitcast(left_data, self.types.f64_type, "lf").unwrap().into_float_value()),
            BasicValueEnum::FloatValue(self.builder.build_signed_int_to_float(left_data, self.types.f64_type, "li2f").unwrap()),
            "left_as_float"
        ).unwrap().into_float_value();
        let right_f = self.builder.build_select(
            right_is_float,
            BasicValueEnum::FloatValue(self.builder.build_bitcast(right_data, self.types.f64_type, "rf").unwrap().into_float_value()),
            BasicValueEnum::FloatValue(self.builder.build_signed_int_to_float(right_data, self.types.f64_type, "ri2f").unwrap()),
            "right_as_float"
        ).unwrap().into_float_value();
        let float_sum = self.builder.build_float_add(left_f, right_f, "fsum").unwrap();
        let float_result = self.make_float(float_sum)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // string + string (concatenation)
        self.builder.position_at_end(string_case);
        let left_ptr = self.builder.build_int_to_ptr(left_data, self.context.i8_type().ptr_type(AddressSpace::default()), "lstr").unwrap();
        let right_ptr = self.builder.build_int_to_ptr(right_data, self.context.i8_type().ptr_type(AddressSpace::default()), "rstr").unwrap();

        // Get lengths
        let left_len = self.builder.build_call(self.libc.strlen, &[left_ptr.into()], "llen")
            .unwrap().try_as_basic_value().left().unwrap().into_int_value();
        let right_len = self.builder.build_call(self.libc.strlen, &[right_ptr.into()], "rlen")
            .unwrap().try_as_basic_value().left().unwrap().into_int_value();

        // Allocate new string (len1 + len2 + 1)
        let total_len = self.builder.build_int_add(left_len, right_len, "total").unwrap();
        let one = self.types.i64_type.const_int(1, false);
        let alloc_size = self.builder.build_int_add(total_len, one, "alloc_size").unwrap();
        let new_str = self.builder.build_call(self.libc.malloc, &[alloc_size.into()], "new_str")
            .unwrap().try_as_basic_value().left().unwrap().into_pointer_value();

        // Copy strings
        self.builder.build_call(self.libc.strcpy, &[new_str.into(), left_ptr.into()], "").unwrap();
        self.builder.build_call(self.libc.strcat, &[new_str.into(), right_ptr.into()], "").unwrap();

        let string_result = self.make_string(new_str)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let string_block = self.builder.get_insert_block().unwrap();

        // Error case - just return nil for now (should be runtime error)
        self.builder.position_at_end(error_case);
        let error_result = self.make_nil();
        self.builder.build_unconditional_branch(merge).unwrap();
        let error_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self.builder.build_phi(self.types.value_type, "add_result").unwrap();
        phi.add_incoming(&[
            (&int_result, int_block),
            (&float_result, float_block),
            (&string_result, string_block),
            (&error_result, error_block),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Subtract two values
    fn inline_sub(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        let function = self.current_function.unwrap();
        let int_int = self.context.append_basic_block(function, "sub_int");
        let float_case = self.context.append_basic_block(function, "sub_float");
        let merge = self.context.append_basic_block(function, "sub_merge");

        let int_tag = self.types.i8_type.const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self.types.i8_type.const_int(ValueTag::Float.as_u8() as u64, false);

        let left_is_int = self.builder.build_int_compare(IntPredicate::EQ, left_tag, int_tag, "l_int").unwrap();
        let right_is_int = self.builder.build_int_compare(IntPredicate::EQ, right_tag, int_tag, "r_int").unwrap();
        let both_int = self.builder.build_and(left_is_int, right_is_int, "both_int").unwrap();

        self.builder.build_conditional_branch(both_int, int_int, float_case).unwrap();

        // int - int
        self.builder.position_at_end(int_int);
        let int_diff = self.builder.build_int_sub(left_data, right_data, "diff").unwrap();
        let int_result = self.make_int(int_diff)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float case
        self.builder.position_at_end(float_case);
        let left_is_float = self.builder.build_int_compare(IntPredicate::EQ, left_tag, float_tag, "lf").unwrap();
        let right_is_float = self.builder.build_int_compare(IntPredicate::EQ, right_tag, float_tag, "rf").unwrap();
        let left_f = self.builder.build_select(
            left_is_float,
            BasicValueEnum::FloatValue(self.builder.build_bitcast(left_data, self.types.f64_type, "lf").unwrap().into_float_value()),
            BasicValueEnum::FloatValue(self.builder.build_signed_int_to_float(left_data, self.types.f64_type, "li2f").unwrap()),
            "left_as_float"
        ).unwrap().into_float_value();
        let right_f = self.builder.build_select(
            right_is_float,
            BasicValueEnum::FloatValue(self.builder.build_bitcast(right_data, self.types.f64_type, "rf").unwrap().into_float_value()),
            BasicValueEnum::FloatValue(self.builder.build_signed_int_to_float(right_data, self.types.f64_type, "ri2f").unwrap()),
            "right_as_float"
        ).unwrap().into_float_value();
        let float_diff = self.builder.build_float_sub(left_f, right_f, "fdiff").unwrap();
        let float_result = self.make_float(float_diff)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self.builder.build_phi(self.types.value_type, "sub_result").unwrap();
        phi.add_incoming(&[
            (&int_result, int_block),
            (&float_result, float_block),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Multiply two values
    fn inline_mul(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        let function = self.current_function.unwrap();
        let int_int = self.context.append_basic_block(function, "mul_int");
        let float_case = self.context.append_basic_block(function, "mul_float");
        let merge = self.context.append_basic_block(function, "mul_merge");

        let int_tag = self.types.i8_type.const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self.types.i8_type.const_int(ValueTag::Float.as_u8() as u64, false);

        let left_is_int = self.builder.build_int_compare(IntPredicate::EQ, left_tag, int_tag, "l_int").unwrap();
        let right_is_int = self.builder.build_int_compare(IntPredicate::EQ, right_tag, int_tag, "r_int").unwrap();
        let both_int = self.builder.build_and(left_is_int, right_is_int, "both_int").unwrap();

        self.builder.build_conditional_branch(both_int, int_int, float_case).unwrap();

        // int * int
        self.builder.position_at_end(int_int);
        let int_prod = self.builder.build_int_mul(left_data, right_data, "prod").unwrap();
        let int_result = self.make_int(int_prod)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float case
        self.builder.position_at_end(float_case);
        let left_is_float = self.builder.build_int_compare(IntPredicate::EQ, left_tag, float_tag, "lf").unwrap();
        let right_is_float = self.builder.build_int_compare(IntPredicate::EQ, right_tag, float_tag, "rf").unwrap();
        let left_f = self.builder.build_select(
            left_is_float,
            BasicValueEnum::FloatValue(self.builder.build_bitcast(left_data, self.types.f64_type, "lf").unwrap().into_float_value()),
            BasicValueEnum::FloatValue(self.builder.build_signed_int_to_float(left_data, self.types.f64_type, "li2f").unwrap()),
            "left_as_float"
        ).unwrap().into_float_value();
        let right_f = self.builder.build_select(
            right_is_float,
            BasicValueEnum::FloatValue(self.builder.build_bitcast(right_data, self.types.f64_type, "rf").unwrap().into_float_value()),
            BasicValueEnum::FloatValue(self.builder.build_signed_int_to_float(right_data, self.types.f64_type, "ri2f").unwrap()),
            "right_as_float"
        ).unwrap().into_float_value();
        let float_prod = self.builder.build_float_mul(left_f, right_f, "fprod").unwrap();
        let float_result = self.make_float(float_prod)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self.builder.build_phi(self.types.value_type, "mul_result").unwrap();
        phi.add_incoming(&[
            (&int_result, int_block),
            (&float_result, float_block),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Divide two values
    fn inline_div(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        let function = self.current_function.unwrap();
        let int_int = self.context.append_basic_block(function, "div_int");
        let float_case = self.context.append_basic_block(function, "div_float");
        let merge = self.context.append_basic_block(function, "div_merge");

        let int_tag = self.types.i8_type.const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self.types.i8_type.const_int(ValueTag::Float.as_u8() as u64, false);

        let left_is_int = self.builder.build_int_compare(IntPredicate::EQ, left_tag, int_tag, "l_int").unwrap();
        let right_is_int = self.builder.build_int_compare(IntPredicate::EQ, right_tag, int_tag, "r_int").unwrap();
        let both_int = self.builder.build_and(left_is_int, right_is_int, "both_int").unwrap();

        self.builder.build_conditional_branch(both_int, int_int, float_case).unwrap();

        // int / int
        self.builder.position_at_end(int_int);
        let int_quot = self.builder.build_int_signed_div(left_data, right_data, "quot").unwrap();
        let int_result = self.make_int(int_quot)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float case
        self.builder.position_at_end(float_case);
        let left_is_float = self.builder.build_int_compare(IntPredicate::EQ, left_tag, float_tag, "lf").unwrap();
        let right_is_float = self.builder.build_int_compare(IntPredicate::EQ, right_tag, float_tag, "rf").unwrap();
        let left_f = self.builder.build_select(
            left_is_float,
            BasicValueEnum::FloatValue(self.builder.build_bitcast(left_data, self.types.f64_type, "lf").unwrap().into_float_value()),
            BasicValueEnum::FloatValue(self.builder.build_signed_int_to_float(left_data, self.types.f64_type, "li2f").unwrap()),
            "left_as_float"
        ).unwrap().into_float_value();
        let right_f = self.builder.build_select(
            right_is_float,
            BasicValueEnum::FloatValue(self.builder.build_bitcast(right_data, self.types.f64_type, "rf").unwrap().into_float_value()),
            BasicValueEnum::FloatValue(self.builder.build_signed_int_to_float(right_data, self.types.f64_type, "ri2f").unwrap()),
            "right_as_float"
        ).unwrap().into_float_value();
        let float_quot = self.builder.build_float_div(left_f, right_f, "fquot").unwrap();
        let float_result = self.make_float(float_quot)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self.builder.build_phi(self.types.value_type, "div_result").unwrap();
        phi.add_incoming(&[
            (&int_result, int_block),
            (&float_result, float_block),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Modulo two values
    fn inline_mod(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let rem = self.builder.build_int_signed_rem(left_data, right_data, "rem").unwrap();
        self.make_int(rem)
    }

    /// Compare two values for equality
    fn inline_eq(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        // Tags must match
        let tags_equal = self.builder.build_int_compare(IntPredicate::EQ, left_tag, right_tag, "tags_eq").unwrap();
        // Data must match
        let data_equal = self.builder.build_int_compare(IntPredicate::EQ, left_data, right_data, "data_eq").unwrap();
        // Both must be true
        let result = self.builder.build_and(tags_equal, data_equal, "eq").unwrap();

        self.make_bool(result)
    }

    /// Compare two values for inequality
    fn inline_ne(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        let tags_equal = self.builder.build_int_compare(IntPredicate::EQ, left_tag, right_tag, "tags_eq").unwrap();
        let data_equal = self.builder.build_int_compare(IntPredicate::EQ, left_data, right_data, "data_eq").unwrap();
        let both_equal = self.builder.build_and(tags_equal, data_equal, "eq").unwrap();
        let result = self.builder.build_not(both_equal, "ne").unwrap();

        self.make_bool(result)
    }

    /// Compare two values: less than
    fn inline_lt(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let result = self.builder.build_int_compare(IntPredicate::SLT, left_data, right_data, "lt").unwrap();
        self.make_bool(result)
    }

    /// Compare two values: less than or equal
    fn inline_le(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let result = self.builder.build_int_compare(IntPredicate::SLE, left_data, right_data, "le").unwrap();
        self.make_bool(result)
    }

    /// Compare two values: greater than
    fn inline_gt(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let result = self.builder.build_int_compare(IntPredicate::SGT, left_data, right_data, "gt").unwrap();
        self.make_bool(result)
    }

    /// Compare two values: greater than or equal
    fn inline_ge(&mut self, left: BasicValueEnum<'ctx>, right: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let result = self.builder.build_int_compare(IntPredicate::SGE, left_data, right_data, "ge").unwrap();
        self.make_bool(result)
    }

    /// Negate a value
    fn inline_neg(&mut self, val: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.extract_tag(val)?;
        let data = self.extract_data(val)?;

        let int_tag = self.types.i8_type.const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self.types.i8_type.const_int(ValueTag::Float.as_u8() as u64, false);

        let is_int = self.builder.build_int_compare(IntPredicate::EQ, tag, int_tag, "is_int").unwrap();
        let is_float = self.builder.build_int_compare(IntPredicate::EQ, tag, float_tag, "is_float").unwrap();

        let function = self.current_function.unwrap();
        let neg_int = self.context.append_basic_block(function, "neg_int");
        let neg_float = self.context.append_basic_block(function, "neg_float");
        let neg_else = self.context.append_basic_block(function, "neg_else");
        let merge = self.context.append_basic_block(function, "neg_merge");

        self.builder.build_conditional_branch(is_int, neg_int, neg_float).unwrap();

        // Negate int
        self.builder.position_at_end(neg_int);
        let neg_data = self.builder.build_int_neg(data, "neg").unwrap();
        let int_result = self.make_int(neg_data)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // Negate float
        self.builder.position_at_end(neg_float);
        self.builder.build_conditional_branch(is_float, neg_else, merge).unwrap();

        self.builder.position_at_end(neg_else);
        let float_val = self.builder.build_bitcast(data, self.types.f64_type, "f").unwrap().into_float_value();
        let neg_float_val = self.builder.build_float_neg(float_val, "fneg").unwrap();
        let float_result = self.make_float(neg_float_val)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self.builder.build_phi(self.types.value_type, "neg_result").unwrap();
        phi.add_incoming(&[
            (&int_result, int_block),
            (&float_result, float_block),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Logical not
    fn inline_not(&mut self, val: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let truthy = self.is_truthy(val)?;
        let result = self.builder.build_not(truthy, "not").unwrap();
        self.make_bool(result)
    }

    // ========== Inline Print (blether) ==========

    /// Print a value using printf
    fn inline_blether(&mut self, val: BasicValueEnum<'ctx>) -> Result<(), HaversError> {
        let tag = self.extract_tag(val)?;
        let data = self.extract_data(val)?;

        let function = self.current_function.unwrap();
        let print_nil = self.context.append_basic_block(function, "print_nil");
        let print_bool = self.context.append_basic_block(function, "print_bool");
        let print_int = self.context.append_basic_block(function, "print_int");
        let print_float = self.context.append_basic_block(function, "print_float");
        let print_string = self.context.append_basic_block(function, "print_string");
        let print_default = self.context.append_basic_block(function, "print_default");
        let print_done = self.context.append_basic_block(function, "print_done");

        let nil_tag = self.types.i8_type.const_int(ValueTag::Nil.as_u8() as u64, false);
        let bool_tag = self.types.i8_type.const_int(ValueTag::Bool.as_u8() as u64, false);
        let int_tag = self.types.i8_type.const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self.types.i8_type.const_int(ValueTag::Float.as_u8() as u64, false);
        let string_tag = self.types.i8_type.const_int(ValueTag::String.as_u8() as u64, false);

        self.builder.build_switch(tag, print_default, &[
            (nil_tag, print_nil),
            (bool_tag, print_bool),
            (int_tag, print_int),
            (float_tag, print_float),
            (string_tag, print_string),
        ]).unwrap();

        // Print nil
        self.builder.position_at_end(print_nil);
        let nil_str = self.get_string_ptr(self.fmt_nil);
        self.builder.build_call(self.libc.printf, &[nil_str.into()], "").unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print bool
        self.builder.position_at_end(print_bool);
        let true_str = self.get_string_ptr(self.fmt_true);
        let false_str = self.get_string_ptr(self.fmt_false);
        let zero = self.types.i64_type.const_int(0, false);
        let is_true = self.builder.build_int_compare(IntPredicate::NE, data, zero, "is_true").unwrap();
        let bool_str = self.builder.build_select(is_true, true_str, false_str, "bool_str").unwrap();
        self.builder.build_call(self.libc.printf, &[bool_str.into()], "").unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print int
        self.builder.position_at_end(print_int);
        let int_fmt = self.get_string_ptr(self.fmt_int);
        self.builder.build_call(self.libc.printf, &[int_fmt.into(), data.into()], "").unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print float
        self.builder.position_at_end(print_float);
        let float_fmt = self.get_string_ptr(self.fmt_float);
        let float_val = self.builder.build_bitcast(data, self.types.f64_type, "f").unwrap();
        self.builder.build_call(self.libc.printf, &[float_fmt.into(), float_val.into()], "").unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print string
        self.builder.position_at_end(print_string);
        let str_fmt = self.get_string_ptr(self.fmt_string);
        let str_ptr = self.builder.build_int_to_ptr(data, self.context.i8_type().ptr_type(AddressSpace::default()), "str").unwrap();
        self.builder.build_call(self.libc.printf, &[str_fmt.into(), str_ptr.into()], "").unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print default (unknown type)
        self.builder.position_at_end(print_default);
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Done - print newline
        self.builder.position_at_end(print_done);
        let newline = self.get_string_ptr(self.fmt_newline);
        self.builder.build_call(self.libc.printf, &[newline.into()], "").unwrap();

        Ok(())
    }

    // ========== Statement Compilation ==========

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), HaversError> {
        match stmt {
            Stmt::VarDecl { name, initializer, .. } => {
                let value = if let Some(init) = initializer {
                    self.compile_expr(init)?
                } else {
                    self.make_nil()
                };

                let alloca = self.create_entry_block_alloca(name);
                self.builder.build_store(alloca, value)
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
                self.inline_blether(val)?;
                Ok(())
            }

            Stmt::If { condition, then_branch, else_branch, .. } => {
                self.compile_if(condition, then_branch, else_branch.as_deref())
            }

            Stmt::While { condition, body, .. } => {
                self.compile_while(condition, body)
            }

            Stmt::For { variable, iterable, body, .. } => {
                self.compile_for(variable, iterable, body)
            }

            Stmt::Function { name, params, body, .. } => {
                self.compile_function(name, params, body)
            }

            Stmt::Return { value, .. } => {
                let ret_val = if let Some(v) = value {
                    self.compile_expr(v)?
                } else {
                    self.make_nil()
                };
                self.builder.build_return(Some(&ret_val))
                    .map_err(|e| HaversError::CompileError(format!("Failed to build return: {}", e)))?;
                Ok(())
            }

            Stmt::Break { .. } => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.builder.build_unconditional_branch(loop_ctx.break_block)
                        .map_err(|e| HaversError::CompileError(format!("Failed to build break: {}", e)))?;
                    Ok(())
                } else {
                    Err(HaversError::CompileError("Break outside loop".to_string()))
                }
            }

            Stmt::Continue { .. } => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    self.builder.build_unconditional_branch(loop_ctx.continue_block)
                        .map_err(|e| HaversError::CompileError(format!("Failed to build continue: {}", e)))?;
                    Ok(())
                } else {
                    Err(HaversError::CompileError("Continue outside loop".to_string()))
                }
            }

            // Not yet implemented
            _ => Err(HaversError::CompileError(format!(
                "Statement not yet supported in LLVM backend: {:?}",
                stmt
            ))),
        }
    }

    // ========== Expression Compilation ==========

    fn compile_expr(&mut self, expr: &Expr) -> Result<BasicValueEnum<'ctx>, HaversError> {
        match expr {
            Expr::Literal { value, .. } => self.compile_literal(value),

            Expr::Variable { name, .. } => {
                if let Some(&alloca) = self.variables.get(name) {
                    let val = self.builder.build_load(self.types.value_type, alloca, name)
                        .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;
                    Ok(val)
                } else {
                    Err(HaversError::CompileError(format!("Undefined variable: {}", name)))
                }
            }

            Expr::Assign { name, value, .. } => {
                let val = self.compile_expr(value)?;
                if let Some(&alloca) = self.variables.get(name) {
                    self.builder.build_store(alloca, val)
                        .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;
                    Ok(val)
                } else {
                    Err(HaversError::CompileError(format!("Undefined variable: {}", name)))
                }
            }

            Expr::Binary { left, operator, right, .. } => {
                self.compile_binary(left, *operator, right)
            }

            Expr::Unary { operator, operand, .. } => {
                self.compile_unary(*operator, operand)
            }

            Expr::Logical { left, operator, right, .. } => {
                self.compile_logical(left, *operator, right)
            }

            Expr::Call { callee, arguments, .. } => {
                self.compile_call(callee, arguments)
            }

            Expr::Grouping { expr, .. } => self.compile_expr(expr),

            Expr::Ternary { condition, then_expr, else_expr, .. } => {
                self.compile_ternary(condition, then_expr, else_expr)
            }

            Expr::Range { start, end, .. } => {
                // For now, just compile as nil - ranges are handled in for loops
                let _start_val = self.compile_expr(start)?;
                let _end_val = self.compile_expr(end)?;
                Ok(self.make_nil())
            }

            // Not yet implemented
            _ => Err(HaversError::CompileError(format!(
                "Expression not yet supported in LLVM backend: {:?}",
                expr
            ))),
        }
    }

    fn compile_literal(&mut self, literal: &Literal) -> Result<BasicValueEnum<'ctx>, HaversError> {
        match literal {
            Literal::Nil => Ok(self.make_nil()),

            Literal::Bool(b) => {
                let bool_val = self.types.bool_type.const_int(*b as u64, false);
                self.make_bool(bool_val)
            }

            Literal::Integer(n) => {
                let int_val = self.types.i64_type.const_int(*n as u64, true);
                self.make_int(int_val)
            }

            Literal::Float(f) => {
                let float_val = self.types.f64_type.const_float(*f);
                self.make_float(float_val)
            }

            Literal::String(s) => {
                let str_ptr = self.builder.build_global_string_ptr(s, "str")
                    .map_err(|e| HaversError::CompileError(format!("Failed to create string: {}", e)))?;
                self.make_string(str_ptr.as_pointer_value())
            }
        }
    }

    fn compile_binary(&mut self, left: &Expr, op: BinaryOp, right: &Expr) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_val = self.compile_expr(left)?;
        let right_val = self.compile_expr(right)?;

        match op {
            BinaryOp::Add => self.inline_add(left_val, right_val),
            BinaryOp::Subtract => self.inline_sub(left_val, right_val),
            BinaryOp::Multiply => self.inline_mul(left_val, right_val),
            BinaryOp::Divide => self.inline_div(left_val, right_val),
            BinaryOp::Modulo => self.inline_mod(left_val, right_val),
            BinaryOp::Equal => self.inline_eq(left_val, right_val),
            BinaryOp::NotEqual => self.inline_ne(left_val, right_val),
            BinaryOp::Less => self.inline_lt(left_val, right_val),
            BinaryOp::LessEqual => self.inline_le(left_val, right_val),
            BinaryOp::Greater => self.inline_gt(left_val, right_val),
            BinaryOp::GreaterEqual => self.inline_ge(left_val, right_val),
        }
    }

    fn compile_unary(&mut self, op: UnaryOp, operand: &Expr) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let val = self.compile_expr(operand)?;
        match op {
            UnaryOp::Negate => self.inline_neg(val),
            UnaryOp::Not => self.inline_not(val),
        }
    }

    fn compile_logical(&mut self, left: &Expr, op: LogicalOp, right: &Expr) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();
        let left_val = self.compile_expr(left)?;
        let left_truthy = self.is_truthy(left_val)?;

        let eval_right = self.context.append_basic_block(function, "eval_right");
        let merge = self.context.append_basic_block(function, "merge");

        match op {
            LogicalOp::And => {
                self.builder.build_conditional_branch(left_truthy, eval_right, merge).unwrap();
            }
            LogicalOp::Or => {
                self.builder.build_conditional_branch(left_truthy, merge, eval_right).unwrap();
            }
        }

        let left_block = self.builder.get_insert_block().unwrap();
        self.builder.position_at_end(eval_right);
        let right_val = self.compile_expr(right)?;
        let right_block = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(merge).unwrap();

        self.builder.position_at_end(merge);
        let phi = self.builder.build_phi(self.types.value_type, "logical").unwrap();
        phi.add_incoming(&[(&left_val, left_block), (&right_val, right_block)]);

        Ok(phi.as_basic_value())
    }

    fn compile_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<BasicValueEnum<'ctx>, HaversError> {
        if let Expr::Variable { name, .. } = callee {
            // Check user-defined functions
            if let Some(&func) = self.functions.get(name) {
                let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::new();
                for arg in args {
                    compiled_args.push(self.compile_expr(arg)?.into());
                }

                let result = self.builder.build_call(func, &compiled_args, "call")
                    .map_err(|e| HaversError::CompileError(format!("Failed to call: {}", e)))?;
                return Ok(result.try_as_basic_value().left().unwrap());
            }
        }

        Err(HaversError::CompileError(format!("Unknown function: {:?}", callee)))
    }

    fn compile_if(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: Option<&Stmt>) -> Result<(), HaversError> {
        let function = self.current_function.unwrap();
        let cond_val = self.compile_expr(condition)?;
        let cond_bool = self.is_truthy(cond_val)?;

        let then_block = self.context.append_basic_block(function, "then");
        let else_block = self.context.append_basic_block(function, "else");
        let merge_block = self.context.append_basic_block(function, "merge");

        self.builder.build_conditional_branch(cond_bool, then_block, else_block).unwrap();

        // Then branch
        self.builder.position_at_end(then_block);
        self.compile_stmt(then_branch)?;
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(merge_block).unwrap();
        }

        // Else branch
        self.builder.position_at_end(else_block);
        if let Some(else_stmt) = else_branch {
            self.compile_stmt(else_stmt)?;
        }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(merge_block).unwrap();
        }

        self.builder.position_at_end(merge_block);
        Ok(())
    }

    fn compile_while(&mut self, condition: &Expr, body: &Stmt) -> Result<(), HaversError> {
        let function = self.current_function.unwrap();

        let loop_block = self.context.append_basic_block(function, "loop");
        let body_block = self.context.append_basic_block(function, "body");
        let after_block = self.context.append_basic_block(function, "after");

        self.loop_stack.push(LoopContext {
            break_block: after_block,
            continue_block: loop_block,
        });

        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        let cond_val = self.compile_expr(condition)?;
        let cond_bool = self.is_truthy(cond_val)?;
        self.builder.build_conditional_branch(cond_bool, body_block, after_block).unwrap();

        self.builder.position_at_end(body_block);
        self.compile_stmt(body)?;
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(loop_block).unwrap();
        }

        self.loop_stack.pop();
        self.builder.position_at_end(after_block);
        Ok(())
    }

    fn compile_for(&mut self, variable: &str, iterable: &Expr, body: &Stmt) -> Result<(), HaversError> {
        if let Expr::Range { start, end, inclusive, .. } = iterable {
            return self.compile_for_range(variable, start, end, *inclusive, body);
        }
        Err(HaversError::CompileError("For loop over non-range not yet supported".to_string()))
    }

    fn compile_for_range(&mut self, variable: &str, start: &Expr, end: &Expr, inclusive: bool, body: &Stmt) -> Result<(), HaversError> {
        let function = self.current_function.unwrap();

        let start_val = self.compile_expr(start)?;
        let end_val = self.compile_expr(end)?;

        let start_data = self.extract_data(start_val)?;
        let end_data = self.extract_data(end_val)?;

        // Create loop variable
        let var_alloca = self.create_entry_block_alloca(variable);
        let start_mdh = self.make_int(start_data)?;
        self.builder.build_store(var_alloca, start_mdh).unwrap();
        self.variables.insert(variable.to_string(), var_alloca);

        // Create counter
        let counter_alloca = self.builder.build_alloca(self.types.i64_type, "counter").unwrap();
        self.builder.build_store(counter_alloca, start_data).unwrap();

        let loop_block = self.context.append_basic_block(function, "for_loop");
        let body_block = self.context.append_basic_block(function, "for_body");
        let incr_block = self.context.append_basic_block(function, "for_incr");
        let after_block = self.context.append_basic_block(function, "for_after");

        self.loop_stack.push(LoopContext {
            break_block: after_block,
            continue_block: incr_block,
        });

        self.builder.build_unconditional_branch(loop_block).unwrap();

        // Loop condition
        self.builder.position_at_end(loop_block);
        let current = self.builder.build_load(self.types.i64_type, counter_alloca, "current").unwrap().into_int_value();
        let cmp = if inclusive {
            self.builder.build_int_compare(IntPredicate::SLE, current, end_data, "cmp")
        } else {
            self.builder.build_int_compare(IntPredicate::SLT, current, end_data, "cmp")
        }.unwrap();
        self.builder.build_conditional_branch(cmp, body_block, after_block).unwrap();

        // Body
        self.builder.position_at_end(body_block);
        self.compile_stmt(body)?;
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(incr_block).unwrap();
        }

        // Increment
        self.builder.position_at_end(incr_block);
        let one = self.types.i64_type.const_int(1, false);
        let next = self.builder.build_int_add(current, one, "next").unwrap();
        self.builder.build_store(counter_alloca, next).unwrap();

        let next_mdh = self.make_int(next)?;
        self.builder.build_store(var_alloca, next_mdh).unwrap();

        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.loop_stack.pop();
        self.builder.position_at_end(after_block);
        Ok(())
    }

    fn compile_function(&mut self, name: &str, params: &[crate::ast::Param], body: &[Stmt]) -> Result<(), HaversError> {
        let function = self.functions.get(name).copied()
            .ok_or_else(|| HaversError::CompileError(format!("Function not declared: {}", name)))?;

        let entry = self.context.append_basic_block(function, "entry");

        let saved_function = self.current_function;
        let saved_variables = std::mem::take(&mut self.variables);

        self.builder.position_at_end(entry);
        self.current_function = Some(function);

        // Set up parameters
        for (i, param) in params.iter().enumerate() {
            let param_val = function.get_nth_param(i as u32)
                .ok_or_else(|| HaversError::CompileError("Missing parameter".to_string()))?;
            let alloca = self.create_entry_block_alloca(&param.name);
            self.builder.build_store(alloca, param_val).unwrap();
            self.variables.insert(param.name.clone(), alloca);
        }

        // Compile body
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // Add implicit return if needed
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_return(Some(&self.make_nil())).unwrap();
        }

        // Restore state
        self.current_function = saved_function;
        self.variables = saved_variables;

        if let Some(func) = saved_function {
            if let Some(last_block) = func.get_last_basic_block() {
                self.builder.position_at_end(last_block);
            }
        }

        Ok(())
    }

    fn compile_ternary(&mut self, condition: &Expr, then_expr: &Expr, else_expr: &Expr) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        let cond_val = self.compile_expr(condition)?;
        let cond_bool = self.is_truthy(cond_val)?;

        let then_block = self.context.append_basic_block(function, "tern_then");
        let else_block = self.context.append_basic_block(function, "tern_else");
        let merge_block = self.context.append_basic_block(function, "tern_merge");

        self.builder.build_conditional_branch(cond_bool, then_block, else_block).unwrap();

        self.builder.position_at_end(then_block);
        let then_val = self.compile_expr(then_expr)?;
        let then_bb = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(merge_block).unwrap();

        self.builder.position_at_end(else_block);
        let else_val = self.compile_expr(else_expr)?;
        let else_bb = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(merge_block).unwrap();

        self.builder.position_at_end(merge_block);
        let phi = self.builder.build_phi(self.types.value_type, "tern").unwrap();
        phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);

        Ok(phi.as_basic_value())
    }

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
