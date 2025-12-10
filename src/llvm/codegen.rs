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
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
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
#[allow(dead_code)]
struct LibcFunctions<'ctx> {
    printf: FunctionValue<'ctx>,
    malloc: FunctionValue<'ctx>,
    realloc: FunctionValue<'ctx>,
    strlen: FunctionValue<'ctx>,
    strcpy: FunctionValue<'ctx>,
    strcat: FunctionValue<'ctx>,
    snprintf: FunctionValue<'ctx>,
    exit: FunctionValue<'ctx>,
    strstr: FunctionValue<'ctx>,
    strcmp: FunctionValue<'ctx>,
    memcpy: FunctionValue<'ctx>,
    toupper: FunctionValue<'ctx>,
    tolower: FunctionValue<'ctx>,
    isspace: FunctionValue<'ctx>,
    // Phase 5: Timing functions
    clock_gettime: FunctionValue<'ctx>,
    nanosleep: FunctionValue<'ctx>,
    // Phase 7: I/O functions
    fgets: FunctionValue<'ctx>,
    // Extra: string operations
    strdup: FunctionValue<'ctx>,
    // Extra: random/time
    rand: FunctionValue<'ctx>,
    srand: FunctionValue<'ctx>,
    time: FunctionValue<'ctx>,
    qsort: FunctionValue<'ctx>,
    // Runtime functions
    get_key: FunctionValue<'ctx>,
    random: FunctionValue<'ctx>,
}

