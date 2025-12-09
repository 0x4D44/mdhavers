//! Runtime Function Declarations
//!
//! Declares external C runtime functions that provide the mdhavers runtime.

use inkwell::module::Module;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

use super::types::MdhTypes;

/// Collection of runtime function declarations
pub struct RuntimeFunctions<'ctx> {
    // Value creation
    pub make_nil: FunctionValue<'ctx>,
    pub make_bool: FunctionValue<'ctx>,
    pub make_int: FunctionValue<'ctx>,
    pub make_float: FunctionValue<'ctx>,
    pub make_string: FunctionValue<'ctx>,
    pub make_list: FunctionValue<'ctx>,

    // Arithmetic
    pub add: FunctionValue<'ctx>,
    pub sub: FunctionValue<'ctx>,
    pub mul: FunctionValue<'ctx>,
    pub div: FunctionValue<'ctx>,
    pub modulo: FunctionValue<'ctx>,
    pub neg: FunctionValue<'ctx>,

    // Comparison
    pub eq: FunctionValue<'ctx>,
    pub ne: FunctionValue<'ctx>,
    pub lt: FunctionValue<'ctx>,
    pub le: FunctionValue<'ctx>,
    pub gt: FunctionValue<'ctx>,
    pub ge: FunctionValue<'ctx>,

    // Logical
    pub not: FunctionValue<'ctx>,
    pub truthy: FunctionValue<'ctx>,

    // Type operations
    pub get_tag: FunctionValue<'ctx>,
    pub type_of: FunctionValue<'ctx>,

    // I/O
    pub blether: FunctionValue<'ctx>,
    pub speir: FunctionValue<'ctx>,
    pub get_key: FunctionValue<'ctx>,

    // List operations
    pub list_get: FunctionValue<'ctx>,
    pub list_set: FunctionValue<'ctx>,
    pub list_push: FunctionValue<'ctx>,
    pub list_pop: FunctionValue<'ctx>,
    pub len: FunctionValue<'ctx>,

    // String operations
    pub str_concat: FunctionValue<'ctx>,
    pub to_string: FunctionValue<'ctx>,
    pub to_int: FunctionValue<'ctx>,
    pub to_float: FunctionValue<'ctx>,

    // Math
    pub abs: FunctionValue<'ctx>,
    pub random: FunctionValue<'ctx>,
    pub floor: FunctionValue<'ctx>,
    pub ceil: FunctionValue<'ctx>,
    pub round: FunctionValue<'ctx>,
}

impl<'ctx> RuntimeFunctions<'ctx> {
    pub fn declare(module: &Module<'ctx>, types: &MdhTypes<'ctx>) -> Self {
        let _context = module.get_context();
        let value_type = types.value_type;
        let i8_type = types.i8_type;
        let i32_type = types.i32_type;
        let i64_type = types.i64_type;
        let f64_type = types.f64_type;
        let bool_type = types.bool_type;
        let void_type = types.void_type;
        let str_ptr = i8_type.ptr_type(AddressSpace::default());

        // Value creation functions
        let make_nil = module.add_function("__mdh_make_nil", value_type.fn_type(&[], false), None);

        let make_bool = module.add_function(
            "__mdh_make_bool",
            value_type.fn_type(&[bool_type.into()], false),
            None,
        );

        let make_int = module.add_function(
            "__mdh_make_int",
            value_type.fn_type(&[i64_type.into()], false),
            None,
        );

        let make_float = module.add_function(
            "__mdh_make_float",
            value_type.fn_type(&[f64_type.into()], false),
            None,
        );

        let make_string = module.add_function(
            "__mdh_make_string",
            value_type.fn_type(&[str_ptr.into()], false),
            None,
        );

        let make_list = module.add_function(
            "__mdh_make_list",
            value_type.fn_type(&[i32_type.into()], false),
            None,
        );

        // Arithmetic functions
        let add = module.add_function(
            "__mdh_add",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let sub = module.add_function(
            "__mdh_sub",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let mul = module.add_function(
            "__mdh_mul",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let div = module.add_function(
            "__mdh_div",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let modulo = module.add_function(
            "__mdh_mod",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let neg = module.add_function(
            "__mdh_neg",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        // Comparison functions
        let eq = module.add_function(
            "__mdh_eq",
            bool_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let ne = module.add_function(
            "__mdh_ne",
            bool_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let lt = module.add_function(
            "__mdh_lt",
            bool_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let le = module.add_function(
            "__mdh_le",
            bool_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let gt = module.add_function(
            "__mdh_gt",
            bool_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let ge = module.add_function(
            "__mdh_ge",
            bool_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        // Logical functions
        let not = module.add_function(
            "__mdh_not",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let truthy = module.add_function(
            "__mdh_truthy",
            bool_type.fn_type(&[value_type.into()], false),
            None,
        );

        // Type operations
        let get_tag = module.add_function(
            "__mdh_get_tag",
            i8_type.fn_type(&[value_type.into()], false),
            None,
        );

        let type_of = module.add_function(
            "__mdh_type_of",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        // I/O
        let blether = module.add_function(
            "__mdh_blether",
            void_type.fn_type(&[value_type.into()], false),
            None,
        );

        let speir = module.add_function(
            "__mdh_speir",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let get_key = module.add_function("__mdh_get_key", value_type.fn_type(&[], false), None);

        // List operations
        let list_get = module.add_function(
            "__mdh_list_get",
            value_type.fn_type(&[value_type.into(), i64_type.into()], false),
            None,
        );

        let list_set = module.add_function(
            "__mdh_list_set",
            void_type.fn_type(
                &[value_type.into(), i64_type.into(), value_type.into()],
                false,
            ),
            None,
        );

        let list_push = module.add_function(
            "__mdh_list_push",
            void_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let list_pop = module.add_function(
            "__mdh_list_pop",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let len = module.add_function(
            "__mdh_len",
            i64_type.fn_type(&[value_type.into()], false),
            None,
        );

        // String operations
        let str_concat = module.add_function(
            "__mdh_str_concat",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let to_string = module.add_function(
            "__mdh_to_string",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let to_int = module.add_function(
            "__mdh_to_int",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let to_float = module.add_function(
            "__mdh_to_float",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        // Math
        let abs = module.add_function(
            "__mdh_abs",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let random = module.add_function(
            "__mdh_random",
            value_type.fn_type(&[i64_type.into(), i64_type.into()], false),
            None,
        );

        let floor = module.add_function(
            "__mdh_floor",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let ceil = module.add_function(
            "__mdh_ceil",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let round = module.add_function(
            "__mdh_round",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        RuntimeFunctions {
            make_nil,
            make_bool,
            make_int,
            make_float,
            make_string,
            make_list,
            add,
            sub,
            mul,
            div,
            modulo,
            neg,
            eq,
            ne,
            lt,
            le,
            gt,
            ge,
            not,
            truthy,
            get_tag,
            type_of,
            blether,
            speir,
            get_key,
            list_get,
            list_set,
            list_push,
            list_pop,
            len,
            str_concat,
            to_string,
            to_int,
            to_float,
            abs,
            random,
            floor,
            ceil,
            round,
        }
    }
}
