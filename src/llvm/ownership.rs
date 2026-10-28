//! Ownership table for the native runtime ABI (RC prerequisite R0.5).
//!
//! The reference-counting design (`wrk_docs/2026.10.23 - DES - LLVM 18-19 upgrade and
//! reference counting.md`, section 3.3) adopts the Python C-API convention: arguments
//! are borrowed and results are owned (+1). This table records what every `__mdh_*`
//! symbol that codegen declares does with its `MdhValue` arguments and result TODAY, so
//! stage R1 knows where the convention needs a retain. It is the one place for that
//! information; the tests below keep it in step with codegen's declarations and with
//! the C and Rust runtimes. Nothing reads it at run time yet.
//!
//! Each row is `(symbol, arg codes, result)`. Arg codes, one per declared parameter:
//! - `-` not an `MdhValue` (an integer, double, flag or raw pointer);
//! - `b` borrowed: only read during the call;
//! - `m` borrowed, and its heap object is mutated in place (push, insert, field set);
//! - `s` stored: the callee keeps the value (container, closure, global, last error,
//!   thread, channel, handle table) beyond the call, so under RC it must retain it.
//!
//! No row may consume or free an argument object (drop the caller's reference to it):
//! RC borrowing relies on that (`rc_touches_slot` in `codegen/rc.rs`). The release
//! functions are the only exception, and borrowing recognises them by name; the test
//! `no_row_consumes_its_argument` keeps new rows honest.

/// What a runtime function's result is, as far as ownership goes. Where several
/// apply, the first in declaration order wins (the one that needs the most care).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ret {
    /// May return a value that is also held elsewhere (a container element, a field,
    /// a global, the last error, a thread's cached result). R1 must retain it.
    Stored,
    /// May return one of its own arguments, or an object an argument references.
    /// R1 must retain it.
    Arg,
    /// A raw pointer into an argument's payload (`const char*`, `MdhList*`): borrowed,
    /// it must not outlive that argument.
    Interior,
    /// May return a value whose payload is static data (not heap); it needs an
    /// immortal header before R1.
    Static,
    /// A heap object this call allocated: already owned (+1).
    Fresh,
    /// Nothing counted: void, an immediate value, a raw integer or a handle index.
    None,
}