/// Inferred type for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarType {
    Unknown,
    Int,
    Float,
    String,
    Bool,
    List,
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

    /// Global variable storage (name -> global pointer) - accessible from all functions
    globals: HashMap<String, PointerValue<'ctx>>,

    /// Shadow i64 storage for integer variables (optimization)
    /// When a variable is known to be Int, we keep an unboxed i64 version
    int_shadows: HashMap<String, PointerValue<'ctx>>,

    /// Inferred types for variables (for optimization)
    var_types: HashMap<String, VarType>,

    /// User-defined functions
    functions: HashMap<String, FunctionValue<'ctx>>,

    /// Loop context stack for break/continue
    loop_stack: Vec<LoopContext<'ctx>>,

    /// Track if we're in a hot loop body (skip MdhValue stores)
    in_loop_body: bool,

    /// Counter for generating unique lambda names
    lambda_counter: u32,

    /// Class definitions: name -> global variable holding class data pointer
    classes: HashMap<String, inkwell::values::GlobalValue<'ctx>>,

    /// Class method tables: class_name -> [(method_name, function)]
    class_methods: HashMap<String, Vec<(String, FunctionValue<'ctx>)>>,

    /// Current 'masel' value (set during method execution)
    current_masel: Option<PointerValue<'ctx>>,

    /// Current class name being compiled (for method naming)
    current_class: Option<String>,

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

        // Set target triple and data layout for x86_64 Linux
        // This ensures proper struct alignment (i64 aligned to 8 bytes)
        use inkwell::targets::{InitializationConfig, Target, TargetTriple};
        Target::initialize_native(&InitializationConfig::default()).unwrap();
        let triple = TargetTriple::create("x86_64-unknown-linux-gnu");
        module.set_triple(&triple);

        // Standard x86_64 data layout - i64 aligned to 8 bytes
        let data_layout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128";
        module
            .set_data_layout(&inkwell::targets::TargetData::create(data_layout).get_data_layout());

        let builder = context.create_builder();
        let types = MdhTypes::new(context);

        // Declare libc functions
        let libc = Self::declare_libc_functions(&module, context, &types);

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
            globals: HashMap::new(),
            int_shadows: HashMap::new(),
            var_types: HashMap::new(),
            functions: HashMap::new(),
            loop_stack: Vec::new(),
            in_loop_body: false,
            lambda_counter: 0,
            classes: HashMap::new(),
            class_methods: HashMap::new(),
            current_masel: None,
            current_class: None,
            fmt_int,
            fmt_float,
            fmt_string,
            fmt_true,
            fmt_false,
            fmt_nil,
            fmt_newline,
        }
    }

    fn declare_libc_functions(
        module: &Module<'ctx>,
        context: &'ctx Context,
        types: &MdhTypes<'ctx>,
    ) -> LibcFunctions<'ctx> {
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

        // realloc(void*, size_t) -> void*
        let realloc_type = i8_ptr.fn_type(&[i8_ptr.into(), i64_type.into()], false);
        let realloc = module.add_function("realloc", realloc_type, Some(Linkage::External));

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
        let snprintf_type =
            i32_type.fn_type(&[i8_ptr.into(), i64_type.into(), i8_ptr.into()], true);
        let snprintf = module.add_function("snprintf", snprintf_type, Some(Linkage::External));

        // exit(int) -> void
        let exit_type = void_type.fn_type(&[i32_type.into()], false);
        let exit = module.add_function("exit", exit_type, Some(Linkage::External));

        // strstr(const char*, const char*) -> char*
        let strstr_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let strstr = module.add_function("strstr", strstr_type, Some(Linkage::External));

        // strcmp(const char*, const char*) -> int
        let strcmp_type = i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let strcmp = module.add_function("strcmp", strcmp_type, Some(Linkage::External));

        // memcpy(void* dest, const void* src, size_t n) -> void*
        let memcpy_type = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into(), i64_type.into()], false);
        let memcpy = module.add_function("memcpy", memcpy_type, Some(Linkage::External));

        // toupper(int) -> int
        let toupper_type = i32_type.fn_type(&[i32_type.into()], false);
        let toupper = module.add_function("toupper", toupper_type, Some(Linkage::External));

        // tolower(int) -> int
        let tolower_type = i32_type.fn_type(&[i32_type.into()], false);
        let tolower = module.add_function("tolower", tolower_type, Some(Linkage::External));

        // isspace(int) -> int
        let isspace_type = i32_type.fn_type(&[i32_type.into()], false);
        let isspace = module.add_function("isspace", isspace_type, Some(Linkage::External));

        // clock_gettime(clockid_t, struct timespec*) -> int
        // struct timespec is {i64 tv_sec, i64 tv_nsec}
        let clock_gettime_type = i32_type.fn_type(&[i32_type.into(), i8_ptr.into()], false);
        let clock_gettime =
            module.add_function("clock_gettime", clock_gettime_type, Some(Linkage::External));

        // nanosleep(const struct timespec*, struct timespec*) -> int
        let nanosleep_type = i32_type.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        let nanosleep = module.add_function("nanosleep", nanosleep_type, Some(Linkage::External));

        // fgets(char* buf, int size, FILE* stream) -> char*
        let fgets_type = i8_ptr.fn_type(&[i8_ptr.into(), i32_type.into(), i8_ptr.into()], false);
        let fgets = module.add_function("fgets", fgets_type, Some(Linkage::External));

        // strdup(const char*) -> char* (allocates a copy)
        let strdup_type = i8_ptr.fn_type(&[i8_ptr.into()], false);
        let strdup = module.add_function("strdup", strdup_type, Some(Linkage::External));

        // rand() -> int
        let rand_type = i32_type.fn_type(&[], false);
        let rand = module.add_function("rand", rand_type, Some(Linkage::External));

        // srand(unsigned int) -> void
        let srand_type = void_type.fn_type(&[i32_type.into()], false);
        let srand = module.add_function("srand", srand_type, Some(Linkage::External));

        // time(time_t*) -> time_t (pass NULL to get current time)
        let time_type = i64_type.fn_type(&[i8_ptr.into()], false);
        let time = module.add_function("time", time_type, Some(Linkage::External));

        // qsort(void*, size_t, size_t, comparator) - we won't use this directly
        let qsort_type = void_type.fn_type(
            &[
                i8_ptr.into(),
                i64_type.into(),
                i64_type.into(),
                i8_ptr.into(),
            ],
            false,
        );
        let qsort = module.add_function("qsort", qsort_type, Some(Linkage::External));

        // __mdh_get_key() -> MdhValue
        let get_key_type = types.value_type.fn_type(&[], false);
        let get_key = module.add_function("__mdh_get_key", get_key_type, Some(Linkage::External));

        // __mdh_random(i64 min, i64 max) -> MdhValue
        let random_type = types
            .value_type
            .fn_type(&[i64_type.into(), i64_type.into()], false);
        let random = module.add_function("__mdh_random", random_type, Some(Linkage::External));

        LibcFunctions {
            printf,
            malloc,
            realloc,
            strlen,
            strcpy,
            strcat,
            snprintf,
            exit,
            strstr,
            strcmp,
            memcpy,
            toupper,
            tolower,
            isspace,
            clock_gettime,
            nanosleep,
            fgets,
            strdup,
            rand,
            srand,
            time,
            qsort,
            get_key,
            random,
        }
    }

    fn create_global_string(
        module: &Module<'ctx>,
        context: &'ctx Context,
        s: &str,
        name: &str,
    ) -> inkwell::values::GlobalValue<'ctx> {
        let bytes: Vec<u8> = s.bytes().chain(std::iter::once(0)).collect();
        let arr_type = context.i8_type().array_type(bytes.len() as u32);
        let global = module.add_global(arr_type, Some(AddressSpace::default()), name);
        global.set_linkage(Linkage::Private);
        global.set_constant(true);
        let values: Vec<_> = bytes
            .iter()
            .map(|b| context.i8_type().const_int(*b as u64, false))
            .collect();
        global.set_initializer(&context.i8_type().const_array(&values));
        global
    }

    fn get_string_ptr(&self, global: inkwell::values::GlobalValue<'ctx>) -> PointerValue<'ctx> {
        self.builder
            .build_pointer_cast(
                global.as_pointer_value(),
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "str_ptr",
            )
            .unwrap()
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

    /// Create a bool value: {tag=1, data=0|1}
    fn make_bool(&self, val: IntValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self
            .types
            .i8_type
            .const_int(ValueTag::Bool.as_u8() as u64, false);
        let data = self
            .builder
            .build_int_z_extend(val, self.types.i64_type, "bool_ext")
            .map_err(|e| HaversError::CompileError(format!("Failed to extend bool: {}", e)))?;

        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self
            .builder
            .build_insert_value(v1, data, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create an int value: {tag=2, data=i64}
    fn make_int(&self, val: IntValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);

        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self
            .builder
            .build_insert_value(v1, val, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create a float value: {tag=3, data=bitcast(f64)}
    fn make_float(
        &self,
        val: inkwell::values::FloatValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);
        let data = self
            .builder
            .build_bitcast(val, self.types.i64_type, "float_bits")
            .map_err(|e| HaversError::CompileError(format!("Failed to bitcast float: {}", e)))?;

        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self
            .builder
            .build_insert_value(v1, data, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create a string value: {tag=4, data=ptr as i64}
    fn make_string(&self, ptr: PointerValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self
            .types
            .i8_type
            .const_int(ValueTag::String.as_u8() as u64, false);
        let data = self
            .builder
            .build_ptr_to_int(ptr, self.types.i64_type, "str_ptr_int")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert ptr: {}", e)))?;

        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self
            .builder
            .build_insert_value(v1, data, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create a list value: {tag=5, data=ptr as i64}
    /// List memory layout: [i64 length, {i8,i64} element0, {i8,i64} element1, ...]
    fn make_list(&self, ptr: PointerValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self
            .types
            .i8_type
            .const_int(ValueTag::List.as_u8() as u64, false);
        let data = self
            .builder
            .build_ptr_to_int(ptr, self.types.i64_type, "list_ptr_int")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert ptr: {}", e)))?;

        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self
            .builder
            .build_insert_value(v1, data, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create a dict value: {tag=6, data=ptr as i64}
    /// Dict memory layout: [i64 count][entry0][entry1]... where entry = [{i8,i64} key][{i8,i64} val]
    fn make_dict(&self, ptr: PointerValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self
            .types
            .i8_type
            .const_int(ValueTag::Dict.as_u8() as u64, false);
        let data = self
            .builder
            .build_ptr_to_int(ptr, self.types.i64_type, "dict_ptr_int")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert ptr: {}", e)))?;

        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self
            .builder
            .build_insert_value(v1, data, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create an instance value: {tag=9, data=ptr as i64}
    /// Instance memory layout: [i64 class_name_ptr][i64 field_count][field_entry0][field_entry1]...
    /// where field_entry = [{i8,i64} key (string)][{i8,i64} value]
    fn make_instance(&self, ptr: PointerValue<'ctx>) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self
            .types
            .i8_type
            .const_int(ValueTag::Instance.as_u8() as u64, false);
        let data = self
            .builder
            .build_ptr_to_int(ptr, self.types.i64_type, "instance_ptr_int")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert ptr: {}", e)))?;

        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self
            .builder
            .build_insert_value(v1, data, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Create a function value: {tag=7, data=func_ptr as i64}
    fn make_function(
        &self,
        func_ptr_int: IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self
            .types
            .i8_type
            .const_int(ValueTag::Function.as_u8() as u64, false);

        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, tag, 0, "v1")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert tag: {}", e)))?;
        let v2 = self
            .builder
            .build_insert_value(v1, func_ptr_int, 1, "v2")
            .map_err(|e| HaversError::CompileError(format!("Failed to insert data: {}", e)))?;

        Ok(v2.into_struct_value().into())
    }

    /// Extract data as list pointer
    #[allow(dead_code)]
    fn extract_list_ptr(
        &self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<PointerValue<'ctx>, HaversError> {
        let data = self.extract_data(val)?;
        let ptr = self
            .builder
            .build_int_to_ptr(
                data,
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "as_list",
            )
            .map_err(|e| HaversError::CompileError(format!("Failed to convert to ptr: {}", e)))?;
        Ok(ptr)
    }

    /// Extract tag from value
    fn extract_tag(&self, val: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, HaversError> {
        let struct_val = val.into_struct_value();
        let tag = self
            .builder
            .build_extract_value(struct_val, 0, "tag")
            .map_err(|e| HaversError::CompileError(format!("Failed to extract tag: {}", e)))?;
        Ok(tag.into_int_value())
    }

    /// Extract data from value as i64
    fn extract_data(&self, val: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, HaversError> {
        let struct_val = val.into_struct_value();
        let data = self
            .builder
            .build_extract_value(struct_val, 1, "data")
            .map_err(|e| HaversError::CompileError(format!("Failed to extract data: {}", e)))?;
        Ok(data.into_int_value())
    }

    /// Extract data as f64 (for float values)
    #[allow(dead_code)]
    fn extract_float(
        &self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::FloatValue<'ctx>, HaversError> {
        let data = self.extract_data(val)?;
        let float_val = self
            .builder
            .build_bitcast(data, self.types.f64_type, "as_float")
            .map_err(|e| HaversError::CompileError(format!("Failed to bitcast to float: {}", e)))?;
        Ok(float_val.into_float_value())
    }

    /// Extract data as string pointer
    #[allow(dead_code)]
    fn extract_string_ptr(
        &self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<PointerValue<'ctx>, HaversError> {
        let data = self.extract_data(val)?;
        let ptr = self
            .builder
            .build_int_to_ptr(
                data,
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "as_str",
            )
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
        let nil_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Nil.as_u8() as u64, false);
        let bool_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Bool.as_u8() as u64, false);
        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);

        self.builder
            .build_switch(
                tag,
                is_other,
                &[(nil_tag, is_nil), (bool_tag, is_bool), (int_tag, is_int)],
            )
            .map_err(|e| HaversError::CompileError(format!("Failed to build switch: {}", e)))?;

        // nil -> false
        self.builder.position_at_end(is_nil);
        let nil_result = self.types.bool_type.const_int(0, false);
        self.builder.build_unconditional_branch(merge).unwrap();
        let nil_block = self.builder.get_insert_block().unwrap();

        // bool -> value
        self.builder.position_at_end(is_bool);
        let bool_result = self
            .builder
            .build_int_truncate(data, self.types.bool_type, "bool_val")
            .unwrap();
        self.builder.build_unconditional_branch(merge).unwrap();
        let bool_block = self.builder.get_insert_block().unwrap();

        // int -> value != 0
        self.builder.position_at_end(is_int);
        let zero = self.types.i64_type.const_int(0, false);
        let int_result = self
            .builder
            .build_int_compare(IntPredicate::NE, data, zero, "int_truthy")
            .unwrap();
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // other -> true
        self.builder.position_at_end(is_other);
        let other_result = self.types.bool_type.const_int(1, false);
        self.builder.build_unconditional_branch(merge).unwrap();
        let other_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self
            .builder
            .build_phi(self.types.bool_type, "truthy")
            .unwrap();
        phi.add_incoming(&[
            (&nil_result, nil_block),
            (&bool_result, bool_block),
            (&int_result, int_block),
            (&other_result, other_block),
        ]);

        Ok(phi.as_basic_value().into_int_value())
    }

    /// Compile a condition expression directly to i1 boolean, bypassing MdhValue boxing.
    /// This is an optimization for loop conditions and if statements.
    /// Returns None if the expression can't be optimized (falls back to is_truthy).
    fn compile_condition_direct(
        &mut self,
        expr: &Expr,
    ) -> Result<Option<IntValue<'ctx>>, HaversError> {
        match expr {
            // Comparison operations can return i1 directly
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                match operator {
                    // For equality comparisons, check if either operand could be a string
                    // If so, fall back to full compilation which uses strcmp
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        let left_type = self.infer_expr_type(left);
                        let right_type = self.infer_expr_type(right);
                        // Only use fast path if both are known to be non-string types
                        if left_type == VarType::String
                            || right_type == VarType::String
                            || left_type == VarType::Unknown
                            || right_type == VarType::Unknown
                        {
                            // Fall back to full compilation with strcmp support
                            return Ok(None);
                        }
                        // Both are known non-string types, safe to compare data directly
                        let get_int_data =
                            |s: &mut Self, expr: &Expr| -> Result<IntValue<'ctx>, HaversError> {
                                if let Some(int_val) = s.compile_int_expr(expr)? {
                                    return Ok(int_val);
                                }
                                let val = s.compile_expr(expr)?;
                                s.extract_data(val)
                            };
                        let left_data = get_int_data(self, left)?;
                        let right_data = get_int_data(self, right)?;
                        let pred = match operator {
                            BinaryOp::Equal => IntPredicate::EQ,
                            BinaryOp::NotEqual => IntPredicate::NE,
                            _ => unreachable!(),
                        };
                        let result = self
                            .builder
                            .build_int_compare(pred, left_data, right_data, "cmp_direct")
                            .unwrap();
                        Ok(Some(result))
                    }
                    BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual => {
                        // Helper to get i64 value from expression (shadow or extract)
                        let get_int_data =
                            |s: &mut Self, expr: &Expr| -> Result<IntValue<'ctx>, HaversError> {
                                // First try int shadow path
                                if let Some(int_val) = s.compile_int_expr(expr)? {
                                    return Ok(int_val);
                                }
                                // Fall back to MdhValue extraction
                                let val = s.compile_expr(expr)?;
                                s.extract_data(val)
                            };

                        let left_data = get_int_data(self, left)?;
                        let right_data = get_int_data(self, right)?;

                        let pred = match operator {
                            BinaryOp::Less => IntPredicate::SLT,
                            BinaryOp::LessEqual => IntPredicate::SLE,
                            BinaryOp::Greater => IntPredicate::SGT,
                            BinaryOp::GreaterEqual => IntPredicate::SGE,
                            _ => unreachable!(),
                        };

                        let result = self
                            .builder
                            .build_int_compare(pred, left_data, right_data, "cmp_direct")
                            .unwrap();
                        Ok(Some(result))
                    }
                    _ => Ok(None), // Other binary ops need full compilation
                }
            }
            // Boolean literals
            Expr::Literal {
                value: Literal::Bool(b),
                ..
            } => {
                let result = self
                    .types
                    .bool_type
                    .const_int(if *b { 1 } else { 0 }, false);
                Ok(Some(result))
            }
            // Boolean variable - extract and compare to 0
            Expr::Variable { name, .. } => {
                if self.var_types.get(name) == Some(&VarType::Bool) {
                    let val = self.compile_expr(expr)?;
                    let data = self.extract_data(val)?;
                    let zero = self.types.i64_type.const_int(0, false);
                    let result = self.builder.build_int_compare(IntPredicate::NE, data, zero, "bool_truthy").unwrap();
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            }
            // Index expression - if result type is bool, optimize truthiness check
            Expr::Index { .. } => {
                // Compile the index and check if non-zero
                // This works for booleans stored as MdhValue{tag:1, data:0|1}
                let val = self.compile_expr(expr)?;
                let data = self.extract_data(val)?;
                let zero = self.types.i64_type.const_int(0, false);
                let result = self.builder.build_int_compare(IntPredicate::NE, data, zero, "index_truthy").unwrap();
                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    /// Infer the type of an expression for optimization purposes
    fn infer_expr_type(&self, expr: &Expr) -> VarType {
        match expr {
            Expr::Literal { value, .. } => match value {
                Literal::Integer(_) => VarType::Int,
                Literal::Float(_) => VarType::Float,
                Literal::String(_) => VarType::String,
                Literal::Bool(_) => VarType::Bool,
                Literal::Nil => VarType::Unknown,
            },
            Expr::Variable { name, .. } => self
                .var_types
                .get(name)
                .copied()
                .unwrap_or(VarType::Unknown),
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => match operator {
                BinaryOp::Add
                | BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Modulo => {
                    let lt = self.infer_expr_type(left);
                    let rt = self.infer_expr_type(right);
                    if lt == VarType::Int && rt == VarType::Int {
                        VarType::Int
                    } else if lt == VarType::Float || rt == VarType::Float {
                        VarType::Float
                    } else {
                        VarType::Unknown
                    }
                }
                BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
                | BinaryOp::Equal
                | BinaryOp::NotEqual => VarType::Bool,
            },
            Expr::List { .. } => VarType::List,
            Expr::Unary { operand, .. } => self.infer_expr_type(operand),
            _ => VarType::Unknown,
        }
    }

    /// Compile an integer expression directly to i64, bypassing MdhValue boxing.
    /// Returns None if the expression can't be compiled as pure integer.
    fn compile_int_expr(&mut self, expr: &Expr) -> Result<Option<IntValue<'ctx>>, HaversError> {
        match expr {
            // Integer literal
            Expr::Literal {
                value: Literal::Integer(n),
                ..
            } => Ok(Some(self.types.i64_type.const_int(*n as u64, true))),

            // Variable with int shadow
            Expr::Variable { name, .. } => {
                if let Some(&shadow) = self.int_shadows.get(name) {
                    let val = self
                        .builder
                        .build_load(self.types.i64_type, shadow, &format!("{}_i64", name))
                        .map_err(|e| {
                            HaversError::CompileError(format!("Failed to load shadow: {}", e))
                        })?;
                    Ok(Some(val.into_int_value()))
                } else if self.var_types.get(name) == Some(&VarType::Int) {
                    // Known int but no shadow - extract from MdhValue
                    if let Some(&alloca) = self.variables.get(name) {
                        let val = self
                            .builder
                            .build_load(self.types.value_type, alloca, name)
                            .map_err(|e| {
                                HaversError::CompileError(format!("Failed to load: {}", e))
                            })?;
                        let data = self.extract_data(val)?;
                        Ok(Some(data))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }

            // Binary operations on integers
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let lt = self.infer_expr_type(left);
                let rt = self.infer_expr_type(right);

                if lt == VarType::Int && rt == VarType::Int {
                    match operator {
                        BinaryOp::Add
                        | BinaryOp::Subtract
                        | BinaryOp::Multiply
                        | BinaryOp::Divide
                        | BinaryOp::Modulo => {
                            let left_i64 = self.compile_int_expr(left)?;
                            let right_i64 = self.compile_int_expr(right)?;

                            if let (Some(l), Some(r)) = (left_i64, right_i64) {
                                let result = match operator {
                                    BinaryOp::Add => {
                                        self.builder.build_int_add(l, r, "add_i64").unwrap()
                                    }
                                    BinaryOp::Subtract => {
                                        self.builder.build_int_sub(l, r, "sub_i64").unwrap()
                                    }
                                    BinaryOp::Multiply => {
                                        self.builder.build_int_mul(l, r, "mul_i64").unwrap()
                                    }
                                    BinaryOp::Divide => {
                                        self.builder.build_int_signed_div(l, r, "div_i64").unwrap()
                                    }
                                    BinaryOp::Modulo => {
                                        self.builder.build_int_signed_rem(l, r, "mod_i64").unwrap()
                                    }
                                    _ => unreachable!(),
                                };
                                return Ok(Some(result));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(None)
            }

            _ => Ok(None),
        }
    }

    /// Sync all int shadows back to their MdhValue counterparts
    /// Called at loop exit to ensure variables are up-to-date
    fn sync_all_shadows(&mut self) -> Result<(), HaversError> {
        // Collect names first to avoid borrow issues
        let shadow_names: Vec<String> = self.int_shadows.keys().cloned().collect();

        for name in shadow_names {
            if let (Some(&shadow), Some(&alloca)) =
                (self.int_shadows.get(&name), self.variables.get(&name))
            {
                // Load from shadow
                let int_val = self
                    .builder
                    .build_load(self.types.i64_type, shadow, &format!("{}_sync", name))
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to load shadow: {}", e))
                    })?
                    .into_int_value();
                // Box to MdhValue
                let boxed = self.make_int(int_val)?;
                // Store to MdhValue
                self.builder
                    .build_store(alloca, boxed)
                    .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;
            }
        }
        Ok(())
    }

    // ========== Inline Arithmetic ==========

    /// Add two values with type checking
    fn inline_add(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
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

        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);
        let string_tag = self
            .types
            .i8_type
            .const_int(ValueTag::String.as_u8() as u64, false);

        // Check if both are ints
        let left_is_int = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, int_tag, "l_int")
            .unwrap();
        let right_is_int = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, int_tag, "r_int")
            .unwrap();
        let both_int = self
            .builder
            .build_and(left_is_int, right_is_int, "both_int")
            .unwrap();

        // Check if either is float
        let left_is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, float_tag, "l_float")
            .unwrap();
        let right_is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, float_tag, "r_float")
            .unwrap();
        let either_float = self
            .builder
            .build_or(left_is_float, right_is_float, "either_float")
            .unwrap();

        // Check if both are strings
        let left_is_string = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, string_tag, "l_str")
            .unwrap();
        let right_is_string = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, string_tag, "r_str")
            .unwrap();
        let both_string = self
            .builder
            .build_and(left_is_string, right_is_string, "both_str")
            .unwrap();

        // Branch based on types
        let check_float = self.context.append_basic_block(function, "check_float");
        let check_string = self.context.append_basic_block(function, "check_string");

        self.builder
            .build_conditional_branch(both_int, int_int, check_float)
            .unwrap();

        self.builder.position_at_end(check_float);
        self.builder
            .build_conditional_branch(either_float, float_case, check_string)
            .unwrap();

        self.builder.position_at_end(check_string);
        self.builder
            .build_conditional_branch(both_string, string_case, error_case)
            .unwrap();

        // int + int
        self.builder.position_at_end(int_int);
        let int_sum = self
            .builder
            .build_int_add(left_data, right_data, "sum")
            .unwrap();
        let int_result = self.make_int(int_sum)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float + float (or int+float)
        self.builder.position_at_end(float_case);
        // Convert both to float
        let left_f = self
            .builder
            .build_select(
                left_is_float,
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_bitcast(left_data, self.types.f64_type, "lf")
                        .unwrap()
                        .into_float_value(),
                ),
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_signed_int_to_float(left_data, self.types.f64_type, "li2f")
                        .unwrap(),
                ),
                "left_as_float",
            )
            .unwrap()
            .into_float_value();
        let right_f = self
            .builder
            .build_select(
                right_is_float,
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_bitcast(right_data, self.types.f64_type, "rf")
                        .unwrap()
                        .into_float_value(),
                ),
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_signed_int_to_float(right_data, self.types.f64_type, "ri2f")
                        .unwrap(),
                ),
                "right_as_float",
            )
            .unwrap()
            .into_float_value();
        let float_sum = self
            .builder
            .build_float_add(left_f, right_f, "fsum")
            .unwrap();
        let float_result = self.make_float(float_sum)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // string + string (concatenation)
        self.builder.position_at_end(string_case);
        let left_ptr = self
            .builder
            .build_int_to_ptr(
                left_data,
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "lstr",
            )
            .unwrap();
        let right_ptr = self
            .builder
            .build_int_to_ptr(
                right_data,
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "rstr",
            )
            .unwrap();

        // Get lengths
        let left_len = self
            .builder
            .build_call(self.libc.strlen, &[left_ptr.into()], "llen")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let right_len = self
            .builder
            .build_call(self.libc.strlen, &[right_ptr.into()], "rlen")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        // Allocate new string (len1 + len2 + 1)
        let total_len = self
            .builder
            .build_int_add(left_len, right_len, "total")
            .unwrap();
        let one = self.types.i64_type.const_int(1, false);
        let alloc_size = self
            .builder
            .build_int_add(total_len, one, "alloc_size")
            .unwrap();
        let new_str = self
            .builder
            .build_call(self.libc.malloc, &[alloc_size.into()], "new_str")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Copy strings using memcpy (faster than strcpy/strcat since we know lengths)
        // memcpy(new_str, left_ptr, left_len)
        self.builder
            .build_call(
                self.libc.memcpy,
                &[new_str.into(), left_ptr.into(), left_len.into()],
                "",
            )
            .unwrap();
        // memcpy(new_str + left_len, right_ptr, right_len + 1) - +1 for null terminator
        let dest_offset = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), new_str, &[left_len], "dest_offset")
                .unwrap()
        };
        let right_len_plus_one = self
            .builder
            .build_int_add(right_len, one, "rlen_plus_one")
            .unwrap();
        self.builder
            .build_call(
                self.libc.memcpy,
                &[dest_offset.into(), right_ptr.into(), right_len_plus_one.into()],
                "",
            )
            .unwrap();

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
        let phi = self
            .builder
            .build_phi(self.types.value_type, "add_result")
            .unwrap();
        phi.add_incoming(&[
            (&int_result, int_block),
            (&float_result, float_block),
            (&string_result, string_block),
            (&error_result, error_block),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Subtract two values
    fn inline_sub(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        let function = self.current_function.unwrap();
        let int_int = self.context.append_basic_block(function, "sub_int");
        let float_case = self.context.append_basic_block(function, "sub_float");
        let merge = self.context.append_basic_block(function, "sub_merge");

        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);

        let left_is_int = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, int_tag, "l_int")
            .unwrap();
        let right_is_int = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, int_tag, "r_int")
            .unwrap();
        let both_int = self
            .builder
            .build_and(left_is_int, right_is_int, "both_int")
            .unwrap();

        self.builder
            .build_conditional_branch(both_int, int_int, float_case)
            .unwrap();

        // int - int
        self.builder.position_at_end(int_int);
        let int_diff = self
            .builder
            .build_int_sub(left_data, right_data, "diff")
            .unwrap();
        let int_result = self.make_int(int_diff)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float case
        self.builder.position_at_end(float_case);
        let left_is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, float_tag, "lf")
            .unwrap();
        let right_is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, float_tag, "rf")
            .unwrap();
        let left_f = self
            .builder
            .build_select(
                left_is_float,
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_bitcast(left_data, self.types.f64_type, "lf")
                        .unwrap()
                        .into_float_value(),
                ),
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_signed_int_to_float(left_data, self.types.f64_type, "li2f")
                        .unwrap(),
                ),
                "left_as_float",
            )
            .unwrap()
            .into_float_value();
        let right_f = self
            .builder
            .build_select(
                right_is_float,
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_bitcast(right_data, self.types.f64_type, "rf")
                        .unwrap()
                        .into_float_value(),
                ),
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_signed_int_to_float(right_data, self.types.f64_type, "ri2f")
                        .unwrap(),
                ),
                "right_as_float",
            )
            .unwrap()
            .into_float_value();
        let float_diff = self
            .builder
            .build_float_sub(left_f, right_f, "fdiff")
            .unwrap();
        let float_result = self.make_float(float_diff)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "sub_result")
            .unwrap();
        phi.add_incoming(&[(&int_result, int_block), (&float_result, float_block)]);

        Ok(phi.as_basic_value())
    }

    /// Multiply two values
    fn inline_mul(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        let function = self.current_function.unwrap();
        let int_int = self.context.append_basic_block(function, "mul_int");
        let float_case = self.context.append_basic_block(function, "mul_float");
        let merge = self.context.append_basic_block(function, "mul_merge");

        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);

        let left_is_int = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, int_tag, "l_int")
            .unwrap();
        let right_is_int = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, int_tag, "r_int")
            .unwrap();
        let both_int = self
            .builder
            .build_and(left_is_int, right_is_int, "both_int")
            .unwrap();

        self.builder
            .build_conditional_branch(both_int, int_int, float_case)
            .unwrap();

        // int * int
        self.builder.position_at_end(int_int);
        let int_prod = self
            .builder
            .build_int_mul(left_data, right_data, "prod")
            .unwrap();
        let int_result = self.make_int(int_prod)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float case
        self.builder.position_at_end(float_case);
        let left_is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, float_tag, "lf")
            .unwrap();
        let right_is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, float_tag, "rf")
            .unwrap();
        let left_f = self
            .builder
            .build_select(
                left_is_float,
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_bitcast(left_data, self.types.f64_type, "lf")
                        .unwrap()
                        .into_float_value(),
                ),
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_signed_int_to_float(left_data, self.types.f64_type, "li2f")
                        .unwrap(),
                ),
                "left_as_float",
            )
            .unwrap()
            .into_float_value();
        let right_f = self
            .builder
            .build_select(
                right_is_float,
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_bitcast(right_data, self.types.f64_type, "rf")
                        .unwrap()
                        .into_float_value(),
                ),
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_signed_int_to_float(right_data, self.types.f64_type, "ri2f")
                        .unwrap(),
                ),
                "right_as_float",
            )
            .unwrap()
            .into_float_value();
        let float_prod = self
            .builder
            .build_float_mul(left_f, right_f, "fprod")
            .unwrap();
        let float_result = self.make_float(float_prod)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "mul_result")
            .unwrap();
        phi.add_incoming(&[(&int_result, int_block), (&float_result, float_block)]);

        Ok(phi.as_basic_value())
    }

    /// Divide two values
    fn inline_div(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        let function = self.current_function.unwrap();
        let int_int = self.context.append_basic_block(function, "div_int");
        let float_case = self.context.append_basic_block(function, "div_float");
        let merge = self.context.append_basic_block(function, "div_merge");

        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);

        let left_is_int = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, int_tag, "l_int")
            .unwrap();
        let right_is_int = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, int_tag, "r_int")
            .unwrap();
        let both_int = self
            .builder
            .build_and(left_is_int, right_is_int, "both_int")
            .unwrap();

        self.builder
            .build_conditional_branch(both_int, int_int, float_case)
            .unwrap();

        // int / int
        self.builder.position_at_end(int_int);
        let int_quot = self
            .builder
            .build_int_signed_div(left_data, right_data, "quot")
            .unwrap();
        let int_result = self.make_int(int_quot)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float case
        self.builder.position_at_end(float_case);
        let left_is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, float_tag, "lf")
            .unwrap();
        let right_is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, float_tag, "rf")
            .unwrap();
        let left_f = self
            .builder
            .build_select(
                left_is_float,
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_bitcast(left_data, self.types.f64_type, "lf")
                        .unwrap()
                        .into_float_value(),
                ),
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_signed_int_to_float(left_data, self.types.f64_type, "li2f")
                        .unwrap(),
                ),
                "left_as_float",
            )
            .unwrap()
            .into_float_value();
        let right_f = self
            .builder
            .build_select(
                right_is_float,
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_bitcast(right_data, self.types.f64_type, "rf")
                        .unwrap()
                        .into_float_value(),
                ),
                BasicValueEnum::FloatValue(
                    self.builder
                        .build_signed_int_to_float(right_data, self.types.f64_type, "ri2f")
                        .unwrap(),
                ),
                "right_as_float",
            )
            .unwrap()
            .into_float_value();
        let float_quot = self
            .builder
            .build_float_div(left_f, right_f, "fquot")
            .unwrap();
        let float_result = self.make_float(float_quot)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "div_result")
            .unwrap();
        phi.add_incoming(&[(&int_result, int_block), (&float_result, float_block)]);

        Ok(phi.as_basic_value())
    }

    /// Modulo two values
    fn inline_mod(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let rem = self
            .builder
            .build_int_signed_rem(left_data, right_data, "rem")
            .unwrap();
        self.make_int(rem)
    }

    /// Compare two values for equality
    fn inline_eq(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_tag = self.extract_tag(left)?;
        let right_tag = self.extract_tag(right)?;
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;

        // Tags must match first
        let tags_equal = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, right_tag, "tags_eq")
            .unwrap();

        // Check if both are strings (tag == 4)
        let string_tag = self.types.i8_type.const_int(4, false);
        let left_is_string = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_tag, string_tag, "left_is_str")
            .unwrap();
        let right_is_string = self
            .builder
            .build_int_compare(IntPredicate::EQ, right_tag, string_tag, "right_is_str")
            .unwrap();
        let both_strings = self
            .builder
            .build_and(left_is_string, right_is_string, "both_str")
            .unwrap();

        // Create basic blocks for string vs non-string comparison
        let function = self.current_function.unwrap();
        let cmp_string = self.context.append_basic_block(function, "cmp_string");
        let cmp_other = self.context.append_basic_block(function, "cmp_other");
        let cmp_merge = self.context.append_basic_block(function, "cmp_merge");

        self.builder
            .build_conditional_branch(both_strings, cmp_string, cmp_other)
            .unwrap();

        // String comparison: use strcmp
        self.builder.position_at_end(cmp_string);
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let left_str = self
            .builder
            .build_int_to_ptr(left_data, i8_ptr_type, "left_str")
            .unwrap();
        let right_str = self
            .builder
            .build_int_to_ptr(right_data, i8_ptr_type, "right_str")
            .unwrap();
        let strcmp_result = self
            .builder
            .build_call(
                self.libc.strcmp,
                &[left_str.into(), right_str.into()],
                "strcmp_res",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let zero = self.types.i32_type.const_int(0, false);
        let str_equal = self
            .builder
            .build_int_compare(IntPredicate::EQ, strcmp_result, zero, "str_eq")
            .unwrap();
        self.builder.build_unconditional_branch(cmp_merge).unwrap();
        let string_block = self.builder.get_insert_block().unwrap();

        // Non-string comparison: compare data directly
        self.builder.position_at_end(cmp_other);
        let data_equal = self
            .builder
            .build_int_compare(IntPredicate::EQ, left_data, right_data, "data_eq")
            .unwrap();
        // Both tags and data must match for non-strings
        let other_equal = self
            .builder
            .build_and(tags_equal, data_equal, "other_eq")
            .unwrap();
        self.builder.build_unconditional_branch(cmp_merge).unwrap();
        let other_block = self.builder.get_insert_block().unwrap();

        // Merge results
        self.builder.position_at_end(cmp_merge);
        let phi = self
            .builder
            .build_phi(self.types.bool_type, "eq_result")
            .unwrap();
        phi.add_incoming(&[(&str_equal, string_block), (&other_equal, other_block)]);
        let result = phi.as_basic_value().into_int_value();

        self.make_bool(result)
    }

    /// Compare two values for inequality
    fn inline_ne(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Use inline_eq and invert the result
        let eq_result = self.inline_eq(left, right)?;
        // Extract the bool data (0 or 1) and truncate to i1
        let eq_data = self.extract_data(eq_result)?;
        let eq_bool = self
            .builder
            .build_int_truncate(eq_data, self.types.bool_type, "eq_as_bool")
            .unwrap();
        let result = self.builder.build_not(eq_bool, "ne").unwrap();
        self.make_bool(result)
    }

    /// Compare two values: less than
    fn inline_lt(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let result = self
            .builder
            .build_int_compare(IntPredicate::SLT, left_data, right_data, "lt")
            .unwrap();
        self.make_bool(result)
    }

    /// Compare two values: less than or equal
    fn inline_le(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let result = self
            .builder
            .build_int_compare(IntPredicate::SLE, left_data, right_data, "le")
            .unwrap();
        self.make_bool(result)
    }

    /// Compare two values: greater than
    fn inline_gt(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let result = self
            .builder
            .build_int_compare(IntPredicate::SGT, left_data, right_data, "gt")
            .unwrap();
        self.make_bool(result)
    }

    /// Compare two values: greater than or equal
    fn inline_ge(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let left_data = self.extract_data(left)?;
        let right_data = self.extract_data(right)?;
        let result = self
            .builder
            .build_int_compare(IntPredicate::SGE, left_data, right_data, "ge")
            .unwrap();
        self.make_bool(result)
    }

    /// Negate a value
    fn inline_neg(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.extract_tag(val)?;
        let data = self.extract_data(val)?;

        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);

        let is_int = self
            .builder
            .build_int_compare(IntPredicate::EQ, tag, int_tag, "is_int")
            .unwrap();

        let function = self.current_function.unwrap();
        let neg_int = self.context.append_basic_block(function, "neg_int");
        let neg_float = self.context.append_basic_block(function, "neg_float");
        let merge = self.context.append_basic_block(function, "neg_merge");

        self.builder
            .build_conditional_branch(is_int, neg_int, neg_float)
            .unwrap();

        // Negate int
        self.builder.position_at_end(neg_int);
        let neg_data = self.builder.build_int_neg(data, "neg").unwrap();
        let int_result = self.make_int(neg_data)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // Negate float (treat anything non-int as float for simplicity)
        self.builder.position_at_end(neg_float);
        let is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, tag, float_tag, "is_float")
            .unwrap();
        let float_val = self
            .builder
            .build_bitcast(data, self.types.f64_type, "f")
            .unwrap()
            .into_float_value();
        let neg_float_val = self.builder.build_float_neg(float_val, "fneg").unwrap();
        // For non-float, just return 0
        let zero_float = self.types.f64_type.const_float(0.0);
        let selected_float = self
            .builder
            .build_select(is_float, neg_float_val, zero_float, "sel_float")
            .map_err(|e| HaversError::CompileError(format!("Failed to select: {}", e)))?
            .into_float_value();
        let float_result = self.make_float(selected_float)?;
        self.builder.build_unconditional_branch(merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "neg_result")
            .unwrap();
        phi.add_incoming(&[(&int_result, int_block), (&float_result, float_block)]);

        Ok(phi.as_basic_value())
    }

    /// Logical not
    fn inline_not(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
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

        let nil_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Nil.as_u8() as u64, false);
        let bool_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Bool.as_u8() as u64, false);
        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);
        let string_tag = self
            .types
            .i8_type
            .const_int(ValueTag::String.as_u8() as u64, false);

        self.builder
            .build_switch(
                tag,
                print_default,
                &[
                    (nil_tag, print_nil),
                    (bool_tag, print_bool),
                    (int_tag, print_int),
                    (float_tag, print_float),
                    (string_tag, print_string),
                ],
            )
            .unwrap();

        // Print nil
        self.builder.position_at_end(print_nil);
        let nil_str = self.get_string_ptr(self.fmt_nil);
        self.builder
            .build_call(self.libc.printf, &[nil_str.into()], "")
            .unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print bool
        self.builder.position_at_end(print_bool);
        let true_str = self.get_string_ptr(self.fmt_true);
        let false_str = self.get_string_ptr(self.fmt_false);
        let zero = self.types.i64_type.const_int(0, false);
        let is_true = self
            .builder
            .build_int_compare(IntPredicate::NE, data, zero, "is_true")
            .unwrap();
        let bool_str = self
            .builder
            .build_select(is_true, true_str, false_str, "bool_str")
            .unwrap();
        self.builder
            .build_call(self.libc.printf, &[bool_str.into()], "")
            .unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print int
        self.builder.position_at_end(print_int);
        let int_fmt = self.get_string_ptr(self.fmt_int);
        self.builder
            .build_call(self.libc.printf, &[int_fmt.into(), data.into()], "")
            .unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print float
        self.builder.position_at_end(print_float);
        let float_fmt = self.get_string_ptr(self.fmt_float);
        let float_val = self
            .builder
            .build_bitcast(data, self.types.f64_type, "f")
            .unwrap();
        self.builder
            .build_call(self.libc.printf, &[float_fmt.into(), float_val.into()], "")
            .unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print string
        self.builder.position_at_end(print_string);
        let str_fmt = self.get_string_ptr(self.fmt_string);
        let str_ptr = self
            .builder
            .build_int_to_ptr(
                data,
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "str",
            )
            .unwrap();
        self.builder
            .build_call(self.libc.printf, &[str_fmt.into(), str_ptr.into()], "")
            .unwrap();
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Print default (unknown type)
        self.builder.position_at_end(print_default);
        self.builder.build_unconditional_branch(print_done).unwrap();

        // Done - print newline
        self.builder.position_at_end(print_done);
        let newline = self.get_string_ptr(self.fmt_newline);
        self.builder
            .build_call(self.libc.printf, &[newline.into()], "")
            .unwrap();

        Ok(())
    }

    // ========== Inline Type Conversion Functions ==========

    /// Convert any value to string (tae_string)
    fn inline_tae_string(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.extract_tag(val)?;
        let data = self.extract_data(val)?;

        let function = self.current_function.unwrap();
        let str_nil = self.context.append_basic_block(function, "str_nil");
        let str_bool = self.context.append_basic_block(function, "str_bool");
        let str_int = self.context.append_basic_block(function, "str_int");
        let str_float = self.context.append_basic_block(function, "str_float");
        let str_string = self.context.append_basic_block(function, "str_string");
        let str_default = self.context.append_basic_block(function, "str_default");
        let str_merge = self.context.append_basic_block(function, "str_merge");

        let nil_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Nil.as_u8() as u64, false);
        let bool_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Bool.as_u8() as u64, false);
        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);
        let string_tag = self
            .types
            .i8_type
            .const_int(ValueTag::String.as_u8() as u64, false);
        let list_tag = self
            .types
            .i8_type
            .const_int(ValueTag::List.as_u8() as u64, false);

        let str_list = self.context.append_basic_block(function, "str_list");

        self.builder
            .build_switch(
                tag,
                str_default,
                &[
                    (nil_tag, str_nil),
                    (bool_tag, str_bool),
                    (int_tag, str_int),
                    (float_tag, str_float),
                    (string_tag, str_string),
                    (list_tag, str_list),
                ],
            )
            .unwrap();

        // nil -> "naething"
        self.builder.position_at_end(str_nil);
        let nil_str = self
            .builder
            .build_global_string_ptr("naething", "nil_str")
            .unwrap();
        let nil_result = self.make_string(nil_str.as_pointer_value())?;
        self.builder.build_unconditional_branch(str_merge).unwrap();
        let nil_block = self.builder.get_insert_block().unwrap();

        // bool -> "aye" or "nae"
        self.builder.position_at_end(str_bool);
        let true_str = self
            .builder
            .build_global_string_ptr("aye", "true_str")
            .unwrap();
        let false_str = self
            .builder
            .build_global_string_ptr("nae", "false_str")
            .unwrap();
        let zero = self.types.i64_type.const_int(0, false);
        let is_true = self
            .builder
            .build_int_compare(IntPredicate::NE, data, zero, "is_true")
            .unwrap();
        let bool_ptr = self
            .builder
            .build_select(
                is_true,
                true_str.as_pointer_value(),
                false_str.as_pointer_value(),
                "bool_ptr",
            )
            .unwrap();
        let bool_result = self.make_string(bool_ptr.into_pointer_value())?;
        self.builder.build_unconditional_branch(str_merge).unwrap();
        let bool_block = self.builder.get_insert_block().unwrap();

        // int -> format with snprintf
        self.builder.position_at_end(str_int);
        // Allocate buffer for int (max 21 chars for i64 + sign + null)
        let buf_size = self.types.i64_type.const_int(32, false);
        let int_buf = self
            .builder
            .build_call(self.libc.malloc, &[buf_size.into()], "int_buf")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let int_fmt = self
            .builder
            .build_global_string_ptr("%lld", "int_fmt")
            .unwrap();
        self.builder
            .build_call(
                self.libc.snprintf,
                &[
                    int_buf.into(),
                    buf_size.into(),
                    int_fmt.as_pointer_value().into(),
                    data.into(),
                ],
                "",
            )
            .unwrap();
        let int_result = self.make_string(int_buf)?;
        self.builder.build_unconditional_branch(str_merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float -> format with snprintf
        self.builder.position_at_end(str_float);
        let float_buf = self
            .builder
            .build_call(self.libc.malloc, &[buf_size.into()], "float_buf")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let float_fmt = self
            .builder
            .build_global_string_ptr("%g", "float_fmt")
            .unwrap();
        let float_val = self
            .builder
            .build_bitcast(data, self.types.f64_type, "f")
            .unwrap();
        self.builder
            .build_call(
                self.libc.snprintf,
                &[
                    float_buf.into(),
                    buf_size.into(),
                    float_fmt.as_pointer_value().into(),
                    float_val.into(),
                ],
                "",
            )
            .unwrap();
        let float_result = self.make_string(float_buf)?;
        self.builder.build_unconditional_branch(str_merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // string -> already a string, just return it
        self.builder.position_at_end(str_string);
        let string_result = val;
        self.builder.build_unconditional_branch(str_merge).unwrap();
        let string_block = self.builder.get_insert_block().unwrap();

        // list -> format as "[elem, elem, ...]"
        self.builder.position_at_end(str_list);
        // Get list pointer and length
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(data, i8_ptr_type, "list_ptr")
            .unwrap();
        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        // Allocate buffer: "[" + up to 20 chars per element * count + ", " separators + "]" + null
        // Estimate: 25 bytes per element should be plenty
        let const_25 = self.types.i64_type.const_int(25, false);
        let const_3 = self.types.i64_type.const_int(3, false);
        let buf_size_mul = self
            .builder
            .build_int_mul(list_len, const_25, "buf_size_mul")
            .unwrap();
        let list_buf_size = self
            .builder
            .build_int_add(buf_size_mul, const_3, "list_buf_size")
            .unwrap();
        let list_buf = self
            .builder
            .build_call(self.libc.malloc, &[list_buf_size.into()], "list_buf")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Start with "["
        let open_bracket = self
            .builder
            .build_global_string_ptr("[", "open_bracket")
            .unwrap();
        self.builder
            .build_call(
                self.libc.snprintf,
                &[
                    list_buf.into(),
                    list_buf_size.into(),
                    open_bracket.as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();

        // Loop through elements
        let list_loop_header = self
            .context
            .append_basic_block(function, "list_loop_header");
        let list_loop_body = self.context.append_basic_block(function, "list_loop_body");
        let list_loop_end = self.context.append_basic_block(function, "list_loop_end");

        // Index starts at 0
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx_ptr")
            .unwrap();
        let zero_64 = self.types.i64_type.const_int(0, false);
        self.builder.build_store(idx_ptr, zero_64).unwrap();
        self.builder
            .build_unconditional_branch(list_loop_header)
            .unwrap();

        // Loop header: check idx < len
        self.builder.position_at_end(list_loop_header);
        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx")
            .unwrap()
            .into_int_value();
        let loop_cond = self
            .builder
            .build_int_compare(IntPredicate::ULT, idx, list_len, "loop_cond")
            .unwrap();
        self.builder
            .build_conditional_branch(loop_cond, list_loop_body, list_loop_end)
            .unwrap();

        // Loop body: append element
        self.builder.position_at_end(list_loop_body);

        // If not first element, add ", "
        let is_first = self
            .builder
            .build_int_compare(IntPredicate::EQ, idx, zero_64, "is_first")
            .unwrap();
        let sep_block = self.context.append_basic_block(function, "sep_block");
        let elem_block = self.context.append_basic_block(function, "elem_block");
        self.builder
            .build_conditional_branch(is_first, elem_block, sep_block)
            .unwrap();

        self.builder.position_at_end(sep_block);
        let comma_sep = self
            .builder
            .build_global_string_ptr(", ", "comma_sep")
            .unwrap();
        self.builder
            .build_call(
                self.libc.strcat,
                &[list_buf.into(), comma_sep.as_pointer_value().into()],
                "",
            )
            .unwrap();
        self.builder.build_unconditional_branch(elem_block).unwrap();

        self.builder.position_at_end(elem_block);
        // Reload idx since we may have come from sep_block
        let idx_in_elem = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_in_elem")
            .unwrap()
            .into_int_value();
        // Get element from list
        let value_ptr_type = self.types.value_type.ptr_type(AddressSpace::default());
        let one_64 = self.types.i64_type.const_int(1, false);
        let elements_base = unsafe {
            self.builder
                .build_gep(self.types.i64_type, len_ptr, &[one_64], "elements_base")
                .unwrap()
        };
        let elements_ptr = self
            .builder
            .build_pointer_cast(elements_base, value_ptr_type, "elements_ptr")
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.value_type,
                    elements_ptr,
                    &[idx_in_elem],
                    "elem_ptr",
                )
                .unwrap()
        };
        let elem_val = self
            .builder
            .build_load(self.types.value_type, elem_ptr, "elem_val")
            .unwrap();

        // Extract element tag and data
        let elem_tag = self
            .builder
            .build_extract_value(elem_val.into_struct_value(), 0, "elem_tag")
            .unwrap()
            .into_int_value();
        let elem_data = self
            .builder
            .build_extract_value(elem_val.into_struct_value(), 1, "elem_data")
            .unwrap()
            .into_int_value();

        // Store elem_data for use in blocks
        let elem_data_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "elem_data_ptr")
            .unwrap();
        self.builder.build_store(elem_data_ptr, elem_data).unwrap();

        // Format based on type (float, string, or int/default)
        let elem_is_float = self
            .builder
            .build_int_compare(IntPredicate::EQ, elem_tag, float_tag, "elem_is_float")
            .unwrap();
        let elem_is_string = self
            .builder
            .build_int_compare(IntPredicate::EQ, elem_tag, string_tag, "elem_is_string")
            .unwrap();
        let elem_float_block = self
            .context
            .append_basic_block(function, "elem_float_block");
        let elem_string_check = self
            .context
            .append_basic_block(function, "elem_string_check");
        let elem_string_print = self
            .context
            .append_basic_block(function, "elem_string_print");
        let elem_int_block = self.context.append_basic_block(function, "elem_int_block");
        let elem_done = self.context.append_basic_block(function, "elem_done");
        self.builder
            .build_conditional_branch(elem_is_float, elem_float_block, elem_string_check)
            .unwrap();

        // Check for string
        self.builder.position_at_end(elem_string_check);
        self.builder
            .build_conditional_branch(elem_is_string, elem_string_print, elem_int_block)
            .unwrap();

        // Format as string
        self.builder.position_at_end(elem_string_print);
        let elem_data_str = self
            .builder
            .build_load(self.types.i64_type, elem_data_ptr, "elem_data_str")
            .unwrap()
            .into_int_value();
        let elem_str_ptr = self
            .builder
            .build_int_to_ptr(elem_data_str, i8_ptr_type, "elem_str_ptr")
            .unwrap();
        self.builder
            .build_call(
                self.libc.strcat,
                &[list_buf.into(), elem_str_ptr.into()],
                "",
            )
            .unwrap();
        self.builder.build_unconditional_branch(elem_done).unwrap();

        // Format as float
        self.builder.position_at_end(elem_float_block);
        let elem_data_float = self
            .builder
            .build_load(self.types.i64_type, elem_data_ptr, "elem_data_float")
            .unwrap()
            .into_int_value();
        let elem_float_buf = self
            .builder
            .build_call(self.libc.malloc, &[const_25.into()], "elem_float_buf")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let float_fmt2 = self
            .builder
            .build_global_string_ptr("%g", "float_fmt2")
            .unwrap();
        let elem_as_float = self
            .builder
            .build_bitcast(elem_data_float, self.types.f64_type, "elem_as_float")
            .unwrap();
        self.builder
            .build_call(
                self.libc.snprintf,
                &[
                    elem_float_buf.into(),
                    const_25.into(),
                    float_fmt2.as_pointer_value().into(),
                    elem_as_float.into(),
                ],
                "",
            )
            .unwrap();
        self.builder
            .build_call(
                self.libc.strcat,
                &[list_buf.into(), elem_float_buf.into()],
                "",
            )
            .unwrap();
        self.builder.build_unconditional_branch(elem_done).unwrap();

        // Format as int (default)
        self.builder.position_at_end(elem_int_block);
        let elem_data_int = self
            .builder
            .build_load(self.types.i64_type, elem_data_ptr, "elem_data_int")
            .unwrap()
            .into_int_value();
        let elem_int_buf = self
            .builder
            .build_call(self.libc.malloc, &[const_25.into()], "elem_int_buf")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let int_fmt2 = self
            .builder
            .build_global_string_ptr("%lld", "int_fmt2")
            .unwrap();
        self.builder
            .build_call(
                self.libc.snprintf,
                &[
                    elem_int_buf.into(),
                    const_25.into(),
                    int_fmt2.as_pointer_value().into(),
                    elem_data_int.into(),
                ],
                "",
            )
            .unwrap();
        self.builder
            .build_call(
                self.libc.strcat,
                &[list_buf.into(), elem_int_buf.into()],
                "",
            )
            .unwrap();
        self.builder.build_unconditional_branch(elem_done).unwrap();

        // Increment and loop
        self.builder.position_at_end(elem_done);
        let idx_for_incr = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_for_incr")
            .unwrap()
            .into_int_value();
        let next_idx = self
            .builder
            .build_int_add(idx_for_incr, one_64, "next_idx")
            .unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder
            .build_unconditional_branch(list_loop_header)
            .unwrap();

        // Loop end: close bracket
        self.builder.position_at_end(list_loop_end);
        let close_bracket = self
            .builder
            .build_global_string_ptr("]", "close_bracket")
            .unwrap();
        self.builder
            .build_call(
                self.libc.strcat,
                &[list_buf.into(), close_bracket.as_pointer_value().into()],
                "",
            )
            .unwrap();
        let list_result = self.make_string(list_buf)?;
        self.builder.build_unconditional_branch(str_merge).unwrap();
        let list_block = self.builder.get_insert_block().unwrap();

        // default -> empty string
        self.builder.position_at_end(str_default);
        let empty_str = self
            .builder
            .build_global_string_ptr("", "empty_str")
            .unwrap();
        let default_result = self.make_string(empty_str.as_pointer_value())?;
        self.builder.build_unconditional_branch(str_merge).unwrap();
        let default_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(str_merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "str_result")
            .unwrap();
        phi.add_incoming(&[
            (&nil_result, nil_block),
            (&bool_result, bool_block),
            (&int_result, int_block),
            (&float_result, float_block),
            (&string_result, string_block),
            (&list_result, list_block),
            (&default_result, default_block),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Convert any value to int (tae_int)
    fn inline_tae_int(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.extract_tag(val)?;
        let data = self.extract_data(val)?;

        let function = self.current_function.unwrap();
        let int_nil = self.context.append_basic_block(function, "int_nil");
        let int_bool = self.context.append_basic_block(function, "int_bool");
        let int_int = self.context.append_basic_block(function, "int_int");
        let int_float = self.context.append_basic_block(function, "int_float");
        let int_default = self.context.append_basic_block(function, "int_default");
        let int_merge = self.context.append_basic_block(function, "int_merge");

        let nil_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Nil.as_u8() as u64, false);
        let bool_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Bool.as_u8() as u64, false);
        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);

        self.builder
            .build_switch(
                tag,
                int_default,
                &[
                    (nil_tag, int_nil),
                    (bool_tag, int_bool),
                    (int_tag, int_int),
                    (float_tag, int_float),
                ],
            )
            .unwrap();

        // nil -> 0
        self.builder.position_at_end(int_nil);
        let zero = self.types.i64_type.const_int(0, false);
        let nil_result = self.make_int(zero)?;
        self.builder.build_unconditional_branch(int_merge).unwrap();
        let nil_block = self.builder.get_insert_block().unwrap();

        // bool -> 0 or 1
        self.builder.position_at_end(int_bool);
        let bool_result = self.make_int(data)?;
        self.builder.build_unconditional_branch(int_merge).unwrap();
        let bool_block = self.builder.get_insert_block().unwrap();

        // int -> already an int
        self.builder.position_at_end(int_int);
        let int_result = val;
        self.builder.build_unconditional_branch(int_merge).unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float -> truncate to int
        self.builder.position_at_end(int_float);
        let float_val = self
            .builder
            .build_bitcast(data, self.types.f64_type, "f")
            .unwrap()
            .into_float_value();
        let truncated = self
            .builder
            .build_float_to_signed_int(float_val, self.types.i64_type, "trunc")
            .unwrap();
        let float_result = self.make_int(truncated)?;
        self.builder.build_unconditional_branch(int_merge).unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // default -> 0
        self.builder.position_at_end(int_default);
        let default_result = self.make_int(zero)?;
        self.builder.build_unconditional_branch(int_merge).unwrap();
        let default_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(int_merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "int_result")
            .unwrap();
        phi.add_incoming(&[
            (&nil_result, nil_block),
            (&bool_result, bool_block),
            (&int_result, int_block),
            (&float_result, float_block),
            (&default_result, default_block),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Convert any value to float (tae_float)
    fn inline_tae_float(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.extract_tag(val)?;
        let data = self.extract_data(val)?;

        let function = self.current_function.unwrap();
        let float_nil = self.context.append_basic_block(function, "float_nil");
        let float_bool = self.context.append_basic_block(function, "float_bool");
        let float_int = self.context.append_basic_block(function, "float_int");
        let float_float = self.context.append_basic_block(function, "float_float");
        let float_default = self.context.append_basic_block(function, "float_default");
        let float_merge = self.context.append_basic_block(function, "float_merge");

        let nil_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Nil.as_u8() as u64, false);
        let bool_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Bool.as_u8() as u64, false);
        let int_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Int.as_u8() as u64, false);
        let float_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Float.as_u8() as u64, false);

        self.builder
            .build_switch(
                tag,
                float_default,
                &[
                    (nil_tag, float_nil),
                    (bool_tag, float_bool),
                    (int_tag, float_int),
                    (float_tag, float_float),
                ],
            )
            .unwrap();

        // nil -> 0.0
        self.builder.position_at_end(float_nil);
        let zero_f = self.types.f64_type.const_float(0.0);
        let nil_result = self.make_float(zero_f)?;
        self.builder
            .build_unconditional_branch(float_merge)
            .unwrap();
        let nil_block = self.builder.get_insert_block().unwrap();

        // bool -> 0.0 or 1.0
        self.builder.position_at_end(float_bool);
        let bool_f = self
            .builder
            .build_signed_int_to_float(data, self.types.f64_type, "bool_f")
            .unwrap();
        let bool_result = self.make_float(bool_f)?;
        self.builder
            .build_unconditional_branch(float_merge)
            .unwrap();
        let bool_block = self.builder.get_insert_block().unwrap();

        // int -> convert to float
        self.builder.position_at_end(float_int);
        let int_f = self
            .builder
            .build_signed_int_to_float(data, self.types.f64_type, "int_f")
            .unwrap();
        let int_result = self.make_float(int_f)?;
        self.builder
            .build_unconditional_branch(float_merge)
            .unwrap();
        let int_block = self.builder.get_insert_block().unwrap();

        // float -> already a float
        self.builder.position_at_end(float_float);
        let float_result = val;
        self.builder
            .build_unconditional_branch(float_merge)
            .unwrap();
        let float_block = self.builder.get_insert_block().unwrap();

        // default -> 0.0
        self.builder.position_at_end(float_default);
        let default_result = self.make_float(zero_f)?;
        self.builder
            .build_unconditional_branch(float_merge)
            .unwrap();
        let default_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(float_merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "float_result")
            .unwrap();
        phi.add_incoming(&[
            (&nil_result, nil_block),
            (&bool_result, bool_block),
            (&int_result, int_block),
            (&float_result, float_block),
            (&default_result, default_block),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Get length of a string (len)
    fn inline_len(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.extract_tag(val)?;
        let data = self.extract_data(val)?;

        let function = self.current_function.unwrap();
        let len_string = self.context.append_basic_block(function, "len_string");
        let len_check_list = self.context.append_basic_block(function, "len_check_list");
        let len_list = self.context.append_basic_block(function, "len_list");
        let len_default = self.context.append_basic_block(function, "len_default");
        let len_merge = self.context.append_basic_block(function, "len_merge");

        // Check if string (tag == 4)
        let string_tag = self
            .types
            .i8_type
            .const_int(ValueTag::String.as_u8() as u64, false);
        let is_string = self
            .builder
            .build_int_compare(IntPredicate::EQ, tag, string_tag, "is_str")
            .unwrap();
        self.builder
            .build_conditional_branch(is_string, len_string, len_check_list)
            .unwrap();

        // String -> strlen
        self.builder.position_at_end(len_string);
        let str_ptr = self
            .builder
            .build_int_to_ptr(
                data,
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "str",
            )
            .unwrap();
        let len = self
            .builder
            .build_call(self.libc.strlen, &[str_ptr.into()], "len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let string_result = self.make_int(len)?;
        self.builder.build_unconditional_branch(len_merge).unwrap();
        let string_block = self.builder.get_insert_block().unwrap();

        // Check if list (tag == 5)
        self.builder.position_at_end(len_check_list);
        let list_tag = self
            .types
            .i8_type
            .const_int(ValueTag::List.as_u8() as u64, false);
        let is_list = self
            .builder
            .build_int_compare(IntPredicate::EQ, tag, list_tag, "is_list")
            .unwrap();
        self.builder
            .build_conditional_branch(is_list, len_list, len_default)
            .unwrap();

        // List -> read length from offset 1 (after capacity)
        // Layout: [capacity: i64][length: i64][elements...]
        self.builder.position_at_end(len_list);
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let header_ptr = self
            .builder
            .build_int_to_ptr(data, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(self.types.i64_type, header_ptr, &[self.types.i64_type.const_int(1, false)], "len_ptr")
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();
        let list_result = self.make_int(list_len)?;
        self.builder.build_unconditional_branch(len_merge).unwrap();
        let list_block = self.builder.get_insert_block().unwrap();

        // Default -> 0
        self.builder.position_at_end(len_default);
        let zero = self.types.i64_type.const_int(0, false);
        let default_result = self.make_int(zero)?;
        self.builder.build_unconditional_branch(len_merge).unwrap();
        let default_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(len_merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "len_result")
            .unwrap();
        phi.add_incoming(&[
            (&string_result, string_block),
            (&list_result, list_block),
            (&default_result, default_block),
        ]);

        Ok(phi.as_basic_value())
    }

    fn inline_shove(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        elem_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // List layout: [capacity: i64][length: i64][elements...]
        let tag = self.extract_tag(list_val)?;
        let data = self.extract_data(list_val)?;

        let function = self.current_function.unwrap();
        let shove_list = self.context.append_basic_block(function, "shove_list");
        let shove_default = self.context.append_basic_block(function, "shove_default");
        let shove_merge = self.context.append_basic_block(function, "shove_merge");

        // Check if it's a list (tag == 5)
        let list_tag = self
            .types
            .i8_type
            .const_int(ValueTag::List.as_u8() as u64, false);
        let is_list = self
            .builder
            .build_int_compare(IntPredicate::EQ, tag, list_tag, "is_list")
            .unwrap();
        self.builder
            .build_conditional_branch(is_list, shove_list, shove_default)
            .unwrap();

        // Handle list case with capacity-based growth
        self.builder.position_at_end(shove_list);

        // Convert data to pointer
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let header_ptr = self
            .builder
            .build_int_to_ptr(data, i64_ptr_type, "header_ptr")
            .unwrap();

        // Load capacity from offset 0
        let old_capacity = self
            .builder
            .build_load(self.types.i64_type, header_ptr, "old_capacity")
            .unwrap()
            .into_int_value();

        // Get length pointer at offset 1
        let one = self.types.i64_type.const_int(1, false);
        let len_ptr = unsafe {
            self.builder
                .build_gep(self.types.i64_type, header_ptr, &[one], "len_ptr")
                .unwrap()
        };

        // Load current length
        let old_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "old_len")
            .unwrap()
            .into_int_value();

        // Calculate new length
        let new_len = self.builder.build_int_add(old_len, one, "new_len").unwrap();

        // Check if we need to grow: new_len > capacity?
        let needs_grow = self.builder.build_int_compare(
            IntPredicate::UGT, new_len, old_capacity, "needs_grow"
        ).unwrap();

        // Create blocks for grow vs no-grow paths
        let grow_block = self.context.append_basic_block(function, "shove_grow");
        let no_grow_block = self.context.append_basic_block(function, "shove_no_grow");
        let store_block = self.context.append_basic_block(function, "shove_store");

        self.builder.build_conditional_branch(needs_grow, grow_block, no_grow_block).unwrap();

        // GROW PATH: double capacity and realloc
        self.builder.position_at_end(grow_block);
        let two = self.types.i64_type.const_int(2, false);
        let doubled_capacity = self.builder.build_int_mul(old_capacity, two, "doubled_capacity").unwrap();
        // Ensure new_capacity is at least 8 (in case old_capacity was 0 or very small)
        let eight = self.types.i64_type.const_int(8, false);
        let cap_ok = self.builder.build_int_compare(IntPredicate::UGE, doubled_capacity, eight, "cap_ok").unwrap();
        let new_capacity = self.builder.build_select(cap_ok, doubled_capacity, eight, "safe_cap").unwrap().into_int_value();

        // Calculate new allocation size: 16 (header) + new_capacity * 16 (elements)
        let header_size = self.types.i64_type.const_int(16, false);
        let value_size = self.types.i64_type.const_int(16, false);
        let elements_size = self.builder.build_int_mul(new_capacity, value_size, "elements_size").unwrap();
        let total_size = self.builder.build_int_add(header_size, elements_size, "total_size").unwrap();

        // Realloc
        let old_ptr_i8 = self.builder.build_pointer_cast(header_ptr, i8_ptr_type, "old_ptr_i8").unwrap();
        let new_ptr = self
            .builder
            .build_call(
                self.libc.realloc,
                &[old_ptr_i8.into(), total_size.into()],
                "new_list_alloc",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let new_header_ptr = self.builder.build_pointer_cast(new_ptr, i64_ptr_type, "new_header_ptr").unwrap();

        // Store new capacity
        self.builder.build_store(new_header_ptr, new_capacity).unwrap();
        self.builder.build_unconditional_branch(store_block).unwrap();
        let grow_end_block = self.builder.get_insert_block().unwrap();

        // NO-GROW PATH: just use existing buffer
        self.builder.position_at_end(no_grow_block);
        self.builder.build_unconditional_branch(store_block).unwrap();
        let no_grow_end_block = self.builder.get_insert_block().unwrap();

        // STORE BLOCK: use PHI to get final header pointer, then store element
        self.builder.position_at_end(store_block);
        let final_header_ptr = self.builder.build_phi(i64_ptr_type, "final_header_ptr").unwrap();
        final_header_ptr.add_incoming(&[
            (&new_header_ptr, grow_end_block),
            (&header_ptr, no_grow_end_block),
        ]);
        let final_header_ptr = final_header_ptr.as_basic_value().into_pointer_value();

        // Get length pointer in final buffer (at offset 1)
        let final_len_ptr = unsafe {
            self.builder
                .build_gep(self.types.i64_type, final_header_ptr, &[one], "final_len_ptr")
                .unwrap()
        };

        // Store new length
        self.builder.build_store(final_len_ptr, new_len).unwrap();

        // Get pointer to elements array (at offset 2)
        let value_ptr_type = self.types.value_type.ptr_type(AddressSpace::default());
        let elements_base = unsafe {
            self.builder
                .build_gep(self.types.i64_type, final_header_ptr, &[two], "elements_base")
                .unwrap()
        };
        let elements_ptr = self
            .builder
            .build_pointer_cast(elements_base, value_ptr_type, "elements_ptr")
            .unwrap();

        // Store new element at index old_len
        let new_elem_ptr = unsafe {
            self.builder
                .build_gep(self.types.value_type, elements_ptr, &[old_len], "new_elem")
                .unwrap()
        };
        self.builder.build_store(new_elem_ptr, elem_val).unwrap();

        // Create result list value
        let list_result = self.make_list(self.builder.build_pointer_cast(final_header_ptr, i8_ptr_type, "list_ptr").unwrap())?;
        self.builder
            .build_unconditional_branch(shove_merge)
            .unwrap();
        let list_block = self.builder.get_insert_block().unwrap();

        // Default case - return nil for non-lists
        self.builder.position_at_end(shove_default);
        let default_result = self.make_nil();
        self.builder
            .build_unconditional_branch(shove_merge)
            .unwrap();
        let default_block = self.builder.get_insert_block().unwrap();

        // Merge
        self.builder.position_at_end(shove_merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "shove_result")
            .unwrap();
        phi.add_incoming(&[(&list_result, list_block), (&default_result, default_block)]);

        Ok(phi.as_basic_value())
    }

    // ========== Phase 1: Math Functions ==========

    /// Get or create an LLVM intrinsic function
    fn get_or_create_intrinsic(
        &self,
        name: &str,
        ret_type: inkwell::types::BasicTypeEnum<'ctx>,
        arg_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
    ) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function(name) {
            func
        } else {
            let fn_type = match ret_type {
                inkwell::types::BasicTypeEnum::FloatType(ft) => ft.fn_type(arg_types, false),
                inkwell::types::BasicTypeEnum::IntType(it) => it.fn_type(arg_types, false),
                _ => panic!("Unsupported intrinsic return type"),
            };
            self.module.add_function(name, fn_type, None)
        }
    }

    /// abs(x) - absolute value (integers only for simplicity, uses select)
    fn inline_abs(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let data = self.extract_data(val)?;

        // Integer abs: (x < 0) ? -x : x
        let zero = self.types.i64_type.const_int(0, false);
        let is_negative = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, data, zero, "is_negative")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
        let negated = self
            .builder
            .build_int_neg(data, "negated")
            .map_err(|e| HaversError::CompileError(format!("Failed to negate: {}", e)))?;
        let abs_val = self
            .builder
            .build_select(is_negative, negated, data, "abs_val")
            .map_err(|e| HaversError::CompileError(format!("Failed to select: {}", e)))?
            .into_int_value();

        self.make_int(abs_val)
    }

    /// min(a, b) - minimum of two values
    fn inline_min(
        &mut self,
        a: BasicValueEnum<'ctx>,
        b: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let a_data = self.extract_data(a)?;
        let b_data = self.extract_data(b)?;

        // Compare as signed integers
        let is_less = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, a_data, b_data, "is_less")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;

        let min_val = self
            .builder
            .build_select(is_less, a_data, b_data, "min_val")
            .map_err(|e| HaversError::CompileError(format!("Failed to select: {}", e)))?
            .into_int_value();

        self.make_int(min_val)
    }

    /// max(a, b) - maximum of two values
    fn inline_max(
        &mut self,
        a: BasicValueEnum<'ctx>,
        b: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let a_data = self.extract_data(a)?;
        let b_data = self.extract_data(b)?;

        // Compare as signed integers
        let is_greater = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SGT, a_data, b_data, "is_greater")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;

        let max_val = self
            .builder
            .build_select(is_greater, a_data, b_data, "max_val")
            .map_err(|e| HaversError::CompileError(format!("Failed to select: {}", e)))?
            .into_int_value();

        self.make_int(max_val)
    }

    /// floor(x) - floor of float, returns int
    fn inline_floor(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let data = self.extract_data(val)?;
        let float_val = self
            .builder
            .build_bitcast(data, self.types.f64_type, "as_float")
            .map_err(|e| HaversError::CompileError(format!("Failed to bitcast: {}", e)))?
            .into_float_value();

        let floor_fn = self.get_or_create_intrinsic(
            "llvm.floor.f64",
            self.types.f64_type.into(),
            &[self.types.f64_type.into()],
        );
        let floored = self
            .builder
            .build_call(floor_fn, &[float_val.into()], "floored")
            .map_err(|e| HaversError::CompileError(format!("Failed to call floor: {}", e)))?
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_float_value();

        let int_val = self
            .builder
            .build_float_to_signed_int(floored, self.types.i64_type, "floor_int")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;

        self.make_int(int_val)
    }

    /// ceil(x) - ceiling of float, returns int
    fn inline_ceil(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let data = self.extract_data(val)?;
        let float_val = self
            .builder
            .build_bitcast(data, self.types.f64_type, "as_float")
            .map_err(|e| HaversError::CompileError(format!("Failed to bitcast: {}", e)))?
            .into_float_value();

        let ceil_fn = self.get_or_create_intrinsic(
            "llvm.ceil.f64",
            self.types.f64_type.into(),
            &[self.types.f64_type.into()],
        );
        let ceiled = self
            .builder
            .build_call(ceil_fn, &[float_val.into()], "ceiled")
            .map_err(|e| HaversError::CompileError(format!("Failed to call ceil: {}", e)))?
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_float_value();

        let int_val = self
            .builder
            .build_float_to_signed_int(ceiled, self.types.i64_type, "ceil_int")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;

        self.make_int(int_val)
    }

    /// round(x) - round float to nearest int
    fn inline_round(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let data = self.extract_data(val)?;
        let float_val = self
            .builder
            .build_bitcast(data, self.types.f64_type, "as_float")
            .map_err(|e| HaversError::CompileError(format!("Failed to bitcast: {}", e)))?
            .into_float_value();

        let round_fn = self.get_or_create_intrinsic(
            "llvm.round.f64",
            self.types.f64_type.into(),
            &[self.types.f64_type.into()],
        );
        let rounded = self
            .builder
            .build_call(round_fn, &[float_val.into()], "rounded")
            .map_err(|e| HaversError::CompileError(format!("Failed to call round: {}", e)))?
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_float_value();

        let int_val = self
            .builder
            .build_float_to_signed_int(rounded, self.types.i64_type, "round_int")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;

        self.make_int(int_val)
    }

    /// sqrt(x) - square root, returns float
    fn inline_sqrt(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let data = self.extract_data(val)?;
        let float_val = self
            .builder
            .build_bitcast(data, self.types.f64_type, "as_float")
            .map_err(|e| HaversError::CompileError(format!("Failed to bitcast: {}", e)))?
            .into_float_value();

        let sqrt_fn = self.get_or_create_intrinsic(
            "llvm.sqrt.f64",
            self.types.f64_type.into(),
            &[self.types.f64_type.into()],
        );
        let sqrt_result = self
            .builder
            .build_call(sqrt_fn, &[float_val.into()], "sqrt_result")
            .map_err(|e| HaversError::CompileError(format!("Failed to call sqrt: {}", e)))?
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_float_value();

        self.make_float(sqrt_result)
    }

    // ========== Phase 2: List Operations ==========

    /// Helper to get element pointer at index in a list
    fn get_list_element_ptr(
        &self,
        list_data: IntValue<'ctx>,
        index: IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, HaversError> {
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let raw_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let len_ptr = self
            .builder
            .build_pointer_cast(raw_ptr, i64_ptr_type, "len_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast: {}", e)))?;
        let value_ptr_type = self.types.value_type.ptr_type(AddressSpace::default());
        let elements_base = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    len_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "elements_base",
                )
                .map_err(|e| HaversError::CompileError(format!("Failed to compute base: {}", e)))?
        };
        let elements_ptr = self
            .builder
            .build_pointer_cast(elements_base, value_ptr_type, "elements_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast: {}", e)))?;
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.types.value_type, elements_ptr, &[index], "elem_ptr")
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to compute element ptr: {}", e))
                })?
        };
        Ok(elem_ptr)
    }

    /// Helper to get list length
    fn get_list_length(&self, list_data: IntValue<'ctx>) -> Result<IntValue<'ctx>, HaversError> {
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let raw_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let len_ptr = self
            .builder
            .build_pointer_cast(raw_ptr, i64_ptr_type, "len_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast: {}", e)))?;
        let length = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();
        Ok(length)
    }

    /// Helper to allocate a new list with given length
    fn allocate_list(&self, length: IntValue<'ctx>) -> Result<PointerValue<'ctx>, HaversError> {
        // Size = 8 (length) + 16 * num_elements (each element is {i8, i64} = 16 bytes with padding)
        let elem_size = self.types.i64_type.const_int(16, false);
        let header_size = self.types.i64_type.const_int(16, false);
        let elems_size = self
            .builder
            .build_int_mul(length, elem_size, "elems_size")
            .map_err(|e| HaversError::CompileError(format!("Failed to multiply: {}", e)))?;
        let total_size = self
            .builder
            .build_int_add(header_size, elems_size, "total_size")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;

        let ptr = self
            .builder
            .build_call(self.libc.malloc, &[total_size.into()], "new_list")
            .map_err(|e| HaversError::CompileError(format!("Failed to malloc: {}", e)))?
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Store the length
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let len_ptr = self
            .builder
            .build_pointer_cast(ptr, i64_ptr_type, "len_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast: {}", e)))?;
        self.builder
            .build_store(len_ptr, length)
            .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;

        Ok(ptr)
    }

    /// yank(list) - pop last element from list
    fn inline_yank(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let list_data = self.extract_data(val)?;
        let length = self.get_list_length(list_data)?;

        // Get last element
        let one = self.types.i64_type.const_int(1, false);
        let last_idx = self
            .builder
            .build_int_sub(length, one, "last_idx")
            .map_err(|e| HaversError::CompileError(format!("Failed to subtract: {}", e)))?;
        let elem_ptr = self.get_list_element_ptr(list_data, last_idx)?;
        let result = self
            .builder
            .build_load(self.types.value_type, elem_ptr, "yanked")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;

        // Decrement length in place
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let raw_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let len_ptr = self
            .builder
            .build_pointer_cast(raw_ptr, i64_ptr_type, "len_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast: {}", e)))?;
        let new_len = self
            .builder
            .build_int_sub(length, one, "new_len")
            .map_err(|e| HaversError::CompileError(format!("Failed to subtract: {}", e)))?;
        self.builder
            .build_store(len_ptr, new_len)
            .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;

        Ok(result)
    }

    /// heid(list) - get first element (head)
    fn inline_heid(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let list_data = self.extract_data(val)?;
        let zero = self.types.i64_type.const_int(0, false);
        let elem_ptr = self.get_list_element_ptr(list_data, zero)?;
        let result = self
            .builder
            .build_load(self.types.value_type, elem_ptr, "heid")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;
        Ok(result)
    }

    /// bum(list) - get last element
    fn inline_bum(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let list_data = self.extract_data(val)?;
        let length = self.get_list_length(list_data)?;
        let one = self.types.i64_type.const_int(1, false);
        let last_idx = self
            .builder
            .build_int_sub(length, one, "last_idx")
            .map_err(|e| HaversError::CompileError(format!("Failed to subtract: {}", e)))?;
        let elem_ptr = self.get_list_element_ptr(list_data, last_idx)?;
        let result = self
            .builder
            .build_load(self.types.value_type, elem_ptr, "bum")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;
        Ok(result)
    }

    /// tail(list) - return new list without first element
    fn inline_tail(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let list_data = self.extract_data(val)?;
        let length = self.get_list_length(list_data)?;

        let one = self.types.i64_type.const_int(1, false);
        let new_len = self
            .builder
            .build_int_sub(length, one, "new_len")
            .map_err(|e| HaversError::CompileError(format!("Failed to subtract: {}", e)))?;

        // Allocate new list
        let new_list_ptr = self.allocate_list(new_len)?;

        // Copy elements 1..length to new list
        let function = self
            .current_function
            .ok_or_else(|| HaversError::CompileError("No current function".to_string()))?;
        let loop_block = self.context.append_basic_block(function, "tail_loop");
        let done_block = self.context.append_basic_block(function, "tail_done");

        // i = 0
        let i_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to alloca: {}", e)))?;
        let zero = self.types.i64_type.const_int(0, false);
        self.builder.build_store(i_ptr, zero).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();
        let cond = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, i, new_len, "cond")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;

        let body_block = self.context.append_basic_block(function, "tail_body");
        self.builder
            .build_conditional_branch(cond, body_block, done_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        // Copy element i+1 from source to i in dest
        let src_idx = self
            .builder
            .build_int_add(i, one, "src_idx")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        let src_ptr = self.get_list_element_ptr(list_data, src_idx)?;
        let elem = self
            .builder
            .build_load(self.types.value_type, src_ptr, "elem")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;

        let new_list_data = self
            .builder
            .build_ptr_to_int(new_list_ptr, self.types.i64_type, "new_data")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;
        let dst_ptr = self.get_list_element_ptr(new_list_data, i)?;
        self.builder.build_store(dst_ptr, elem).unwrap();

        // i++
        let next_i = self
            .builder
            .build_int_add(i, one, "next_i")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);

        self.make_list(new_list_ptr)
    }

    /// scran(list, start, end) - slice list[start:end]
    fn inline_scran(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        start_val: BasicValueEnum<'ctx>,
        end_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let list_data = self.extract_data(list_val)?;
        let start = self.extract_data(start_val)?;
        let end = self.extract_data(end_val)?;

        // new_len = end - start
        let new_len = self
            .builder
            .build_int_sub(end, start, "new_len")
            .map_err(|e| HaversError::CompileError(format!("Failed to subtract: {}", e)))?;

        let new_list_ptr = self.allocate_list(new_len)?;

        let function = self
            .current_function
            .ok_or_else(|| HaversError::CompileError("No current function".to_string()))?;
        let loop_block = self.context.append_basic_block(function, "scran_loop");
        let done_block = self.context.append_basic_block(function, "scran_done");

        let i_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to alloca: {}", e)))?;
        let zero = self.types.i64_type.const_int(0, false);
        self.builder.build_store(i_ptr, zero).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();
        let cond = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, i, new_len, "cond")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;

        let body_block = self.context.append_basic_block(function, "scran_body");
        self.builder
            .build_conditional_branch(cond, body_block, done_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        let src_idx = self
            .builder
            .build_int_add(start, i, "src_idx")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        let src_ptr = self.get_list_element_ptr(list_data, src_idx)?;
        let elem = self
            .builder
            .build_load(self.types.value_type, src_ptr, "elem")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;

        let new_list_data = self
            .builder
            .build_ptr_to_int(new_list_ptr, self.types.i64_type, "new_data")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;
        let dst_ptr = self.get_list_element_ptr(new_list_data, i)?;
        self.builder.build_store(dst_ptr, elem).unwrap();

        let one = self.types.i64_type.const_int(1, false);
        let next_i = self
            .builder
            .build_int_add(i, one, "next_i")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);

        self.make_list(new_list_ptr)
    }

    /// slap(a, b) - concatenate two lists
    fn inline_slap(
        &mut self,
        a: BasicValueEnum<'ctx>,
        b: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let a_data = self.extract_data(a)?;
        let b_data = self.extract_data(b)?;
        let a_len = self.get_list_length(a_data)?;
        let b_len = self.get_list_length(b_data)?;

        let new_len = self
            .builder
            .build_int_add(a_len, b_len, "new_len")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;

        let new_list_ptr = self.allocate_list(new_len)?;
        let new_list_data = self
            .builder
            .build_ptr_to_int(new_list_ptr, self.types.i64_type, "new_data")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;

        let function = self
            .current_function
            .ok_or_else(|| HaversError::CompileError("No current function".to_string()))?;

        // Copy first list
        let loop1 = self.context.append_basic_block(function, "slap_loop1");
        let body1 = self.context.append_basic_block(function, "slap_body1");
        let done1 = self.context.append_basic_block(function, "slap_done1");

        let i_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to alloca: {}", e)))?;
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        self.builder.build_store(i_ptr, zero).unwrap();
        self.builder.build_unconditional_branch(loop1).unwrap();

        self.builder.position_at_end(loop1);
        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();
        let cond1 = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, i, a_len, "cond1")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
        self.builder
            .build_conditional_branch(cond1, body1, done1)
            .unwrap();

        self.builder.position_at_end(body1);
        let src_ptr = self.get_list_element_ptr(a_data, i)?;
        let elem = self
            .builder
            .build_load(self.types.value_type, src_ptr, "elem")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;
        let dst_ptr = self.get_list_element_ptr(new_list_data, i)?;
        self.builder.build_store(dst_ptr, elem).unwrap();
        let next_i = self
            .builder
            .build_int_add(i, one, "next_i")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder.build_unconditional_branch(loop1).unwrap();

        // Copy second list
        self.builder.position_at_end(done1);
        let loop2 = self.context.append_basic_block(function, "slap_loop2");
        let body2 = self.context.append_basic_block(function, "slap_body2");
        let done2 = self.context.append_basic_block(function, "slap_done2");

        self.builder.build_store(i_ptr, zero).unwrap();
        self.builder.build_unconditional_branch(loop2).unwrap();

        self.builder.position_at_end(loop2);
        let i2 = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i2")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();
        let cond2 = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, i2, b_len, "cond2")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
        self.builder
            .build_conditional_branch(cond2, body2, done2)
            .unwrap();

        self.builder.position_at_end(body2);
        let src_ptr2 = self.get_list_element_ptr(b_data, i2)?;
        let elem2 = self
            .builder
            .build_load(self.types.value_type, src_ptr2, "elem2")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;
        let dst_idx = self
            .builder
            .build_int_add(a_len, i2, "dst_idx")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        let dst_ptr2 = self.get_list_element_ptr(new_list_data, dst_idx)?;
        self.builder.build_store(dst_ptr2, elem2).unwrap();
        let next_i2 = self
            .builder
            .build_int_add(i2, one, "next_i2")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        self.builder.build_store(i_ptr, next_i2).unwrap();
        self.builder.build_unconditional_branch(loop2).unwrap();

        self.builder.position_at_end(done2);

        self.make_list(new_list_ptr)
    }

    /// reverse(list) - return reversed copy
    fn inline_reverse(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let list_data = self.extract_data(val)?;
        let length = self.get_list_length(list_data)?;

        let new_list_ptr = self.allocate_list(length)?;
        let new_list_data = self
            .builder
            .build_ptr_to_int(new_list_ptr, self.types.i64_type, "new_data")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert: {}", e)))?;

        let function = self
            .current_function
            .ok_or_else(|| HaversError::CompileError("No current function".to_string()))?;
        let loop_block = self.context.append_basic_block(function, "rev_loop");
        let body_block = self.context.append_basic_block(function, "rev_body");
        let done_block = self.context.append_basic_block(function, "rev_done");

        let i_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to alloca: {}", e)))?;
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        self.builder.build_store(i_ptr, zero).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();
        let cond = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, i, length, "cond")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
        self.builder
            .build_conditional_branch(cond, body_block, done_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        let src_ptr = self.get_list_element_ptr(list_data, i)?;
        let elem = self
            .builder
            .build_load(self.types.value_type, src_ptr, "elem")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;
        // dst_idx = length - 1 - i
        let len_minus_1 = self
            .builder
            .build_int_sub(length, one, "len_minus_1")
            .map_err(|e| HaversError::CompileError(format!("Failed to subtract: {}", e)))?;
        let dst_idx = self
            .builder
            .build_int_sub(len_minus_1, i, "dst_idx")
            .map_err(|e| HaversError::CompileError(format!("Failed to subtract: {}", e)))?;
        let dst_ptr = self.get_list_element_ptr(new_list_data, dst_idx)?;
        self.builder.build_store(dst_ptr, elem).unwrap();

        let next_i = self
            .builder
            .build_int_add(i, one, "next_i")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);

        self.make_list(new_list_ptr)
    }

    /// sumaw(list) - sum all numeric elements
    fn inline_sumaw(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let list_data = self.extract_data(val)?;
        let length = self.get_list_length(list_data)?;

        let function = self
            .current_function
            .ok_or_else(|| HaversError::CompileError("No current function".to_string()))?;
        let loop_block = self.context.append_basic_block(function, "sum_loop");
        let body_block = self.context.append_basic_block(function, "sum_body");
        let done_block = self.context.append_basic_block(function, "sum_done");

        let i_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to alloca: {}", e)))?;
        let sum_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "sum")
            .map_err(|e| HaversError::CompileError(format!("Failed to alloca: {}", e)))?;
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        self.builder.build_store(i_ptr, zero).unwrap();
        self.builder.build_store(sum_ptr, zero).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();
        let cond = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, i, length, "cond")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;
        self.builder
            .build_conditional_branch(cond, body_block, done_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        let elem_ptr = self.get_list_element_ptr(list_data, i)?;
        let elem = self
            .builder
            .build_load(self.types.value_type, elem_ptr, "elem")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;
        let elem_data = self.extract_data(elem)?;

        let sum = self
            .builder
            .build_load(self.types.i64_type, sum_ptr, "sum")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();
        let new_sum = self
            .builder
            .build_int_add(sum, elem_data, "new_sum")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        self.builder.build_store(sum_ptr, new_sum).unwrap();

        let next_i = self
            .builder
            .build_int_add(i, one, "next_i")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);
        let final_sum = self
            .builder
            .build_load(self.types.i64_type, sum_ptr, "final_sum")
            .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?
            .into_int_value();

        self.make_int(final_sum)
    }

    // ========== Phase 3: String Operations ==========

    /// contains(str, substr) -> bool - check if string contains substring
    fn inline_contains(
        &mut self,
        container: BasicValueEnum<'ctx>,
        substr: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let container_data = self.extract_data(container)?;
        let substr_data = self.extract_data(substr)?;

        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let str_ptr = self
            .builder
            .build_int_to_ptr(container_data, i8_ptr_type, "str_ptr")
            .unwrap();
        let sub_ptr = self
            .builder
            .build_int_to_ptr(substr_data, i8_ptr_type, "sub_ptr")
            .unwrap();

        // Call strstr(str, substr) - returns NULL if not found
        let result_ptr = self
            .builder
            .build_call(
                self.libc.strstr,
                &[str_ptr.into(), sub_ptr.into()],
                "strstr_result",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Compare with null
        let null = i8_ptr_type.const_null();
        let is_found = self
            .builder
            .build_int_compare(inkwell::IntPredicate::NE, result_ptr, null, "is_found")
            .unwrap();

        // Convert to i64 (0 or 1) for the bool tag's data
        let found_i64 = self
            .builder
            .build_int_z_extend(is_found, self.types.i64_type, "found_i64")
            .unwrap();

        // Create bool tagged value
        let bool_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Bool.as_u8() as u64, false);
        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, bool_tag, 0, "v1")
            .unwrap();
        let v2 = self
            .builder
            .build_insert_value(v1, found_i64, 1, "v2")
            .unwrap();
        Ok(v2.into_struct_value().into())
    }

    /// upper(str) -> string - convert to uppercase
    fn inline_upper(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let str_data = self.extract_data(val)?;
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let str_ptr = self
            .builder
            .build_int_to_ptr(str_data, i8_ptr_type, "str_ptr")
            .unwrap();

        // Get length
        let len = self
            .builder
            .build_call(self.libc.strlen, &[str_ptr.into()], "len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        // Allocate result buffer (len + 1)
        let one = self.types.i64_type.const_int(1, false);
        let buf_size = self.builder.build_int_add(len, one, "buf_size").unwrap();
        let result_buf = self
            .builder
            .build_call(self.libc.malloc, &[buf_size.into()], "upper_buf")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Loop through each character and uppercase
        let function = self.current_function.unwrap();
        let loop_block = self.context.append_basic_block(function, "upper_loop");
        let body_block = self.context.append_basic_block(function, "upper_body");
        let done_block = self.context.append_basic_block(function, "upper_done");

        let i_ptr = self.builder.build_alloca(self.types.i64_type, "i").unwrap();
        let zero = self.types.i64_type.const_int(0, false);
        self.builder.build_store(i_ptr, zero).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i")
            .unwrap()
            .into_int_value();
        let cond = self
            .builder
            .build_int_compare(inkwell::IntPredicate::ULT, i, len, "cond")
            .unwrap();
        self.builder
            .build_conditional_branch(cond, body_block, done_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        let src_char_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), str_ptr, &[i], "src_char_ptr")
                .unwrap()
        };
        let char_val = self
            .builder
            .build_load(self.context.i8_type(), src_char_ptr, "char")
            .unwrap()
            .into_int_value();
        let char_i32 = self
            .builder
            .build_int_z_extend(char_val, self.types.i32_type, "char_i32")
            .unwrap();
        let upper_i32 = self
            .builder
            .build_call(self.libc.toupper, &[char_i32.into()], "upper_char")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let upper_i8 = self
            .builder
            .build_int_truncate(upper_i32, self.context.i8_type(), "upper_i8")
            .unwrap();

        let dst_char_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), result_buf, &[i], "dst_char_ptr")
                .unwrap()
        };
        self.builder.build_store(dst_char_ptr, upper_i8).unwrap();

        let next_i = self.builder.build_int_add(i, one, "next_i").unwrap();
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);
        // Null-terminate
        let null_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), result_buf, &[len], "null_ptr")
                .unwrap()
        };
        self.builder
            .build_store(null_ptr, self.context.i8_type().const_int(0, false))
            .unwrap();

        self.make_string(result_buf)
    }

    /// lower(str) -> string - convert to lowercase
    fn inline_lower(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let str_data = self.extract_data(val)?;
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let str_ptr = self
            .builder
            .build_int_to_ptr(str_data, i8_ptr_type, "str_ptr")
            .unwrap();

        let len = self
            .builder
            .build_call(self.libc.strlen, &[str_ptr.into()], "len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        let one = self.types.i64_type.const_int(1, false);
        let buf_size = self.builder.build_int_add(len, one, "buf_size").unwrap();
        let result_buf = self
            .builder
            .build_call(self.libc.malloc, &[buf_size.into()], "lower_buf")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let function = self.current_function.unwrap();
        let loop_block = self.context.append_basic_block(function, "lower_loop");
        let body_block = self.context.append_basic_block(function, "lower_body");
        let done_block = self.context.append_basic_block(function, "lower_done");

        let i_ptr = self.builder.build_alloca(self.types.i64_type, "i").unwrap();
        let zero = self.types.i64_type.const_int(0, false);
        self.builder.build_store(i_ptr, zero).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i")
            .unwrap()
            .into_int_value();
        let cond = self
            .builder
            .build_int_compare(inkwell::IntPredicate::ULT, i, len, "cond")
            .unwrap();
        self.builder
            .build_conditional_branch(cond, body_block, done_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        let src_char_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), str_ptr, &[i], "src_char_ptr")
                .unwrap()
        };
        let char_val = self
            .builder
            .build_load(self.context.i8_type(), src_char_ptr, "char")
            .unwrap()
            .into_int_value();
        let char_i32 = self
            .builder
            .build_int_z_extend(char_val, self.types.i32_type, "char_i32")
            .unwrap();
        let lower_i32 = self
            .builder
            .build_call(self.libc.tolower, &[char_i32.into()], "lower_char")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let lower_i8 = self
            .builder
            .build_int_truncate(lower_i32, self.context.i8_type(), "lower_i8")
            .unwrap();

        let dst_char_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), result_buf, &[i], "dst_char_ptr")
                .unwrap()
        };
        self.builder.build_store(dst_char_ptr, lower_i8).unwrap();

        let next_i = self.builder.build_int_add(i, one, "next_i").unwrap();
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);
        let null_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), result_buf, &[len], "null_ptr")
                .unwrap()
        };
        self.builder
            .build_store(null_ptr, self.context.i8_type().const_int(0, false))
            .unwrap();

        self.make_string(result_buf)
    }

    /// wheesht(str) -> string - trim whitespace
    fn inline_wheesht(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let str_data = self.extract_data(val)?;
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let str_ptr = self
            .builder
            .build_int_to_ptr(str_data, i8_ptr_type, "str_ptr")
            .unwrap();

        let len = self
            .builder
            .build_call(self.libc.strlen, &[str_ptr.into()], "len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        let function = self.current_function.unwrap();

        // Find start (skip leading whitespace)
        let find_start = self.context.append_basic_block(function, "find_start");
        let find_start_body = self.context.append_basic_block(function, "find_start_body");
        let find_end = self.context.append_basic_block(function, "find_end");
        let find_end_body = self.context.append_basic_block(function, "find_end_body");
        let copy = self.context.append_basic_block(function, "copy");

        let start_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "start")
            .unwrap();
        let end_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "end")
            .unwrap();
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        self.builder.build_store(start_ptr, zero).unwrap();
        self.builder.build_store(end_ptr, len).unwrap();
        self.builder.build_unconditional_branch(find_start).unwrap();

        // Find start loop
        self.builder.position_at_end(find_start);
        let start = self
            .builder
            .build_load(self.types.i64_type, start_ptr, "start")
            .unwrap()
            .into_int_value();
        let cond1 = self
            .builder
            .build_int_compare(inkwell::IntPredicate::ULT, start, len, "cond1")
            .unwrap();
        self.builder
            .build_conditional_branch(cond1, find_start_body, find_end)
            .unwrap();

        self.builder.position_at_end(find_start_body);
        let char_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), str_ptr, &[start], "char_ptr")
                .unwrap()
        };
        let char_val = self
            .builder
            .build_load(self.context.i8_type(), char_ptr, "char")
            .unwrap()
            .into_int_value();
        let char_i32 = self
            .builder
            .build_int_z_extend(char_val, self.types.i32_type, "char_i32")
            .unwrap();
        let is_space = self
            .builder
            .build_call(self.libc.isspace, &[char_i32.into()], "is_space")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let is_ws = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                is_space,
                self.types.i32_type.const_int(0, false),
                "is_ws",
            )
            .unwrap();

        // Create a continue block for when we find whitespace
        let start_continue = self.context.append_basic_block(function, "start_continue");

        // If whitespace, increment and loop; otherwise go to find_end
        self.builder
            .build_conditional_branch(is_ws, start_continue, find_end)
            .unwrap();

        // Increment start and continue looping
        self.builder.position_at_end(start_continue);
        let next_start = self
            .builder
            .build_int_add(start, one, "next_start")
            .unwrap();
        self.builder.build_store(start_ptr, next_start).unwrap();
        self.builder.build_unconditional_branch(find_start).unwrap();

        // Find end loop (from end backwards)
        self.builder.position_at_end(find_end);
        let end = self
            .builder
            .build_load(self.types.i64_type, end_ptr, "end")
            .unwrap()
            .into_int_value();
        let start_val = self
            .builder
            .build_load(self.types.i64_type, start_ptr, "start_val")
            .unwrap()
            .into_int_value();
        let cond2 = self
            .builder
            .build_int_compare(inkwell::IntPredicate::UGT, end, start_val, "cond2")
            .unwrap();
        self.builder
            .build_conditional_branch(cond2, find_end_body, copy)
            .unwrap();

        self.builder.position_at_end(find_end_body);
        let prev_end = self.builder.build_int_sub(end, one, "prev_end").unwrap();
        let char_ptr2 = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), str_ptr, &[prev_end], "char_ptr2")
                .unwrap()
        };
        let char_val2 = self
            .builder
            .build_load(self.context.i8_type(), char_ptr2, "char2")
            .unwrap()
            .into_int_value();
        let char_i32_2 = self
            .builder
            .build_int_z_extend(char_val2, self.types.i32_type, "char_i32_2")
            .unwrap();
        let is_space2 = self
            .builder
            .build_call(self.libc.isspace, &[char_i32_2.into()], "is_space2")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let is_ws2 = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                is_space2,
                self.types.i32_type.const_int(0, false),
                "is_ws2",
            )
            .unwrap();

        // If whitespace at prev_end, store the new end and loop; otherwise go to copy
        let end_continue = self.context.append_basic_block(function, "end_continue");
        self.builder
            .build_conditional_branch(is_ws2, end_continue, copy)
            .unwrap();

        self.builder.position_at_end(end_continue);
        self.builder.build_store(end_ptr, prev_end).unwrap();
        self.builder.build_unconditional_branch(find_end).unwrap();

        // Copy substring
        self.builder.position_at_end(copy);
        let final_start = self
            .builder
            .build_load(self.types.i64_type, start_ptr, "final_start")
            .unwrap()
            .into_int_value();
        let final_end = self
            .builder
            .build_load(self.types.i64_type, end_ptr, "final_end")
            .unwrap()
            .into_int_value();
        let new_len = self
            .builder
            .build_int_sub(final_end, final_start, "new_len")
            .unwrap();
        let buf_size = self
            .builder
            .build_int_add(new_len, one, "buf_size")
            .unwrap();

        let result_buf = self
            .builder
            .build_call(self.libc.malloc, &[buf_size.into()], "trim_buf")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let src_start = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), str_ptr, &[final_start], "src_start")
                .unwrap()
        };
        self.builder
            .build_call(
                self.libc.memcpy,
                &[result_buf.into(), src_start.into(), new_len.into()],
                "",
            )
            .unwrap();

        let null_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), result_buf, &[new_len], "null_ptr")
                .unwrap()
        };
        self.builder
            .build_store(null_ptr, self.context.i8_type().const_int(0, false))
            .unwrap();

        self.make_string(result_buf)
    }

    /// coont(str, substr) -> int - count occurrences
    fn inline_coont(
        &mut self,
        str_val: BasicValueEnum<'ctx>,
        sub_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let str_data = self.extract_data(str_val)?;
        let sub_data = self.extract_data(sub_val)?;
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

        let str_ptr = self
            .builder
            .build_int_to_ptr(str_data, i8_ptr_type, "str_ptr")
            .unwrap();
        let sub_ptr = self
            .builder
            .build_int_to_ptr(sub_data, i8_ptr_type, "sub_ptr")
            .unwrap();

        let sub_len = self
            .builder
            .build_call(self.libc.strlen, &[sub_ptr.into()], "sub_len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        let function = self.current_function.unwrap();
        let loop_block = self.context.append_basic_block(function, "coont_loop");
        let found_block = self.context.append_basic_block(function, "coont_found");
        let done_block = self.context.append_basic_block(function, "coont_done");

        let count_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "count")
            .unwrap();
        let pos_ptr = self.builder.build_alloca(i8_ptr_type, "pos").unwrap();
        let zero = self.types.i64_type.const_int(0, false);
        self.builder.build_store(count_ptr, zero).unwrap();
        self.builder.build_store(pos_ptr, str_ptr).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        let pos = self
            .builder
            .build_load(i8_ptr_type, pos_ptr, "pos")
            .unwrap()
            .into_pointer_value();
        let found = self
            .builder
            .build_call(self.libc.strstr, &[pos.into(), sub_ptr.into()], "found")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let null = i8_ptr_type.const_null();
        let is_found = self
            .builder
            .build_int_compare(inkwell::IntPredicate::NE, found, null, "is_found")
            .unwrap();
        self.builder
            .build_conditional_branch(is_found, found_block, done_block)
            .unwrap();

        self.builder.position_at_end(found_block);
        let count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "count")
            .unwrap()
            .into_int_value();
        let one = self.types.i64_type.const_int(1, false);
        let new_count = self.builder.build_int_add(count, one, "new_count").unwrap();
        self.builder.build_store(count_ptr, new_count).unwrap();

        // Move past this occurrence
        let next_pos = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), found, &[sub_len], "next_pos")
                .unwrap()
        };
        self.builder.build_store(pos_ptr, next_pos).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);
        let final_count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "final_count")
            .unwrap()
            .into_int_value();
        self.make_int(final_count)
    }

    // ========== Phase 4: Type & Utility Functions ==========

    /// whit_kind(x) -> string - return type name
    fn inline_whit_kind(
        &mut self,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let tag = self.extract_tag(val)?;

        let function = self.current_function.unwrap();
        let nil_block = self.context.append_basic_block(function, "kind_nil");
        let bool_block = self.context.append_basic_block(function, "kind_bool");
        let int_block = self.context.append_basic_block(function, "kind_int");
        let float_block = self.context.append_basic_block(function, "kind_float");
        let string_block = self.context.append_basic_block(function, "kind_string");
        let list_block = self.context.append_basic_block(function, "kind_list");
        let dict_block = self.context.append_basic_block(function, "kind_dict");
        let default_block = self.context.append_basic_block(function, "kind_default");
        let merge_block = self.context.append_basic_block(function, "kind_merge");

        // Create type name strings
        let nil_str = self
            .builder
            .build_global_string_ptr("naething", "nil_kind")
            .unwrap();
        let bool_str = self
            .builder
            .build_global_string_ptr("boolean", "bool_kind")
            .unwrap();
        let number_str = self
            .builder
            .build_global_string_ptr("number", "number_kind")
            .unwrap();
        let string_str = self
            .builder
            .build_global_string_ptr("string", "string_kind")
            .unwrap();
        let list_str = self
            .builder
            .build_global_string_ptr("list", "list_kind")
            .unwrap();
        let dict_str = self
            .builder
            .build_global_string_ptr("dict", "dict_kind")
            .unwrap();

        // Switch on tag
        self.builder
            .build_switch(
                tag,
                default_block,
                &[
                    (self.types.i8_type.const_int(0, false), nil_block),
                    (self.types.i8_type.const_int(1, false), bool_block),
                    (self.types.i8_type.const_int(2, false), int_block),
                    (self.types.i8_type.const_int(3, false), float_block),
                    (self.types.i8_type.const_int(4, false), string_block),
                    (self.types.i8_type.const_int(5, false), list_block),
                    (self.types.i8_type.const_int(6, false), dict_block),
                ],
            )
            .unwrap();

        self.builder.position_at_end(nil_block);
        let nil_result = self.make_string(nil_str.as_pointer_value())?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let nil_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(bool_block);
        let bool_result = self.make_string(bool_str.as_pointer_value())?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let bool_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(int_block);
        let int_result = self.make_string(number_str.as_pointer_value())?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let int_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(float_block);
        let float_result = self.make_string(number_str.as_pointer_value())?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let float_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(string_block);
        let string_result = self.make_string(string_str.as_pointer_value())?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let string_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(list_block);
        let list_result = self.make_string(list_str.as_pointer_value())?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let list_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(dict_block);
        let dict_result = self.make_string(dict_str.as_pointer_value())?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let dict_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(default_block);
        let default_result = self.make_string(nil_str.as_pointer_value())?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let default_bb = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(merge_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "kind_result")
            .unwrap();
        phi.add_incoming(&[
            (&nil_result, nil_bb),
            (&bool_result, bool_bb),
            (&int_result, int_bb),
            (&float_result, float_bb),
            (&string_result, string_bb),
            (&list_result, list_bb),
            (&dict_result, dict_bb),
            (&default_result, default_bb),
        ]);

        Ok(phi.as_basic_value())
    }

    /// range(start, end) -> list - create list [start, start+1, ..., end-1]
    fn inline_range(
        &mut self,
        start_val: BasicValueEnum<'ctx>,
        end_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let start = self.extract_data(start_val)?;
        let end = self.extract_data(end_val)?;

        let function = self.current_function.unwrap();

        // Calculate length = max(0, end - start)
        let diff = self.builder.build_int_sub(end, start, "diff").unwrap();
        let zero = self.types.i64_type.const_int(0, false);
        let is_positive = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SGT, diff, zero, "is_pos")
            .unwrap();
        let length = self
            .builder
            .build_select(is_positive, diff, zero, "length")
            .unwrap()
            .into_int_value();

        // Allocate list: 8 bytes header + length * 16 bytes
        let header_size = self.types.i64_type.const_int(16, false);
        let value_size = self.types.i64_type.const_int(16, false);
        let elements_size = self
            .builder
            .build_int_mul(length, value_size, "elem_size")
            .unwrap();
        let total_size = self
            .builder
            .build_int_add(header_size, elements_size, "total")
            .unwrap();

        let raw_ptr = self
            .builder
            .build_call(self.libc.malloc, &[total_size.into()], "range_list")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Store length
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let len_ptr = self
            .builder
            .build_pointer_cast(raw_ptr, i64_ptr_type, "len_ptr")
            .unwrap();
        self.builder.build_store(len_ptr, length).unwrap();

        // Get elements base
        let one = self.types.i64_type.const_int(1, false);
        let value_ptr_type = self.types.value_type.ptr_type(AddressSpace::default());
        let elements_base = unsafe {
            self.builder
                .build_gep(self.types.i64_type, len_ptr, &[one], "elements_base")
                .unwrap()
        };
        let elements_ptr = self
            .builder
            .build_pointer_cast(elements_base, value_ptr_type, "elements_ptr")
            .unwrap();

        // Loop to fill elements
        let loop_block = self.context.append_basic_block(function, "range_loop");
        let body_block = self.context.append_basic_block(function, "range_body");
        let done_block = self.context.append_basic_block(function, "range_done");

        let i_ptr = self.builder.build_alloca(self.types.i64_type, "i").unwrap();
        self.builder.build_store(i_ptr, zero).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(loop_block);
        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i")
            .unwrap()
            .into_int_value();
        let cond = self
            .builder
            .build_int_compare(inkwell::IntPredicate::ULT, i, length, "cond")
            .unwrap();
        self.builder
            .build_conditional_branch(cond, body_block, done_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        let val = self.builder.build_int_add(start, i, "val").unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.types.value_type, elements_ptr, &[i], "elem_ptr")
                .unwrap()
        };
        let elem = self.make_int(val)?;
        self.builder.build_store(elem_ptr, elem).unwrap();

        let next_i = self.builder.build_int_add(i, one, "next_i").unwrap();
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);
        self.make_list(raw_ptr)
    }

    // ========== Statement Compilation ==========

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), HaversError> {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                // Track the inferred type for optimization
                let var_type = if let Some(init) = initializer {
                    self.infer_expr_type(init)
                } else {
                    VarType::Unknown
                };
                self.var_types.insert(name.clone(), var_type);

                // Check if this is a top-level declaration (needs LLVM global)
                let is_top_level = self.current_class.is_none()
                    && self.loop_stack.is_empty()
                    && !self.variables.contains_key(name)
                    && !self.globals.contains_key(name);

                // For int variables, try to use optimized path
                // Note: we skip the optimization for top-level vars since they need LLVM globals
                if var_type == VarType::Int && !is_top_level {
                    // Check if shadow already exists (re-declaration in loop)
                    let shadow = if let Some(&existing) = self.int_shadows.get(name) {
                        existing
                    } else {
                        let s = self.create_entry_block_alloca_i64(&format!("{}_shadow", name));
                        self.int_shadows.insert(name.clone(), s);
                        s
                    };

                    // Try to get the int value directly
                    if let Some(init) = initializer {
                        if let Some(int_val) = self.compile_int_expr(init)? {
                            // Store to shadow
                            self.builder.build_store(shadow, int_val).map_err(|e| {
                                HaversError::CompileError(format!("Failed to store shadow: {}", e))
                            })?;

                            // Skip MdhValue store in loop body
                            if !self.in_loop_body {
                                // Ensure MdhValue alloca exists
                                let alloca = if let Some(&existing) = self.variables.get(name) {
                                    existing
                                } else {
                                    let a = self.create_entry_block_alloca(name);
                                    self.variables.insert(name.clone(), a);
                                    a
                                };
                                let boxed = self.make_int(int_val)?;
                                self.builder.build_store(alloca, boxed).map_err(|e| {
                                    HaversError::CompileError(format!("Failed to store: {}", e))
                                })?;
                            } else {
                                // In loop: just ensure alloca exists
                                if !self.variables.contains_key(name) {
                                    let a = self.create_entry_block_alloca(name);
                                    self.variables.insert(name.clone(), a);
                                }
                            }
                            return Ok(());
                        }
                    }
                }

                // Fall back to standard path
                let value = if let Some(init) = initializer {
                    self.compile_expr(init)?
                } else {
                    self.make_nil()
                };

                // Check if we need to use a global variable (top-level declaration)
                // Top-level vars need to be true LLVM globals to be accessible from methods
                let is_top_level = self.current_class.is_none()
                    && self.loop_stack.is_empty()
                    && !self.variables.contains_key(name);

                let alloca = if let Some(&existing) = self.variables.get(name) {
                    existing
                } else if let Some(&existing) = self.globals.get(name) {
                    // Already exists as a global
                    existing
                } else if is_top_level {
                    // Create an LLVM global variable
                    let global = self.module.add_global(self.types.value_type, None, name);
                    global.set_initializer(&self.types.value_type.const_zero());
                    let global_ptr = global.as_pointer_value();
                    self.globals.insert(name.clone(), global_ptr);
                    // Also add to variables so main function can find it easily
                    self.variables.insert(name.clone(), global_ptr);
                    global_ptr
                } else {
                    let a = self.create_entry_block_alloca(name);
                    self.variables.insert(name.clone(), a);
                    a
                };
                self.builder
                    .build_store(alloca, value)
                    .map_err(|e| HaversError::CompileError(format!("Failed to store: {}", e)))?;

                // Create shadow if needed
                if var_type == VarType::Int && !self.int_shadows.contains_key(name) {
                    let shadow = self.create_entry_block_alloca_i64(&format!("{}_shadow", name));
                    let data = self.extract_data(value)?;
                    self.builder.build_store(shadow, data).map_err(|e| {
                        HaversError::CompileError(format!("Failed to store shadow: {}", e))
                    })?;
                    self.int_shadows.insert(name.clone(), shadow);
                }
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

            Stmt::Class {
                name,
                superclass: _,
                methods,
                ..
            } => self.compile_class(name, methods),

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
                    // If we're in a loop body and have a shadow, construct fresh MdhValue from shadow
                    // This ensures function calls get the correct value even though we've been
                    // skipping MdhValue stores for optimization
                    if self.in_loop_body {
                        if let Some(&shadow) = self.int_shadows.get(name) {
                            let int_val = self
                                .builder
                                .build_load(
                                    self.types.i64_type,
                                    shadow,
                                    &format!("{}_shadow_load", name),
                                )
                                .map_err(|e| {
                                    HaversError::CompileError(format!(
                                        "Failed to load shadow: {}",
                                        e
                                    ))
                                })?
                                .into_int_value();
                            return self.make_int(int_val);
                        }
                    }
                    let val = self
                        .builder
                        .build_load(self.types.value_type, alloca, name)
                        .map_err(|e| HaversError::CompileError(format!("Failed to load: {}", e)))?;
                    Ok(val)
                } else if let Some(&global) = self.globals.get(name) {
                    // Global variable
                    let val = self
                        .builder
                        .build_load(self.types.value_type, global, &format!("{}_global", name))
                        .map_err(|e| {
                            HaversError::CompileError(format!("Failed to load global: {}", e))
                        })?;
                    Ok(val)
                } else if let Some(&func) = self.functions.get(name) {
                    // User-defined function referenced as a value - return function pointer
                    let func_ptr = func.as_global_value().as_pointer_value();
                    let func_int = self
                        .builder
                        .build_ptr_to_int(func_ptr, self.types.i64_type, "func_int")
                        .unwrap();
                    self.make_function(func_int)
                } else {
                    Err(HaversError::CompileError(format!(
                        "Undefined variable: {}",
                        name
                    )))
                }
            }

            Expr::Assign { name, value, .. } => {
                // Try to use optimized int path if we have an int shadow
                if let Some(&shadow) = self.int_shadows.get(name) {
                    // Try to compile the value directly as i64
                    if let Some(int_val) = self.compile_int_expr(value)? {
                        // Update the shadow with the new i64 value
                        self.builder.build_store(shadow, int_val).map_err(|e| {
                            HaversError::CompileError(format!("Failed to store shadow: {}", e))
                        })?;

                        // Skip MdhValue store in loop body (will sync at loop exit)
                        if self.in_loop_body {
                            // Still need to return a valid MdhValue
                            let boxed = self.make_int(int_val)?;
                            return Ok(boxed);
                        }

                        // Outside loop: also update the MdhValue
                        let boxed = self.make_int(int_val)?;
                        if let Some(&alloca) = self.variables.get(name) {
                            self.builder.build_store(alloca, boxed).map_err(|e| {
                                HaversError::CompileError(format!("Failed to store: {}", e))
                            })?;
                        }
                        return Ok(boxed);
                    }
                }

                // Fall back to standard path
                let val = self.compile_expr(value)?;
                if let Some(&alloca) = self.variables.get(name) {
                    self.builder.build_store(alloca, val).map_err(|e| {
                        HaversError::CompileError(format!("Failed to store: {}", e))
                    })?;
                    // Update int shadow if we have one
                    if let Some(&shadow) = self.int_shadows.get(name) {
                        let data = self.extract_data(val)?;
                        self.builder.build_store(shadow, data).map_err(|e| {
                            HaversError::CompileError(format!("Failed to store shadow: {}", e))
                        })?;
                    }
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

            Expr::Grouping { expr, .. } => self.compile_expr(expr),

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => self.compile_ternary(condition, then_expr, else_expr),

            Expr::Range { start, end, .. } => {
                // For now, just compile as nil - ranges are handled in for loops
                let _start_val = self.compile_expr(start)?;
                let _end_val = self.compile_expr(end)?;
                Ok(self.make_nil())
            }

            Expr::List { elements, .. } => self.compile_list(elements),

            Expr::Dict { pairs, .. } => self.compile_dict(pairs),

            Expr::Index { object, index, .. } => self.compile_index(object, index),

            Expr::IndexSet {
                object,
                index,
                value,
                ..
            } => self.compile_index_set(object, index, value),

            Expr::Input { prompt, .. } => {
                let prompt_val = Some(self.compile_expr(prompt)?);
                self.inline_speir(prompt_val)
            }

            Expr::Lambda { params, body, .. } => self.compile_lambda(params, body),

            Expr::Masel { .. } => self.compile_masel(),

            Expr::Get {
                object, property, ..
            } => self.compile_get(object, property),

            Expr::Set {
                object,
                property,
                value,
                ..
            } => self.compile_set(object, property, value),

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
                let str_ptr = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to create string: {}", e))
                    })?;
                self.make_string(str_ptr.as_pointer_value())
            }
        }
    }

    fn compile_binary(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Type-based optimization: if both operands are known to be Int, use fast path
        let left_type = self.infer_expr_type(left);
        let right_type = self.infer_expr_type(right);

        // Integer fast path for arithmetic operations
        if left_type == VarType::Int && right_type == VarType::Int {
            match op {
                BinaryOp::Add
                | BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Modulo => {
                    return self.compile_binary_int_fast(left, op, right);
                }
                _ => {} // Comparisons already optimized via compile_condition_direct
            }
        }

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

    /// Fast path for integer arithmetic - uses shadows when available
    fn compile_binary_int_fast(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Try to use int shadows directly (avoids MdhValue load)
        let left_data = if let Some(l) = self.compile_int_expr(left)? {
            l
        } else {
            let left_val = self.compile_expr(left)?;
            self.extract_data(left_val)?
        };

        let right_data = if let Some(r) = self.compile_int_expr(right)? {
            r
        } else {
            let right_val = self.compile_expr(right)?;
            self.extract_data(right_val)?
        };

        // Perform operation directly on i64
        let result = match op {
            BinaryOp::Add => self
                .builder
                .build_int_add(left_data, right_data, "add_fast")
                .unwrap(),
            BinaryOp::Subtract => self
                .builder
                .build_int_sub(left_data, right_data, "sub_fast")
                .unwrap(),
            BinaryOp::Multiply => self
                .builder
                .build_int_mul(left_data, right_data, "mul_fast")
                .unwrap(),
            BinaryOp::Divide => self
                .builder
                .build_int_signed_div(left_data, right_data, "div_fast")
                .unwrap(),
            BinaryOp::Modulo => self
                .builder
                .build_int_signed_rem(left_data, right_data, "mod_fast")
                .unwrap(),
            _ => unreachable!(),
        };

        // Box the result back to MdhValue
        self.make_int(result)
    }

    fn compile_unary(
        &mut self,
        op: UnaryOp,
        operand: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let val = self.compile_expr(operand)?;
        match op {
            UnaryOp::Negate => self.inline_neg(val),
            UnaryOp::Not => self.inline_not(val),
        }
    }

    fn compile_logical(
        &mut self,
        left: &Expr,
        op: LogicalOp,
        right: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();
        let left_val = self.compile_expr(left)?;
        let left_truthy = self.is_truthy(left_val)?;

        let eval_right = self.context.append_basic_block(function, "eval_right");
        let merge = self.context.append_basic_block(function, "merge");

        match op {
            LogicalOp::And => {
                self.builder
                    .build_conditional_branch(left_truthy, eval_right, merge)
                    .unwrap();
            }
            LogicalOp::Or => {
                self.builder
                    .build_conditional_branch(left_truthy, merge, eval_right)
                    .unwrap();
            }
        }

        let left_block = self.builder.get_insert_block().unwrap();
        self.builder.position_at_end(eval_right);
        let right_val = self.compile_expr(right)?;
        let right_block = self.builder.get_insert_block().unwrap();
        self.builder.build_unconditional_branch(merge).unwrap();

        self.builder.position_at_end(merge);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "logical")
            .unwrap();
        phi.add_incoming(&[(&left_val, left_block), (&right_val, right_block)]);

        Ok(phi.as_basic_value())
    }

    fn compile_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Check for method call: obj.method(args)
        if let Expr::Get {
            object, property, ..
        } = callee
        {
            return self.compile_method_call(object, property, args);
        }

        if let Expr::Variable { name, .. } = callee {
            // Check for class instantiation: ClassName()
            if self.classes.contains_key(name) {
                return self.compile_class_instantiation(name, args);
            }

            // Check for user-defined functions FIRST to allow shadowing built-ins
            if let Some(&func) = self.functions.get(name) {
                let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::new();
                for arg in args {
                    compiled_args.push(self.compile_expr(arg)?.into());
                }

                let call_site = self
                    .builder
                    .build_call(func, &compiled_args, "call")
                    .map_err(|e| HaversError::CompileError(format!("Failed to call: {}", e)))?;

                call_site.set_tail_call(true);

                return Ok(call_site.try_as_basic_value().left().unwrap());
            }

            // Check for built-in functions
            match name.as_str() {
                "tae_string" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "tae_string expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_tae_string(arg);
                }
                "tae_int" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "tae_int expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_tae_int(arg);
                }
                "tae_float" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "tae_float expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_tae_float(arg);
                }
                "len" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "len expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_len(arg);
                }
                "shove" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "shove expects 2 arguments (list, element)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let elem_arg = self.compile_expr(&args[1])?;
                    let result = self.inline_shove(list_arg, elem_arg)?;

                    // If first argument is a variable, update it with the (possibly new) list pointer
                    if let Expr::Variable { name, .. } = &args[0] {
                        if let Some(var_ptr) = self.variables.get(name).copied() {
                            self.builder.build_store(var_ptr, result).unwrap();
                        }
                    }
                    return Ok(result);
                }
                // Phase 1: Math functions
                "abs" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "abs expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_abs(arg);
                }
                "min" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "min expects 2 arguments".to_string(),
                        ));
                    }
                    let a = self.compile_expr(&args[0])?;
                    let b = self.compile_expr(&args[1])?;
                    return self.inline_min(a, b);
                }
                "max" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "max expects 2 arguments".to_string(),
                        ));
                    }
                    let a = self.compile_expr(&args[0])?;
                    let b = self.compile_expr(&args[1])?;
                    return self.inline_max(a, b);
                }
                "floor" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "floor expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_floor(arg);
                }
                "ceil" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "ceil expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_ceil(arg);
                }
                "round" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "round expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_round(arg);
                }
                "sqrt" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "sqrt expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_sqrt(arg);
                }
                // Phase 2: List operations
                "yank" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "yank expects 1 argument".to_string(),
                        ));
                    }
                    let list = self.compile_expr(&args[0])?;
                    return self.inline_yank(list);
                }
                "heid" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "heid expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_heid(arg);
                }
                "tail" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "tail expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_tail(arg);
                }
                "bum" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "bum expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_bum(arg);
                }
                "scran" => {
                    if args.len() != 3 {
                        return Err(HaversError::CompileError(
                            "scran expects 3 arguments (list, start, end)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let start_arg = self.compile_expr(&args[1])?;
                    let end_arg = self.compile_expr(&args[2])?;
                    return self.inline_scran(list_arg, start_arg, end_arg);
                }
                "slap" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "slap expects 2 arguments".to_string(),
                        ));
                    }
                    let a = self.compile_expr(&args[0])?;
                    let b = self.compile_expr(&args[1])?;
                    return self.inline_slap(a, b);
                }
                "reverse" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "reverse expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_reverse(arg);
                }
                "sumaw" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "sumaw expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_sumaw(arg);
                }
                // Phase 3: String operations
                "contains" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "contains expects 2 arguments".to_string(),
                        ));
                    }
                    let container = self.compile_expr(&args[0])?;
                    let substr = self.compile_expr(&args[1])?;
                    return self.inline_contains(container, substr);
                }
                "upper" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "upper expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_upper(arg);
                }
                "lower" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "lower expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_lower(arg);
                }
                "wheesht" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "wheesht expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_wheesht(arg);
                }
                "coont" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "coont expects 2 arguments".to_string(),
                        ));
                    }
                    let str_arg = self.compile_expr(&args[0])?;
                    let sub_arg = self.compile_expr(&args[1])?;
                    return self.inline_coont(str_arg, sub_arg);
                }
                // Phase 4: Type & Utility functions
                "whit_kind" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "whit_kind expects 1 argument".to_string(),
                        ));
                    }
                    let arg = self.compile_expr(&args[0])?;
                    return self.inline_whit_kind(arg);
                }
                "range" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "range expects 2 arguments".to_string(),
                        ));
                    }
                    let start = self.compile_expr(&args[0])?;
                    let end = self.compile_expr(&args[1])?;
                    return self.inline_range(start, end);
                }
                // Phase 5: Timing functions
                "noo" => {
                    if !args.is_empty() {
                        return Err(HaversError::CompileError(
                            "noo expects 0 arguments".to_string(),
                        ));
                    }
                    return self.inline_noo();
                }
                "tick" => {
                    if !args.is_empty() {
                        return Err(HaversError::CompileError(
                            "tick expects 0 arguments".to_string(),
                        ));
                    }
                    return self.inline_tick();
                }
                "bide" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "bide expects 1 argument (milliseconds)".to_string(),
                        ));
                    }
                    let ms = self.compile_expr(&args[0])?;
                    return self.inline_bide(ms);
                }
                // Phase 7: I/O functions
                "speir" => {
                    let prompt = if args.is_empty() {
                        None
                    } else {
                        Some(self.compile_expr(&args[0])?)
                    };
                    return self.inline_speir(prompt);
                }
                // Extra: String operations
                "split" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "split expects 2 arguments (string, delimiter)".to_string(),
                        ));
                    }
                    let str_arg = self.compile_expr(&args[0])?;
                    let delim_arg = self.compile_expr(&args[1])?;
                    return self.inline_split(str_arg, delim_arg);
                }
                "join" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "join expects 2 arguments (list, delimiter)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let delim_arg = self.compile_expr(&args[1])?;
                    return self.inline_join(list_arg, delim_arg);
                }
                "sort" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "sort expects 1 argument (list)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    return self.inline_sort(list_arg);
                }
                "shuffle" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "shuffle expects 1 argument (list)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    return self.inline_shuffle(list_arg);
                }
                // Phase 6: Higher-order functions
                "gaun" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "gaun expects 2 arguments (list, function)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let func_arg = self.compile_expr(&args[1])?;
                    return self.inline_gaun(list_arg, func_arg);
                }
                "sieve" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "sieve expects 2 arguments (list, function)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let func_arg = self.compile_expr(&args[1])?;
                    return self.inline_sieve(list_arg, func_arg);
                }
                "tumble" => {
                    if args.len() != 3 {
                        return Err(HaversError::CompileError(
                            "tumble expects 3 arguments (list, initial, function)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let init_arg = self.compile_expr(&args[1])?;
                    let func_arg = self.compile_expr(&args[2])?;
                    return self.inline_tumble(list_arg, init_arg, func_arg);
                }
                "aw" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "aw expects 2 arguments (list, function)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let func_arg = self.compile_expr(&args[1])?;
                    return self.inline_aw(list_arg, func_arg);
                }
                "ony" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "ony expects 2 arguments (list, function)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let func_arg = self.compile_expr(&args[1])?;
                    return self.inline_ony(list_arg, func_arg);
                }
                "hunt" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "hunt expects 2 arguments (list, function)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let func_arg = self.compile_expr(&args[1])?;
                    return self.inline_hunt(list_arg, func_arg);
                }
                "ilk" => {
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "ilk expects 2 arguments (list, function)".to_string(),
                        ));
                    }
                    let list_arg = self.compile_expr(&args[0])?;
                    let func_arg = self.compile_expr(&args[1])?;
                    return self.inline_ilk(list_arg, func_arg);
                }
                // Dict functions
                "keys" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "keys expects 1 argument (dict)".to_string(),
                        ));
                    }
                    let dict_arg = self.compile_expr(&args[0])?;
                    return self.inline_keys(dict_arg);
                }
                "values" => {
                    if args.len() != 1 {
                        return Err(HaversError::CompileError(
                            "values expects 1 argument (dict)".to_string(),
                        ));
                    }
                    let dict_arg = self.compile_expr(&args[0])?;
                    return self.inline_values(dict_arg);
                }
                "jammy" => {
                    // jammy(min, max) - random number between min and max (inclusive)
                    if args.len() != 2 {
                        return Err(HaversError::CompileError(
                            "jammy expects 2 arguments (min, max)".to_string(),
                        ));
                    }
                    let min_arg = self.compile_expr(&args[0])?;
                    let max_arg = self.compile_expr(&args[1])?;
                    return self.inline_jammy(min_arg, max_arg);
                }
                "get_key" => {
                    // get_key() - read a single key press
                    if !args.is_empty() {
                        return Err(HaversError::CompileError(
                            "get_key expects 0 arguments".to_string(),
                        ));
                    }
                    return self.inline_get_key();
                }
                _ => {}
            }

            // Check if it's a variable containing a function value (lambda)
            if let Some(&var_ptr) = self.variables.get(name) {
                let func_val = self
                    .builder
                    .build_load(self.types.value_type, var_ptr, "func_val")
                    .map_err(|e| HaversError::CompileError(format!("Failed to load var: {}", e)))?;

                // Compile arguments
                let mut compiled_args: Vec<BasicValueEnum<'ctx>> = Vec::new();
                for arg in args {
                    compiled_args.push(self.compile_expr(arg)?);
                }

                return self.call_function_value(func_val, &compiled_args);
            }
        }

        // Check if callee is any expression that evaluates to a function value
        let func_val = self.compile_expr(callee)?;
        let mut compiled_args: Vec<BasicValueEnum<'ctx>> = Vec::new();
        for arg in args {
            compiled_args.push(self.compile_expr(arg)?);
        }
        self.call_function_value(func_val, &compiled_args)
    }

    fn compile_if(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Result<(), HaversError> {
        let function = self.current_function.unwrap();
        // Optimization: try to compile condition directly to i1 without boxing
        let cond_bool = if let Some(direct) = self.compile_condition_direct(condition)? {
            direct
        } else {
            let cond_val = self.compile_expr(condition)?;
            self.is_truthy(cond_val)?
        };

        let then_block = self.context.append_basic_block(function, "then");
        let else_block = self.context.append_basic_block(function, "else");
        let merge_block = self.context.append_basic_block(function, "merge");

        self.builder
            .build_conditional_branch(cond_bool, then_block, else_block)
            .unwrap();

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
                .unwrap();
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
                .unwrap();
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
        // Optimization: try to compile condition directly to i1 without boxing
        let cond_bool = if let Some(direct) = self.compile_condition_direct(condition)? {
            direct
        } else {
            // Fallback: compile expression and check truthiness
            let cond_val = self.compile_expr(condition)?;
            self.is_truthy(cond_val)?
        };
        self.builder
            .build_conditional_branch(cond_bool, body_block, after_block)
            .unwrap();

        self.builder.position_at_end(body_block);
        // Mark that we're in a hot loop body (skip MdhValue stores)
        let was_in_loop = self.in_loop_body;
        self.in_loop_body = true;
        self.compile_stmt(body)?;
        self.in_loop_body = was_in_loop;
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(loop_block).unwrap();
        }

        self.loop_stack.pop();
        self.builder.position_at_end(after_block);

        // Sync all dirty shadows to MdhValue at loop exit
        self.sync_all_shadows()?;

        Ok(())
    }

    fn compile_for(
        &mut self,
        variable: &str,
        iterable: &Expr,
        body: &Stmt,
    ) -> Result<(), HaversError> {
        if let Expr::Range {
            start,
            end,
            inclusive,
            ..
        } = iterable
        {
            return self.compile_for_range(variable, start, end, *inclusive, body);
        }
        // For-each loop over list
        self.compile_for_list(variable, iterable, body)
    }

    fn compile_for_list(
        &mut self,
        variable: &str,
        iterable: &Expr,
        body: &Stmt,
    ) -> Result<(), HaversError> {
        let function = self.current_function.unwrap();

        // Compile the iterable (should be a list)
        let list_val = self.compile_expr(iterable)?;

        // Extract list pointer from the MdhValue
        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let i64_ptr_type = self
            .types
            .i64_type
            .ptr_type(inkwell::AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        // Get list length (first 8 bytes)
        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        // Create loop variable (holds element value)
        let var_alloca = self.create_entry_block_alloca(variable);
        self.variables.insert(variable.to_string(), var_alloca);

        // Create index counter
        let idx_alloca = self
            .builder
            .build_alloca(self.types.i64_type, "for_idx")
            .unwrap();
        let zero = self.types.i64_type.const_int(0, false);
        self.builder.build_store(idx_alloca, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "for_list_loop");
        let body_block = self.context.append_basic_block(function, "for_list_body");
        let incr_block = self.context.append_basic_block(function, "for_list_incr");
        let after_block = self.context.append_basic_block(function, "for_list_after");

        self.loop_stack.push(LoopContext {
            break_block: after_block,
            continue_block: incr_block,
        });

        self.builder.build_unconditional_branch(loop_block).unwrap();

        // Loop condition: idx < len
        self.builder.position_at_end(loop_block);
        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_alloca, "idx")
            .unwrap()
            .into_int_value();
        let cmp = self
            .builder
            .build_int_compare(IntPredicate::ULT, idx, list_len, "for_cmp")
            .unwrap();
        self.builder
            .build_conditional_branch(cmp, body_block, after_block)
            .unwrap();

        // Body: load element at idx into variable
        self.builder.position_at_end(body_block);

        // Calculate element offset: 8 (header) + idx * 16 (element size)
        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "idx_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();

        // Get pointer to element
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), list_ptr, &[elem_offset], "elem_ptr")
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types
                    .value_type
                    .ptr_type(inkwell::AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let elem_val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "elem_val")
            .unwrap();

        // Store element in loop variable
        self.builder.build_store(var_alloca, elem_val).unwrap();

        // Compile body
        self.compile_stmt(body)?;
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder.build_unconditional_branch(incr_block).unwrap();
        }

        // Increment
        self.builder.position_at_end(incr_block);
        let one = self.types.i64_type.const_int(1, false);
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_alloca, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.loop_stack.pop();
        self.builder.position_at_end(after_block);
        Ok(())
    }

    fn compile_for_range(
        &mut self,
        variable: &str,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
        body: &Stmt,
    ) -> Result<(), HaversError> {
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
        let counter_alloca = self
            .builder
            .build_alloca(self.types.i64_type, "counter")
            .unwrap();
        self.builder
            .build_store(counter_alloca, start_data)
            .unwrap();

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
        let current = self
            .builder
            .build_load(self.types.i64_type, counter_alloca, "current")
            .unwrap()
            .into_int_value();
        let cmp = if inclusive {
            self.builder
                .build_int_compare(IntPredicate::SLE, current, end_data, "cmp")
        } else {
            self.builder
                .build_int_compare(IntPredicate::SLT, current, end_data, "cmp")
        }
        .unwrap();
        self.builder
            .build_conditional_branch(cmp, body_block, after_block)
            .unwrap();

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

        let saved_function = self.current_function;
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_var_types = std::mem::take(&mut self.var_types);
        let saved_int_shadows = std::mem::take(&mut self.int_shadows);

        self.builder.position_at_end(entry);
        self.current_function = Some(function);

        // Set up parameters
        // Create shadows for all parameters (optimistic: assume they're integers)
        // The shadow will be used if the parameter is used in integer context
        for (i, param) in params.iter().enumerate() {
            let param_val = function
                .get_nth_param(i as u32)
                .ok_or_else(|| HaversError::CompileError("Missing parameter".to_string()))?;
            let alloca = self.create_entry_block_alloca(&param.name);
            self.builder.build_store(alloca, param_val).unwrap();
            self.variables.insert(param.name.clone(), alloca);

            // Create shadow with extracted int value (optimistic)
            // If parameter isn't actually an int at runtime, this is still safe
            // because we only use the shadow when we KNOW it's an int context
            let shadow = self.create_entry_block_alloca_i64(&format!("{}_shadow", param.name));
            let data = self.extract_data(param_val)?;
            self.builder.build_store(shadow, data).unwrap();
            self.int_shadows.insert(param.name.clone(), shadow);

            // Mark as Unknown - but shadow is available for optimizations
            self.var_types.insert(param.name.clone(), VarType::Unknown);
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
            self.builder.build_return(Some(&self.make_nil())).unwrap();
        }

        // Restore state
        self.current_function = saved_function;
        self.variables = saved_variables;
        self.var_types = saved_var_types;
        self.int_shadows = saved_int_shadows;

        if let Some(func) = saved_function {
            if let Some(last_block) = func.get_last_basic_block() {
                self.builder.position_at_end(last_block);
            }
        }

        Ok(())
    }

    fn compile_ternary(
        &mut self,
        condition: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        let cond_val = self.compile_expr(condition)?;
        let cond_bool = self.is_truthy(cond_val)?;

        let then_block = self.context.append_basic_block(function, "tern_then");
        let else_block = self.context.append_basic_block(function, "tern_else");
        let merge_block = self.context.append_basic_block(function, "tern_merge");

        self.builder
            .build_conditional_branch(cond_bool, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(then_block);
        let then_val = self.compile_expr(then_expr)?;
        let then_bb = self.builder.get_insert_block().unwrap();
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();

        self.builder.position_at_end(else_block);
        let else_val = self.compile_expr(else_expr)?;
        let else_bb = self.builder.get_insert_block().unwrap();
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();

        self.builder.position_at_end(merge_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "tern")
            .unwrap();
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

    /// Create alloca for i64 shadow variable in entry block (hoisted from loop)
    fn create_entry_block_alloca_i64(&self, name: &str) -> PointerValue<'ctx> {
        let function = self.current_function.unwrap();
        let entry = function.get_first_basic_block().unwrap();

        let builder = self.context.create_builder();
        match entry.get_first_instruction() {
            Some(instr) => builder.position_before(&instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.types.i64_type, name).unwrap()
    }

    /// Compile a list expression: allocate memory and store elements
    /// Memory layout: [i64 length, {i8,i64} elem0, {i8,i64} elem1, ...]
    fn compile_list(&mut self, elements: &[Expr]) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let len = elements.len();

        // New layout: [capacity: i64][length: i64][elements...]
        // Calculate capacity: at least 8 elements or len, whichever is larger
        let initial_capacity = std::cmp::max(8, len);
        let value_size = 16u64; // sizeof({i8, i64}) with alignment
        let header_size = 16u64; // 2 * sizeof(i64) for capacity + length
        let total_size = header_size + (initial_capacity as u64) * value_size;

        // Allocate memory
        let size_val = self.types.i64_type.const_int(total_size, false);
        let raw_ptr = self
            .builder
            .build_call(self.libc.malloc, &[size_val.into()], "list_alloc")
            .map_err(|e| HaversError::CompileError(format!("Failed to call malloc: {}", e)))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| HaversError::CompileError("malloc returned void".to_string()))?
            .into_pointer_value();

        // Cast to i64* for storing capacity and length
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let header_ptr = self
            .builder
            .build_pointer_cast(raw_ptr, i64_ptr_type, "header_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast pointer: {}", e)))?;

        // Store capacity at offset 0
        let cap_val = self.types.i64_type.const_int(initial_capacity as u64, false);
        self.builder
            .build_store(header_ptr, cap_val)
            .map_err(|e| HaversError::CompileError(format!("Failed to store capacity: {}", e)))?;

        // Store length at offset 1
        let len_ptr = unsafe {
            self.builder
                .build_gep(self.types.i64_type, header_ptr, &[self.types.i64_type.const_int(1, false)], "len_ptr")
                .map_err(|e| HaversError::CompileError(format!("Failed to get len ptr: {}", e)))?
        };
        let len_val = self.types.i64_type.const_int(len as u64, false);
        self.builder
            .build_store(len_ptr, len_val)
            .map_err(|e| HaversError::CompileError(format!("Failed to store length: {}", e)))?;

        // Get pointer to elements array (after capacity and length = offset 2)
        let value_ptr_type = self.types.value_type.ptr_type(AddressSpace::default());
        let elements_base = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(2, false)],
                    "elements_base",
                )
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to compute elements base: {}", e))
                })?
        };
        let elements_ptr = self
            .builder
            .build_pointer_cast(elements_base, value_ptr_type, "elements_ptr")
            .map_err(|e| {
                HaversError::CompileError(format!("Failed to cast elements pointer: {}", e))
            })?;

        // Compile and store each element
        for (i, elem) in elements.iter().enumerate() {
            let compiled = self.compile_expr(elem)?;

            // Get pointer to this element slot
            let elem_ptr = unsafe {
                self.builder
                    .build_gep(
                        self.types.value_type,
                        elements_ptr,
                        &[self.types.i64_type.const_int(i as u64, false)],
                        &format!("elem_{}", i),
                    )
                    .map_err(|e| {
                        HaversError::CompileError(format!(
                            "Failed to compute element pointer: {}",
                            e
                        ))
                    })?
            };

            // Store the element
            self.builder.build_store(elem_ptr, compiled).map_err(|e| {
                HaversError::CompileError(format!("Failed to store element: {}", e))
            })?;
        }

        // Return the list as a tagged value
        self.make_list(raw_ptr)
    }

    /// Compile a dict literal expression: {key1: value1, key2: value2, ...}
    /// Dict memory layout: [i64 count][entry0][entry1]... where entry = [{i8,i64} key][{i8,i64} val]
    fn compile_dict(
        &mut self,
        pairs: &[(Expr, Expr)],
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let count = pairs.len();

        // Calculate memory size: 8 bytes for count + count * 32 bytes for entries (16 bytes key + 16 bytes value)
        let entry_size = 32u64; // 16 bytes for key + 16 bytes for value
        let header_size = 8u64; // sizeof(i64) for count
        let total_size = header_size + (count as u64) * entry_size;

        // Allocate memory
        let size_val = self.types.i64_type.const_int(total_size, false);
        let raw_ptr = self
            .builder
            .build_call(self.libc.malloc, &[size_val.into()], "dict_alloc")
            .map_err(|e| HaversError::CompileError(format!("Failed to call malloc: {}", e)))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| HaversError::CompileError("malloc returned void".to_string()))?
            .into_pointer_value();

        // Cast to i64* for storing the count
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let count_ptr = self
            .builder
            .build_pointer_cast(raw_ptr, i64_ptr_type, "count_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast pointer: {}", e)))?;

        // Store count
        let count_val = self.types.i64_type.const_int(count as u64, false);
        self.builder
            .build_store(count_ptr, count_val)
            .map_err(|e| HaversError::CompileError(format!("Failed to store count: {}", e)))?;

        // Get pointer to entries array (after the count)
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let entries_base = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    count_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "entries_base",
                )
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to compute entries base: {}", e))
                })?
        };

        // Compile and store each key-value pair
        for (i, (key_expr, val_expr)) in pairs.iter().enumerate() {
            let compiled_key = self.compile_expr(key_expr)?;
            let compiled_val = self.compile_expr(val_expr)?;

            // Calculate entry offset: entry_size * i
            let entry_offset = self
                .types
                .i64_type
                .const_int((i as u64) * entry_size, false);

            // Get pointer to key slot
            let entry_ptr = unsafe {
                self.builder
                    .build_gep(
                        self.context.i8_type(),
                        self.builder
                            .build_pointer_cast(entries_base, i8_ptr_type, "entries_i8")
                            .unwrap(),
                        &[entry_offset],
                        &format!("entry_{}", i),
                    )
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to compute entry pointer: {}", e))
                    })?
            };

            // Store key at entry start
            let key_ptr = self
                .builder
                .build_pointer_cast(
                    entry_ptr,
                    self.types.value_type.ptr_type(AddressSpace::default()),
                    &format!("key_ptr_{}", i),
                )
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to cast key pointer: {}", e))
                })?;
            self.builder
                .build_store(key_ptr, compiled_key)
                .map_err(|e| HaversError::CompileError(format!("Failed to store key: {}", e)))?;

            // Store value at entry start + 16 bytes
            let value_offset = self.types.i64_type.const_int(16, false);
            let val_ptr = unsafe {
                self.builder
                    .build_gep(
                        self.context.i8_type(),
                        entry_ptr,
                        &[value_offset],
                        &format!("val_gep_{}", i),
                    )
                    .map_err(|e| {
                        HaversError::CompileError(format!("Failed to compute value pointer: {}", e))
                    })?
            };
            let val_typed_ptr = self
                .builder
                .build_pointer_cast(
                    val_ptr,
                    self.types.value_type.ptr_type(AddressSpace::default()),
                    &format!("val_ptr_{}", i),
                )
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to cast value pointer: {}", e))
                })?;
            self.builder
                .build_store(val_typed_ptr, compiled_val)
                .map_err(|e| HaversError::CompileError(format!("Failed to store value: {}", e)))?;
        }

        // Return the dict as a tagged value
        self.make_dict(raw_ptr)
    }

    /// Compile an index expression: list[index], string[index], or dict[key]
    fn compile_index(
        &mut self,
        object: &Expr,
        index: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Fast path: if we know the object is a list and index is an int,
        // skip type checking and negative index handling
        let obj_type = self.infer_expr_type(object);
        let idx_type = self.infer_expr_type(index);

        if obj_type == VarType::List && idx_type == VarType::Int {
            return self.compile_list_index_fast(object, index);
        }

        let obj_val = self.compile_expr(object)?;
        let idx_val = self.compile_expr(index)?;

        // Extract the tag and data from the object
        let obj_tag = self.extract_tag(obj_val)?;
        let obj_data = self.extract_data(obj_val)?;

        // Create basic blocks for branching
        let function = self
            .current_function
            .ok_or_else(|| HaversError::CompileError("No current function".to_string()))?;
        let list_block = self.context.append_basic_block(function, "index_list");
        let check_dict_block = self.context.append_basic_block(function, "check_dict");
        let dict_block = self.context.append_basic_block(function, "index_dict");
        let string_block = self.context.append_basic_block(function, "index_string");
        let merge_block = self.context.append_basic_block(function, "index_merge");

        // Check if object is a list (tag == 5)
        let list_tag = self
            .types
            .i8_type
            .const_int(ValueTag::List.as_u8() as u64, false);
        let is_list = self
            .builder
            .build_int_compare(inkwell::IntPredicate::EQ, obj_tag, list_tag, "is_list")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare tags: {}", e)))?;

        self.builder
            .build_conditional_branch(is_list, list_block, check_dict_block)
            .map_err(|e| HaversError::CompileError(format!("Failed to build branch: {}", e)))?;

        // List indexing - use index as integer
        self.builder.position_at_end(list_block);
        let idx_data = self.extract_data(idx_val)?;
        let list_result = self.compile_list_index(obj_data, idx_data)?;
        let list_bb = self.builder.get_insert_block().unwrap();
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();

        // Check if object is a dict (tag == 6)
        self.builder.position_at_end(check_dict_block);
        let dict_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Dict.as_u8() as u64, false);
        let is_dict = self
            .builder
            .build_int_compare(inkwell::IntPredicate::EQ, obj_tag, dict_tag, "is_dict")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare tags: {}", e)))?;

        self.builder
            .build_conditional_branch(is_dict, dict_block, string_block)
            .unwrap();

        // Dict indexing - use key for lookup
        self.builder.position_at_end(dict_block);
        let dict_result = self.compile_dict_index(obj_data, idx_val)?;
        let dict_bb = self.builder.get_insert_block().unwrap();
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();

        // String indexing (return character as string) - use index as integer
        self.builder.position_at_end(string_block);
        let idx_data_str = self.extract_data(idx_val)?;
        let string_result = self.compile_string_index(obj_data, idx_data_str)?;
        let string_bb = self.builder.get_insert_block().unwrap();
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();

        // Merge
        self.builder.position_at_end(merge_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "index_result")
            .unwrap();
        phi.add_incoming(&[
            (&list_result, list_bb),
            (&dict_result, dict_bb),
            (&string_result, string_bb),
        ]);

        Ok(phi.as_basic_value())
    }

    /// Helper for list indexing
    fn compile_list_index(
        &self,
        list_data: IntValue<'ctx>,
        index: IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // List layout: [capacity: i64][length: i64][elements...]
        // Convert data to pointer
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let header_ptr = self
            .builder
            .build_int_to_ptr(list_data, i64_ptr_type, "list_ptr")
            .map_err(|e| {
                HaversError::CompileError(format!("Failed to convert to pointer: {}", e))
            })?;

        // Get length pointer (at offset 1)
        let len_ptr = unsafe {
            self.builder
                .build_gep(self.types.i64_type, header_ptr, &[self.types.i64_type.const_int(1, false)], "len_ptr")
                .map_err(|e| HaversError::CompileError(format!("Failed to get len ptr: {}", e)))?
        };

        // Load length
        let length = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .map_err(|e| HaversError::CompileError(format!("Failed to load length: {}", e)))?
            .into_int_value();

        // Handle negative indices: if index < 0, index = length + index
        let zero = self.types.i64_type.const_int(0, false);
        let is_negative = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, index, zero, "is_negative")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;

        let adjusted_index = self
            .builder
            .build_int_add(length, index, "adjusted")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;

        let final_index = self
            .builder
            .build_select(is_negative, adjusted_index, index, "final_index")
            .map_err(|e| HaversError::CompileError(format!("Failed to select: {}", e)))?
            .into_int_value();

        // Get pointer to elements array (at offset 2, after capacity and length)
        let value_ptr_type = self.types.value_type.ptr_type(AddressSpace::default());
        let elements_base = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(2, false)],
                    "elements_base",
                )
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to compute elements base: {}", e))
                })?
        };
        let elements_ptr = self
            .builder
            .build_pointer_cast(elements_base, value_ptr_type, "elements_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast pointer: {}", e)))?;

        // Get pointer to the indexed element
        let elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.value_type,
                    elements_ptr,
                    &[final_index],
                    "elem_ptr",
                )
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to compute element pointer: {}", e))
                })?
        };

        // Load and return the element
        let result = self
            .builder
            .build_load(self.types.value_type, elem_ptr, "elem_val")
            .map_err(|e| HaversError::CompileError(format!("Failed to load element: {}", e)))?;

        Ok(result)
    }

    /// Fast path for list indexing when types are known at compile time
    /// Skips type checking, negative index handling, and uses direct pointer arithmetic
    fn compile_list_index_fast(
        &mut self,
        object: &Expr,
        index: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Get list data pointer directly
        let obj_val = self.compile_expr(object)?;
        let list_data = self.extract_data(obj_val)?;

        // Get index as i64 directly (use shadow if available)
        let idx_i64 = if let Some(i) = self.compile_int_expr(index)? {
            i
        } else {
            let idx_val = self.compile_expr(index)?;
            self.extract_data(idx_val)?
        };

        // Convert data to pointer - list layout is [capacity: i64][length: i64][elem0: {i8, i64}][elem1: {i8, i64}]...
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i64_ptr_type, "list_ptr_fast")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert to pointer: {}", e)))?;

        // Skip past capacity and length (16 bytes = offset 2) to reach elements
        let value_ptr_type = self.types.value_type.ptr_type(AddressSpace::default());
        let elements_base = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    list_ptr,
                    &[self.types.i64_type.const_int(2, false)],
                    "elements_base_fast",
                )
                .map_err(|e| HaversError::CompileError(format!("Failed to compute elements base: {}", e)))?
        };
        let elements_ptr = self
            .builder
            .build_pointer_cast(elements_base, value_ptr_type, "elements_ptr_fast")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast pointer: {}", e)))?;

        // Get pointer to the indexed element
        let elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.value_type,
                    elements_ptr,
                    &[idx_i64],
                    "elem_ptr_fast",
                )
                .map_err(|e| HaversError::CompileError(format!("Failed to compute element pointer: {}", e)))?
        };

        // Load and return the element
        let result = self
            .builder
            .build_load(self.types.value_type, elem_ptr, "elem_val_fast")
            .map_err(|e| HaversError::CompileError(format!("Failed to load element: {}", e)))?;

        Ok(result)
    }

    /// Helper for dict indexing - searches for key and returns corresponding value
    /// Dict layout: [i64 count][entry0][entry1]... where entry = [{i8,i64} key][{i8,i64} val]
    fn compile_dict_index(
        &mut self,
        dict_data: IntValue<'ctx>,
        key_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Convert dict data to pointer
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let dict_ptr = self
            .builder
            .build_int_to_ptr(dict_data, i8_ptr_type, "dict_ptr")
            .map_err(|e| {
                HaversError::CompileError(format!("Failed to convert to pointer: {}", e))
            })?;

        // Get dict count
        let count_ptr = self
            .builder
            .build_pointer_cast(dict_ptr, i64_ptr_type, "count_ptr")
            .unwrap();
        let dict_count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "dict_count")
            .unwrap()
            .into_int_value();

        // Extract key tag and data for comparison
        let key_tag = self.extract_tag(key_val)?;
        let key_data = self.extract_data(key_val)?;

        // Allocate result pointer and found flag
        let result_ptr = self
            .builder
            .build_alloca(self.types.value_type, "result_ptr")
            .unwrap();
        self.builder
            .build_store(result_ptr, self.make_nil())
            .unwrap();

        // Loop through entries to find matching key
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let header_size = self.types.i64_type.const_int(16, false);
        let entry_size = self.types.i64_type.const_int(32, false);
        let value_offset_in_entry = self.types.i64_type.const_int(16, false);

        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self
            .context
            .append_basic_block(function, "dict_lookup_loop");
        let body_block = self
            .context
            .append_basic_block(function, "dict_lookup_body");
        let found_block = self.context.append_basic_block(function, "dict_found");
        let continue_block = self.context.append_basic_block(function, "dict_continue");
        let done_block = self
            .context
            .append_basic_block(function, "dict_lookup_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, dict_count, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, done_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        // Get entry key from dict
        let dict_entry_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, entry_size, "entry_mul")
                    .unwrap(),
                "entry_offset",
            )
            .unwrap();
        let dict_key_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    dict_ptr,
                    &[dict_entry_offset],
                    "dict_key_ptr",
                )
                .unwrap()
        };
        let key_value_ptr = self
            .builder
            .build_pointer_cast(
                dict_key_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "key_value_ptr",
            )
            .unwrap();
        let entry_key = self
            .builder
            .build_load(self.types.value_type, key_value_ptr, "entry_key")
            .unwrap();

        // Compare keys - check both tag and data match
        let entry_key_tag = self.extract_tag(entry_key)?;
        let entry_key_data = self.extract_data(entry_key)?;

        let tags_match = self
            .builder
            .build_int_compare(IntPredicate::EQ, key_tag, entry_key_tag, "tags_match")
            .unwrap();
        let data_match = self
            .builder
            .build_int_compare(IntPredicate::EQ, key_data, entry_key_data, "data_match")
            .unwrap();
        let keys_match = self
            .builder
            .build_and(tags_match, data_match, "keys_match")
            .unwrap();

        self.builder
            .build_conditional_branch(keys_match, found_block, continue_block)
            .unwrap();

        // Found - get the value
        self.builder.position_at_end(found_block);
        let dict_value_offset = self
            .builder
            .build_int_add(dict_entry_offset, value_offset_in_entry, "value_offset")
            .unwrap();
        let dict_value_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    dict_ptr,
                    &[dict_value_offset],
                    "dict_value_ptr",
                )
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                dict_value_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let found_val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "found_val")
            .unwrap();
        self.builder.build_store(result_ptr, found_val).unwrap();
        self.builder.build_unconditional_branch(done_block).unwrap();

        // Continue loop
        self.builder.position_at_end(continue_block);
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        // Done - return result (nil if not found)
        self.builder.position_at_end(done_block);
        let result = self
            .builder
            .build_load(self.types.value_type, result_ptr, "dict_result")
            .unwrap();
        Ok(result)
    }

    /// Compile an index set expression: list[index] = value
    fn compile_index_set(
        &mut self,
        object: &Expr,
        index: &Expr,
        value: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Fast path: if we know the object is a list and index is an int,
        // skip type checking and negative index handling
        let obj_type = self.infer_expr_type(object);
        let idx_type = self.infer_expr_type(index);

        if obj_type == VarType::List && idx_type == VarType::Int {
            return self.compile_list_index_set_fast(object, index, value);
        }

        // Compile the object, index, and value
        let obj_val = self.compile_expr(object)?;
        let idx_val = self.compile_expr(index)?;
        let new_val = self.compile_expr(value)?;

        // Extract the object's data (pointer to list)
        let obj_data = self.extract_data(obj_val)?;

        // Extract the index (assume it's an integer)
        let idx_data = self.extract_data(idx_val)?;

        // Convert list data to pointer - layout: [capacity][length][elements...]
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let header_ptr = self
            .builder
            .build_int_to_ptr(obj_data, i64_ptr_type, "list_header_ptr")
            .map_err(|e| {
                HaversError::CompileError(format!("Failed to convert to pointer: {}", e))
            })?;

        // Get length pointer at offset 1
        let len_ptr = unsafe {
            self.builder
                .build_gep(self.types.i64_type, header_ptr, &[self.types.i64_type.const_int(1, false)], "len_ptr")
                .map_err(|e| HaversError::CompileError(format!("Failed to get len ptr: {}", e)))?
        };

        // Load length for bounds checking and negative index handling
        let length = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .map_err(|e| HaversError::CompileError(format!("Failed to load length: {}", e)))?
            .into_int_value();

        // Handle negative indices: if index < 0, index = length + index
        let zero = self.types.i64_type.const_int(0, false);
        let is_negative = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, idx_data, zero, "is_negative")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;

        let adjusted_index = self
            .builder
            .build_int_add(length, idx_data, "adjusted")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;

        let final_index = self
            .builder
            .build_select(is_negative, adjusted_index, idx_data, "final_index")
            .map_err(|e| HaversError::CompileError(format!("Failed to select: {}", e)))?
            .into_int_value();

        // Get pointer to elements array (at offset 2, after capacity and length)
        let value_ptr_type = self.types.value_type.ptr_type(AddressSpace::default());
        let elements_base = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(2, false)],
                    "elements_base",
                )
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to compute elements base: {}", e))
                })?
        };
        let elements_ptr = self
            .builder
            .build_pointer_cast(elements_base, value_ptr_type, "elements_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast pointer: {}", e)))?;

        // Get pointer to the indexed element
        let elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.value_type,
                    elements_ptr,
                    &[final_index],
                    "elem_ptr",
                )
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to compute element pointer: {}", e))
                })?
        };

        // Store the new value at that location
        self.builder
            .build_store(elem_ptr, new_val)
            .map_err(|e| HaversError::CompileError(format!("Failed to store element: {}", e)))?;

        // Return the value that was set (for chained assignments)
        Ok(new_val)
    }

    /// Fast path for list index assignment when types are known at compile time
    fn compile_list_index_set_fast(
        &mut self,
        object: &Expr,
        index: &Expr,
        value: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Get list data pointer directly
        let obj_val = self.compile_expr(object)?;
        let list_data = self.extract_data(obj_val)?;

        // Get index as i64 directly (use shadow if available)
        let idx_i64 = if let Some(i) = self.compile_int_expr(index)? {
            i
        } else {
            let idx_val = self.compile_expr(index)?;
            self.extract_data(idx_val)?
        };

        // Compile the value to store
        let new_val = self.compile_expr(value)?;

        // Convert data to pointer - list layout is [capacity: i64][length: i64][elem0: {i8, i64}][elem1: {i8, i64}]...
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i64_ptr_type, "list_ptr_set_fast")
            .map_err(|e| HaversError::CompileError(format!("Failed to convert to pointer: {}", e)))?;

        // Skip past capacity and length (16 bytes = offset 2) to reach elements
        let value_ptr_type = self.types.value_type.ptr_type(AddressSpace::default());
        let elements_base = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    list_ptr,
                    &[self.types.i64_type.const_int(2, false)],
                    "elements_base_set_fast",
                )
                .map_err(|e| HaversError::CompileError(format!("Failed to compute elements base: {}", e)))?
        };
        let elements_ptr = self
            .builder
            .build_pointer_cast(elements_base, value_ptr_type, "elements_ptr_set_fast")
            .map_err(|e| HaversError::CompileError(format!("Failed to cast pointer: {}", e)))?;

        // Get pointer to the indexed element
        let elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.value_type,
                    elements_ptr,
                    &[idx_i64],
                    "elem_ptr_set_fast",
                )
                .map_err(|e| HaversError::CompileError(format!("Failed to compute element pointer: {}", e)))?
        };

        // Store the new value at that location
        self.builder
            .build_store(elem_ptr, new_val)
            .map_err(|e| HaversError::CompileError(format!("Failed to store element: {}", e)))?;

        // Return the value that was set
        Ok(new_val)
    }

    /// Helper for string indexing (return single character as string)
    fn compile_string_index(
        &self,
        str_data: IntValue<'ctx>,
        index: IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Convert data to string pointer
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let str_ptr = self
            .builder
            .build_int_to_ptr(str_data, i8_ptr_type, "str_ptr")
            .map_err(|e| {
                HaversError::CompileError(format!("Failed to convert to pointer: {}", e))
            })?;

        // Get string length
        let length = self
            .builder
            .build_call(self.libc.strlen, &[str_ptr.into()], "str_len")
            .map_err(|e| HaversError::CompileError(format!("Failed to call strlen: {}", e)))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| HaversError::CompileError("strlen returned void".to_string()))?
            .into_int_value();

        // Handle negative indices
        let zero = self.types.i64_type.const_int(0, false);
        let is_negative = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SLT, index, zero, "is_negative")
            .map_err(|e| HaversError::CompileError(format!("Failed to compare: {}", e)))?;

        let adjusted_index = self
            .builder
            .build_int_add(length, index, "adjusted")
            .map_err(|e| HaversError::CompileError(format!("Failed to add: {}", e)))?;

        let final_index = self
            .builder
            .build_select(is_negative, adjusted_index, index, "final_index")
            .map_err(|e| HaversError::CompileError(format!("Failed to select: {}", e)))?
            .into_int_value();

        // Allocate 2 bytes for the new string (char + null terminator)
        let two = self.types.i64_type.const_int(2, false);
        let new_str = self
            .builder
            .build_call(self.libc.malloc, &[two.into()], "char_str")
            .map_err(|e| HaversError::CompileError(format!("Failed to call malloc: {}", e)))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| HaversError::CompileError("malloc returned void".to_string()))?
            .into_pointer_value();

        // Get pointer to the character
        let char_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), str_ptr, &[final_index], "char_ptr")
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to compute char pointer: {}", e))
                })?
        };

        // Load the character
        let char_val = self
            .builder
            .build_load(self.context.i8_type(), char_ptr, "char_val")
            .map_err(|e| HaversError::CompileError(format!("Failed to load char: {}", e)))?;

        // Store the character in new string
        self.builder
            .build_store(new_str, char_val)
            .map_err(|e| HaversError::CompileError(format!("Failed to store char: {}", e)))?;

        // Store null terminator
        let null_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_str,
                    &[self.types.i64_type.const_int(1, false)],
                    "null_ptr",
                )
                .map_err(|e| {
                    HaversError::CompileError(format!("Failed to compute null pointer: {}", e))
                })?
        };
        let null_byte = self.context.i8_type().const_int(0, false);
        self.builder
            .build_store(null_ptr, null_byte)
            .map_err(|e| HaversError::CompileError(format!("Failed to store null: {}", e)))?;

        // Return as string value
        self.make_string(new_str)
    }

    // ===== Phase 5: Timing functions =====

    /// noo() - Returns current time in milliseconds since epoch (CLOCK_REALTIME)
    fn inline_noo(&mut self) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Create struct timespec { i64 tv_sec; i64 tv_nsec; } on stack
        let timespec_type = self.context.struct_type(
            &[self.types.i64_type.into(), self.types.i64_type.into()],
            false,
        );

        let timespec_ptr = self
            .builder
            .build_alloca(timespec_type, "timespec")
            .unwrap();

        // CLOCK_REALTIME = 0
        let clock_id = self.context.i32_type().const_int(0, false);

        // Cast timespec_ptr to i8* for clock_gettime
        let timespec_i8_ptr = self
            .builder
            .build_pointer_cast(
                timespec_ptr,
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "timespec_i8_ptr",
            )
            .unwrap();

        // Call clock_gettime(CLOCK_REALTIME, &ts)
        self.builder
            .build_call(
                self.libc.clock_gettime,
                &[clock_id.into(), timespec_i8_ptr.into()],
                "clock_result",
            )
            .unwrap();

        // Read tv_sec
        let sec_ptr = self
            .builder
            .build_struct_gep(timespec_type, timespec_ptr, 0, "sec_ptr")
            .unwrap();
        let tv_sec = self
            .builder
            .build_load(self.types.i64_type, sec_ptr, "tv_sec")
            .unwrap()
            .into_int_value();

        // Read tv_nsec
        let nsec_ptr = self
            .builder
            .build_struct_gep(timespec_type, timespec_ptr, 1, "nsec_ptr")
            .unwrap();
        let tv_nsec = self
            .builder
            .build_load(self.types.i64_type, nsec_ptr, "tv_nsec")
            .unwrap()
            .into_int_value();

        // Convert to milliseconds: (tv_sec * 1000) + (tv_nsec / 1_000_000)
        let thousand = self.types.i64_type.const_int(1000, false);
        let million = self.types.i64_type.const_int(1_000_000, false);

        let sec_ms = self
            .builder
            .build_int_mul(tv_sec, thousand, "sec_ms")
            .unwrap();
        let nsec_ms = self
            .builder
            .build_int_signed_div(tv_nsec, million, "nsec_ms")
            .unwrap();
        let total_ms = self
            .builder
            .build_int_add(sec_ms, nsec_ms, "total_ms")
            .unwrap();

        self.make_int(total_ms)
    }

    /// tick() - Returns high-resolution time in nanoseconds (CLOCK_MONOTONIC)
    fn inline_tick(&mut self) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Create struct timespec { i64 tv_sec; i64 tv_nsec; } on stack
        let timespec_type = self.context.struct_type(
            &[self.types.i64_type.into(), self.types.i64_type.into()],
            false,
        );

        let timespec_ptr = self
            .builder
            .build_alloca(timespec_type, "timespec")
            .unwrap();

        // CLOCK_MONOTONIC = 1
        let clock_id = self.context.i32_type().const_int(1, false);

        // Cast timespec_ptr to i8* for clock_gettime
        let timespec_i8_ptr = self
            .builder
            .build_pointer_cast(
                timespec_ptr,
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "timespec_i8_ptr",
            )
            .unwrap();

        // Call clock_gettime(CLOCK_MONOTONIC, &ts)
        self.builder
            .build_call(
                self.libc.clock_gettime,
                &[clock_id.into(), timespec_i8_ptr.into()],
                "clock_result",
            )
            .unwrap();

        // Read tv_sec
        let sec_ptr = self
            .builder
            .build_struct_gep(timespec_type, timespec_ptr, 0, "sec_ptr")
            .unwrap();
        let tv_sec = self
            .builder
            .build_load(self.types.i64_type, sec_ptr, "tv_sec")
            .unwrap()
            .into_int_value();

        // Read tv_nsec
        let nsec_ptr = self
            .builder
            .build_struct_gep(timespec_type, timespec_ptr, 1, "nsec_ptr")
            .unwrap();
        let tv_nsec = self
            .builder
            .build_load(self.types.i64_type, nsec_ptr, "tv_nsec")
            .unwrap()
            .into_int_value();

        // Convert to nanoseconds: (tv_sec * 1_000_000_000) + tv_nsec
        let billion = self.types.i64_type.const_int(1_000_000_000, false);
        let sec_ns = self
            .builder
            .build_int_mul(tv_sec, billion, "sec_ns")
            .unwrap();
        let total_ns = self
            .builder
            .build_int_add(sec_ns, tv_nsec, "total_ns")
            .unwrap();

        self.make_int(total_ns)
    }

    /// bide(ms) - Sleep for specified milliseconds
    fn inline_bide(
        &mut self,
        ms_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Extract the integer value from the tagged value
        let ms_struct = ms_val.into_struct_value();
        let ms = self
            .builder
            .build_extract_value(ms_struct, 1, "ms_data")
            .unwrap()
            .into_int_value();

        // Create struct timespec { i64 tv_sec; i64 tv_nsec; } on stack
        let timespec_type = self.context.struct_type(
            &[self.types.i64_type.into(), self.types.i64_type.into()],
            false,
        );

        let req_ptr = self.builder.build_alloca(timespec_type, "req").unwrap();

        // Convert ms to seconds and nanoseconds
        // tv_sec = ms / 1000
        // tv_nsec = (ms % 1000) * 1_000_000
        let thousand = self.types.i64_type.const_int(1000, false);
        let million = self.types.i64_type.const_int(1_000_000, false);

        let tv_sec = self
            .builder
            .build_int_signed_div(ms, thousand, "tv_sec")
            .unwrap();
        let ms_remainder = self
            .builder
            .build_int_signed_rem(ms, thousand, "ms_remainder")
            .unwrap();
        let tv_nsec = self
            .builder
            .build_int_mul(ms_remainder, million, "tv_nsec")
            .unwrap();

        // Store tv_sec and tv_nsec
        let sec_ptr = self
            .builder
            .build_struct_gep(timespec_type, req_ptr, 0, "sec_ptr")
            .unwrap();
        self.builder.build_store(sec_ptr, tv_sec).unwrap();

        let nsec_ptr = self
            .builder
            .build_struct_gep(timespec_type, req_ptr, 1, "nsec_ptr")
            .unwrap();
        self.builder.build_store(nsec_ptr, tv_nsec).unwrap();

        // Cast req_ptr to i8* for nanosleep
        let req_i8_ptr = self
            .builder
            .build_pointer_cast(
                req_ptr,
                self.context.i8_type().ptr_type(AddressSpace::default()),
                "req_i8_ptr",
            )
            .unwrap();

        // Pass null for the second argument (remaining time)
        let null = self
            .context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .const_null();

        // Call nanosleep(&req, NULL)
        self.builder
            .build_call(
                self.libc.nanosleep,
                &[req_i8_ptr.into(), null.into()],
                "sleep_result",
            )
            .unwrap();

        // Return nil
        Ok(self.make_nil())
    }

    // ===== Phase 7: I/O functions =====

    /// speir(prompt?) - Read a line from stdin, optionally printing a prompt first
    fn inline_speir(
        &mut self,
        prompt: Option<BasicValueEnum<'ctx>>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // If there's a prompt, print it first (without newline)
        if let Some(prompt_val) = prompt {
            // Extract string pointer from prompt
            let prompt_struct = prompt_val.into_struct_value();
            let prompt_data = self
                .builder
                .build_extract_value(prompt_struct, 1, "prompt_data")
                .unwrap()
                .into_int_value();
            let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
            let prompt_ptr = self
                .builder
                .build_int_to_ptr(prompt_data, i8_ptr_type, "prompt_ptr")
                .unwrap();

            // Print the prompt using printf with %s format (no newline)
            let fmt_ptr = self.get_string_ptr(self.fmt_string);
            self.builder
                .build_call(
                    self.libc.printf,
                    &[fmt_ptr.into(), prompt_ptr.into()],
                    "print_prompt",
                )
                .unwrap();
        }

        // Allocate buffer for input (1024 bytes should be enough for most input)
        let buf_size = self.types.i64_type.const_int(1024, false);
        let buffer = self
            .builder
            .build_call(self.libc.malloc, &[buf_size.into()], "input_buffer")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Get stdin - declare it as an external global
        let stdin_global = self.module.add_global(
            self.context.i8_type().ptr_type(AddressSpace::default()),
            Some(AddressSpace::default()),
            "stdin",
        );
        stdin_global.set_linkage(Linkage::External);

        // Load stdin pointer
        let stdin_ptr = self
            .builder
            .build_load(
                self.context.i8_type().ptr_type(AddressSpace::default()),
                stdin_global.as_pointer_value(),
                "stdin_val",
            )
            .unwrap()
            .into_pointer_value();

        // Call fgets(buffer, 1024, stdin)
        let size_i32 = self.context.i32_type().const_int(1024, false);
        let result = self
            .builder
            .build_call(
                self.libc.fgets,
                &[buffer.into(), size_i32.into(), stdin_ptr.into()],
                "fgets_result",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Check if fgets returned NULL (EOF or error)
        let null = self
            .context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .const_null();
        let is_null = self
            .builder
            .build_int_compare(IntPredicate::EQ, result, null, "is_null")
            .unwrap();

        let function = self.current_function.unwrap();
        let eof_block = self.context.append_basic_block(function, "speir_eof");
        let ok_block = self.context.append_basic_block(function, "speir_ok");
        let done_block = self.context.append_basic_block(function, "speir_done");

        self.builder
            .build_conditional_branch(is_null, eof_block, ok_block)
            .unwrap();

        // EOF case - return empty string
        self.builder.position_at_end(eof_block);
        let empty_str = self
            .builder
            .build_global_string_ptr("", "empty_str")
            .unwrap();
        let eof_result = self.make_string(empty_str.as_pointer_value())?;
        self.builder.build_unconditional_branch(done_block).unwrap();
        let eof_block_end = self.builder.get_insert_block().unwrap();

        // OK case - strip the trailing newline if present
        self.builder.position_at_end(ok_block);

        // Get string length
        let len = self
            .builder
            .build_call(self.libc.strlen, &[buffer.into()], "input_len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        // Check if last char is newline
        let one = self.types.i64_type.const_int(1, false);
        let last_idx = self.builder.build_int_sub(len, one, "last_idx").unwrap();
        let last_char_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), buffer, &[last_idx], "last_char_ptr")
                .unwrap()
        };
        let last_char = self
            .builder
            .build_load(self.context.i8_type(), last_char_ptr, "last_char")
            .unwrap()
            .into_int_value();
        let newline = self.context.i8_type().const_int(10, false); // '\n'
        let is_newline = self
            .builder
            .build_int_compare(IntPredicate::EQ, last_char, newline, "is_newline")
            .unwrap();

        let strip_block = self.context.append_basic_block(function, "strip_newline");
        let no_strip_block = self.context.append_basic_block(function, "no_strip");

        self.builder
            .build_conditional_branch(is_newline, strip_block, no_strip_block)
            .unwrap();

        // Strip newline
        self.builder.position_at_end(strip_block);
        let null_byte = self.context.i8_type().const_int(0, false);
        self.builder.build_store(last_char_ptr, null_byte).unwrap();
        self.builder
            .build_unconditional_branch(no_strip_block)
            .unwrap();

        // No strip (or after strip)
        self.builder.position_at_end(no_strip_block);
        let ok_result = self.make_string(buffer)?;
        self.builder.build_unconditional_branch(done_block).unwrap();
        let ok_block_end = self.builder.get_insert_block().unwrap();

        // Done - use phi to select result
        self.builder.position_at_end(done_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "speir_result")
            .unwrap();
        phi.add_incoming(&[(&eof_result, eof_block_end), (&ok_result, ok_block_end)]);

        Ok(phi.as_basic_value())
    }

    // ===== Extra: String operations =====

    /// split(str, delimiter) - Split string into list of strings
    fn inline_split(
        &mut self,
        str_val: BasicValueEnum<'ctx>,
        delim_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Extract string pointers
        let str_struct = str_val.into_struct_value();
        let str_data = self
            .builder
            .build_extract_value(str_struct, 1, "str_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let str_ptr = self
            .builder
            .build_int_to_ptr(str_data, i8_ptr_type, "str_ptr")
            .unwrap();

        let delim_struct = delim_val.into_struct_value();
        let delim_data = self
            .builder
            .build_extract_value(delim_struct, 1, "delim_data")
            .unwrap()
            .into_int_value();
        let delim_ptr = self
            .builder
            .build_int_to_ptr(delim_data, i8_ptr_type, "delim_ptr")
            .unwrap();

        // Get delimiter length
        let delim_len = self
            .builder
            .build_call(self.libc.strlen, &[delim_ptr.into()], "delim_len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        // Get string length
        let str_len = self
            .builder
            .build_call(self.libc.strlen, &[str_ptr.into()], "str_len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        // Handle empty delimiter - return list with single element (the whole string)
        // This prevents infinite loop when delimiter is ""
        let zero = self.types.i64_type.const_int(0, false);
        let delim_is_empty = self
            .builder
            .build_int_compare(IntPredicate::EQ, delim_len, zero, "delim_empty")
            .unwrap();

        let empty_delim_block = self.context.append_basic_block(function, "empty_delim");
        let normal_split_block = self.context.append_basic_block(function, "normal_split");
        let merge_block = self.context.append_basic_block(function, "split_merge");

        self.builder
            .build_conditional_branch(delim_is_empty, empty_delim_block, normal_split_block)
            .unwrap();

        // Empty delimiter case: return list containing the original string
        self.builder.position_at_end(empty_delim_block);
        let one_elem_list = self.allocate_list(self.types.i64_type.const_int(1, false))?;
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let one_len_ptr = self
            .builder
            .build_pointer_cast(one_elem_list, i64_ptr_type, "one_len_ptr")
            .unwrap();
        self.builder
            .build_store(one_len_ptr, self.types.i64_type.const_int(1, false))
            .unwrap();
        // Store original string as element 0
        let header_size_const = self.types.i64_type.const_int(16, false);
        let elem_ptr_empty = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    one_elem_list,
                    &[header_size_const],
                    "elem_ptr_empty",
                )
                .unwrap()
        };
        let value_ptr_empty = self
            .builder
            .build_pointer_cast(
                elem_ptr_empty,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr_empty",
            )
            .unwrap();
        self.builder.build_store(value_ptr_empty, str_val).unwrap();
        let empty_result = self.make_list(one_elem_list)?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let empty_delim_end = self.builder.get_insert_block().unwrap();

        // Normal split case
        self.builder.position_at_end(normal_split_block);

        // Allocate list with space for up to 100 elements initially
        // List format: [i64 length][value elem0][value elem1]...
        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let max_elems = self.types.i64_type.const_int(100, false);
        let initial_size = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(max_elems, elem_size, "elems_size")
                    .unwrap(),
                "initial_size",
            )
            .unwrap();

        let list_ptr = self
            .builder
            .build_call(self.libc.malloc, &[initial_size.into()], "list_ptr")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Initialize length to 0
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let len_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "len_ptr")
            .unwrap();
        self.builder
            .build_store(len_ptr, self.types.i64_type.const_int(0, false))
            .unwrap();

        // Allocate counters on stack
        let pos_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "pos")
            .unwrap();
        let count_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "count")
            .unwrap();
        self.builder
            .build_store(pos_ptr, self.types.i64_type.const_int(0, false))
            .unwrap();
        self.builder
            .build_store(count_ptr, self.types.i64_type.const_int(0, false))
            .unwrap();

        // Store list_ptr in an alloca so we can read it in the loop
        let list_ptr_alloca = self
            .builder
            .build_alloca(i8_ptr_type, "list_ptr_alloca")
            .unwrap();
        self.builder.build_store(list_ptr_alloca, list_ptr).unwrap();

        // Loop to find delimiters and split
        let loop_block = self.context.append_basic_block(function, "split_loop");
        let found_block = self.context.append_basic_block(function, "split_found");
        let not_found_block = self.context.append_basic_block(function, "split_not_found");
        let add_token_block = self.context.append_basic_block(function, "add_token");
        let done_block = self.context.append_basic_block(function, "split_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        // Get current position
        let pos = self
            .builder
            .build_load(self.types.i64_type, pos_ptr, "pos_val")
            .unwrap()
            .into_int_value();

        // Check if we've reached end of string
        let at_end = self
            .builder
            .build_int_compare(IntPredicate::UGE, pos, str_len, "at_end")
            .unwrap();
        self.builder
            .build_conditional_branch(at_end, done_block, found_block)
            .unwrap();

        // Search for delimiter starting at current position
        self.builder.position_at_end(found_block);
        let search_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), str_ptr, &[pos], "search_ptr")
                .unwrap()
        };
        let found_ptr = self
            .builder
            .build_call(
                self.libc.strstr,
                &[search_ptr.into(), delim_ptr.into()],
                "found_ptr",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let null = i8_ptr_type.const_null();
        let is_found = self
            .builder
            .build_int_compare(IntPredicate::NE, found_ptr, null, "is_found")
            .unwrap();
        self.builder
            .build_conditional_branch(is_found, add_token_block, not_found_block)
            .unwrap();

        // Found delimiter - add token from pos to found_ptr
        self.builder.position_at_end(add_token_block);
        let found_as_int = self
            .builder
            .build_ptr_to_int(found_ptr, self.types.i64_type, "found_int")
            .unwrap();
        let search_as_int = self
            .builder
            .build_ptr_to_int(search_ptr, self.types.i64_type, "search_int")
            .unwrap();
        let token_len = self
            .builder
            .build_int_sub(found_as_int, search_as_int, "token_len")
            .unwrap();

        // Allocate and copy token
        let token_alloc_size = self
            .builder
            .build_int_add(
                token_len,
                self.types.i64_type.const_int(1, false),
                "alloc_size",
            )
            .unwrap();
        let token_ptr = self
            .builder
            .build_call(self.libc.malloc, &[token_alloc_size.into()], "token_ptr")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Copy token bytes
        self.builder
            .build_call(
                self.libc.memcpy,
                &[token_ptr.into(), search_ptr.into(), token_len.into()],
                "copy_token",
            )
            .unwrap();

        // Null-terminate
        let token_end = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), token_ptr, &[token_len], "token_end")
                .unwrap()
        };
        self.builder
            .build_store(token_end, self.context.i8_type().const_int(0, false))
            .unwrap();

        // Create string value and add to list
        let token_value = self.make_string(token_ptr)?;

        // Get current count and list pointer
        let count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "count_val")
            .unwrap()
            .into_int_value();
        let current_list_ptr = self
            .builder
            .build_load(i8_ptr_type, list_ptr_alloca, "current_list")
            .unwrap()
            .into_pointer_value();

        // Calculate element offset: 8 + count * 16
        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(count, elem_size, "elem_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    current_list_ptr,
                    &[elem_offset],
                    "elem_ptr",
                )
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        self.builder.build_store(value_ptr, token_value).unwrap();

        // Increment count
        let new_count = self
            .builder
            .build_int_add(count, self.types.i64_type.const_int(1, false), "new_count")
            .unwrap();
        self.builder.build_store(count_ptr, new_count).unwrap();

        // Update position to after delimiter
        let new_pos = self
            .builder
            .build_int_add(pos, token_len, "after_token")
            .unwrap();
        let new_pos = self
            .builder
            .build_int_add(new_pos, delim_len, "after_delim")
            .unwrap();
        self.builder.build_store(pos_ptr, new_pos).unwrap();

        self.builder.build_unconditional_branch(loop_block).unwrap();

        // Not found - add rest of string as final token
        self.builder.position_at_end(not_found_block);
        let rest_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), str_ptr, &[pos], "rest_ptr")
                .unwrap()
        };

        // Duplicate the rest of the string
        let rest_copy = self
            .builder
            .build_call(self.libc.strdup, &[rest_ptr.into()], "rest_copy")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let rest_value = self.make_string(rest_copy)?;

        // Get current count and list pointer for final add
        let final_count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "final_count")
            .unwrap()
            .into_int_value();
        let final_list_ptr = self
            .builder
            .build_load(i8_ptr_type, list_ptr_alloca, "final_list")
            .unwrap()
            .into_pointer_value();

        // Add rest to list
        let final_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(final_count, elem_size, "final_mul")
                    .unwrap(),
                "final_offset",
            )
            .unwrap();
        let final_elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    final_list_ptr,
                    &[final_offset],
                    "final_elem_ptr",
                )
                .unwrap()
        };
        let final_value_ptr = self
            .builder
            .build_pointer_cast(
                final_elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "final_value_ptr",
            )
            .unwrap();
        self.builder
            .build_store(final_value_ptr, rest_value)
            .unwrap();

        // Update count
        let total_count = self
            .builder
            .build_int_add(
                final_count,
                self.types.i64_type.const_int(1, false),
                "total_count",
            )
            .unwrap();
        self.builder.build_store(count_ptr, total_count).unwrap();

        self.builder.build_unconditional_branch(done_block).unwrap();

        // Done - set length and return list
        self.builder.position_at_end(done_block);
        let done_list_ptr = self
            .builder
            .build_load(i8_ptr_type, list_ptr_alloca, "done_list")
            .unwrap()
            .into_pointer_value();
        let done_count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "done_count")
            .unwrap()
            .into_int_value();

        // Store final length
        let done_len_ptr = self
            .builder
            .build_pointer_cast(done_list_ptr, i64_ptr_type, "done_len_ptr")
            .unwrap();
        self.builder.build_store(done_len_ptr, done_count).unwrap();

        // Create list value for normal case
        let normal_result = self.make_list(done_list_ptr)?;
        self.builder
            .build_unconditional_branch(merge_block)
            .unwrap();
        let normal_split_end = self.builder.get_insert_block().unwrap();

        // Merge block - use phi to select result
        self.builder.position_at_end(merge_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "split_result")
            .unwrap();
        phi.add_incoming(&[
            (&empty_result, empty_delim_end),
            (&normal_result, normal_split_end),
        ]);
        Ok(phi.as_basic_value())
    }

    /// join(list, delimiter) - Join list elements with delimiter
    fn inline_join(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        delim_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Extract list pointer
        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        // Extract delimiter string
        let delim_struct = delim_val.into_struct_value();
        let delim_data = self
            .builder
            .build_extract_value(delim_struct, 1, "delim_data")
            .unwrap()
            .into_int_value();
        let delim_ptr = self
            .builder
            .build_int_to_ptr(delim_data, i8_ptr_type, "delim_ptr")
            .unwrap();

        // Get delimiter length
        let delim_len = self
            .builder
            .build_call(self.libc.strlen, &[delim_ptr.into()], "delim_len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        // Get list length from offset 1 (after capacity)
        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        // Check for empty list
        let zero = self.types.i64_type.const_int(0, false);
        let is_empty = self
            .builder
            .build_int_compare(IntPredicate::EQ, list_len, zero, "is_empty")
            .unwrap();

        let empty_block = self.context.append_basic_block(function, "join_empty");
        let calc_block = self.context.append_basic_block(function, "join_calc");
        let done_block = self.context.append_basic_block(function, "join_done");

        self.builder
            .build_conditional_branch(is_empty, empty_block, calc_block)
            .unwrap();

        // Empty list - return empty string
        self.builder.position_at_end(empty_block);
        let empty_str = self
            .builder
            .build_global_string_ptr("", "empty_str")
            .unwrap();
        let empty_result = self.make_string(empty_str.as_pointer_value())?;
        self.builder.build_unconditional_branch(done_block).unwrap();
        let empty_block_end = self.builder.get_insert_block().unwrap();

        // Calculate total size needed
        self.builder.position_at_end(calc_block);

        // First pass: calculate total length
        let total_len_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "total_len")
            .unwrap();
        self.builder.build_store(total_len_ptr, zero).unwrap();

        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let calc_loop = self.context.append_basic_block(function, "calc_loop");
        let calc_body = self.context.append_basic_block(function, "calc_body");
        let calc_done = self.context.append_basic_block(function, "calc_done");

        self.builder.build_unconditional_branch(calc_loop).unwrap();
        self.builder.position_at_end(calc_loop);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let calc_done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, list_len, "calc_done")
            .unwrap();
        self.builder
            .build_conditional_branch(calc_done_cond, calc_done, calc_body)
            .unwrap();

        self.builder.position_at_end(calc_body);

        // Get element at index
        let elem_size = self.types.i64_type.const_int(16, false);
        let header_size = self.types.i64_type.const_int(16, false);
        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "elem_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), list_ptr, &[elem_offset], "elem_ptr")
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let elem_value = self
            .builder
            .build_load(self.types.value_type, value_ptr, "elem_value")
            .unwrap();

        // Get string pointer from element (assuming all elements are strings)
        let elem_data = self
            .builder
            .build_extract_value(elem_value.into_struct_value(), 1, "elem_data")
            .unwrap()
            .into_int_value();
        let elem_str_ptr = self
            .builder
            .build_int_to_ptr(elem_data, i8_ptr_type, "elem_str_ptr")
            .unwrap();

        // Get length of this element
        let elem_len = self
            .builder
            .build_call(self.libc.strlen, &[elem_str_ptr.into()], "elem_len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();

        // Add to total
        let current_total = self
            .builder
            .build_load(self.types.i64_type, total_len_ptr, "current_total")
            .unwrap()
            .into_int_value();
        let new_total = self
            .builder
            .build_int_add(current_total, elem_len, "new_total")
            .unwrap();

        // Add delimiter length if not last element
        let one = self.types.i64_type.const_int(1, false);
        let list_len_minus_one = self
            .builder
            .build_int_sub(list_len, one, "len_minus_one")
            .unwrap();
        let is_last = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, list_len_minus_one, "is_last")
            .unwrap();
        let delim_add = self
            .builder
            .build_select(is_last, zero, delim_len, "delim_add")
            .unwrap()
            .into_int_value();
        let with_delim = self
            .builder
            .build_int_add(new_total, delim_add, "with_delim")
            .unwrap();
        self.builder.build_store(total_len_ptr, with_delim).unwrap();

        // Increment index
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(calc_loop).unwrap();

        // Allocate result buffer
        self.builder.position_at_end(calc_done);
        let final_total = self
            .builder
            .build_load(self.types.i64_type, total_len_ptr, "final_total")
            .unwrap()
            .into_int_value();
        let alloc_size = self
            .builder
            .build_int_add(final_total, one, "alloc_size")
            .unwrap();

        let result_buf = self
            .builder
            .build_call(self.libc.malloc, &[alloc_size.into()], "result_buf")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Initialize buffer to empty string
        self.builder
            .build_store(result_buf, self.context.i8_type().const_int(0, false))
            .unwrap();

        // Second pass: concatenate strings
        self.builder.build_store(idx_ptr, zero).unwrap();

        let concat_loop = self.context.append_basic_block(function, "concat_loop");
        let concat_body = self.context.append_basic_block(function, "concat_body");
        let concat_done = self.context.append_basic_block(function, "concat_done");

        self.builder
            .build_unconditional_branch(concat_loop)
            .unwrap();
        self.builder.position_at_end(concat_loop);

        let idx2 = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx2_val")
            .unwrap()
            .into_int_value();
        let concat_done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx2, list_len, "concat_done")
            .unwrap();
        self.builder
            .build_conditional_branch(concat_done_cond, concat_done, concat_body)
            .unwrap();

        self.builder.position_at_end(concat_body);

        // Get element at index
        let elem_offset2 = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx2, elem_size, "elem_mul2")
                    .unwrap(),
                "elem_offset2",
            )
            .unwrap();
        let elem_ptr2 = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    list_ptr,
                    &[elem_offset2],
                    "elem_ptr2",
                )
                .unwrap()
        };
        let value_ptr2 = self
            .builder
            .build_pointer_cast(
                elem_ptr2,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr2",
            )
            .unwrap();
        let elem_value2 = self
            .builder
            .build_load(self.types.value_type, value_ptr2, "elem_value2")
            .unwrap();

        let elem_data2 = self
            .builder
            .build_extract_value(elem_value2.into_struct_value(), 1, "elem_data2")
            .unwrap()
            .into_int_value();
        let elem_str_ptr2 = self
            .builder
            .build_int_to_ptr(elem_data2, i8_ptr_type, "elem_str_ptr2")
            .unwrap();

        // Concatenate element
        self.builder
            .build_call(
                self.libc.strcat,
                &[result_buf.into(), elem_str_ptr2.into()],
                "cat_elem",
            )
            .unwrap();

        // Add delimiter if not last
        let one2 = self.types.i64_type.const_int(1, false);
        let list_len_minus_one2 = self
            .builder
            .build_int_sub(list_len, one2, "len_minus_one2")
            .unwrap();
        let is_last2 = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx2, list_len_minus_one2, "is_last2")
            .unwrap();

        let add_delim_block = self.context.append_basic_block(function, "add_delim");
        let skip_delim_block = self.context.append_basic_block(function, "skip_delim");

        self.builder
            .build_conditional_branch(is_last2, skip_delim_block, add_delim_block)
            .unwrap();

        self.builder.position_at_end(add_delim_block);
        self.builder
            .build_call(
                self.libc.strcat,
                &[result_buf.into(), delim_ptr.into()],
                "cat_delim",
            )
            .unwrap();
        self.builder
            .build_unconditional_branch(skip_delim_block)
            .unwrap();

        self.builder.position_at_end(skip_delim_block);
        let next_idx2 = self.builder.build_int_add(idx2, one, "next_idx2").unwrap();
        self.builder.build_store(idx_ptr, next_idx2).unwrap();
        self.builder
            .build_unconditional_branch(concat_loop)
            .unwrap();

        // Done concatenating
        self.builder.position_at_end(concat_done);
        let concat_result = self.make_string(result_buf)?;
        self.builder.build_unconditional_branch(done_block).unwrap();
        let concat_block_end = self.builder.get_insert_block().unwrap();

        // Final done block with phi
        self.builder.position_at_end(done_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "join_result")
            .unwrap();
        phi.add_incoming(&[
            (&empty_result, empty_block_end),
            (&concat_result, concat_block_end),
        ]);

        Ok(phi.as_basic_value())
    }

    /// sort(list) - Return a sorted copy of the list (bubble sort for simplicity)
    fn inline_sort(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // First, create a copy of the list
        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        // Get list length
        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        // Calculate total size: 8 + len * 16
        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let elems_total = self
            .builder
            .build_int_mul(list_len, elem_size, "elems_total")
            .unwrap();
        let total_size = self
            .builder
            .build_int_add(header_size, elems_total, "total_size")
            .unwrap();

        // Allocate new list
        let new_list_ptr = self
            .builder
            .build_call(self.libc.malloc, &[total_size.into()], "new_list")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Copy entire list
        self.builder
            .build_call(
                self.libc.memcpy,
                &[new_list_ptr.into(), list_ptr.into(), total_size.into()],
                "copy_list",
            )
            .unwrap();

        // Bubble sort: outer loop i from 0 to len-1
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);

        let i_ptr = self.builder.build_alloca(self.types.i64_type, "i").unwrap();
        self.builder.build_store(i_ptr, zero).unwrap();

        let outer_loop = self.context.append_basic_block(function, "sort_outer");
        let outer_body = self.context.append_basic_block(function, "sort_outer_body");
        let inner_loop = self.context.append_basic_block(function, "sort_inner");
        let inner_body = self.context.append_basic_block(function, "sort_inner_body");
        let inner_done = self.context.append_basic_block(function, "sort_inner_done");
        let outer_done = self.context.append_basic_block(function, "sort_outer_done");

        self.builder.build_unconditional_branch(outer_loop).unwrap();
        self.builder.position_at_end(outer_loop);

        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i_val")
            .unwrap()
            .into_int_value();
        let len_minus_one = self
            .builder
            .build_int_sub(list_len, one, "len_minus_one")
            .unwrap();
        let outer_done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, i, len_minus_one, "outer_done")
            .unwrap();
        self.builder
            .build_conditional_branch(outer_done_cond, outer_done, outer_body)
            .unwrap();

        self.builder.position_at_end(outer_body);
        let j_ptr = self.builder.build_alloca(self.types.i64_type, "j").unwrap();
        self.builder.build_store(j_ptr, zero).unwrap();

        self.builder.build_unconditional_branch(inner_loop).unwrap();
        self.builder.position_at_end(inner_loop);

        let j = self
            .builder
            .build_load(self.types.i64_type, j_ptr, "j_val")
            .unwrap()
            .into_int_value();
        let bound = self
            .builder
            .build_int_sub(len_minus_one, i, "bound")
            .unwrap();
        let inner_done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, j, bound, "inner_done")
            .unwrap();
        self.builder
            .build_conditional_branch(inner_done_cond, inner_done, inner_body)
            .unwrap();

        self.builder.position_at_end(inner_body);

        // Get element at j
        let j_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder.build_int_mul(j, elem_size, "j_mul").unwrap(),
                "j_offset",
            )
            .unwrap();
        let j_elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_list_ptr,
                    &[j_offset],
                    "j_elem_ptr",
                )
                .unwrap()
        };
        let j_value_ptr = self
            .builder
            .build_pointer_cast(
                j_elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "j_value_ptr",
            )
            .unwrap();
        let j_value = self
            .builder
            .build_load(self.types.value_type, j_value_ptr, "j_value")
            .unwrap();

        // Get element at j+1
        let j_plus_one = self.builder.build_int_add(j, one, "j_plus_one").unwrap();
        let j1_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(j_plus_one, elem_size, "j1_mul")
                    .unwrap(),
                "j1_offset",
            )
            .unwrap();
        let j1_elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_list_ptr,
                    &[j1_offset],
                    "j1_elem_ptr",
                )
                .unwrap()
        };
        let j1_value_ptr = self
            .builder
            .build_pointer_cast(
                j1_elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "j1_value_ptr",
            )
            .unwrap();
        let j1_value = self
            .builder
            .build_load(self.types.value_type, j1_value_ptr, "j1_value")
            .unwrap();

        // Extract integer values for comparison (assuming numeric list)
        let j_data = self
            .builder
            .build_extract_value(j_value.into_struct_value(), 1, "j_data")
            .unwrap()
            .into_int_value();
        let j1_data = self
            .builder
            .build_extract_value(j1_value.into_struct_value(), 1, "j1_data")
            .unwrap()
            .into_int_value();

        // Compare: if j_data > j1_data, swap
        let should_swap = self
            .builder
            .build_int_compare(IntPredicate::SGT, j_data, j1_data, "should_swap")
            .unwrap();

        let swap_block = self.context.append_basic_block(function, "sort_swap");
        let no_swap_block = self.context.append_basic_block(function, "sort_no_swap");

        self.builder
            .build_conditional_branch(should_swap, swap_block, no_swap_block)
            .unwrap();

        self.builder.position_at_end(swap_block);
        // Swap values
        self.builder.build_store(j_value_ptr, j1_value).unwrap();
        self.builder.build_store(j1_value_ptr, j_value).unwrap();
        self.builder
            .build_unconditional_branch(no_swap_block)
            .unwrap();

        self.builder.position_at_end(no_swap_block);
        let next_j = self.builder.build_int_add(j, one, "next_j").unwrap();
        self.builder.build_store(j_ptr, next_j).unwrap();
        self.builder.build_unconditional_branch(inner_loop).unwrap();

        self.builder.position_at_end(inner_done);
        let next_i = self.builder.build_int_add(i, one, "next_i").unwrap();
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder.build_unconditional_branch(outer_loop).unwrap();

        self.builder.position_at_end(outer_done);

        // Return new list as value
        let list_as_int = self
            .builder
            .build_ptr_to_int(new_list_ptr, self.types.i64_type, "list_as_int")
            .unwrap();
        let list_tag = self
            .types
            .i8_type
            .const_int(ValueTag::List.as_u8() as u64, false);
        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, list_tag, 0, "v1")
            .unwrap();
        let v2 = self
            .builder
            .build_insert_value(v1, list_as_int, 1, "v2")
            .unwrap();
        Ok(v2.into_struct_value().into())
    }

    /// shuffle(list) - Return a shuffled copy of the list (Fisher-Yates shuffle)
    fn inline_shuffle(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // First, create a copy of the list
        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        // Get list length
        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        // Calculate total size: 8 + len * 16
        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let elems_total = self
            .builder
            .build_int_mul(list_len, elem_size, "elems_total")
            .unwrap();
        let total_size = self
            .builder
            .build_int_add(header_size, elems_total, "total_size")
            .unwrap();

        // Allocate new list
        let new_list_ptr = self
            .builder
            .build_call(self.libc.malloc, &[total_size.into()], "new_list")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Copy entire list
        self.builder
            .build_call(
                self.libc.memcpy,
                &[new_list_ptr.into(), list_ptr.into(), total_size.into()],
                "copy_list",
            )
            .unwrap();

        // Seed random number generator with time
        let null = i8_ptr_type.const_null();
        let time_val = self
            .builder
            .build_call(self.libc.time, &[null.into()], "time_val")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let time_i32 = self
            .builder
            .build_int_truncate(time_val, self.context.i32_type(), "time_i32")
            .unwrap();
        self.builder
            .build_call(self.libc.srand, &[time_i32.into()], "seed_rand")
            .unwrap();

        // Fisher-Yates shuffle: for i from len-1 down to 1
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);

        let len_minus_one = self
            .builder
            .build_int_sub(list_len, one, "len_minus_one")
            .unwrap();
        let i_ptr = self.builder.build_alloca(self.types.i64_type, "i").unwrap();
        self.builder.build_store(i_ptr, len_minus_one).unwrap();

        let shuffle_loop = self.context.append_basic_block(function, "shuffle_loop");
        let shuffle_body = self.context.append_basic_block(function, "shuffle_body");
        let shuffle_done = self.context.append_basic_block(function, "shuffle_done");

        self.builder
            .build_unconditional_branch(shuffle_loop)
            .unwrap();
        self.builder.position_at_end(shuffle_loop);

        let i = self
            .builder
            .build_load(self.types.i64_type, i_ptr, "i_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::ULE, i, zero, "shuffle_done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, shuffle_done, shuffle_body)
            .unwrap();

        self.builder.position_at_end(shuffle_body);

        // Generate random index j in [0, i]
        let rand_val = self
            .builder
            .build_call(self.libc.rand, &[], "rand_val")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let rand_i64 = self
            .builder
            .build_int_z_extend(rand_val, self.types.i64_type, "rand_i64")
            .unwrap();
        let i_plus_one = self.builder.build_int_add(i, one, "i_plus_one").unwrap();
        let j = self
            .builder
            .build_int_unsigned_rem(rand_i64, i_plus_one, "j")
            .unwrap();

        // Get element at i
        let i_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder.build_int_mul(i, elem_size, "i_mul").unwrap(),
                "i_offset",
            )
            .unwrap();
        let i_elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_list_ptr,
                    &[i_offset],
                    "i_elem_ptr",
                )
                .unwrap()
        };
        let i_value_ptr = self
            .builder
            .build_pointer_cast(
                i_elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "i_value_ptr",
            )
            .unwrap();
        let i_value = self
            .builder
            .build_load(self.types.value_type, i_value_ptr, "i_value")
            .unwrap();

        // Get element at j
        let j_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder.build_int_mul(j, elem_size, "j_mul").unwrap(),
                "j_offset",
            )
            .unwrap();
        let j_elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_list_ptr,
                    &[j_offset],
                    "j_elem_ptr",
                )
                .unwrap()
        };
        let j_value_ptr = self
            .builder
            .build_pointer_cast(
                j_elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "j_value_ptr",
            )
            .unwrap();
        let j_value = self
            .builder
            .build_load(self.types.value_type, j_value_ptr, "j_value")
            .unwrap();

        // Swap values
        self.builder.build_store(i_value_ptr, j_value).unwrap();
        self.builder.build_store(j_value_ptr, i_value).unwrap();

        // Decrement i
        let next_i = self.builder.build_int_sub(i, one, "next_i").unwrap();
        self.builder.build_store(i_ptr, next_i).unwrap();
        self.builder
            .build_unconditional_branch(shuffle_loop)
            .unwrap();

        self.builder.position_at_end(shuffle_done);

        // Return new list as value
        let list_as_int = self
            .builder
            .build_ptr_to_int(new_list_ptr, self.types.i64_type, "list_as_int")
            .unwrap();
        let list_tag = self
            .types
            .i8_type
            .const_int(ValueTag::List.as_u8() as u64, false);
        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, list_tag, 0, "v1")
            .unwrap();
        let v2 = self
            .builder
            .build_insert_value(v1, list_as_int, 1, "v2")
            .unwrap();
        Ok(v2.into_struct_value().into())
    }

    // ===== Lambda and Higher-Order Functions =====

    /// Compile a lambda expression into an LLVM function and return a function pointer value
    fn compile_lambda(
        &mut self,
        params: &[String],
        body: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Generate unique lambda name
        let lambda_name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Create function type: (value, value, ...) -> value
        let param_types: Vec<BasicMetadataTypeEnum> = params
            .iter()
            .map(|_| self.types.value_type.into())
            .collect();
        let fn_type = self.types.value_type.fn_type(&param_types, false);
        let lambda_fn = self.module.add_function(&lambda_name, fn_type, None);

        // Save current state
        let saved_function = self.current_function;
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_block = self.builder.get_insert_block();

        // Set up lambda function
        self.current_function = Some(lambda_fn);
        let entry = self.context.append_basic_block(lambda_fn, "entry");
        self.builder.position_at_end(entry);

        // Create allocas for parameters
        for (i, param_name) in params.iter().enumerate() {
            let alloca = self
                .builder
                .build_alloca(self.types.value_type, param_name)
                .unwrap();
            let param_val = lambda_fn.get_nth_param(i as u32).unwrap();
            self.builder.build_store(alloca, param_val).unwrap();
            self.variables.insert(param_name.clone(), alloca);
        }

        // Compile the lambda body
        let result = self.compile_expr(body)?;
        self.builder.build_return(Some(&result)).unwrap();

        // Restore state
        self.current_function = saved_function;
        self.variables = saved_variables;
        if let Some(block) = saved_block {
            self.builder.position_at_end(block);
        }

        // Register lambda as a callable function
        self.functions.insert(lambda_name.clone(), lambda_fn);

        // Return function pointer as value (tag=7 for Function)
        let fn_ptr = lambda_fn.as_global_value().as_pointer_value();
        let fn_ptr_int = self
            .builder
            .build_ptr_to_int(fn_ptr, self.types.i64_type, "fn_ptr_int")
            .unwrap();
        let fn_tag = self
            .types
            .i8_type
            .const_int(ValueTag::Function.as_u8() as u64, false);
        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, fn_tag, 0, "v1")
            .unwrap();
        let v2 = self
            .builder
            .build_insert_value(v1, fn_ptr_int, 1, "v2")
            .unwrap();
        Ok(v2.into_struct_value().into())
    }

    /// Helper to call a function value with arguments
    fn call_function_value(
        &mut self,
        func_val: BasicValueEnum<'ctx>,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Extract function pointer from value
        let func_struct = func_val.into_struct_value();
        let func_data = self
            .builder
            .build_extract_value(func_struct, 1, "func_data")
            .unwrap()
            .into_int_value();

        // Create function type based on number of args
        let param_types: Vec<BasicMetadataTypeEnum> =
            args.iter().map(|_| self.types.value_type.into()).collect();
        let fn_type = self.types.value_type.fn_type(&param_types, false);
        let fn_ptr_type = fn_type.ptr_type(AddressSpace::default());

        let fn_ptr = self
            .builder
            .build_int_to_ptr(func_data, fn_ptr_type, "fn_ptr")
            .unwrap();

        // Prepare arguments
        let call_args: Vec<BasicMetadataValueEnum> = args.iter().map(|a| (*a).into()).collect();

        // Call the function
        let result = self
            .builder
            .build_indirect_call(fn_type, fn_ptr, &call_args, "call_result")
            .unwrap();

        Ok(result.try_as_basic_value().left().unwrap())
    }

    /// gaun(list, fn) - map function over list
    fn inline_gaun(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        func_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Extract list pointer and length
        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        // Allocate new list
        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let elems_total = self
            .builder
            .build_int_mul(list_len, elem_size, "elems_total")
            .unwrap();
        let total_size = self
            .builder
            .build_int_add(header_size, elems_total, "total_size")
            .unwrap();

        let new_list_ptr = self
            .builder
            .build_call(self.libc.malloc, &[total_size.into()], "new_list")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Store length
        let new_len_ptr = self
            .builder
            .build_pointer_cast(new_list_ptr, i64_ptr_type, "new_len_ptr")
            .unwrap();
        self.builder.build_store(new_len_ptr, list_len).unwrap();

        // Store func_val in an alloca so we can use it in the loop
        let func_alloca = self
            .builder
            .build_alloca(self.types.value_type, "func_alloca")
            .unwrap();
        self.builder.build_store(func_alloca, func_val).unwrap();

        // Loop and apply function
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "gaun_loop");
        let body_block = self.context.append_basic_block(function, "gaun_body");
        let done_block = self.context.append_basic_block(function, "gaun_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, list_len, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, done_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        // Get element at idx
        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "idx_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), list_ptr, &[elem_offset], "elem_ptr")
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let elem_val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "elem_val")
            .unwrap();

        // Load function and call it
        let func = self
            .builder
            .build_load(self.types.value_type, func_alloca, "func")
            .unwrap();
        let mapped = self.call_function_value(func, &[elem_val])?;

        // Store result in new list
        let new_elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_list_ptr,
                    &[elem_offset],
                    "new_elem_ptr",
                )
                .unwrap()
        };
        let new_value_ptr = self
            .builder
            .build_pointer_cast(
                new_elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "new_value_ptr",
            )
            .unwrap();
        self.builder.build_store(new_value_ptr, mapped).unwrap();

        // Increment index
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);

        // Return new list
        let list_as_int = self
            .builder
            .build_ptr_to_int(new_list_ptr, self.types.i64_type, "list_as_int")
            .unwrap();
        let list_tag = self
            .types
            .i8_type
            .const_int(ValueTag::List.as_u8() as u64, false);
        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, list_tag, 0, "v1")
            .unwrap();
        let v2 = self
            .builder
            .build_insert_value(v1, list_as_int, 1, "v2")
            .unwrap();
        Ok(v2.into_struct_value().into())
    }

    /// sieve(list, fn) - filter list by predicate
    fn inline_sieve(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        func_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Extract list pointer and length
        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        // Allocate new list (max size = original size)
        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let elems_total = self
            .builder
            .build_int_mul(list_len, elem_size, "elems_total")
            .unwrap();
        let total_size = self
            .builder
            .build_int_add(header_size, elems_total, "total_size")
            .unwrap();

        let new_list_ptr = self
            .builder
            .build_call(self.libc.malloc, &[total_size.into()], "new_list")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Store func_val in an alloca
        let func_alloca = self
            .builder
            .build_alloca(self.types.value_type, "func_alloca")
            .unwrap();
        self.builder.build_store(func_alloca, func_val).unwrap();

        // Counters
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        let count_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "count")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();
        self.builder.build_store(count_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "sieve_loop");
        let body_block = self.context.append_basic_block(function, "sieve_body");
        let keep_block = self.context.append_basic_block(function, "sieve_keep");
        let next_block = self.context.append_basic_block(function, "sieve_next");
        let done_block = self.context.append_basic_block(function, "sieve_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, list_len, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, done_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        // Get element at idx
        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "idx_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), list_ptr, &[elem_offset], "elem_ptr")
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let elem_val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "elem_val")
            .unwrap();

        // Store elem_val in alloca for use in keep_block
        let elem_alloca = self
            .builder
            .build_alloca(self.types.value_type, "elem_alloca")
            .unwrap();
        self.builder.build_store(elem_alloca, elem_val).unwrap();

        // Call predicate
        let func = self
            .builder
            .build_load(self.types.value_type, func_alloca, "func")
            .unwrap();
        let pred_result = self.call_function_value(func, &[elem_val])?;

        // Check if truthy
        let is_truthy = self.is_truthy(pred_result)?;
        self.builder
            .build_conditional_branch(is_truthy, keep_block, next_block)
            .unwrap();

        // Keep element
        self.builder.position_at_end(keep_block);
        let count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "count_val")
            .unwrap()
            .into_int_value();
        let new_elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(count, elem_size, "count_mul")
                    .unwrap(),
                "new_elem_offset",
            )
            .unwrap();
        let new_elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_list_ptr,
                    &[new_elem_offset],
                    "new_elem_ptr",
                )
                .unwrap()
        };
        let new_value_ptr = self
            .builder
            .build_pointer_cast(
                new_elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "new_value_ptr",
            )
            .unwrap();
        let elem_to_store = self
            .builder
            .build_load(self.types.value_type, elem_alloca, "elem_to_store")
            .unwrap();
        self.builder
            .build_store(new_value_ptr, elem_to_store)
            .unwrap();
        let next_count = self
            .builder
            .build_int_add(count, one, "next_count")
            .unwrap();
        self.builder.build_store(count_ptr, next_count).unwrap();
        self.builder.build_unconditional_branch(next_block).unwrap();

        // Next iteration
        self.builder.position_at_end(next_block);
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);

        // Store final count as length
        let final_count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "final_count")
            .unwrap()
            .into_int_value();
        let new_len_ptr = self
            .builder
            .build_pointer_cast(new_list_ptr, i64_ptr_type, "new_len_ptr")
            .unwrap();
        self.builder.build_store(new_len_ptr, final_count).unwrap();

        // Return new list
        let list_as_int = self
            .builder
            .build_ptr_to_int(new_list_ptr, self.types.i64_type, "list_as_int")
            .unwrap();
        let list_tag = self
            .types
            .i8_type
            .const_int(ValueTag::List.as_u8() as u64, false);
        let undef = self.types.value_type.get_undef();
        let v1 = self
            .builder
            .build_insert_value(undef, list_tag, 0, "v1")
            .unwrap();
        let v2 = self
            .builder
            .build_insert_value(v1, list_as_int, 1, "v2")
            .unwrap();
        Ok(v2.into_struct_value().into())
    }

    /// tumble(list, init, fn) - reduce/fold
    fn inline_tumble(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        init_val: BasicValueEnum<'ctx>,
        func_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Extract list pointer and length
        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        // Store func_val and accumulator
        let func_alloca = self
            .builder
            .build_alloca(self.types.value_type, "func_alloca")
            .unwrap();
        self.builder.build_store(func_alloca, func_val).unwrap();
        let acc_alloca = self
            .builder
            .build_alloca(self.types.value_type, "acc_alloca")
            .unwrap();
        self.builder.build_store(acc_alloca, init_val).unwrap();

        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "tumble_loop");
        let body_block = self.context.append_basic_block(function, "tumble_body");
        let done_block = self.context.append_basic_block(function, "tumble_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, list_len, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, done_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        // Get element at idx
        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "idx_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), list_ptr, &[elem_offset], "elem_ptr")
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let elem_val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "elem_val")
            .unwrap();

        // Call fn(acc, elem)
        let func = self
            .builder
            .build_load(self.types.value_type, func_alloca, "func")
            .unwrap();
        let acc = self
            .builder
            .build_load(self.types.value_type, acc_alloca, "acc")
            .unwrap();
        let new_acc = self.call_function_value(func, &[acc, elem_val])?;
        self.builder.build_store(acc_alloca, new_acc).unwrap();

        // Increment index
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);
        let final_acc = self
            .builder
            .build_load(self.types.value_type, acc_alloca, "final_acc")
            .unwrap();
        Ok(final_acc)
    }

    /// aw(list, fn) - all elements satisfy predicate
    fn inline_aw(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        func_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        let func_alloca = self
            .builder
            .build_alloca(self.types.value_type, "func_alloca")
            .unwrap();
        self.builder.build_store(func_alloca, func_val).unwrap();

        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "aw_loop");
        let body_block = self.context.append_basic_block(function, "aw_body");
        let false_block = self.context.append_basic_block(function, "aw_false");
        let true_block = self.context.append_basic_block(function, "aw_true");
        let done_block = self.context.append_basic_block(function, "aw_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, list_len, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, true_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "idx_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), list_ptr, &[elem_offset], "elem_ptr")
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let elem_val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "elem_val")
            .unwrap();

        let func = self
            .builder
            .build_load(self.types.value_type, func_alloca, "func")
            .unwrap();
        let pred_result = self.call_function_value(func, &[elem_val])?;
        let is_truthy = self.is_truthy(pred_result)?;

        let next_block = self.context.append_basic_block(function, "aw_next");
        self.builder
            .build_conditional_branch(is_truthy, next_block, false_block)
            .unwrap();

        self.builder.position_at_end(next_block);
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(false_block);
        let false_result = self.make_bool(self.types.bool_type.const_int(0, false))?;
        self.builder.build_unconditional_branch(done_block).unwrap();
        let false_block_end = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(true_block);
        let true_result = self.make_bool(self.types.bool_type.const_int(1, false))?;
        self.builder.build_unconditional_branch(done_block).unwrap();
        let true_block_end = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(done_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "aw_result")
            .unwrap();
        phi.add_incoming(&[
            (&false_result, false_block_end),
            (&true_result, true_block_end),
        ]);
        Ok(phi.as_basic_value())
    }

    /// ony(list, fn) - any element satisfies predicate
    fn inline_ony(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        func_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        let func_alloca = self
            .builder
            .build_alloca(self.types.value_type, "func_alloca")
            .unwrap();
        self.builder.build_store(func_alloca, func_val).unwrap();

        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "ony_loop");
        let body_block = self.context.append_basic_block(function, "ony_body");
        let true_block = self.context.append_basic_block(function, "ony_true");
        let false_block = self.context.append_basic_block(function, "ony_false");
        let done_block = self.context.append_basic_block(function, "ony_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, list_len, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, false_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "idx_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), list_ptr, &[elem_offset], "elem_ptr")
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let elem_val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "elem_val")
            .unwrap();

        let func = self
            .builder
            .build_load(self.types.value_type, func_alloca, "func")
            .unwrap();
        let pred_result = self.call_function_value(func, &[elem_val])?;
        let is_truthy = self.is_truthy(pred_result)?;

        let next_block = self.context.append_basic_block(function, "ony_next");
        self.builder
            .build_conditional_branch(is_truthy, true_block, next_block)
            .unwrap();

        self.builder.position_at_end(next_block);
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(true_block);
        let true_result = self.make_bool(self.types.bool_type.const_int(1, false))?;
        self.builder.build_unconditional_branch(done_block).unwrap();
        let true_block_end = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(false_block);
        let false_result = self.make_bool(self.types.bool_type.const_int(0, false))?;
        self.builder.build_unconditional_branch(done_block).unwrap();
        let false_block_end = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(done_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "ony_result")
            .unwrap();
        phi.add_incoming(&[
            (&true_result, true_block_end),
            (&false_result, false_block_end),
        ]);
        Ok(phi.as_basic_value())
    }

    /// hunt(list, fn) - find first element satisfying predicate
    fn inline_hunt(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        func_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        let func_alloca = self
            .builder
            .build_alloca(self.types.value_type, "func_alloca")
            .unwrap();
        self.builder.build_store(func_alloca, func_val).unwrap();

        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "hunt_loop");
        let body_block = self.context.append_basic_block(function, "hunt_body");
        let found_block = self.context.append_basic_block(function, "hunt_found");
        let notfound_block = self.context.append_basic_block(function, "hunt_notfound");
        let done_block = self.context.append_basic_block(function, "hunt_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, list_len, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, notfound_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "idx_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), list_ptr, &[elem_offset], "elem_ptr")
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let elem_val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "elem_val")
            .unwrap();

        // Store elem in alloca for use in found block
        let found_alloca = self
            .builder
            .build_alloca(self.types.value_type, "found_alloca")
            .unwrap();
        self.builder.build_store(found_alloca, elem_val).unwrap();

        let func = self
            .builder
            .build_load(self.types.value_type, func_alloca, "func")
            .unwrap();
        let pred_result = self.call_function_value(func, &[elem_val])?;
        let is_truthy = self.is_truthy(pred_result)?;

        let next_block = self.context.append_basic_block(function, "hunt_next");
        self.builder
            .build_conditional_branch(is_truthy, found_block, next_block)
            .unwrap();

        self.builder.position_at_end(next_block);
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(found_block);
        let found_result = self
            .builder
            .build_load(self.types.value_type, found_alloca, "found_result")
            .unwrap();
        self.builder.build_unconditional_branch(done_block).unwrap();
        let found_block_end = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(notfound_block);
        let nil_result = self.make_nil();
        self.builder.build_unconditional_branch(done_block).unwrap();
        let notfound_block_end = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(done_block);
        let phi = self
            .builder
            .build_phi(self.types.value_type, "hunt_result")
            .unwrap();
        phi.add_incoming(&[
            (&found_result, found_block_end),
            (&nil_result, notfound_block_end),
        ]);
        Ok(phi.as_basic_value())
    }

    /// ilk(list, fn) - for-each: calls fn for each element in the list
    fn inline_ilk(
        &mut self,
        list_val: BasicValueEnum<'ctx>,
        func_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Get list pointer and length
        let list_struct = list_val.into_struct_value();
        let list_data = self
            .builder
            .build_extract_value(list_struct, 1, "list_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let list_ptr = self
            .builder
            .build_int_to_ptr(list_data, i8_ptr_type, "list_ptr")
            .unwrap();

        // Get list length
        let header_ptr = self
            .builder
            .build_pointer_cast(list_ptr, i64_ptr_type, "header_ptr")
            .unwrap();
        let len_ptr = unsafe {
            self.builder
                .build_gep(
                    self.types.i64_type,
                    header_ptr,
                    &[self.types.i64_type.const_int(1, false)],
                    "len_ptr",
                )
                .unwrap()
        };
        let list_len = self
            .builder
            .build_load(self.types.i64_type, len_ptr, "list_len")
            .unwrap()
            .into_int_value();

        // Store function value for use in loop
        let func_alloca = self
            .builder
            .build_alloca(self.types.value_type, "func_alloca")
            .unwrap();
        self.builder.build_store(func_alloca, func_val).unwrap();

        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "ilk_loop");
        let body_block = self.context.append_basic_block(function, "ilk_body");
        let done_block = self.context.append_basic_block(function, "ilk_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, list_len, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, done_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        // Get current element
        let elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "idx_mul")
                    .unwrap(),
                "elem_offset",
            )
            .unwrap();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(self.context.i8_type(), list_ptr, &[elem_offset], "elem_ptr")
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let elem_val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "elem_val")
            .unwrap();

        // Call function with element (ignore result)
        let func = self
            .builder
            .build_load(self.types.value_type, func_alloca, "func")
            .unwrap();
        let _result = self.call_function_value(func, &[elem_val])?;

        // Increment and continue
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);
        // Return nil (for-each doesn't return a value)
        Ok(self.make_nil())
    }

    /// keys(dict) - returns a list of all keys in the dict
    fn inline_keys(
        &mut self,
        dict_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Dict layout: [i64 count][entry0][entry1]... where entry = [value key][value val]
        let dict_struct = dict_val.into_struct_value();
        let dict_data = self
            .builder
            .build_extract_value(dict_struct, 1, "dict_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let dict_ptr = self
            .builder
            .build_int_to_ptr(dict_data, i8_ptr_type, "dict_ptr")
            .unwrap();

        // Get dict count
        let count_ptr = self
            .builder
            .build_pointer_cast(dict_ptr, i64_ptr_type, "count_ptr")
            .unwrap();
        let dict_count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "dict_count")
            .unwrap()
            .into_int_value();

        // Allocate result list: 8 bytes header + 16 bytes per key
        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let result_data_size = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(dict_count, elem_size, "data_size")
                    .unwrap(),
                "result_size",
            )
            .unwrap();
        let result_ptr = self
            .builder
            .build_call(self.libc.malloc, &[result_data_size.into()], "result_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to malloc: {}", e)))?
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let result_len_ptr = self
            .builder
            .build_pointer_cast(result_ptr, i64_ptr_type, "result_len_ptr")
            .unwrap();
        self.builder
            .build_store(result_len_ptr, dict_count)
            .unwrap();

        // Loop to copy keys
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let entry_size = self.types.i64_type.const_int(32, false); // 16 bytes key + 16 bytes value
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "keys_loop");
        let body_block = self.context.append_basic_block(function, "keys_body");
        let done_block = self.context.append_basic_block(function, "keys_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, dict_count, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, done_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        // Get key from dict entry
        let dict_entry_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, entry_size, "entry_mul")
                    .unwrap(),
                "entry_offset",
            )
            .unwrap();
        let dict_key_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    dict_ptr,
                    &[dict_entry_offset],
                    "dict_key_ptr",
                )
                .unwrap()
        };
        let key_value_ptr = self
            .builder
            .build_pointer_cast(
                dict_key_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "key_value_ptr",
            )
            .unwrap();
        let key_val = self
            .builder
            .build_load(self.types.value_type, key_value_ptr, "key_val")
            .unwrap();

        // Store key in result list
        let result_elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "result_mul")
                    .unwrap(),
                "result_offset",
            )
            .unwrap();
        let result_elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    result_ptr,
                    &[result_elem_offset],
                    "result_elem_ptr",
                )
                .unwrap()
        };
        let result_value_ptr = self
            .builder
            .build_pointer_cast(
                result_elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "result_value_ptr",
            )
            .unwrap();
        self.builder.build_store(result_value_ptr, key_val).unwrap();

        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);
        self.make_list(result_ptr)
    }

    /// values(dict) - returns a list of all values in the dict
    fn inline_values(
        &mut self,
        dict_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Dict layout: [i64 count][entry0][entry1]... where entry = [value key][value val]
        let dict_struct = dict_val.into_struct_value();
        let dict_data = self
            .builder
            .build_extract_value(dict_struct, 1, "dict_data")
            .unwrap()
            .into_int_value();
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let dict_ptr = self
            .builder
            .build_int_to_ptr(dict_data, i8_ptr_type, "dict_ptr")
            .unwrap();

        // Get dict count
        let count_ptr = self
            .builder
            .build_pointer_cast(dict_ptr, i64_ptr_type, "count_ptr")
            .unwrap();
        let dict_count = self
            .builder
            .build_load(self.types.i64_type, count_ptr, "dict_count")
            .unwrap()
            .into_int_value();

        // Allocate result list: 8 bytes header + 16 bytes per value
        let header_size = self.types.i64_type.const_int(16, false);
        let elem_size = self.types.i64_type.const_int(16, false);
        let result_data_size = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(dict_count, elem_size, "data_size")
                    .unwrap(),
                "result_size",
            )
            .unwrap();
        let result_ptr = self
            .builder
            .build_call(self.libc.malloc, &[result_data_size.into()], "result_ptr")
            .map_err(|e| HaversError::CompileError(format!("Failed to malloc: {}", e)))?
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let result_len_ptr = self
            .builder
            .build_pointer_cast(result_ptr, i64_ptr_type, "result_len_ptr")
            .unwrap();
        self.builder
            .build_store(result_len_ptr, dict_count)
            .unwrap();

        // Loop to copy values
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let entry_size = self.types.i64_type.const_int(32, false); // 16 bytes key + 16 bytes value
        let value_offset_in_entry = self.types.i64_type.const_int(16, false); // Value comes after key
        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "values_loop");
        let body_block = self.context.append_basic_block(function, "values_body");
        let done_block = self.context.append_basic_block(function, "values_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, dict_count, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, done_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        // Get value from dict entry (offset by 16 bytes from entry start)
        let dict_entry_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, entry_size, "entry_mul")
                    .unwrap(),
                "entry_offset",
            )
            .unwrap();
        let dict_value_offset = self
            .builder
            .build_int_add(dict_entry_offset, value_offset_in_entry, "value_offset")
            .unwrap();
        let dict_value_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    dict_ptr,
                    &[dict_value_offset],
                    "dict_value_ptr",
                )
                .unwrap()
        };
        let value_ptr = self
            .builder
            .build_pointer_cast(
                dict_value_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_ptr",
            )
            .unwrap();
        let val = self
            .builder
            .build_load(self.types.value_type, value_ptr, "val")
            .unwrap();

        // Store value in result list
        let result_elem_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, elem_size, "result_mul")
                    .unwrap(),
                "result_offset",
            )
            .unwrap();
        let result_elem_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    result_ptr,
                    &[result_elem_offset],
                    "result_elem_ptr",
                )
                .unwrap()
        };
        let result_value_ptr = self
            .builder
            .build_pointer_cast(
                result_elem_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "result_value_ptr",
            )
            .unwrap();
        self.builder.build_store(result_value_ptr, val).unwrap();

        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        self.builder.position_at_end(done_block);
        self.make_list(result_ptr)
    }

    // ========== Class/OOP Support ==========

    /// Compile `masel` expression - returns the current instance
    fn compile_masel(&self) -> Result<BasicValueEnum<'ctx>, HaversError> {
        if let Some(masel_ptr) = self.current_masel {
            // Load the masel value from the pointer
            let masel_val = self
                .builder
                .build_load(self.types.value_type, masel_ptr, "masel_val")
                .map_err(|e| HaversError::CompileError(format!("Failed to load masel: {}", e)))?;
            Ok(masel_val)
        } else {
            Err(HaversError::CompileError(
                "'masel' used outside of a method".to_string(),
            ))
        }
    }

    /// Compile property get expression: obj.property
    fn compile_get(
        &mut self,
        object: &Expr,
        property: &str,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let obj_val = self.compile_expr(object)?;
        self.compile_instance_get_field(obj_val, property)
    }

    /// Compile property set expression: obj.property = value
    fn compile_set(
        &mut self,
        object: &Expr,
        property: &str,
        value: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let obj_val = self.compile_expr(object)?;
        let val = self.compile_expr(value)?;
        self.compile_instance_set_field(obj_val, property, val)
    }

    /// Get a field from an instance
    /// Instance layout: [i64 class_name_ptr][i64 field_count][field_entry0][field_entry1]...
    /// where field_entry = [{i8,i64} key (string)][{i8,i64} value]
    fn compile_instance_get_field(
        &mut self,
        instance_val: BasicValueEnum<'ctx>,
        field_name: &str,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Extract instance data pointer
        let instance_struct = instance_val.into_struct_value();
        let instance_data = self
            .builder
            .build_extract_value(instance_struct, 1, "instance_data")
            .unwrap()
            .into_int_value();

        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let instance_ptr = self
            .builder
            .build_int_to_ptr(instance_data, i8_ptr_type, "instance_ptr")
            .unwrap();

        // Skip class name pointer (8 bytes) to get to field count
        let field_count_offset = self.types.i64_type.const_int(8, false);
        let field_count_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    instance_ptr,
                    &[field_count_offset],
                    "field_count_ptr",
                )
                .unwrap()
        };
        let field_count_i64_ptr = self
            .builder
            .build_pointer_cast(field_count_ptr, i64_ptr_type, "field_count_i64")
            .unwrap();
        let field_count = self
            .builder
            .build_load(self.types.i64_type, field_count_i64_ptr, "field_count")
            .unwrap()
            .into_int_value();

        // Create field name as a global string for comparison
        let field_name_global = self
            .builder
            .build_global_string_ptr(field_name, "field_name")
            .unwrap();

        // Loop through fields to find matching name
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let header_size = self.types.i64_type.const_int(16, false); // class_name_ptr + field_count
        let entry_size = self.types.i64_type.const_int(32, false); // 16 bytes key + 16 bytes value
        let value_offset_in_entry = self.types.i64_type.const_int(16, false);

        let result_ptr = self
            .builder
            .build_alloca(self.types.value_type, "result_ptr")
            .unwrap();
        self.builder
            .build_store(result_ptr, self.make_nil())
            .unwrap();

        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let loop_block = self.context.append_basic_block(function, "get_field_loop");
        let body_block = self.context.append_basic_block(function, "get_field_body");
        let found_block = self.context.append_basic_block(function, "get_field_found");
        let continue_block = self
            .context
            .append_basic_block(function, "get_field_continue");
        let done_block = self.context.append_basic_block(function, "get_field_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, field_count, "done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, done_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        // Get entry key (string value)
        let entry_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, entry_size, "entry_mul")
                    .unwrap(),
                "entry_offset",
            )
            .unwrap();
        let entry_key_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    instance_ptr,
                    &[entry_offset],
                    "entry_key_ptr",
                )
                .unwrap()
        };
        let key_value_ptr = self
            .builder
            .build_pointer_cast(
                entry_key_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "key_value_ptr",
            )
            .unwrap();
        let entry_key = self
            .builder
            .build_load(self.types.value_type, key_value_ptr, "entry_key")
            .unwrap();

        // Extract string pointer from key and compare with field name
        let entry_key_data = self
            .builder
            .build_extract_value(entry_key.into_struct_value(), 1, "key_data")
            .unwrap()
            .into_int_value();
        let entry_key_str = self
            .builder
            .build_int_to_ptr(entry_key_data, i8_ptr_type, "key_str")
            .unwrap();

        // Use strstr to check if strings match (simple equality check)
        let cmp_result = self
            .builder
            .build_call(
                self.libc.strstr,
                &[
                    entry_key_str.into(),
                    field_name_global.as_pointer_value().into(),
                ],
                "cmp_result",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Check if strstr returned the start of the string (exact match at beginning)
        let cmp_eq = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                self.builder
                    .build_ptr_to_int(cmp_result, self.types.i64_type, "cmp_int")
                    .unwrap(),
                self.builder
                    .build_ptr_to_int(entry_key_str, self.types.i64_type, "key_int")
                    .unwrap(),
                "cmp_eq",
            )
            .unwrap();

        // Also check string lengths are equal
        let field_name_len = self
            .builder
            .build_call(
                self.libc.strlen,
                &[field_name_global.as_pointer_value().into()],
                "field_len",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let entry_key_len = self
            .builder
            .build_call(self.libc.strlen, &[entry_key_str.into()], "key_len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let len_eq = self
            .builder
            .build_int_compare(IntPredicate::EQ, field_name_len, entry_key_len, "len_eq")
            .unwrap();

        let keys_match = self
            .builder
            .build_and(cmp_eq, len_eq, "keys_match")
            .unwrap();
        self.builder
            .build_conditional_branch(keys_match, found_block, continue_block)
            .unwrap();

        // Found - get the value
        self.builder.position_at_end(found_block);
        let value_offset = self
            .builder
            .build_int_add(entry_offset, value_offset_in_entry, "value_offset")
            .unwrap();
        let value_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    instance_ptr,
                    &[value_offset],
                    "value_ptr",
                )
                .unwrap()
        };
        let value_typed_ptr = self
            .builder
            .build_pointer_cast(
                value_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_typed_ptr",
            )
            .unwrap();
        let found_val = self
            .builder
            .build_load(self.types.value_type, value_typed_ptr, "found_val")
            .unwrap();
        self.builder.build_store(result_ptr, found_val).unwrap();
        self.builder.build_unconditional_branch(done_block).unwrap();

        // Continue loop
        self.builder.position_at_end(continue_block);
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        // Done
        self.builder.position_at_end(done_block);
        let result = self
            .builder
            .build_load(self.types.value_type, result_ptr, "get_result")
            .unwrap();
        Ok(result)
    }

    /// Set a field on an instance (add or update)
    /// This is complex because we may need to grow the instance if adding a new field
    fn compile_instance_set_field(
        &mut self,
        instance_val: BasicValueEnum<'ctx>,
        field_name: &str,
        value: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let function = self.current_function.unwrap();

        // Extract instance data pointer
        let instance_struct = instance_val.into_struct_value();
        let instance_data = self
            .builder
            .build_extract_value(instance_struct, 1, "instance_data")
            .unwrap()
            .into_int_value();

        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());
        let instance_ptr = self
            .builder
            .build_int_to_ptr(instance_data, i8_ptr_type, "instance_ptr")
            .unwrap();

        // Get field count
        let field_count_offset = self.types.i64_type.const_int(8, false);
        let field_count_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    instance_ptr,
                    &[field_count_offset],
                    "field_count_ptr",
                )
                .unwrap()
        };
        let field_count_i64_ptr = self
            .builder
            .build_pointer_cast(field_count_ptr, i64_ptr_type, "field_count_i64")
            .unwrap();
        let field_count = self
            .builder
            .build_load(self.types.i64_type, field_count_i64_ptr, "field_count")
            .unwrap()
            .into_int_value();

        // Create field name as a global string
        let field_name_global = self
            .builder
            .build_global_string_ptr(field_name, "field_name_set")
            .unwrap();

        // Loop through fields to find existing field or add new
        let zero = self.types.i64_type.const_int(0, false);
        let one = self.types.i64_type.const_int(1, false);
        let header_size = self.types.i64_type.const_int(16, false);
        let entry_size = self.types.i64_type.const_int(32, false);
        let value_offset_in_entry = self.types.i64_type.const_int(16, false);

        let idx_ptr = self
            .builder
            .build_alloca(self.types.i64_type, "idx")
            .unwrap();
        self.builder.build_store(idx_ptr, zero).unwrap();

        let found_flag = self
            .builder
            .build_alloca(self.types.bool_type, "found_flag")
            .unwrap();
        self.builder
            .build_store(found_flag, self.types.bool_type.const_int(0, false))
            .unwrap();

        let loop_block = self.context.append_basic_block(function, "set_field_loop");
        let body_block = self.context.append_basic_block(function, "set_field_body");
        let found_block = self.context.append_basic_block(function, "set_field_found");
        let continue_block = self
            .context
            .append_basic_block(function, "set_field_continue");
        let add_block = self.context.append_basic_block(function, "set_field_add");
        let done_block = self.context.append_basic_block(function, "set_field_done");

        self.builder.build_unconditional_branch(loop_block).unwrap();
        self.builder.position_at_end(loop_block);

        let idx = self
            .builder
            .build_load(self.types.i64_type, idx_ptr, "idx_val")
            .unwrap()
            .into_int_value();
        let done_cond = self
            .builder
            .build_int_compare(IntPredicate::UGE, idx, field_count, "loop_done")
            .unwrap();
        self.builder
            .build_conditional_branch(done_cond, add_block, body_block)
            .unwrap();

        self.builder.position_at_end(body_block);

        // Get entry key
        let entry_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(idx, entry_size, "entry_mul")
                    .unwrap(),
                "entry_offset",
            )
            .unwrap();
        let entry_key_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    instance_ptr,
                    &[entry_offset],
                    "entry_key_ptr",
                )
                .unwrap()
        };
        let key_value_ptr = self
            .builder
            .build_pointer_cast(
                entry_key_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "key_value_ptr",
            )
            .unwrap();
        let entry_key = self
            .builder
            .build_load(self.types.value_type, key_value_ptr, "entry_key")
            .unwrap();

        // Compare keys
        let entry_key_data = self
            .builder
            .build_extract_value(entry_key.into_struct_value(), 1, "key_data")
            .unwrap()
            .into_int_value();
        let entry_key_str = self
            .builder
            .build_int_to_ptr(entry_key_data, i8_ptr_type, "key_str")
            .unwrap();

        let cmp_result = self
            .builder
            .build_call(
                self.libc.strstr,
                &[
                    entry_key_str.into(),
                    field_name_global.as_pointer_value().into(),
                ],
                "cmp_result",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        let cmp_eq = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                self.builder
                    .build_ptr_to_int(cmp_result, self.types.i64_type, "cmp_int")
                    .unwrap(),
                self.builder
                    .build_ptr_to_int(entry_key_str, self.types.i64_type, "key_int")
                    .unwrap(),
                "cmp_eq",
            )
            .unwrap();

        let field_name_len = self
            .builder
            .build_call(
                self.libc.strlen,
                &[field_name_global.as_pointer_value().into()],
                "field_len",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let entry_key_len = self
            .builder
            .build_call(self.libc.strlen, &[entry_key_str.into()], "key_len")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_int_value();
        let len_eq = self
            .builder
            .build_int_compare(IntPredicate::EQ, field_name_len, entry_key_len, "len_eq")
            .unwrap();

        let keys_match = self
            .builder
            .build_and(cmp_eq, len_eq, "keys_match")
            .unwrap();
        self.builder
            .build_conditional_branch(keys_match, found_block, continue_block)
            .unwrap();

        // Found - update the value in place
        self.builder.position_at_end(found_block);
        let value_offset = self
            .builder
            .build_int_add(entry_offset, value_offset_in_entry, "value_offset")
            .unwrap();
        let value_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    instance_ptr,
                    &[value_offset],
                    "value_ptr",
                )
                .unwrap()
        };
        let value_typed_ptr = self
            .builder
            .build_pointer_cast(
                value_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "value_typed_ptr",
            )
            .unwrap();
        self.builder.build_store(value_typed_ptr, value).unwrap();
        self.builder
            .build_store(found_flag, self.types.bool_type.const_int(1, false))
            .unwrap();
        self.builder.build_unconditional_branch(done_block).unwrap();

        // Continue loop
        self.builder.position_at_end(continue_block);
        let next_idx = self.builder.build_int_add(idx, one, "next_idx").unwrap();
        self.builder.build_store(idx_ptr, next_idx).unwrap();
        self.builder.build_unconditional_branch(loop_block).unwrap();

        // Add new field (not found) - need to realloc and add field
        self.builder.position_at_end(add_block);

        // Calculate new size: header (16) + (field_count + 1) * entry_size (32)
        let new_count = self
            .builder
            .build_int_add(field_count, one, "new_count")
            .unwrap();
        let new_size = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(new_count, entry_size, "data_size")
                    .unwrap(),
                "new_size",
            )
            .unwrap();

        // Realloc the instance
        let new_ptr = self
            .builder
            .build_call(
                self.libc.realloc,
                &[instance_ptr.into(), new_size.into()],
                "new_instance",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();

        // Update field count
        let new_count_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_ptr,
                    &[field_count_offset],
                    "new_count_ptr",
                )
                .unwrap()
        };
        let new_count_i64_ptr = self
            .builder
            .build_pointer_cast(new_count_ptr, i64_ptr_type, "new_count_i64")
            .unwrap();
        self.builder
            .build_store(new_count_i64_ptr, new_count)
            .unwrap();

        // Add new field at end
        let new_entry_offset = self
            .builder
            .build_int_add(
                header_size,
                self.builder
                    .build_int_mul(field_count, entry_size, "entry_mul")
                    .unwrap(),
                "new_entry_offset",
            )
            .unwrap();

        // Store key (string)
        let new_key_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_ptr,
                    &[new_entry_offset],
                    "new_key_ptr",
                )
                .unwrap()
        };
        let new_key_typed_ptr = self
            .builder
            .build_pointer_cast(
                new_key_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "new_key_typed_ptr",
            )
            .unwrap();
        // Create string key value
        let key_str = self
            .builder
            .build_call(
                self.libc.strdup,
                &[field_name_global.as_pointer_value().into()],
                "key_str_dup",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value();
        let key_val = self.make_string(key_str)?;
        self.builder
            .build_store(new_key_typed_ptr, key_val)
            .unwrap();

        // Store value
        let new_value_offset = self
            .builder
            .build_int_add(new_entry_offset, value_offset_in_entry, "new_value_offset")
            .unwrap();
        let new_value_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    new_ptr,
                    &[new_value_offset],
                    "new_value_ptr",
                )
                .unwrap()
        };
        let new_value_typed_ptr = self
            .builder
            .build_pointer_cast(
                new_value_ptr,
                self.types.value_type.ptr_type(AddressSpace::default()),
                "new_value_typed_ptr",
            )
            .unwrap();
        self.builder
            .build_store(new_value_typed_ptr, value)
            .unwrap();

        // Update the instance pointer in the masel variable if this is masel
        // (This handles the case where realloc moved the memory)
        if let Some(masel_ptr) = self.current_masel {
            let new_instance = self.make_instance(new_ptr)?;
            self.builder.build_store(masel_ptr, new_instance).unwrap();
        }

        self.builder.build_unconditional_branch(done_block).unwrap();

        // Done
        self.builder.position_at_end(done_block);
        Ok(value)
    }

    /// Compile a class definition
    fn compile_class(&mut self, name: &str, methods: &[Stmt]) -> Result<(), HaversError> {
        // Save the current builder position (we're in main or another function)
        let saved_block = self.builder.get_insert_block();
        let saved_function = self.current_function;

        // Store current class name for method naming
        self.current_class = Some(name.to_string());

        // First pass: declare all methods (create function signatures)
        // This allows methods to call each other regardless of definition order
        let mut method_list: Vec<(String, FunctionValue<'ctx>)> = Vec::new();
        for method in methods {
            if let Stmt::Function {
                name: method_name,
                params,
                ..
            } = method
            {
                let func_name = format!("{}_{}", name, method_name);
                let param_types: Vec<BasicMetadataTypeEnum> =
                    std::iter::once(self.types.value_type.into())
                        .chain(params.iter().map(|_| self.types.value_type.into()))
                        .collect();
                let fn_type = self.types.value_type.fn_type(&param_types, false);
                let function = self.module.add_function(&func_name, fn_type, None);
                self.functions.insert(func_name, function);
                method_list.push((method_name.clone(), function));
            }
        }

        // Store method table and class name early so methods can be looked up
        self.class_methods
            .insert(name.to_string(), method_list.clone());
        let class_name_global = self
            .builder
            .build_global_string_ptr(name, &format!("class_{}", name))
            .unwrap();
        self.classes.insert(name.to_string(), class_name_global);

        // Second pass: define all methods (compile function bodies)
        for method in methods {
            if let Stmt::Function {
                name: method_name,
                params,
                body,
                ..
            } = method
            {
                self.compile_method_body(name, method_name, params, body)?;
            }
        }

        self.current_class = None;

        // Restore the builder position to where we were before compiling the class
        if let Some(block) = saved_block {
            self.builder.position_at_end(block);
        }
        self.current_function = saved_function;

        Ok(())
    }

    /// Compile the body of a method (function within a class)
    /// The function signature is already declared in compile_class
    fn compile_method_body(
        &mut self,
        class_name: &str,
        method_name: &str,
        params: &[crate::ast::Param],
        body: &[Stmt],
    ) -> Result<(), HaversError> {
        // Get the already-declared function
        let func_name = format!("{}_{}", class_name, method_name);
        let function = *self.functions.get(&func_name).ok_or_else(|| {
            HaversError::CompileError(format!("Method {} not declared", func_name))
        })?;

        // Save current state
        let old_function = self.current_function;
        let old_variables = std::mem::take(&mut self.variables);
        let old_int_shadows = std::mem::take(&mut self.int_shadows);
        let old_var_types = std::mem::take(&mut self.var_types);
        let old_masel = self.current_masel;

        self.current_function = Some(function);

        // Create entry block
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);

        // First parameter is masel - allocate and store it
        let masel_alloca = self
            .builder
            .build_alloca(self.types.value_type, "masel")
            .map_err(|e| HaversError::CompileError(format!("Failed to alloca masel: {}", e)))?;
        let masel_param = function.get_nth_param(0).unwrap();
        self.builder
            .build_store(masel_alloca, masel_param)
            .map_err(|e| HaversError::CompileError(format!("Failed to store masel: {}", e)))?;
        self.current_masel = Some(masel_alloca);
        self.variables.insert("masel".to_string(), masel_alloca);

        // Bind remaining parameters
        for (i, param) in params.iter().enumerate() {
            let param_val = function.get_nth_param((i + 1) as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(self.types.value_type, &param.name)
                .map_err(|e| HaversError::CompileError(format!("Failed to alloca param: {}", e)))?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| HaversError::CompileError(format!("Failed to store param: {}", e)))?;
            self.variables.insert(param.name.clone(), alloca);
        }

        // Compile method body
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // Add implicit return of masel (the possibly-modified instance) if needed
        // This ensures that init() and other methods that modify the instance
        // return the updated instance pointer to the caller
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            // Return the current masel value (may have been reallocated)
            let masel_val = self
                .builder
                .build_load(self.types.value_type, masel_alloca, "return_masel")
                .map_err(|e| HaversError::CompileError(format!("Failed to load masel: {}", e)))?;
            self.builder
                .build_return(Some(&masel_val))
                .map_err(|e| HaversError::CompileError(format!("Failed to build return: {}", e)))?;
        }

        // Restore state
        self.current_function = old_function;
        self.variables = old_variables;
        self.int_shadows = old_int_shadows;
        self.var_types = old_var_types;
        self.current_masel = old_masel;

        Ok(())
    }

    /// Create a new instance of a class
    fn compile_class_instantiation(
        &mut self,
        class_name: &str,
        args: &[Expr],
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Allocate instance memory: [class_name_ptr][field_count=0]
        // Start with just header, fields will be added by init method
        let header_size = self.types.i64_type.const_int(16, false); // class_name_ptr + field_count
        let instance_ptr = self
            .builder
            .build_call(self.libc.malloc, &[header_size.into()], "instance_alloc")
            .map_err(|e| HaversError::CompileError(format!("Failed to malloc instance: {}", e)))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| HaversError::CompileError("malloc returned void".to_string()))?
            .into_pointer_value();

        let i64_ptr_type = self.types.i64_type.ptr_type(AddressSpace::default());

        // Store class name pointer
        let class_name_global = self
            .classes
            .get(class_name)
            .ok_or_else(|| HaversError::CompileError(format!("Unknown class: {}", class_name)))?;
        let class_name_ptr_slot = self
            .builder
            .build_pointer_cast(instance_ptr, i64_ptr_type, "class_name_slot")
            .unwrap();
        let class_name_int = self
            .builder
            .build_ptr_to_int(
                class_name_global.as_pointer_value(),
                self.types.i64_type,
                "class_name_int",
            )
            .unwrap();
        self.builder
            .build_store(class_name_ptr_slot, class_name_int)
            .unwrap();

        // Store field count = 0
        let field_count_offset = self.types.i64_type.const_int(8, false);
        let field_count_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type(),
                    instance_ptr,
                    &[field_count_offset],
                    "field_count_ptr",
                )
                .unwrap()
        };
        let field_count_i64_ptr = self
            .builder
            .build_pointer_cast(field_count_ptr, i64_ptr_type, "field_count_i64")
            .unwrap();
        let zero = self.types.i64_type.const_int(0, false);
        self.builder.build_store(field_count_i64_ptr, zero).unwrap();

        // Create instance value
        let instance = self.make_instance(instance_ptr)?;

        // Call init method if it exists
        let init_func_name = format!("{}_init", class_name);
        if let Some(&init_func) = self.functions.get(&init_func_name) {
            // Compile arguments
            let mut call_args: Vec<BasicMetadataValueEnum> = vec![instance.into()];
            for arg in args {
                let arg_val = self.compile_expr(arg)?;
                call_args.push(arg_val.into());
            }

            // Call init - it may modify the instance via masel.field = value
            // init returns the (possibly reallocated) instance, which we must use
            let init_result = self
                .builder
                .build_call(init_func, &call_args, "init_result")
                .map_err(|e| HaversError::CompileError(format!("Failed to call init: {}", e)))?
                .try_as_basic_value()
                .left()
                .ok_or_else(|| {
                    HaversError::CompileError("init returned void".to_string())
                })?;
            return Ok(init_result);
        }

        Ok(instance)
    }

    /// Compile a method call: obj.method(args)
    fn compile_method_call(
        &mut self,
        object: &Expr,
        method_name: &str,
        args: &[Expr],
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Compile the object (instance)
        let instance = self.compile_expr(object)?;

        // Try to find the method
        let mut found_method: Option<FunctionValue<'ctx>> = None;

        // If we're currently compiling a class, check if method is from current class first
        if let Some(ref current_class) = self.current_class.clone() {
            let func_name = format!("{}_{}", current_class, method_name);
            if let Some(&func) = self.functions.get(&func_name) {
                found_method = Some(func);
            }
        }

        // Check in class_methods table
        if found_method.is_none() {
            for methods in self.class_methods.values() {
                for (name, func) in methods {
                    if name == method_name {
                        found_method = Some(*func);
                        break;
                    }
                }
                if found_method.is_some() {
                    break;
                }
            }
        }

        // Also check directly in functions map with class prefixes
        if found_method.is_none() {
            for class_name in self.classes.clone().keys() {
                let func_name = format!("{}_{}", class_name, method_name);
                if let Some(&func) = self.functions.get(&func_name) {
                    found_method = Some(func);
                    break;
                }
            }
        }

        let method_func = found_method.ok_or_else(|| {
            HaversError::CompileError(format!("Method '{}' not found", method_name))
        })?;

        // Build call arguments: instance first, then regular args
        let mut call_args: Vec<BasicMetadataValueEnum> = vec![instance.into()];
        for arg in args {
            let arg_val = self.compile_expr(arg)?;
            call_args.push(arg_val.into());
        }

        // Call the method
        let result = self
            .builder
            .build_call(method_func, &call_args, "method_result")
            .map_err(|e| HaversError::CompileError(format!("Failed to call method: {}", e)))?
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| self.make_nil());

        Ok(result)
    }

    /// jammy(min, max) - random number between min and max (inclusive)
    fn inline_jammy(
        &mut self,
        min_val: BasicValueEnum<'ctx>,
        max_val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, HaversError> {
        // Extract integer data from both arguments
        let min_data = self.extract_data(min_val)?;
        let max_data = self.extract_data(max_val)?;

        // Get or declare the __mdh_random function
        let random_fn = self
            .module
            .get_function("__mdh_random")
            .ok_or_else(|| HaversError::CompileError("__mdh_random not found".to_string()))?;

        // Call __mdh_random(min, max)
        let result = self
            .builder
            .build_call(
                random_fn,
                &[min_data.into(), max_data.into()],
                "random_result",
            )
            .map_err(|e| HaversError::CompileError(format!("Failed to call __mdh_random: {}", e)))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| HaversError::CompileError("__mdh_random returned void".to_string()))?;

        Ok(result)
    }

    /// get_key() - read a single key press from terminal
    fn inline_get_key(&mut self) -> Result<BasicValueEnum<'ctx>, HaversError> {
        let get_key_fn = self
            .module
            .get_function("__mdh_get_key")
            .ok_or_else(|| HaversError::CompileError("__mdh_get_key not found".to_string()))?;

        let result = self
            .builder
            .build_call(get_key_fn, &[], "key_result")
            .map_err(|e| HaversError::CompileError(format!("Failed to call __mdh_get_key: {}", e)))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| HaversError::CompileError("__mdh_get_key returned void".to_string()))?;

        Ok(result)
    }
}
