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

    // Bytes operations
    pub bytes_new: FunctionValue<'ctx>,
    pub bytes_from_string: FunctionValue<'ctx>,
    pub bytes_len: FunctionValue<'ctx>,
    pub bytes_slice: FunctionValue<'ctx>,
    pub bytes_get: FunctionValue<'ctx>,
    pub bytes_set: FunctionValue<'ctx>,
    pub bytes_append: FunctionValue<'ctx>,
    pub bytes_read_u16be: FunctionValue<'ctx>,
    pub bytes_read_u32be: FunctionValue<'ctx>,
    pub bytes_write_u16be: FunctionValue<'ctx>,
    pub bytes_write_u32be: FunctionValue<'ctx>,

    // Math
    pub abs: FunctionValue<'ctx>,
    pub random: FunctionValue<'ctx>,
    pub floor: FunctionValue<'ctx>,
    pub ceil: FunctionValue<'ctx>,
    pub round: FunctionValue<'ctx>,

    // Timing
    pub mono_ms: FunctionValue<'ctx>,
    pub mono_ns: FunctionValue<'ctx>,

    // Audio
    pub soond_stairt: FunctionValue<'ctx>,
    pub soond_steek: FunctionValue<'ctx>,
    pub soond_wheesht: FunctionValue<'ctx>,
    pub soond_luid: FunctionValue<'ctx>,
    pub soond_hou_luid: FunctionValue<'ctx>,
    pub soond_haud_gang: FunctionValue<'ctx>,
    pub soond_lade: FunctionValue<'ctx>,
    pub soond_spiel: FunctionValue<'ctx>,
    pub soond_haud: FunctionValue<'ctx>,
    pub soond_gae_on: FunctionValue<'ctx>,
    pub soond_stap: FunctionValue<'ctx>,
    pub soond_unlade: FunctionValue<'ctx>,
    pub soond_is_spielin: FunctionValue<'ctx>,
    pub soond_pit_luid: FunctionValue<'ctx>,
    pub soond_pit_pan: FunctionValue<'ctx>,
    pub soond_pit_tune: FunctionValue<'ctx>,
    pub soond_pit_rin_roond: FunctionValue<'ctx>,
    pub soond_ready: FunctionValue<'ctx>,

    pub muisic_lade: FunctionValue<'ctx>,
    pub muisic_spiel: FunctionValue<'ctx>,
    pub muisic_haud: FunctionValue<'ctx>,
    pub muisic_gae_on: FunctionValue<'ctx>,
    pub muisic_stap: FunctionValue<'ctx>,
    pub muisic_unlade: FunctionValue<'ctx>,
    pub muisic_is_spielin: FunctionValue<'ctx>,
    pub muisic_loup: FunctionValue<'ctx>,
    pub muisic_hou_lang: FunctionValue<'ctx>,
    pub muisic_whaur: FunctionValue<'ctx>,
    pub muisic_pit_luid: FunctionValue<'ctx>,
    pub muisic_pit_pan: FunctionValue<'ctx>,
    pub muisic_pit_tune: FunctionValue<'ctx>,
    pub muisic_pit_rin_roond: FunctionValue<'ctx>,

    pub midi_lade: FunctionValue<'ctx>,
    pub midi_spiel: FunctionValue<'ctx>,
    pub midi_haud: FunctionValue<'ctx>,
    pub midi_gae_on: FunctionValue<'ctx>,
    pub midi_stap: FunctionValue<'ctx>,
    pub midi_unlade: FunctionValue<'ctx>,
    pub midi_is_spielin: FunctionValue<'ctx>,
    pub midi_loup: FunctionValue<'ctx>,
    pub midi_hou_lang: FunctionValue<'ctx>,
    pub midi_whaur: FunctionValue<'ctx>,
    pub midi_pit_luid: FunctionValue<'ctx>,
    pub midi_pit_pan: FunctionValue<'ctx>,
    pub midi_pit_rin_roond: FunctionValue<'ctx>,
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

        // Bytes operations
        let bytes_new = module.add_function(
            "__mdh_bytes_new",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let bytes_from_string = module.add_function(
            "__mdh_bytes_from_string",
            value_type.fn_type(&[value_type.into()], false),
            None,
        );

        let bytes_len = module.add_function(
            "__mdh_bytes_len",
            i64_type.fn_type(&[value_type.into()], false),
            None,
        );

        let bytes_slice = module.add_function(
            "__mdh_bytes_slice",
            value_type.fn_type(
                &[value_type.into(), value_type.into(), value_type.into()],
                false,
            ),
            None,
        );

        let bytes_get = module.add_function(
            "__mdh_bytes_get",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let bytes_set = module.add_function(
            "__mdh_bytes_set",
            value_type.fn_type(
                &[value_type.into(), value_type.into(), value_type.into()],
                false,
            ),
            None,
        );

        let bytes_append = module.add_function(
            "__mdh_bytes_append",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let bytes_read_u16be = module.add_function(
            "__mdh_bytes_read_u16be",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let bytes_read_u32be = module.add_function(
            "__mdh_bytes_read_u32be",
            value_type.fn_type(&[value_type.into(), value_type.into()], false),
            None,
        );

        let bytes_write_u16be = module.add_function(
            "__mdh_bytes_write_u16be",
            value_type.fn_type(
                &[value_type.into(), value_type.into(), value_type.into()],
                false,
            ),
            None,
        );

        let bytes_write_u32be = module.add_function(
            "__mdh_bytes_write_u32be",
            value_type.fn_type(
                &[value_type.into(), value_type.into(), value_type.into()],
                false,
            ),
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

        // Timing
        let mono_ms = module.add_function("__mdh_mono_ms", value_type.fn_type(&[], false), None);

        let mono_ns = module.add_function("__mdh_mono_ns", value_type.fn_type(&[], false), None);

        // Audio
        let audio_0_type = value_type.fn_type(&[], false);
        let audio_1_type = value_type.fn_type(&[value_type.into()], false);
        let audio_2_type = value_type.fn_type(&[value_type.into(), value_type.into()], false);

        let soond_stairt = module.add_function("__mdh_soond_stairt", audio_0_type, None);
        let soond_steek = module.add_function("__mdh_soond_steek", audio_0_type, None);
        let soond_wheesht = module.add_function("__mdh_soond_wheesht", audio_1_type, None);
        let soond_luid = module.add_function("__mdh_soond_luid", audio_1_type, None);
        let soond_hou_luid = module.add_function("__mdh_soond_hou_luid", audio_0_type, None);
        let soond_haud_gang = module.add_function("__mdh_soond_haud_gang", audio_0_type, None);
        let soond_lade = module.add_function("__mdh_soond_lade", audio_1_type, None);
        let soond_spiel = module.add_function("__mdh_soond_spiel", audio_1_type, None);
        let soond_haud = module.add_function("__mdh_soond_haud", audio_1_type, None);
        let soond_gae_on = module.add_function("__mdh_soond_gae_on", audio_1_type, None);
        let soond_stap = module.add_function("__mdh_soond_stap", audio_1_type, None);
        let soond_unlade = module.add_function("__mdh_soond_unlade", audio_1_type, None);
        let soond_is_spielin = module.add_function("__mdh_soond_is_spielin", audio_1_type, None);
        let soond_pit_luid = module.add_function("__mdh_soond_pit_luid", audio_2_type, None);
        let soond_pit_pan = module.add_function("__mdh_soond_pit_pan", audio_2_type, None);
        let soond_pit_tune = module.add_function("__mdh_soond_pit_tune", audio_2_type, None);
        let soond_pit_rin_roond =
            module.add_function("__mdh_soond_pit_rin_roond", audio_2_type, None);
        let soond_ready = module.add_function("__mdh_soond_ready", audio_1_type, None);

        let muisic_lade = module.add_function("__mdh_muisic_lade", audio_1_type, None);
        let muisic_spiel = module.add_function("__mdh_muisic_spiel", audio_1_type, None);
        let muisic_haud = module.add_function("__mdh_muisic_haud", audio_1_type, None);
        let muisic_gae_on = module.add_function("__mdh_muisic_gae_on", audio_1_type, None);
        let muisic_stap = module.add_function("__mdh_muisic_stap", audio_1_type, None);
        let muisic_unlade = module.add_function("__mdh_muisic_unlade", audio_1_type, None);
        let muisic_is_spielin = module.add_function("__mdh_muisic_is_spielin", audio_1_type, None);
        let muisic_loup = module.add_function("__mdh_muisic_loup", audio_2_type, None);
        let muisic_hou_lang = module.add_function("__mdh_muisic_hou_lang", audio_1_type, None);
        let muisic_whaur = module.add_function("__mdh_muisic_whaur", audio_1_type, None);
        let muisic_pit_luid = module.add_function("__mdh_muisic_pit_luid", audio_2_type, None);
        let muisic_pit_pan = module.add_function("__mdh_muisic_pit_pan", audio_2_type, None);
        let muisic_pit_tune = module.add_function("__mdh_muisic_pit_tune", audio_2_type, None);
        let muisic_pit_rin_roond =
            module.add_function("__mdh_muisic_pit_rin_roond", audio_2_type, None);

        let midi_lade = module.add_function("__mdh_midi_lade", audio_2_type, None);
        let midi_spiel = module.add_function("__mdh_midi_spiel", audio_1_type, None);
        let midi_haud = module.add_function("__mdh_midi_haud", audio_1_type, None);
        let midi_gae_on = module.add_function("__mdh_midi_gae_on", audio_1_type, None);
        let midi_stap = module.add_function("__mdh_midi_stap", audio_1_type, None);
        let midi_unlade = module.add_function("__mdh_midi_unlade", audio_1_type, None);
        let midi_is_spielin = module.add_function("__mdh_midi_is_spielin", audio_1_type, None);
        let midi_loup = module.add_function("__mdh_midi_loup", audio_2_type, None);
        let midi_hou_lang = module.add_function("__mdh_midi_hou_lang", audio_1_type, None);
        let midi_whaur = module.add_function("__mdh_midi_whaur", audio_1_type, None);
        let midi_pit_luid = module.add_function("__mdh_midi_pit_luid", audio_2_type, None);
        let midi_pit_pan = module.add_function("__mdh_midi_pit_pan", audio_2_type, None);
        let midi_pit_rin_roond =
            module.add_function("__mdh_midi_pit_rin_roond", audio_2_type, None);

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
            bytes_new,
            bytes_from_string,
            bytes_len,
            bytes_slice,
            bytes_get,
            bytes_set,
            bytes_append,
            bytes_read_u16be,
            bytes_read_u32be,
            bytes_write_u16be,
            bytes_write_u32be,
            abs,
            random,
            floor,
            ceil,
            round,
            mono_ms,
            mono_ns,
            soond_stairt,
            soond_steek,
            soond_wheesht,
            soond_luid,
            soond_hou_luid,
            soond_haud_gang,
            soond_lade,
            soond_spiel,
            soond_haud,
            soond_gae_on,
            soond_stap,
            soond_unlade,
            soond_is_spielin,
            soond_pit_luid,
            soond_pit_pan,
            soond_pit_tune,
            soond_pit_rin_roond,
            soond_ready,
            muisic_lade,
            muisic_spiel,
            muisic_haud,
            muisic_gae_on,
            muisic_stap,
            muisic_unlade,
            muisic_is_spielin,
            muisic_loup,
            muisic_hou_lang,
            muisic_whaur,
            muisic_pit_luid,
            muisic_pit_pan,
            muisic_pit_tune,
            muisic_pit_rin_roond,
            midi_lade,
            midi_spiel,
            midi_haud,
            midi_gae_on,
            midi_stap,
            midi_unlade,
            midi_is_spielin,
            midi_loup,
            midi_hou_lang,
            midi_whaur,
            midi_pit_luid,
            midi_pit_pan,
            midi_pit_rin_roond,
        }
    }
}