/// `(symbol, arg codes, result)` for every runtime symbol codegen declares, sorted
/// by symbol.
pub const RUNTIME_OWNERSHIP: &[(&str, &str, Ret)] = &[
    ("__mdh_args", "", Ret::Fresh),
    ("__mdh_assert", "bs", Ret::None), // msg stored as last error (if string) via hurl
    ("__mdh_assert_equal", "bb", Ret::None), // aye, or hurls a fresh error string
    ("__mdh_assert_fail", "b-", Ret::None), // hurls a fresh "Mak siccar failed at line N!" string
    ("__mdh_atomic_add", "bb", Ret::None), // handles are int table indices
    ("__mdh_atomic_cas", "bbb", Ret::None),
    ("__mdh_atomic_load", "b", Ret::None),
    ("__mdh_atomic_new", "b", Ret::None),
    ("__mdh_atomic_store", "bb", Ret::None),
    ("__mdh_average", "b", Ret::None),
    ("__mdh_bampot_mode", "b", Ret::Fresh), // shares elements of arg 1
    ("__mdh_banter", "bb", Ret::Fresh),
    ("__mdh_bit_and", "bb", Ret::None),
    ("__mdh_bit_or", "bb", Ret::None),
    ("__mdh_bit_xor", "bb", Ret::None),
    ("__mdh_blether", "b", Ret::None),
    ("__mdh_blether_format", "bb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_blooter", "b", Ret::Fresh),         // R1: +1 on every path
    ("__mdh_bonnie", "b", Ret::Fresh),
    ("__mdh_braw", "b", Ret::None),
    ("__mdh_braw_date", "b", Ret::Fresh),
    ("__mdh_braw_time", "", Ret::Fresh),
    // The single allocator (runtime/gc_stub.c): raw memory, no MdhValue in or out.
    ("__mdh_buf_alloc", "-", Ret::None),
    ("__mdh_buf_alloc_checked", "-", Ret::None), // raises "ower muckle"; declared on first use
    ("__mdh_buf_realloc", "--", Ret::None),
    ("__mdh_buf_realloc_checked", "--", Ret::None), // raises "ower muckle"; declared on first use
    ("__mdh_builtin_error", "-", Ret::None),        // never returns; declared on first use
    ("__mdh_builtin_type_error", "-", Ret::None),   // never returns; declared on first use
    ("__mdh_bytes_append", "mb", Ret::Arg),         // returns bytes arg; bytes mutated in place
    ("__mdh_bytes_from_string", "b", Ret::Fresh),
    ("__mdh_bytes_get", "bb", Ret::None),
    ("__mdh_bytes_len", "b", Ret::None),
    ("__mdh_bytes_new", "b", Ret::Fresh),
    ("__mdh_bytes_read_u16be", "bb", Ret::None),
    ("__mdh_bytes_read_u32be", "bb", Ret::None),
    ("__mdh_bytes_set", "mbb", Ret::Arg), // returns bytes arg; mutated in place
    ("__mdh_bytes_slice", "bbb", Ret::Fresh), // byte copy, no MdhValue elements
    ("__mdh_bytes_write_u16be", "mbb", Ret::Arg), // returns bytes arg; mutated in place
    ("__mdh_bytes_write_u32be", "mbb", Ret::Arg), // returns bytes arg; mutated in place
    ("__mdh_cannie", "b", Ret::None),
    ("__mdh_capitalize", "b", Ret::Fresh),
    ("__mdh_center", "bb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_chan_close", "b", Ret::None),
    ("__mdh_chan_is_closed", "b", Ret::None),
    ("__mdh_chan_new", "b", Ret::None),
    ("__mdh_chan_recv", "b", Ret::Fresh), // R1: +1 on every path
    ("__mdh_chan_send", "bs", Ret::None), // value stored in channel buffer
    ("__mdh_chan_try_recv", "b", Ret::Fresh), // R1: +1 on every path
    ("__mdh_char_at", "bb", Ret::Fresh),
    ("__mdh_chars", "b", Ret::Fresh),
    ("__mdh_chdir", "b", Ret::None),
    ("__mdh_chr", "b", Ret::Fresh),
    ("__mdh_chunks", "bb", Ret::Fresh), // shares elements of arg 1
    ("__mdh_chynge", "bbs", Ret::Fresh), // list insert copy; shares elements of arg 1
    ("__mdh_clarty", "b", Ret::None),
    ("__mdh_clype", "b", Ret::Fresh),
    ("__mdh_condvar_broadcast", "b", Ret::None),
    ("__mdh_condvar_new", "", Ret::None),
    ("__mdh_condvar_signal", "b", Ret::None),
    ("__mdh_condvar_timed_wait", "bbb", Ret::None),
    ("__mdh_condvar_wait", "bb", Ret::None),
    ("__mdh_contains", "bb", Ret::None),
    ("__mdh_count_val", "bb", Ret::None),
    ("__mdh_crabbit", "b", Ret::None),
    ("__mdh_creel_tae_list", "b", Ret::Fresh), // shares elements of arg 1
    ("__mdh_creels_baith", "bb", Ret::Fresh),  // shares elements of args 1,2
    ("__mdh_creels_differ", "bb", Ret::Fresh), // shares elements of args 1,2
    ("__mdh_creels_thegither", "bb", Ret::Fresh), // shares elements of args 1,2
    ("__mdh_crivvens", "b", Ret::Fresh),
    ("__mdh_cwd", "", Ret::Fresh),
    ("__mdh_date_add", "bbb", Ret::None),
    ("__mdh_date_diff", "bbb", Ret::None),
    ("__mdh_date_format", "bb", Ret::Fresh),
    ("__mdh_date_now", "", Ret::Fresh),
    ("__mdh_date_parse", "bb", Ret::None),
    ("__mdh_dicht", "bb", Ret::Fresh), // list copy minus one; shares elements of arg 1
    ("__mdh_dict_contains", "bb", Ret::None),
    ("__mdh_dict_get", "bb", Ret::Stored), // returns the stored value in the dict
    ("__mdh_dict_get_default", "bbb", Ret::Stored), // may return default (arg 3) when key missing
    ("__mdh_dict_invert", "b", Ret::Fresh), // shares keys and values of arg 1
    ("__mdh_dict_keys", "b", Ret::Fresh),  // shares elements of arg 1
    ("__mdh_dict_merge", "bb", Ret::Fresh), // shares keys and values of args 1,2
    ("__mdh_dict_remove", "bb", Ret::Fresh), // new dict; shares entries of arg 1
    ("__mdh_dict_set", "mss", Ret::Arg),   // in place; returns dict arg
    ("__mdh_dict_values", "b", Ret::Fresh), // shares elements of arg 1
    ("__mdh_div_by_zero", "", Ret::None),  // never returns; declared on first use
    ("__mdh_dns_lookup", "b", Ret::Fresh),
    ("__mdh_dns_naptr", "b", Ret::Fresh),
    ("__mdh_dns_srv", "bb", Ret::Fresh),
    ("__mdh_drap", "bb", Ret::Fresh), // shares elements of arg 1
    ("__mdh_dreich", "b", Ret::None),
    ("__mdh_drookit", "b", Ret::None),
    ("__mdh_dtls_handshake", "bb", Ret::Fresh), // int handles; copies dup(fd)
    ("__mdh_dtls_server_new", "b", Ret::Fresh), // config copied into Rust strings
    ("__mdh_empty_creel", "", Ret::Fresh),
    ("__mdh_empty_dict", "", Ret::Fresh),
    ("__mdh_ends_with", "bb", Ret::None),
    ("__mdh_env_all", "", Ret::Fresh),
    ("__mdh_env_get", "b", Ret::Fresh),
    ("__mdh_env_set", "bb", Ret::None),
    ("__mdh_eq", "bb", Ret::None),
    ("__mdh_event_loop_new", "", Ret::None),
    ("__mdh_event_loop_poll", "bb", Ret::Fresh), // events embed callbacks held by loop; shares callbacks
    ("__mdh_event_loop_stop", "b", Ret::None),
    ("__mdh_event_unwatch", "bb", Ret::None),
    ("__mdh_event_watch_read", "bbs", Ret::None), // callback kept in loop watch table
    ("__mdh_event_watch_write", "bbs", Ret::None), // callback kept in loop watch table
    ("__mdh_fae_binary", "b", Ret::None),
    ("__mdh_fae_hex", "b", Ret::None),
    ("__mdh_fae_pairs", "b", Ret::Fresh), // shares keys and values of arg 1
    ("__mdh_file_delete", "b", Ret::None),
    ("__mdh_file_exists", "b", Ret::None),
    ("__mdh_file_size", "b", Ret::None),
    ("__mdh_gallus", "b", Ret::None),
    ("__mdh_geggie", "b", Ret::Fresh),
    ("__mdh_get_key", "", Ret::Fresh),
    ("__mdh_get_last_error", "", Ret::Stored), // thread-local last error
    ("__mdh_get_log_level", "", Ret::None),
    ("__mdh_glaikit", "b", Ret::None),
    ("__mdh_grup", "bb", Ret::Fresh),     // shares elements of arg 1
    ("__mdh_grup_runs", "b", Ret::Fresh), // shares elements of arg 1
    ("__mdh_haggis_hunt", "bb", Ret::Fresh),
    ("__mdh_haver", "", Ret::Fresh),
    ("__mdh_haverin", "b", Ret::None),
    ("__mdh_heave_oot", "mb", Ret::Arg), // in place tombstone; returns set arg
    ("__mdh_help_ma_boab", "b", Ret::Fresh),
    ("__mdh_hurl", "s", Ret::None), // msg stored as thread-local last error
    ("__mdh_hurl_at", "b-", Ret::None), // hurls a fresh "Hurled at line N" string
    ("__mdh_hurl_text_at_line", "-", Ret::None), // raw text; hurls a fresh line-bearing string
    ("__mdh_index_oob", "--", Ret::None), // never returns; declared on first use
    ("__mdh_indices_o", "bb", Ret::Fresh),
    ("__mdh_int_overflow_error", "", Ret::None),
    ("__mdh_interleave", "bb", Ret::Fresh), // shares elements of args 1 and 2
    ("__mdh_is_a", "bb", Ret::None),
    ("__mdh_is_blank", "b", Ret::None),
    ("__mdh_is_bool", "b", Ret::None),
    ("__mdh_is_dict", "b", Ret::None),
    ("__mdh_is_digit", "b", Ret::None),
    ("__mdh_is_dir", "b", Ret::None),
    ("__mdh_is_disjoint", "bb", Ret::None),
    ("__mdh_is_float", "b", Ret::None),
    ("__mdh_is_function", "b", Ret::None),
    ("__mdh_is_int", "b", Ret::None),
    ("__mdh_is_list", "b", Ret::None),
    ("__mdh_is_muckle", "b", Ret::None),
    ("__mdh_is_nil", "b", Ret::None),
    ("__mdh_is_space", "b", Ret::None),
    ("__mdh_is_string", "b", Ret::None),
    ("__mdh_is_subset", "bb", Ret::None),
    ("__mdh_is_superset", "bb", Ret::None),
    ("__mdh_is_wee", "b", Ret::None),
    ("__mdh_jammy", "bb", Ret::None),
    ("__mdh_jings", "b", Ret::Fresh),
    ("__mdh_jmp_buf_size", "", Ret::None),
    ("__mdh_join_general", "bb", Ret::Fresh),
    ("__mdh_json_parse", "b", Ret::Fresh),
    ("__mdh_json_pretty", "b", Ret::Fresh),
    ("__mdh_json_stringify", "b", Ret::Fresh),
    ("__mdh_key_down", "b", Ret::None),
    ("__mdh_key_not_found", "b", Ret::None), // hurls fresh message; key only read
    ("__mdh_key_pressed", "b", Ret::None),
    ("__mdh_last_index_of", "bb", Ret::None),
    ("__mdh_leftpad", "bbb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_lines", "b", Ret::Fresh),
    ("__mdh_list_all_strings", "b", Ret::None),
    ("__mdh_list_clear", "m", Ret::Arg), // returns the same list
    ("__mdh_list_contains", "bb", Ret::None),
    ("__mdh_list_copy", "b", Ret::Fresh), // R1: +1 on every path
    ("__mdh_list_dir", "b", Ret::Fresh),
    ("__mdh_list_index", "bb", Ret::None),
    ("__mdh_list_index_of", "bb", Ret::None),
    ("__mdh_list_max", "b", Ret::None), // returns an element, but only Int/Float (immediate)
    ("__mdh_list_min", "b", Ret::None), // returns an element, but only Int/Float (immediate)
    ("__mdh_list_push", "ms", Ret::None),
    ("__mdh_list_slice", "b--", Ret::Fresh), // R1: +1 on every path
    ("__mdh_list_sort", "b", Ret::Fresh),    // shares elements of arg 1
    ("__mdh_list_uniq", "b", Ret::Fresh),    // shares elements of arg 1
    ("__mdh_log_enabled", "bb", Ret::None),
    ("__mdh_log_event", "bbbbbb", Ret::None), // msg/fields go in transient record passed to callback
    ("__mdh_log_get_filter", "", Ret::Fresh),
    ("__mdh_log_init", "b", Ret::None), // global callback = fn element of config.sinks (shares)
    ("__mdh_log_set_filter", "b", Ret::None), // copies the spec string
    ("__mdh_log_span_begin", "sbss", Ret::Fresh), // name/fields/target stored in span object dict
    ("__mdh_log_span_current", "", Ret::Stored), // native span object held by thread-local span stack
    ("__mdh_log_span_enter", "s", Ret::None),    // span pointer pushed on thread-local span stack
    ("__mdh_log_span_exit", "b", Ret::None),     // pops span stack
    ("__mdh_log_span_in", "bb", Ret::Fresh),     // R1: +1 on every path
    ("__mdh_ltrim", "b", Ret::Fresh),            // R1: +1 on every path
    ("__mdh_make_creel", "b", Ret::Fresh),       // shares elements of arg 1 (inserted as set keys)
    ("__mdh_make_dir", "b", Ret::None),
    ("__mdh_make_list", "-", Ret::Fresh),
    ("__mdh_median", "b", Ret::None),
    ("__mdh_midi_gae_on", "b", Ret::None),   // handle is an int
    ("__mdh_midi_haud", "b", Ret::None),     // handle is an int
    ("__mdh_midi_hou_lang", "b", Ret::None), // handle is an int
    ("__mdh_midi_is_spielin", "b", Ret::None), // handle is an int
    ("__mdh_midi_lade", "bb", Ret::None),    // paths copied
    ("__mdh_midi_loup", "bb", Ret::None),
    ("__mdh_midi_pit_luid", "bb", Ret::None),
    ("__mdh_midi_pit_pan", "bb", Ret::None),
    ("__mdh_midi_pit_rin_roond", "bb", Ret::None),
    ("__mdh_midi_spiel", "b", Ret::None),  // handle is an int
    ("__mdh_midi_stap", "b", Ret::None),   // handle is an int
    ("__mdh_midi_unlade", "b", Ret::None), // handle is an int
    ("__mdh_midi_whaur", "b", Ret::None),  // handle is an int
    ("__mdh_mono_ms", "", Ret::None),
    ("__mdh_mono_ns", "", Ret::None),
    ("__mdh_mouse_down", "b", Ret::None),
    ("__mdh_mouse_dx", "", Ret::None),
    ("__mdh_mouse_dy", "", Ret::None),
    ("__mdh_mouse_x", "", Ret::None),
    ("__mdh_mouse_y", "", Ret::None),
    ("__mdh_muckle", "bb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_muisic_gae_on", "b", Ret::None), // handle is an int
    ("__mdh_muisic_haud", "b", Ret::None), // handle is an int
    ("__mdh_muisic_hou_lang", "b", Ret::None), // handle is an int
    ("__mdh_muisic_is_spielin", "b", Ret::None), // handle is an int
    ("__mdh_muisic_lade", "b", Ret::None), // path copied to Rust String
    ("__mdh_muisic_loup", "bb", Ret::None),
    ("__mdh_muisic_pit_luid", "bb", Ret::None),
    ("__mdh_muisic_pit_pan", "bb", Ret::None),
    ("__mdh_muisic_pit_rin_roond", "bb", Ret::None),
    ("__mdh_muisic_pit_tune", "bb", Ret::None),
    ("__mdh_muisic_spiel", "b", Ret::None), // handle is an int
    ("__mdh_muisic_stap", "b", Ret::None),  // handle is an int
    ("__mdh_muisic_unlade", "b", Ret::None), // handle is an int
    ("__mdh_muisic_whaur", "b", Ret::None), // handle is an int
    ("__mdh_mutex_lock", "b", Ret::None),
    ("__mdh_mutex_new", "", Ret::None), // handle is a table index
    ("__mdh_mutex_try_lock", "b", Ret::None),
    ("__mdh_mutex_unlock", "b", Ret::None),
    ("__mdh_mutter", "b", Ret::Fresh),
    ("__mdh_native_call0", "mb", Ret::Fresh), // obj fields may be mutated
    ("__mdh_native_call1", "mbs", Ret::Fresh), // args stored in object fields/state; obj mutated
    ("__mdh_native_call2", "mbss", Ret::Fresh), // args stored in object fields/state; obj mutated
    ("__mdh_native_call3", "mbsss", Ret::Fresh), // args stored in object fields/state; obj mutated
    ("__mdh_native_call4", "mbssss", Ret::Fresh), // args stored in object fields/state; obj mutated
    ("__mdh_native_call5", "mbsssss", Ret::Fresh), // args stored in object fields/state; obj mutated
    ("__mdh_native_call6", "mbssssss", Ret::Fresh), // args stored in object fields/state; obj mutated
    ("__mdh_native_call7", "mbsssssss", Ret::Fresh), // args stored in object fields/state; obj mutated
    ("__mdh_native_call8", "mbssssssss", Ret::Fresh), // args stored in object fields/state; obj mutated
    ("__mdh_native_get", "bb", Ret::Stored), // returns object field value (held in tri state)
    ("__mdh_native_set", "mbs", Ret::Stored), // returns value arg; key copied to Rust string (graphics3d)
    ("__mdh_no_window", "", Ret::None),       // never returns; declared on first use
    ("__mdh_note_line", "-", Ret::None),
    ("__mdh_numpty_check", "b", Ret::Fresh),
    // The single allocator: a raw object payload the caller wraps in a value (the
    // caller's +1 once R1 counts).
    ("__mdh_obj_alloc", "--", Ret::None),
    ("__mdh_obj_alloc_checked", "--", Ret::None), // raises "ower muckle"; declared on first use
    ("__mdh_obj_realloc", "--", Ret::None),
    ("__mdh_obj_realloc_checked", "--", Ret::None), // raises "ower muckle"; declared on first use
    ("__mdh_obj_strdup", "-", Ret::None),
    ("__mdh_obj_strdup_checked", "-", Ret::None), // raises "ower muckle"; declared on first use
    ("__mdh_och", "b", Ret::Fresh),
    ("__mdh_ord", "b", Ret::None),
    ("__mdh_pair_adjacent", "b", Ret::Fresh), // shares elements of arg 1
    ("__mdh_pair_up", "bb", Ret::Fresh),      // shares elements of args 1 and 2
    ("__mdh_path_join", "bb", Ret::Fresh),
    ("__mdh_random", "--", Ret::None),
    ("__mdh_random_int", "bb", Ret::None),
    ("__mdh_range", "---", Ret::Fresh),
    ("__mdh_range_o", "b", Ret::None),
    // Reference counting (runtime/gc_stub.c): turned on by main's prologue (MDH_RC=1), then
    // the object's data after the inline tag test.
    ("__mdh_rc_check", "b-", Ret::None), // debug builds: aborts on a DEAD object
    ("__mdh_rc_enable", "", Ret::None),
    ("__mdh_rc_release_obj", "-", Ret::None),
    ("__mdh_rc_retain_obj", "-", Ret::None),
    ("__mdh_redd_up", "b", Ret::Fresh), // shares elements of arg 1
    ("__mdh_regex_match", "bb", Ret::Fresh),
    ("__mdh_regex_match_all", "bb", Ret::Fresh),
    ("__mdh_regex_replace", "bbb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_regex_replace_first", "bbb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_regex_split", "bb", Ret::Fresh),
    ("__mdh_regex_test", "bb", Ret::None),
    ("__mdh_register_fn_names", "--", Ret::None), // stores a raw pointer to a constant table
    ("__mdh_release", "b", Ret::None),
    ("__mdh_repeat_say", "bb", Ret::Fresh),
    ("__mdh_replace", "bbb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_replace_first", "bbb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_retain", "b", Ret::None),
    ("__mdh_reverse_str", "b", Ret::Fresh),
    ("__mdh_rightpad", "bbb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_rtrim", "b", Ret::Fresh),      // R1: +1 on every path
    ("__mdh_sclaff", "b-", Ret::Fresh),    // a new list; list_push retains the items
    ("__mdh_scottify", "b", Ret::Fresh),   // R1: +1 on every path
    ("__mdh_screen_close", "", Ret::None),
    ("__mdh_screen_grab_mouse", "b", Ret::None),
    ("__mdh_screen_height", "", Ret::None),
    ("__mdh_screen_open", "bbb", Ret::None),
    ("__mdh_screen_shaw", "bb", Ret::None),
    ("__mdh_screen_should_close", "", Ret::None),
    ("__mdh_screen_width", "", Ret::None),
    ("__mdh_scrieve", "bb", Ret::None),
    ("__mdh_scrieve_append", "bb", Ret::None),
    ("__mdh_scunner", "b", Ret::None),
    ("__mdh_scunner_check", "bb", Ret::Fresh), // bool or fresh error string
    ("__mdh_set_args", "--", Ret::None),       // stores raw argv pointer, not an MdhValue
    ("__mdh_set_contains", "bb", Ret::None),
    ("__mdh_set_log_level", "b", Ret::None),
    // Shared slots (globals, captured cells) once threads exist: load retains under the
    // slot lock; store takes the caller's +1 and releases the old value (gc_stub.c).
    ("__mdh_shared_load", "-", Ret::Fresh),
    ("__mdh_shared_store", "-s", Ret::None),
    ("__mdh_shell", "b", Ret::Fresh),
    ("__mdh_shell_status", "b", Ret::None),
    ("__mdh_shuffle", "b", Ret::Fresh), // shares elements of arg 1
    ("__mdh_skelp", "bb", Ret::Fresh),
    ("__mdh_skip", "b", Ret::None),
    ("__mdh_slainte", "", Ret::Fresh),
    ("__mdh_slice_string", "bbbb", Ret::Fresh),
    ("__mdh_slurp", "b", Ret::Fresh),
    ("__mdh_socket_accept", "b", Ret::Fresh), // result dict
    ("__mdh_socket_bind", "bbb", Ret::Fresh), // result dict
    ("__mdh_socket_close", "b", Ret::Fresh),  // result dict
    ("__mdh_socket_connect", "bbb", Ret::Fresh), // result dict
    ("__mdh_socket_listen", "bb", Ret::Fresh), // result dict
    ("__mdh_socket_set_nodelay", "bb", Ret::Fresh), // result dict
    ("__mdh_socket_set_nonblocking", "bb", Ret::Fresh), // result dict
    ("__mdh_socket_set_rcvbuf", "bb", Ret::Fresh), // result dict
    ("__mdh_socket_set_reuseaddr", "bb", Ret::Fresh), // result dict
    ("__mdh_socket_set_reuseport", "bb", Ret::Fresh), // result dict
    ("__mdh_socket_set_sndbuf", "bb", Ret::Fresh), // result dict
    ("__mdh_socket_set_ttl", "bb", Ret::Fresh), // result dict
    ("__mdh_socket_tcp", "", Ret::Fresh),     // result dict
    ("__mdh_socket_udp", "", Ret::Fresh),     // result dict
    ("__mdh_soond_gae_on", "b", Ret::None),
    ("__mdh_soond_haud", "b", Ret::None),
    ("__mdh_soond_haud_gang", "", Ret::None),
    ("__mdh_soond_hou_luid", "", Ret::None),
    ("__mdh_soond_is_spielin", "b", Ret::None),
    ("__mdh_soond_lade", "b", Ret::None),
    ("__mdh_soond_luid", "b", Ret::None),
    ("__mdh_soond_pit_luid", "bb", Ret::None),
    ("__mdh_soond_pit_pan", "bb", Ret::None),
    ("__mdh_soond_pit_rin_roond", "bb", Ret::None),
    ("__mdh_soond_pit_tune", "bb", Ret::None),
    ("__mdh_soond_ready", "b", Ret::None),
    ("__mdh_soond_spiel", "b", Ret::None),
    ("__mdh_soond_stairt", "", Ret::None),
    ("__mdh_soond_stap", "b", Ret::None),
    ("__mdh_soond_steek", "", Ret::None),
    ("__mdh_soond_unlade", "b", Ret::None),
    ("__mdh_soond_wheesht", "b", Ret::None),
    ("__mdh_speir", "b", Ret::Fresh),
    ("__mdh_split_by", "bb", Ret::Fresh), // shares elements of arg 1
    ("__mdh_sporran_fill", "bbb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_srtp_create", "b", Ret::Fresh), // result dict
    ("__mdh_srtp_protect", "bb", Ret::Fresh), // result dict
    ("__mdh_srtp_unprotect", "bb", Ret::Fresh), // result dict
    ("__mdh_stacktrace", "", Ret::Fresh),
    ("__mdh_starts_with", "bb", Ret::None),
    ("__mdh_stoater", "b", Ret::Fresh),  // R1: +1 on every path
    ("__mdh_stooshie", "b", Ret::Fresh), // R1: +1 on every path
    ("__mdh_str_index_of", "bb", Ret::None),
    ("__mdh_str_len", "b", Ret::None),
    ("__mdh_strip_left", "bb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_strip_right", "bb", Ret::Fresh), // R1: +1 on every path
    ("__mdh_substring", "bbb", Ret::Fresh),
    ("__mdh_swapcase", "b", Ret::Fresh),
    ("__mdh_tae_binary", "b", Ret::Fresh),
    ("__mdh_tae_hex", "b", Ret::Fresh),
    ("__mdh_tae_octal", "b", Ret::Fresh),
    ("__mdh_tak", "bb", Ret::Fresh), // shares elements of arg 1
    ("__mdh_tattie_scone", "bb", Ret::Fresh),
    ("__mdh_tcp_recv", "bb", Ret::Fresh), // result dict
    ("__mdh_tcp_send", "bb", Ret::Fresh), // result dict
    ("__mdh_term_height", "", Ret::None),
    ("__mdh_term_width", "", Ret::None),
    ("__mdh_thread_detach", "b", Ret::None),
    ("__mdh_thread_join", "b", Ret::Stored), // cached t->result, returned on every join
    ("__mdh_thread_spawn", "ss", Ret::None), // func and args kept until the worker's function returns; returns handle id
    ("__mdh_timer_after", "bbs", Ret::None), // callback kept in MdhTimer; returns timer id
    ("__mdh_timer_cancel", "bb", Ret::None),
    ("__mdh_timer_every", "bbs", Ret::None), // callback kept in MdhTimer; returns timer id
    ("__mdh_title_case", "b", Ret::Fresh),   // R1: +1 on every path
    ("__mdh_tls_client_new", "b", Ret::Fresh), // result dict; config copied into Rust
    ("__mdh_tls_close", "b", Ret::Fresh),    // result dict
    ("__mdh_tls_connect", "bb", Ret::Fresh), // result dict
    ("__mdh_tls_recv", "bb", Ret::Fresh),    // result dict
    ("__mdh_tls_send", "bb", Ret::Fresh),    // result dict
    ("__mdh_to_float", "b", Ret::None),
    ("__mdh_to_int", "b", Ret::None),
    ("__mdh_to_string", "b", Ret::Fresh), // R1: +1 on every path
    ("__mdh_toss_in", "ms", Ret::Arg),    // returns the set it was given
    ("__mdh_tri_module", "", Ret::Stored), // cached static module native object
    ("__mdh_try_pop", "", Ret::None),
    ("__mdh_try_push", "-", Ret::None), // keeps jmp_buf pointer, not an MdhValue
    ("__mdh_type_error", "---", Ret::None), // raises via hurl with a fresh string
    ("__mdh_type_of", "b", Ret::Fresh),
    ("__mdh_udp_recv_from", "bb", Ret::Fresh), // result dict
    ("__mdh_udp_send_to", "bbbb", Ret::Fresh), // result dict
    ("__mdh_unique", "b", Ret::Fresh),         // shares elements of arg 1
    ("__mdh_wee", "bb", Ret::Fresh),           // R1: +1 on every path
    ("__mdh_wheesht_aw", "b", Ret::Fresh),
    ("__mdh_window", "bb", Ret::Fresh),
    ("__mdh_words", "b", Ret::Fresh),
    ("__mdh_wrang_sort", "bb", Ret::None),
];

/// The ownership row of a runtime symbol: its arg codes and result.
pub fn ownership_of(symbol: &str) -> Option<(&'static str, Ret)> {
    RUNTIME_OWNERSHIP
        .binary_search_by(|(name, _, _)| (*name).cmp(symbol))
        .ok()
        .map(|i| (RUNTIME_OWNERSHIP[i].1, RUNTIME_OWNERSHIP[i].2))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::llvm::codegen::CodeGen;
    use inkwell::context::Context;

    /// Exact source-audited rows whose names trip the destructive-operation lint.
    const AUDITED_BORROWING_REMOVALS: &[(&str, &str, Ret)] = &[
        ("__mdh_env_set", "bb", Ret::None), // releases only __mdh_to_string's retained/fresh temporary
        ("__mdh_file_delete", "b", Ret::None), // reads path; failure hurls a fresh message
        ("__mdh_is_subset", "bb", Ret::None), // scans membership and returns only booleans
        ("__mdh_is_superset", "bb", Ret::None), // delegates with borrowed operands swapped
        ("__mdh_log_set_filter", "b", Ret::None), // validates then copies spec bytes into raw buffer
        ("__mdh_set_args", "--", Ret::None),      // stores argc and raw argv pointer, not MdhValues
        ("__mdh_set_contains", "bb", Ret::None),  // searches entries and returns a boolean
        ("__mdh_set_log_level", "b", Ret::None),  // stores only parsed scalar level in a global
        ("__mdh_try_pop", "", Ret::None), // decrements depth of the raw jmp_buf pointer stack
    ];

    /// RC borrowing treats a runtime function with a row as borrowing its arguments: it
    /// never drops the caller's reference to an argument object. A function that does
    /// (by name: consume, take, drop, destroy, ...) needs a new arg code, not a row. The
    /// release functions (release/free/realloc) have rows but are caught by name in
    /// `rc_touches_slot`, and none of their names contains a word below.
    #[test]
    fn no_row_consumes_its_argument() {
        let consuming = [
            "owned", "destroy", "drop", "kill", "dispose", "take", "consume",
        ];
        for (name, _, _) in RUNTIME_OWNERSHIP {
            assert!(
                !consuming.iter().any(|w| name.contains(w)),
                "{name} looks like it consumes an argument: RC borrowing assumes rows borrow"
            );
        }
    }

    /// Name lint for RC7 summaries: a destructive-sounding operation must declare
    /// mutation/storage, or return a copy while borrowing every argument.
    #[test]
    fn no_borrowing_row_removes() {
        let removing = [
            "remove", "pop", "set", "insert", "clear", "delete", "heave", "toss", "drain", "swap",
            "replace",
        ];
        for row in AUDITED_BORROWING_REMOVALS {
            assert!(
                RUNTIME_OWNERSHIP.contains(row),
                "stale audited row: {} ({}, {:?})",
                row.0,
                row.1,
                row.2
            );
        }
        let problems: Vec<_> = RUNTIME_OWNERSHIP
            .iter()
            .filter(|(name, args, ret)| {
                removing.iter().any(|word| name.contains(word))
                    && !args.contains(['m', 's'])
                    && !(*ret == Ret::Fresh && args.chars().all(|arg| arg == 'b'))
                    && !AUDITED_BORROWING_REMOVALS.contains(&(*name, *args, *ret))
            })
            .map(|(name, args, ret)| {
                format!("{name} ({args}, {ret:?}) looks destructive but only borrows")
            })
            .collect();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    /// Uses the runtime helpers codegen declares only on first use: division (by
    /// zero), indexing (out of bounds), `hurl` with a line and split's string copy.
    const LAZY_DECLS: &str = r#"
dae f(xs, n) {
    gin n == 0 {
        hurl "nae"
    }
    gie xs[n] / n
}
blether f([1, 2], 1)
blether join(["a"], ",")
blether aw([1], |x| x)
mak_siccar 1 == 1, "one"
screen_clear("black")
blether sclaff([[1]])
blether split("a,b", ",")
blether assert_equal(1, 1)
ken s = "a"
s = s + "b"
kin Pot {
    dae init() { masel.n = 1 }
}
blether Pot().n
"#;

    /// Every `__mdh_*` function declared (not defined) in a compiled module, with its
    /// parameter count.
    fn declared_runtime_symbols() -> Vec<(String, u32)> {
        let context = Context::create();
        // With reference counting on, so its declarations are kept and listed too.
        let mut codegen = CodeGen::new_with_refcounting(&context, "ownership_table", true);
        let program = crate::parse(LAZY_DECLS).expect("parse");
        codegen.compile(&program).expect("compile");
        let mut out: Vec<(String, u32)> = codegen
            .get_module()
            .get_functions()
            .filter(|f| f.count_basic_blocks() == 0)
            .map(|f| {
                (
                    f.get_name().to_string_lossy().into_owned(),
                    f.count_params(),
                )
            })
            .filter(|(name, _)| name.starts_with("__mdh_"))
            .collect();
        out.sort();
        out
    }

    #[test]
    fn table_is_sorted_unique_and_well_formed() {
        for pair in RUNTIME_OWNERSHIP.windows(2) {
            assert!(pair[0].0 < pair[1].0, "not sorted/unique at {}", pair[1].0);
        }
        for (name, args, _) in RUNTIME_OWNERSHIP {
            assert!(
                args.chars().all(|c| matches!(c, '-' | 'b' | 'm' | 's')),
                "{name}: bad arg codes {args:?}"
            );
        }
    }

    #[test]
    fn every_declared_runtime_symbol_has_an_entry_with_its_arity() {
        let declared = declared_runtime_symbols();
        assert!(declared.len() > 300, "too few declarations found");
        let mut problems = Vec::new();
        for (name, params) in &declared {
            match ownership_of(name) {
                None => problems.push(format!("{name}: no ownership entry")),
                Some((args, _)) if args.len() != *params as usize => problems.push(format!(
                    "{name}: {} arg codes, {params} declared params",
                    args.len()
                )),
                Some(_) => {}
            }
        }
        for (name, _, _) in RUNTIME_OWNERSHIP {
            if !declared.iter().any(|(d, _)| d == name) {
                problems.push(format!(
                    "{name}: entry for a symbol codegen does not declare"
                ));
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    #[test]
    fn every_entry_is_defined_by_a_runtime() {
        let c_runtime = concat!(
            include_str!("../../runtime/mdh_runtime.c"),
            include_str!("../../runtime/mdh_screen.c"),
            include_str!("../../runtime/gc_stub.c")
        );
        let rust_runtime = [
            include_str!("../../runtime/mdh_runtime_rs/src/lib.rs"),
            include_str!("../../runtime/mdh_runtime_rs/src/tri_runtime.rs"),
            include_str!("../../runtime/mdh_runtime_rs/src/audio.rs"),
        ];
        let defined = |name: &str| {
            let call = format!(" {name}(");
            let ptr_call = format!("*{name}("); // `void *__mdh_buf_alloc(`
                                                // A C definition starts in column 0 and is not a prototype.
            let in_c = c_runtime.lines().any(|line| {
                !line.starts_with([' ', '\t', '#', '/', '*'])
                    && (line.contains(&call) || line.contains(&ptr_call))
                    && !line.trim_end().ends_with(';')
            });
            let fn_def = format!("fn {name}(");
            in_c || rust_runtime.iter().any(|src| src.contains(&fn_def))
        };
        let missing: Vec<&str> = RUNTIME_OWNERSHIP
            .iter()
            .map(|(name, _, _)| *name)
            .filter(|name| !defined(name))
            .collect();
        assert!(missing.is_empty(), "not defined in a runtime: {missing:?}");
    }
}
