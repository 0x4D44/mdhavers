use crate::ast::*;
use crate::error::{HaversError, HaversResult};
use std::collections::{HashMap, HashSet};

/// Higher-order builtins whose calls pass their line tae `__havers.hof` (MDH-BUG-FLU-00113).
const HOF_BUILTINS: &[&str] = &[
    "gaun", "sieve", "tumble", "ilk", "hunt", "ony", "aw", "grup_up", "pairt_by",
];

/// Runtime helpers bound as ordinary (shadowable) names at the top o' the user's block.
const RUNTIME_BINDINGS: &[&str] = &[
    "len",
    // Creel (set) builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00104).
    "creel",
    "empty_creel",
    "toss_in",
    "heave_oot",
    "is_in_creel",
    "creels_thegither",
    "creels_baith",
    "creels_differ",
    "creel_tae_list",
    "is_subset",
    "is_superset",
    "is_disjoint",
    "whit_kind",
    "tae_string",
    "tae_int",
    "tae_float",
    "shove",
    "yank",
    "keys",
    "values",
    // Dict builtins wi' typed keys (core_runtime.js, MDH-BUG-FLU-00109).
    "items",
    "dict_merge",
    "json_stringify",
    "range",
    "abs",
    "min",
    "max",
    "floor",
    "ceil",
    "round",
    "sqrt",
    // Maths builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00100).
    "pooer",
    "pow",
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    "atan2",
    "hypot",
    "log",
    "log10",
    "log2",
    "exp",
    "degrees",
    "radians",
    "PI",
    "E",
    "TAU",
    "pi",
    "e",
    "tau",
    "sign",
    "trunc",
    "clamp",
    "lerp",
    "gcd",
    "lcm",
    "factorial",
    "is_even",
    "is_odd",
    "is_prime",
    "average",
    "median",
    "product",
    "minaw",
    "maxaw",
    "range_o",
    "random",
    "random_int",
    "random_choice",
    "split",
    "join",
    "contains",
    "reverse",
    "sort",
    "blether",
    "set_log_level",
    "get_log_level",
    "log_set_filter",
    "log_get_filter",
    "log_enabled",
    "log_event",
    "log_init",
    "log_span",
    "log_span_enter",
    "log_span_exit",
    "log_span_current",
    "log_span_in",
    "speir",
    "heid",
    "tail",
    "bum",
    "scran",
    "slap",
    "sumaw",
    "coont",
    "wheesht",
    "upper",
    "lower",
    "shuffle",
    "noo",
    "tick",
    "bide",
    "snooze",
    "get_key",
    "gaun",
    "sieve",
    "tumble",
    "aw",
    "ony",
    "hunt",
    // Higher-order and dict builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00113).
    "ilk",
    "grup_up",
    "pairt_by",
    "dict_get",
    "dict_has",
    "dict_remove",
    "dict_invert",
    "fae_pairs",
    // Pure string builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00114).
    "chars",
    "char_at",
    "ord",
    "chr",
    "trim",
    "trim_start",
    "trim_end",
    "strip_left",
    "strip_right",
    "words",
    "lines",
    "title",
    "capitalize",
    "swapcase",
    "center",
    "pad_left",
    "pad_right",
    "repeat",
    "replace",
    "replace_first",
    "starts_with",
    "starts_wi",
    "ends_with",
    "ends_wi",
    "index_of",
    "last_index_of",
    "substring",
    "substr_between",
    "is_alpha",
    "is_digit",
    "is_space",
    "is_upper",
    "is_lower",
    "is_blank",
    "is_empty",
    "split_by",
    "indices_o",
    // Pure list builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00114).
    "zip",
    "enumerate",
    "flatten",
    "uniq",
    "unique",
    "chunks",
    "interleave",
    "pair_up",
    "tak",
    "drap",
    "grup",
    "grup_runs",
    "fankle",
    "ceilidh",
    // Conversion builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00114).
    "tae_bool",
    "tae_binary",
    "tae_hex",
    "tae_octal",
    "fae_binary",
    "fae_hex",
    // Bit builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00114).
    "bit_an",
    "bit_or",
    "bit_xor",
    "bit_nae",
    "bit_shove_left",
    "bit_shove_right",
    "bit_coont",
    // JSON builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00114).
    "json_parse",
    "json_pretty",
    "json_stringify_pretty",
    // Assertion builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00114).
    "assert",
    "assert_equal",
    "assert_nae_equal",
    // Scots string an' list builtins (core_runtime.js), matchin' the interpreter's (MDH-BUG-FLU-00114).
    "roar",
    "mutter",
    "blooter",
    "stooshie",
    "wheesht_aw",
    "clarty",
    "dreich",
    "geggie",
    "banter",
    "skelp",
    "haverin",
    "glaikit",
    "haggis_hunt",
    "sporran_fill",
    "tattie_scone",
    "birl",
    "chynge",
    "clype",
    "soond_stairt",
    "soond_steek",
    "soond_wheesht",
    "soond_luid",
    "soond_hou_luid",
    "soond_haud_gang",
    "soond_lade",
    "soond_spiel",
    "soond_haud",
    "soond_gae_on",
    "soond_stap",
    "soond_unlade",
    "soond_is_spielin",
    "soond_pit_luid",
    "soond_pit_pan",
    "soond_pit_tune",
    "soond_pit_rin_roond",
    "soond_ready",
    "muisic_lade",
    "muisic_spiel",
    "muisic_haud",
    "muisic_gae_on",
    "muisic_stap",
    "muisic_unlade",
    "muisic_is_spielin",
    "muisic_loup",
    "muisic_hou_lang",
    "muisic_whaur",
    "muisic_pit_luid",
    "muisic_pit_pan",
    "muisic_pit_tune",
    "muisic_pit_rin_roond",
    "midi_lade",
    "midi_spiel",
    "midi_haud",
    "midi_gae_on",
    "midi_stap",
    "midi_unlade",
    "midi_is_spielin",
    "midi_loup",
    "midi_hou_lang",
    "midi_whaur",
    "midi_pit_luid",
    "midi_pit_pan",
    "midi_pit_rin_roond",
    // bytes, date/time, regex, file an' environment builtins, plus the interpreter-only ones that
    // raise a clear error (core_runtime.js, MDH-BUG-FLU-00114).
    "bytes",
    "bytes_from_string",
    "bytes_len",
    "bytes_slice",
    "bytes_get",
    "bytes_set",
    "bytes_append",
    "bytes_read_u16be",
    "bytes_read_u32be",
    "bytes_write_u16be",
    "bytes_write_u32be",
    "date_now",
    "date_format",
    "date_parse",
    "date_add",
    "date_diff",
    "timestamp",
    "timestamp_millis",
    "the_noo",
    "mono_ms",
    "mono_ns",
    "regex_test",
    "regex_match",
    "regex_match_all",
    "regex_replace",
    "regex_replace_first",
    "regex_split",
    "scrieve",
    "scrieve_append",
    "append_file",
    "read_file",
    "read_lines",
    "file_exists",
    "file_delete",
    "list_dir",
    "make_dir",
    "is_dir",
    "file_size",
    "path_join",
    "env_get",
    "env_set",
    "env_all",
    "cwd",
    "chdir",
    "args",
    "exit",
    "socket_udp",
    "socket_tcp",
    "socket_bind",
    "socket_connect",
    "socket_listen",
    "socket_accept",
    "socket_set_nonblocking",
    "socket_set_reuseaddr",
    "socket_set_reuseport",
    "socket_set_ttl",
    "socket_set_nodelay",
    "socket_set_rcvbuf",
    "socket_set_sndbuf",
    "socket_close",
    "udp_send_to",
    "udp_recv_from",
    "tcp_send",
    "tcp_recv",
    "dns_lookup",
    "dns_srv",
    "dns_naptr",
    "tls_client_new",
    "tls_connect",
    "tls_send",
    "tls_recv",
    "tls_close",
    "dtls_server_new",
    "dtls_handshake",
    "srtp_create",
    "srtp_protect",
    "srtp_unprotect",
    "event_loop_new",
    "event_loop_stop",
    "event_watch_read",
    "event_watch_write",
    "event_unwatch",
    "event_loop_poll",
    "timer_after",
    "timer_every",
    "timer_cancel",
    "thread_spawn",
    "thread_join",
    "thread_detach",
    "mutex_new",
    "mutex_lock",
    "mutex_unlock",
    "mutex_try_lock",
    "condvar_new",
    "condvar_wait",
    "condvar_timed_wait",
    "condvar_signal",
    "condvar_broadcast",
    "atomic_new",
    "atomic_load",
    "atomic_store",
    "atomic_add",
    "atomic_cas",
    "chan_new",
    "chan_send",
    "chan_recv",
    "chan_try_recv",
    "chan_close",
    "chan_is_closed",
    "shell",
    "shell_status",
];

/// JS reserved words, strict-mode restricted names an' awkward globals. An mdhavers
/// identifier spelled like ane o' these gets a `$` suffix (mdhavers names cannae contain `$`).
const JS_RESERVED: &[&str] = &[
    "arguments",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "undefined",
    "NaN",
    "Infinity",
];

/// Map an mdhavers identifier tae a safe JS binding name. Reserved words an' names
/// startin' `__` (the runtime's ain namespace: `__havers`, `__match_val_N`, ...) are
/// suffixed wi' `$`, which cannae appear in an mdhavers identifier.
fn js_ident(name: &str) -> String {
    if name.starts_with("__") || JS_RESERVED.contains(&name) {
        format!("{}$", name)
    } else {
        name.to_string()
    }
}

/// Escape text so it is safe inside ony JS string form: '...', "..." or `...`.
/// Every string the compiler emits gaes through here.
fn js_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\'' => out.push_str("\\'"),
            '`' => out.push_str("\\`"),
            '$' => out.push_str("\\$"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// A double-quoted JS string literal.
fn js_string(s: &str) -> String {
    format!("\"{}\"", js_escape(s))
}

/// The binding name fer `fetch "path"` wi'oot an alias: the file stem, made intae
/// a valid identifier (`js_ident` then handles reserved words).
fn module_binding_name(path: &str) -> String {
    let stem = path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(path)
        .replace(".braw", "")
        .replace(".js", "");
    let mut ident: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if ident.is_empty() || ident.starts_with(|c: char| c.is_ascii_digit()) {
        ident.insert(0, '_');
    }
    ident
}

/// True when the expression always yields a JS boolean, so JS truthiness
/// already matches the interpreter's.
fn is_bool_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Literal {
            value: Literal::Bool(_),
            ..
        } => true,
        Expr::Binary { operator, .. } => matches!(
            operator,
            BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
        ),
        Expr::Unary {
            operator: UnaryOp::Not,
            ..
        } => true,
        Expr::Logical { left, right, .. } => is_bool_expr(left) && is_bool_expr(right),
        Expr::Grouping { expr, .. } => is_bool_expr(expr),
        _ => false,
    }
}

/// Whether a string dict key is its ain JS property name; mirrors `plainKey`
/// in runtime/js/core_runtime.js. Array-index names (JS reorders them),
/// `__proto__` an' names startin' wi' NUL are encoded by `__havers.dkey`.
fn is_plain_dict_key(s: &str) -> bool {
    let index_like =
        !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && (s == "0" || !s.starts_with('0'));
    !(index_like || s.starts_with('\0') || s == "__proto__")
}

fn is_number_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Literal {
            value: Literal::Integer(_) | Literal::Float(_),
            ..
        } => true,
        Expr::Grouping { expr, .. } => is_number_literal(expr),
        Expr::Unary {
            operator: UnaryOp::Negate,
            operand,
            ..
        } => is_number_literal(operand),
        _ => false,
    }
}

fn is_string_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Literal {
            value: Literal::String(_),
            ..
        }
        | Expr::FString { .. } => true,
        Expr::Grouping { expr, .. } => is_string_literal(expr),
        _ => false,
    }
}

/// A literal whose JS `===` agrees wi' the interpreter's `==` against ony value.
/// No' numbers: an integer (BigInt) equals a float (JS number) o' the same value.
fn is_scalar_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Literal { value, .. } => matches!(value, Literal::String(_) | Literal::Bool(_)),
        Expr::Grouping { expr, .. } => is_scalar_literal(expr),
        _ => false,
    }
}

/// An operand that is statically a string or bool, never an instance.
fn is_primitive_operand(expr: &Expr) -> bool {
    is_scalar_literal(expr) || is_string_literal(expr)
}

/// Compiler - transpiles mdhavers tae JavaScript
pub struct Compiler {
    indent: usize,
    output: String,
    match_counter: usize,
    /// How many method bodies we're inside: a nested function sees `masel` through `__masel`.
    method_depth: usize,
    /// Set just afore a class method is compiled, so its body binds `__masel`.
    compiling_method: bool,
    /// Plain function declarations entered inside the current method body: each has its
    /// ain `this`, so `masel` is read through `__masel` there.
    nested_fn_depth: usize,
    /// mdhavers name -> JS name fer each enclosing JS block, innermost last.
    scopes: Vec<HashMap<String, String>>,
    /// Counter fer renamin' bindings that shadow an outer name.
    rename_counter: usize,
    /// `kin`/`thing` names never bound ony ither way: calls tae these get `new`.
    static_classes: HashSet<String>,
    /// One entry per enclosing JS function, the top level first.
    fn_contexts: Vec<FnContext>,
    /// Counter fer namin' the `gie` sentinels.
    gie_counter: usize,
}

/// Where a `gie` is, relative tae the JS function it returns frae.
#[derive(Default)]
struct FnContext {
    /// How many block-expression IIFEs deep the cursor is.
    block_depth: usize,
    /// Set once a `gie` inside a block expression needs it: that `gie`
    /// throws this object, and the function catches it (MDH-BUG-FLU-00094).
    sentinel: Option<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            indent: 0,
            output: String::new(),
            match_counter: 0,
            method_depth: 0,
            compiling_method: false,
            nested_fn_depth: 0,
            scopes: vec![HashMap::new()],
            rename_counter: 0,
            static_classes: HashSet::new(),
            fn_contexts: vec![FnContext::default()],
            gie_counter: 0,
        }
    }

    /// Compile a program tae JavaScript
    pub fn compile(&mut self, program: &Program) -> HaversResult<String> {
        self.output.clear();
        self.indent = 0;
        self.match_counter = 0;
        self.rename_counter = 0;
        self.fn_contexts = vec![FnContext::default()];
        self.gie_counter = 0;

        let mut needs_tri_runtime = false;
        for stmt in &program.statements {
            Self::scan_stmt_for_runtime_requirements(stmt, &mut needs_tri_runtime)?;
        }

        let mut kin_names = HashSet::new();
        let mut other_names = HashSet::new();
        for stmt in &program.statements {
            Self::collect_bindings_stmt(stmt, &mut kin_names, &mut other_names);
        }
        self.static_classes = kin_names.difference(&other_names).cloned().collect();

        // Add runtime helpers
        self.emit_runtime(needs_tri_runtime);

        // User code lives in its ain block, wi' the runtime helpers bound as ordinary
        // `let`s, so a user `ken len` or `dae max` shadows (or rebinds) the builtin
        // just as it does in the interpreter.
        self.emit_line("{");
        self.indent += 1;
        self.emit_line(&format!(
            "let {{ {} }} = __havers;",
            RUNTIME_BINDINGS.join(", ")
        ));
        self.scopes = vec![RUNTIME_BINDINGS
            .iter()
            .map(|name| (name.to_string(), name.to_string()))
            .collect()];

        // Compile all statements
        for stmt in &program.statements {
            self.compile_stmt(stmt);
        }

        self.indent -= 1;
        self.emit_line("}");

        Ok(self.output.clone())
    }

    /// Record every name bound by a `kin`/`thing` declaration, an' every name bound
    /// ony ither way (ken, dae, parameters, loop variables, assignment...).
    fn collect_bindings_stmt(
        stmt: &Stmt,
        kin_names: &mut HashSet<String>,
        other: &mut HashSet<String>,
    ) {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                other.insert(name.clone());
                if let Some(init) = initializer {
                    Self::collect_bindings_expr(init, kin_names, other);
                }
            }
            Stmt::Expression { expr, .. } | Stmt::Print { value: expr, .. } => {
                Self::collect_bindings_expr(expr, kin_names, other);
            }
            Stmt::Block { statements, .. } => {
                for s in statements {
                    Self::collect_bindings_stmt(s, kin_names, other);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                Self::collect_bindings_expr(condition, kin_names, other);
                Self::collect_bindings_stmt(then_branch, kin_names, other);
                if let Some(else_branch) = else_branch {
                    Self::collect_bindings_stmt(else_branch, kin_names, other);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                Self::collect_bindings_expr(condition, kin_names, other);
                Self::collect_bindings_stmt(body, kin_names, other);
            }
            Stmt::For {
                variable,
                iterable,
                body,
                ..
            } => {
                other.insert(variable.clone());
                Self::collect_bindings_expr(iterable, kin_names, other);
                Self::collect_bindings_stmt(body, kin_names, other);
            }
            Stmt::Function {
                name, params, body, ..
            } => {
                other.insert(name.clone());
                for param in params {
                    other.insert(param.name.clone());
                    if let Some(default) = &param.default {
                        Self::collect_bindings_expr(default, kin_names, other);
                    }
                }
                for s in body {
                    Self::collect_bindings_stmt(s, kin_names, other);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    Self::collect_bindings_expr(value, kin_names, other);
                }
            }
            Stmt::Class { name, methods, .. } => {
                kin_names.insert(name.clone());
                for method in methods {
                    if let Stmt::Function { params, body, .. } = method {
                        for param in params {
                            other.insert(param.name.clone());
                            if let Some(default) = &param.default {
                                Self::collect_bindings_expr(default, kin_names, other);
                            }
                        }
                        for s in body {
                            Self::collect_bindings_stmt(s, kin_names, other);
                        }
                    }
                }
            }
            Stmt::Struct { name, .. } => {
                kin_names.insert(name.clone());
            }
            Stmt::Import { path, alias, .. } => {
                other.insert(alias.clone().unwrap_or_else(|| path.clone()));
            }
            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                other.insert(error_name.clone());
                Self::collect_bindings_stmt(try_block, kin_names, other);
                Self::collect_bindings_stmt(catch_block, kin_names, other);
            }
            Stmt::Match { value, arms, .. } => {
                Self::collect_bindings_expr(value, kin_names, other);
                for arm in arms {
                    if let Pattern::Identifier(name) = &arm.pattern {
                        other.insert(name.clone());
                    }
                    Self::collect_bindings_stmt(&arm.body, kin_names, other);
                }
            }
            Stmt::Assert {
                condition, message, ..
            } => {
                Self::collect_bindings_expr(condition, kin_names, other);
                if let Some(message) = message {
                    Self::collect_bindings_expr(message, kin_names, other);
                }
            }
            Stmt::Destructure {
                patterns, value, ..
            } => {
                for pattern in patterns {
                    if let DestructPattern::Variable(name) | DestructPattern::Rest(name) = pattern {
                        other.insert(name.clone());
                    }
                }
                Self::collect_bindings_expr(value, kin_names, other);
            }
            Stmt::Log {
                message, extras, ..
            } => {
                Self::collect_bindings_expr(message, kin_names, other);
                for extra in extras {
                    Self::collect_bindings_expr(extra, kin_names, other);
                }
            }
            Stmt::Hurl { message, .. } => {
                Self::collect_bindings_expr(message, kin_names, other);
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }

    fn collect_bindings_expr(
        expr: &Expr,
        kin_names: &mut HashSet<String>,
        other: &mut HashSet<String>,
    ) {
        macro_rules! walk {
            ($e:expr) => {
                Self::collect_bindings_expr($e, kin_names, other)
            };
        }
        match expr {
            Expr::Literal { .. } | Expr::Variable { .. } | Expr::Masel { .. } => {}
            Expr::Assign { name, value, .. } => {
                other.insert(name.clone());
                Self::collect_bindings_expr(value, kin_names, other);
            }
            Expr::Binary { left, right, .. }
            | Expr::Logical { left, right, .. }
            | Expr::Pipe { left, right, .. } => {
                walk!(left);
                walk!(right);
            }
            Expr::Unary { operand: e, .. }
            | Expr::Grouping { expr: e, .. }
            | Expr::Input { prompt: e, .. }
            | Expr::Spread { expr: e, .. }
            | Expr::Get { object: e, .. } => walk!(e),
            Expr::Call {
                callee, arguments, ..
            } => {
                walk!(callee);
                for arg in arguments {
                    walk!(arg);
                }
            }
            Expr::Set { object, value, .. } => {
                walk!(object);
                walk!(value);
            }
            Expr::Index { object, index, .. } => {
                walk!(object);
                walk!(index);
            }
            Expr::IndexSet {
                object,
                index,
                value,
                ..
            } => {
                walk!(object);
                walk!(index);
                walk!(value);
            }
            Expr::Slice {
                object,
                start,
                end,
                step,
                ..
            } => {
                walk!(object);
                for part in [start, end, step].into_iter().flatten() {
                    walk!(part);
                }
            }
            Expr::List { elements, .. } => {
                for e in elements {
                    walk!(e);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (k, v) in pairs {
                    walk!(k);
                    walk!(v);
                }
            }
            Expr::Range { start, end, .. } => {
                walk!(start);
                walk!(end);
            }
            Expr::Lambda { params, body, .. } => {
                for param in params {
                    other.insert(param.clone());
                }
                Self::collect_bindings_expr(body, kin_names, other);
            }
            Expr::BlockExpr { statements, .. } => {
                for s in statements {
                    Self::collect_bindings_stmt(s, kin_names, other);
                }
            }
            Expr::FString { parts, .. } => {
                for part in parts {
                    if let FStringPart::Expr(e) = part {
                        walk!(e);
                    }
                }
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                walk!(condition);
                walk!(then_expr);
                walk!(else_expr);
            }
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
    }

    /// The JS name an mdhavers name refers tae at this point in the program.
    fn resolve(&self, name: &str) -> String {
        for scope in self.scopes.iter().rev() {
            if let Some(js) = scope.get(name) {
                return js.clone();
            }
        }
        js_ident(name)
    }

    /// Bind a parameter-like name (initialised on entry, so nae TDZ hazard).
    fn bind_param(&mut self, name: &str) -> String {
        let js = js_ident(name);
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), js.clone());
        }
        js
    }

    /// Declare a name in the current block. Returns the JS name an' whether it is
    /// fresh here (emit `let`) or a re-declaration (emit plain assignment, as the
    /// interpreter just redefines it). A fresh binding that shadows an outer name
    /// gets a unique JS name, so uses o' the outer name earlier in this block (or
    /// in its ain initialiser) never hit JS's temporal dead zone.
    fn declare(&mut self, name: &str) -> (String, bool) {
        if let Some(js) = self.scopes.last().and_then(|scope| scope.get(name)) {
            return (js.clone(), false);
        }
        let shadows = self.scopes.iter().any(|scope| scope.contains_key(name));
        let js = if shadows {
            self.rename_counter += 1;
            format!("{}${}", js_ident(name), self.rename_counter)
        } else {
            js_ident(name)
        };
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), js.clone());
        }
        (js, true)
    }

    /// Compile an expression intae a string wi'oot touchin' the main output.
    fn expr_to_string(&mut self, expr: &Expr) -> String {
        let old_output = std::mem::take(&mut self.output);
        self.compile_expr(expr);
        std::mem::replace(&mut self.output, old_output)
    }

    /// Compile a condition usin' mdhavers truthiness.
    fn compile_condition(&mut self, expr: &Expr) {
        if is_bool_expr(expr) {
            self.compile_expr(expr);
        } else {
            self.output.push_str("__havers.truthy(");
            self.compile_expr(expr);
            self.output.push(')');
        }
    }

    fn is_tri_import_path(path: &str) -> bool {
        path == "tri" || path == "tri.js" || path == "tri.braw"
    }

    fn scan_stmt_for_runtime_requirements(
        stmt: &Stmt,
        needs_tri_runtime: &mut bool,
    ) -> HaversResult<()> {
        match stmt {
            Stmt::Import { path, alias, span } => {
                if Self::is_tri_import_path(path) {
                    if alias.is_none() {
                        return Err(HaversError::TypeError {
                            message: "tri import requires an alias (fetch \"tri\" tae name)"
                                .to_string(),
                            line: span.line,
                        });
                    }
                    *needs_tri_runtime = true;
                }
            }

            Stmt::Block { statements, .. } => {
                for stmt in statements {
                    Self::scan_stmt_for_runtime_requirements(stmt, needs_tri_runtime)?;
                }
            }

            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                Self::scan_stmt_for_runtime_requirements(then_branch, needs_tri_runtime)?;
                if let Some(else_branch) = else_branch {
                    Self::scan_stmt_for_runtime_requirements(else_branch, needs_tri_runtime)?;
                }
            }

            Stmt::While { body, .. } => {
                Self::scan_stmt_for_runtime_requirements(body, needs_tri_runtime)?;
            }

            Stmt::For { body, .. } => {
                Self::scan_stmt_for_runtime_requirements(body, needs_tri_runtime)?;
            }

            Stmt::Function { body, .. } => {
                for stmt in body {
                    Self::scan_stmt_for_runtime_requirements(stmt, needs_tri_runtime)?;
                }
            }

            Stmt::Class { methods, .. } => {
                for method in methods {
                    Self::scan_stmt_for_runtime_requirements(method, needs_tri_runtime)?;
                }
            }

            Stmt::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                Self::scan_stmt_for_runtime_requirements(try_block, needs_tri_runtime)?;
                Self::scan_stmt_for_runtime_requirements(catch_block, needs_tri_runtime)?;
            }

            Stmt::Match { arms, .. } => {
                for arm in arms {
                    Self::scan_stmt_for_runtime_requirements(&arm.body, needs_tri_runtime)?;
                }
            }

            Stmt::VarDecl { .. }
            | Stmt::Expression { .. }
            | Stmt::Return { .. }
            | Stmt::Print { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Struct { .. }
            | Stmt::Assert { .. }
            | Stmt::Destructure { .. }
            | Stmt::Log { .. }
            | Stmt::Hurl { .. } => {}
        }

        Ok(())
    }

    fn emit_runtime(&mut self, include_tri: bool) {
        self.emit_line("// mdhavers runtime - pure havers, but working havers!");
        self.output
            .push_str(include_str!("../runtime/js/audio_runtime.js"));
        self.output
            .push_str(include_str!("../runtime/js/logging_runtime.js"));
        self.output
            .push_str(include_str!("../runtime/js/core_runtime.js"));
        // Normalize to exactly one trailing newline before we start emitting runtime bindings.
        while self.output.ends_with('\n') {
            self.output.pop();
        }
        self.output.push('\n');
        self.emit_line("const __havers = {");
        self.indent += 1;

        // Core helpers the compiled code calls as __havers.<name> (show, truthy, eq, ...)
        self.emit_line("...__havers_core,");

        // Builtins that take or gie integers (len, range, abs, tae_int, ...) live
        // in core_runtime.js, which carries the BigInt integer model.
        self.emit_line("...__havers_core.builtins,");

        // whit_kind (type) function
        self.emit_line("whit_kind: (x) => __havers_core.typeName(x),");

        // tae_string function (same text as blether prints)
        self.emit_line("tae_string: (x) => __havers_core.show(x),");

        // shove (push) function
        self.emit_line("shove: (arr, val) => { arr.push(val); },");

        // yank (pop) function
        self.emit_line("yank: (arr) => {");
        self.indent += 1;
        self.emit_line("if (arr.length === 0) throw new Error('Cannae yank fae an empty list!');");
        self.emit_line("return arr.pop();");
        self.indent -= 1;
        self.emit_line("},");

        // keys, values, items an' the dict helpers live in core_runtime.js,
        // which keeps dict keys typed (MDH-BUG-FLU-00109).

        // join function
        self.emit_line("join: (arr, delim) => arr.map(__havers_core.show).join(delim),");

        // contains function
        self.emit_line("contains: (container, item) => {");
        self.indent += 1;
        self.emit_line("if (typeof container === 'string') return container.includes(item);");
        // Lists find items by ==, so a list or dict member matches structurally.
        self.emit_line(
            "if (Array.isArray(container)) return container.some((x) => __havers_core.eq(x, item));",
        );
        self.emit_line(
            "if (__havers_core.isDict(container)) return Object.prototype.hasOwnProperty.call(container, __havers_core.dkey(item));",
        );
        self.emit_line("if (typeof container === 'object') return item in container;");
        self.emit_line("return false;");
        self.indent -= 1;
        self.emit_line("},");

        // sort function
        // Same total order as the interpreter: numbers (NaN last) < strings
        // (code point / UTF-8 byte order) < everything else (all equal, so the
        // stable sort keeps their order).
        self.emit_line("sort: (arr) => {");
        self.indent += 1;
        self.emit_line("if (!Array.isArray(arr)) throw new Error('sort() expects a list');");
        self.emit_line(
            "const rank = (v) => typeof v === 'bigint' || typeof v === 'number' ? 0 : typeof v === 'string' ? 1 : 2;",
        );
        self.emit_line("const cpCmp = (x, y) => {");
        self.indent += 1;
        self.emit_line("const xs = Array.from(x), ys = Array.from(y);");
        self.emit_line("const n = Math.min(xs.length, ys.length);");
        self.emit_line("for (let i = 0; i < n; i++) {");
        self.indent += 1;
        self.emit_line("const d = xs[i].codePointAt(0) - ys[i].codePointAt(0);");
        self.emit_line("if (d !== 0) return d;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("return xs.length - ys.length;");
        self.indent -= 1;
        self.emit_line("};");
        self.emit_line("return [...arr].sort((a, b) => {");
        self.indent += 1;
        self.emit_line("const ra = rank(a), rb = rank(b);");
        self.emit_line("if (ra !== rb) return ra - rb;");
        self.emit_line("if (ra === 0) {");
        self.indent += 1;
        self.emit_line("const na = Number.isNaN(a), nb = Number.isNaN(b);");
        self.emit_line("if (na || nb) return na === nb ? 0 : (na ? 1 : -1);");
        self.emit_line("return a < b ? -1 : a > b ? 1 : 0;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("if (ra === 1) return cpCmp(a, b);");
        self.emit_line("return 0;");
        self.indent -= 1;
        self.emit_line("});");
        self.indent -= 1;
        self.emit_line("},");

        // blether (print) function - prints like the interpreter's Value Display
        self.emit_line("blether: (x) => console.log(__havers_core.show(x)),");

        // logging helpers
        self.emit_line("set_log_level: __mdh_log_set_level,");
        self.emit_line("get_log_level: __mdh_log_get_level,");
        self.emit_line("log_set_filter: __mdh_log_set_filter,");
        self.emit_line("log_get_filter: () => __mdh_log_state.filter,");
        self.emit_line("log_enabled: __mdh_log_enabled,");
        self.emit_line("log_event: __mdh_log_event,");
        self.emit_line("log_init: __mdh_log_init,");
        self.emit_line("log_span: __mdh_log_span,");
        self.emit_line("log_span_enter: __mdh_log_span_enter,");
        self.emit_line("log_span_exit: __mdh_log_span_exit,");
        self.emit_line("log_span_current: __mdh_log_span_current,");
        self.emit_line("log_span_in: __mdh_log_span_in,");

        // speir (input) - for Node.js
        self.emit_line("speir: (prompt) => {");
        self.indent += 1;
        self.emit_line("const fs = require('fs');");
        self.emit_line("process.stdout.write(String(prompt));");
        self.emit_line("const buf = Buffer.alloc(1024);");
        self.emit_line("let n = 0;");
        self.emit_line(
            "try { n = fs.readSync(0, buf); } catch (e) { if (e.code !== 'EOF') throw e; }",
        );
        self.emit_line("if (n === 0) return null;");
        self.emit_line("return buf.toString('utf8', 0, n).trim();");
        self.indent -= 1;
        self.emit_line("},");

        // Scots-flavored functions

        // heid, tail, bum, upper, lower, split an' reverse live in core_runtime.js (code points, MDH-BUG-FLU-00116).

        // slap - concatenate
        self.emit_line("slap: (a, b) => {");
        self.indent += 1;
        self.emit_line("if (typeof a === 'string' && typeof b === 'string') return a + b;");
        self.emit_line("if (Array.isArray(a) && Array.isArray(b)) return [...a, ...b];");
        self.emit_line("throw new Error('slap() expects two lists or two strings');");
        self.indent -= 1;
        self.emit_line("},");

        // wheesht - trim ASCII whitespace aff a string, or drop the falsy items o' a list
        self.emit_line("wheesht: (x) => {");
        self.indent += 1;
        self.emit_line(
            "if (typeof x === 'string') return x.replace(/^[ \\t\\n\\r]+|[ \\t\\n\\r]+$/g, '');",
        );
        self.emit_line("if (Array.isArray(x)) return x.filter(__havers_core.truthy);");
        self.emit_line("throw new Error('Jings! Something went awfy wrang: wheesht() expects a string or list');");
        self.indent -= 1;
        self.emit_line("},");

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

        // Audio functions
        self.emit_line("// Audio functions");
        self.emit_line("soond_stairt: __havers_audio.soond_stairt,");
        self.emit_line("soond_steek: __havers_audio.soond_steek,");
        self.emit_line("soond_wheesht: __havers_audio.soond_wheesht,");
        self.emit_line("soond_luid: __havers_audio.soond_luid,");
        self.emit_line("soond_hou_luid: __havers_audio.soond_hou_luid,");
        self.emit_line("soond_haud_gang: __havers_audio.soond_haud_gang,");
        self.emit_line("soond_lade: __havers_audio.soond_lade,");
        self.emit_line("soond_spiel: __havers_audio.soond_spiel,");
        self.emit_line("soond_haud: __havers_audio.soond_haud,");
        self.emit_line("soond_gae_on: __havers_audio.soond_gae_on,");
        self.emit_line("soond_stap: __havers_audio.soond_stap,");
        self.emit_line("soond_unlade: __havers_audio.soond_unlade,");
        self.emit_line("soond_is_spielin: __havers_audio.soond_is_spielin,");
        self.emit_line("soond_pit_luid: __havers_audio.soond_pit_luid,");
        self.emit_line("soond_pit_pan: __havers_audio.soond_pit_pan,");
        self.emit_line("soond_pit_tune: __havers_audio.soond_pit_tune,");
        self.emit_line("soond_pit_rin_roond: __havers_audio.soond_pit_rin_roond,");
        self.emit_line("soond_ready: __havers_audio.soond_ready,");
        self.emit_line("muisic_lade: __havers_audio.muisic_lade,");
        self.emit_line("muisic_spiel: __havers_audio.muisic_spiel,");
        self.emit_line("muisic_haud: __havers_audio.muisic_haud,");
        self.emit_line("muisic_gae_on: __havers_audio.muisic_gae_on,");
        self.emit_line("muisic_stap: __havers_audio.muisic_stap,");
        self.emit_line("muisic_unlade: __havers_audio.muisic_unlade,");
        self.emit_line("muisic_is_spielin: __havers_audio.muisic_is_spielin,");
        self.emit_line("muisic_loup: __havers_audio.muisic_loup,");
        self.emit_line("muisic_hou_lang: __havers_audio.muisic_hou_lang,");
        self.emit_line("muisic_whaur: __havers_audio.muisic_whaur,");
        self.emit_line("muisic_pit_luid: __havers_audio.muisic_pit_luid,");
        self.emit_line("muisic_pit_pan: __havers_audio.muisic_pit_pan,");
        self.emit_line("muisic_pit_tune: __havers_audio.muisic_pit_tune,");
        self.emit_line("muisic_pit_rin_roond: __havers_audio.muisic_pit_rin_roond,");
        self.emit_line("midi_lade: __havers_audio.midi_lade,");
        self.emit_line("midi_spiel: __havers_audio.midi_spiel,");
        self.emit_line("midi_haud: __havers_audio.midi_haud,");
        self.emit_line("midi_gae_on: __havers_audio.midi_gae_on,");
        self.emit_line("midi_stap: __havers_audio.midi_stap,");
        self.emit_line("midi_unlade: __havers_audio.midi_unlade,");
        self.emit_line("midi_is_spielin: __havers_audio.midi_is_spielin,");
        self.emit_line("midi_loup: __havers_audio.midi_loup,");
        self.emit_line("midi_hou_lang: __havers_audio.midi_hou_lang,");
        self.emit_line("midi_whaur: __havers_audio.midi_whaur,");
        self.emit_line("midi_pit_luid: __havers_audio.midi_pit_luid,");
        self.emit_line("midi_pit_pan: __havers_audio.midi_pit_pan,");
        self.emit_line("midi_pit_rin_roond: __havers_audio.midi_pit_rin_roond,");

        self.indent -= 1;
        self.emit_line("};");
        // The audio runtime checks fer JS numbers, so integer (BigInt) arguments
        // reach it as plain numbers.
        self.emit_line("for (const k of Object.keys(__havers)) if (/^(soond|muisic|midi)_/.test(k)) __havers[k] = __havers_core.numArgs(__havers[k]);");
        self.emit_line("");

        if include_tri {
            // tri module (three.js adapter with Scots naming)
            // three.js objects are handed oot through the interop Proxy (core_runtime.js),
            // which turns integers intae numbers on every property write an' method
            // call, an' carries the Scots method names (MDH-BUG-FLU-00097).
            self.emit_line("const __havers_tri_wrap = __havers_core.foreign;");
            self.emit_line("");
            self.emit_line("const __havers_tri = (() => {");
            self.indent += 1;
            self.emit_line("const THREE = (typeof globalThis !== 'undefined' && globalThis.THREE)");
            self.emit_line("  ? globalThis.THREE");
            self.emit_line("  : (typeof require !== 'undefined' ? require('three') : null);");
            self.emit_line(
                "if (!THREE) throw new Error(\"three.js (THREE) is required for tri\");",
            );
            self.emit_line("return {");
            self.indent += 1;
            self.emit_line("Sicht: (...args) => __havers_tri_wrap(new THREE.Scene(...__havers_core.jsArgs(args))),");
            self.emit_line("Thing3D: (...args) => __havers_tri_wrap(new THREE.Object3D(...__havers_core.jsArgs(args))),");
            self.emit_line("Clump: (...args) => __havers_tri_wrap(new THREE.Group(...__havers_core.jsArgs(args))),");
            self.emit_line("Mesch: (...args) => __havers_tri_wrap(new THREE.Mesh(...__havers_core.jsArgs(args))),");
            self.emit_line("Kamera: (...args) => __havers_tri_wrap(new THREE.Camera(...__havers_core.jsArgs(args))),");
            self.emit_line("PerspectivKamera: (...args) => __havers_tri_wrap(new THREE.PerspectiveCamera(...__havers_core.jsArgs(args))),");
            self.emit_line("OrthograffikKamera: (...args) => __havers_tri_wrap(new THREE.OrthographicCamera(...__havers_core.jsArgs(args))),");
            self.emit_line(
                "Geometrie: (...args) => __havers_tri_wrap(new THREE.BufferGeometry(...__havers_core.jsArgs(args))),",
            );
            self.emit_line(
                "BoxGeometrie: (...args) => __havers_tri_wrap(new THREE.BoxGeometry(...__havers_core.jsArgs(args))),",
            );
            self.emit_line(
                "SpherGeometrie: (...args) => __havers_tri_wrap(new THREE.SphereGeometry(...__havers_core.jsArgs(args))),",
            );
            self.emit_line(
                "Maiterial: (...args) => __havers_tri_wrap(new THREE.Material(...__havers_core.jsArgs(args))),",
            );
            self.emit_line("MeshBasicMaiterial: (...args) => __havers_tri_wrap(new THREE.MeshBasicMaterial(...__havers_core.jsArgs(args))),");
            self.emit_line("MeshStandardMaiterial: (...args) => __havers_tri_wrap(new THREE.MeshStandardMaterial(...__havers_core.jsArgs(args))),");
            self.emit_line("Licht: (...args) => __havers_tri_wrap(new THREE.Light(...__havers_core.jsArgs(args))),");
            self.emit_line(
                "AmbiantLicht: (...args) => __havers_tri_wrap(new THREE.AmbientLight(...__havers_core.jsArgs(args))),",
            );
            self.emit_line("DireksionalLicht: (...args) => __havers_tri_wrap(new THREE.DirectionalLight(...__havers_core.jsArgs(args))),");
            self.emit_line(
                "PyntLicht: (...args) => __havers_tri_wrap(new THREE.PointLight(...__havers_core.jsArgs(args))),",
            );
            self.emit_line("Textur: (...args) => __havers_tri_wrap(new THREE.Texture(...__havers_core.jsArgs(args))),");
            self.emit_line(
                "Renderar: (...args) => __havers_tri_wrap(new THREE.WebGLRenderer(...__havers_core.jsArgs(args))),",
            );
            self.emit_line("Colour: (...args) => __havers_tri_wrap(new THREE.Color(...__havers_core.jsArgs(args))),");
            self.emit_line("DEG_TO_RAD: Math.PI / 180,");
            self.emit_line("RAD_TO_DEG: 180 / Math.PI,");
            self.indent -= 1;
            self.emit_line("};");
            self.indent -= 1;
            self.emit_line("})();");
            self.emit_line("");
        }

        // The runtime helpers are bound tae plain names at the top o' the user's
        // block (see `compile`), never at module scope, so user code can shadow them.
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                // The initialiser is compiled before the name is bound, so `ken x = x + 1`
                // in an inner block reads the outer `x`, as in the interpreter.
                let init_js = match initializer {
                    Some(init) => self.expr_to_string(init),
                    None => "null".to_string(),
                };
                let (js, fresh) = self.declare(name);
                let keyword = if fresh { "let " } else { "" };
                self.emit_line(&format!("{}{} = {};", keyword, js, init_js));
            }

            Stmt::Expression { expr, .. } => {
                self.emit_indent();
                if matches!(expr, Expr::Dict { .. }) {
                    // A statement startin' wi' `{` would parse as a JS block.
                    self.output.push('(');
                    self.compile_expr(expr);
                    self.output.push(')');
                } else {
                    self.compile_expr(expr);
                }
                self.output.push_str(";\n");
            }

            Stmt::Block { statements, .. } => {
                self.emit_line("{");
                self.indent += 1;
                self.push_scope();
                for stmt in statements {
                    self.compile_stmt(stmt);
                }
                self.pop_scope();
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
                self.compile_condition(condition);
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
                self.compile_condition(condition);
                self.output.push_str(") ");
                self.compile_stmt_inline(body);
                self.output.push('\n');
            }

            Stmt::For {
                variable,
                iterable,
                body,
                span,
            } => {
                let iter_js = self.expr_to_string(iterable);
                self.push_scope();
                let (js, _) = self.declare(variable);
                self.emit_indent();
                self.output.push_str(&format!(
                    "for (let {} of __havers.iter({}, {})) ",
                    js, iter_js, span.line
                ));
                self.compile_stmt_inline(body);
                self.output.push('\n');
                self.pop_scope();
            }

            Stmt::Function {
                name, params, body, ..
            } => {
                let (js, fresh) = self.declare(name);
                self.emit_indent();
                if fresh {
                    self.output.push_str(&format!("function {}(", js));
                    self.compile_callable(params, body);
                    self.output.push('\n');
                } else {
                    // Redefinin' a name in the same block: rebind it, like the interpreter.
                    self.output.push_str(&format!("{} = function {}(", js, js));
                    self.compile_callable(params, body);
                    self.output.push_str(";\n");
                }
            }

            Stmt::Return { value, .. } => {
                if self.current_fn().block_depth > 0 {
                    // A JS `return` here would only leave the block's IIFE: throw
                    // the value oot tae the enclosing function instead.
                    let sentinel = self.block_return_sentinel();
                    self.emit_indent();
                    self.output.push_str(&format!("{}.v = ", sentinel));
                    match value {
                        Some(expr) => self.compile_expr(expr),
                        None => self.output.push_str("undefined"),
                    }
                    self.output.push_str(";\n");
                    self.emit_line(&format!("throw {};", sentinel));
                    return;
                }
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
                self.output.push_str("__havers.blether(");
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
                let super_js = superclass.as_ref().map(|s| self.resolve(s));
                let (js, fresh) = self.declare(name);
                self.emit_indent();
                if fresh {
                    self.output.push_str("let ");
                }
                self.output
                    .push_str(&format!("{} = __havers.kin(class {}", js, js_ident(name)));
                if let Some(super_js) = &super_js {
                    self.output.push_str(&format!(" extends {}", super_js));
                }
                self.output.push_str(" {\n");
                self.indent += 1;

                if super_js.is_none() {
                    // `init` stays an ordinary method. The root constructor dispatches tae
                    // the maist-derived `init`, so a subclass init replaces its parent's
                    // (mdhavers has nae super call), exactly as the interpreter does.
                    self.emit_line("constructor(...args) {");
                    self.indent += 1;
                    self.emit_line("if (typeof this.init === \"function\") this.init(...args);");
                    self.indent -= 1;
                    self.emit_line("}");
                }

                let outer_method_depth = std::mem::take(&mut self.method_depth);
                let outer_nested_fn_depth = std::mem::take(&mut self.nested_fn_depth);
                for method in methods {
                    if let Stmt::Function {
                        name: method_name,
                        params,
                        body,
                        ..
                    } = method
                    {
                        self.emit_indent();
                        self.output.push_str(&format!("{}(", method_name));
                        self.compiling_method = true;
                        self.compile_callable(params, body);
                        self.output.push('\n');
                    }
                }
                self.method_depth = outer_method_depth;
                self.nested_fn_depth = outer_nested_fn_depth;

                self.indent -= 1;
                self.emit_line("});");
            }

            Stmt::Struct { name, fields, .. } => {
                // Compile struct as a (callable) class with a constructor
                let (js, fresh) = self.declare(name);
                let params: Vec<String> = fields.iter().map(|f| js_ident(f)).collect();
                self.emit_indent();
                if fresh {
                    self.output.push_str("let ");
                }
                self.output.push_str(&format!(
                    "{} = __havers.kin(class {} {{\n",
                    js,
                    js_ident(name)
                ));
                self.indent += 1;
                self.emit_line(&format!("constructor({}) {{", params.join(", ")));
                self.indent += 1;
                for (field, param) in fields.iter().zip(&params) {
                    self.emit_line(&format!("this.{} = {};", field, param));
                }
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("}, true);");
            }

            Stmt::Import { path, alias, .. } => {
                if Self::is_tri_import_path(path) {
                    let (js, fresh) = self.declare(alias.as_deref().unwrap_or("tri"));
                    let keyword = if fresh { "let " } else { "" };
                    self.emit_line(&format!("{}{} = __havers_tri;", keyword, js));
                } else {
                    let binding = alias.clone().unwrap_or_else(|| module_binding_name(path));
                    let (js, fresh) = self.declare(&binding);
                    let keyword = if fresh { "let " } else { "" };
                    // A JS module crosses the interop boundary: integers passed
                    // tae it become numbers (MDH-BUG-FLU-00097).
                    self.emit_line(&format!(
                        "{}{} = __havers.foreign(require('{}'));",
                        keyword,
                        js,
                        js_escape(path)
                    ));
                }
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
                // The catch binding sees the error's message string, as in the interpreter.
                self.output.push_str(" catch (__err) {\n");
                self.indent += 1;
                self.push_scope();
                let (js, _) = self.declare(error_name);
                if let Some(sentinel) = self.current_fn().sentinel.clone() {
                    // A `gie` frae a block expression is no' an error.
                    self.emit_line(&format!("if (__err === {}) throw __err;", sentinel));
                }
                self.emit_line(&format!("let {} = __havers.caught(__err);", js));
                self.compile_block_body(catch_block);
                self.pop_scope();
                self.indent -= 1;
                self.emit_indent();
                self.output.push_str("}\n");
            }

            Stmt::Match { value, arms, span } => {
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
                    self.push_scope();

                    // Bind pattern variable if identifier
                    if let Pattern::Identifier(name) = &arm.pattern {
                        let (js, _) = self.declare(name);
                        self.emit_line(&format!("let {} = {};", js, match_var));
                    }

                    // Compile the body - unwrap block to avoid double braces
                    self.compile_block_body(&arm.body);

                    self.pop_scope();
                    self.indent -= 1;
                    self.emit_indent();
                    self.output.push_str("} ");
                }

                if !arms.is_empty() {
                    self.output.push_str("else {\n");
                    self.indent += 1;
                    let prefix = js_string(&format!(
                        "That's pure mince! Type error at line {}: Nae match found fer ",
                        span.line
                    ));
                    self.emit_line(&format!(
                        "throw __havers.fail({} + __havers.show({}));",
                        prefix, match_var
                    ));
                    self.indent -= 1;
                    self.emit_line("}");
                }
            }

            Stmt::Assert {
                condition,
                message,
                span,
            } => {
                self.emit_indent();
                self.output.push_str("if (!(");
                self.compile_condition(condition);
                self.output.push_str(")) {\n");
                self.indent += 1;
                self.emit_indent();
                self.output.push_str("throw __havers.fail(");
                self.output.push_str(&js_string(&format!(
                    "Mak siccar failed at line {}! ",
                    span.line
                )));
                self.output.push_str(" + ");
                if let Some(msg) = message {
                    self.output.push_str("__havers.show(");
                    self.compile_expr(msg);
                    self.output.push(')');
                } else {
                    self.output.push_str(&js_string("Assertion failed"));
                }
                self.output.push_str(");\n");
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::Destructure {
                patterns, value, ..
            } => {
                // JavaScript destructuring: let [a, , ...rest] = value. Ignored slots are
                // holes (so `_` can appear twice); names already bound in this block are
                // reassigned rather than redeclared.
                let value_js = self.expr_to_string(value);
                let mut parts = Vec::new();
                let mut fresh_names = Vec::new();
                let mut named = 0;
                for pattern in patterns {
                    match pattern {
                        DestructPattern::Variable(name) | DestructPattern::Rest(name) => {
                            named += 1;
                            let (js, fresh) = self.declare(name);
                            if fresh {
                                fresh_names.push(js.clone());
                            }
                            if matches!(pattern, DestructPattern::Rest(_)) {
                                parts.push(format!("...{}", js));
                            } else {
                                parts.push(js);
                            }
                        }
                        DestructPattern::Ignore => parts.push(String::new()),
                    }
                }
                let target = format!("[{}]", parts.join(", "));
                if fresh_names.len() == named {
                    self.emit_line(&format!("let {} = {};", target, value_js));
                } else {
                    if !fresh_names.is_empty() {
                        self.emit_line(&format!("let {};", fresh_names.join(", ")));
                    }
                    self.emit_line(&format!("{} = {};", target, value_js));
                }
            }

            Stmt::Log {
                level,
                message,
                extras,
                ..
            } => {
                // Compile log to log_event with optional fields/target
                let level_name = match level {
                    crate::ast::LogLevel::Wheesht => return, // Silent - no output
                    crate::ast::LogLevel::Roar => "roar",
                    crate::ast::LogLevel::Holler => "holler",
                    crate::ast::LogLevel::Blether => "blether",
                    crate::ast::LogLevel::Mutter => "mutter",
                    crate::ast::LogLevel::Whisper => "whisper",
                };
                self.emit_indent();
                self.output
                    .push_str(&format!("__havers.log_event(\"{}\", ", level_name));
                self.compile_expr(message);
                for extra in extras {
                    self.output.push_str(", ");
                    self.compile_expr(extra);
                }
                self.output.push_str(");\n");
            }

            Stmt::Hurl { message, span } => {
                self.emit_indent();
                self.output.push_str("throw __havers.hurl(");
                self.compile_expr(message);
                self.output.push_str(&format!(", {});\n", span.line));
            }
        }
    }

    /// Compile a statement as a braced JS block wi' its ain scope.
    fn compile_stmt_inline(&mut self, stmt: &Stmt) {
        self.output.push_str("{\n");
        self.indent += 1;
        self.push_scope();
        self.compile_block_body(stmt);
        self.pop_scope();
        self.indent -= 1;
        self.emit_indent();
        self.output.push('}');
    }

    /// Compile the statements o' a block (or a lone statement) intae the current block.
    fn compile_block_body(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { statements, .. } => {
                for s in statements {
                    self.compile_stmt(s);
                }
            }
            other => self.compile_stmt(other),
        }
    }

    /// Emit `params) { body }` fer a function or method, in its ain scope.
    fn compile_callable(&mut self, params: &[Param], body: &[Stmt]) {
        self.push_scope();
        self.fn_contexts.push(FnContext::default());
        let is_method = std::mem::take(&mut self.compiling_method);
        // A function declared inside a method has its ain `this`: its defaults an' body
        // read the method's `masel` through `__masel`.
        let is_nested = !is_method && self.method_depth > 0;
        if is_nested {
            self.nested_fn_depth += 1;
        }
        let params_str = self.compile_params(params);
        if is_method {
            self.method_depth += 1;
        }
        if let Some(sentinel) = self.current_fn().sentinel.clone() {
            // A `gie` in a default value's block throws the sentinel while the
            // parameters are bound, afore ony body `try` is entered. So bind them
            // in an inner arrow (same scopin', same `this`) ca'd inside the
            // `try` (MDH-BUG-FLU-00098).
            let args = format!("{}_args", sentinel);
            self.output.push_str(&format!("...{}) {{\n", args));
            self.indent += 1;
            if is_method {
                self.emit_line("const __masel = this;");
            }
            self.emit_line(&format!("const {} = {{}};", sentinel));
            self.emit_line("try {");
            self.emit_line(&format!("return (({}) => {{", params_str));
            self.indent += 1;
            for stmt in body {
                self.compile_stmt(stmt);
            }
            self.fn_contexts.pop();
            self.indent -= 1;
            self.emit_line(&format!("}})(...{});", args));
            self.emit_line(&Self::block_return_catch(&sentinel));
            self.indent -= 1;
            self.emit_indent();
            self.output.push('}');
            if is_method {
                self.method_depth -= 1;
            }
            if is_nested {
                self.nested_fn_depth -= 1;
            }
            self.pop_scope();
            return;
        }
        self.output.push_str(&params_str);
        self.output.push_str(") {\n");
        self.indent += 1;
        if is_method {
            // A nested function declaration has its ain `this`: it reads `masel` here.
            self.emit_line("const __masel = this;");
        }
        let body_start = self.output.len();
        for stmt in body {
            self.compile_stmt(stmt);
        }
        if let Some(sentinel) = self.fn_contexts.pop().and_then(|ctx| ctx.sentinel) {
            let body_js = self.output.split_off(body_start);
            self.emit_line(&format!("const {} = {{}};", sentinel));
            self.emit_line("try {");
            self.output.push_str(&body_js);
            self.emit_line(&Self::block_return_catch(&sentinel));
        }
        self.indent -= 1;
        self.emit_indent();
        self.output.push('}');
        if is_method {
            self.method_depth -= 1;
        }
        if is_nested {
            self.nested_fn_depth -= 1;
        }
        self.pop_scope();
    }

    fn current_fn(&self) -> &FnContext {
        self.fn_contexts
            .last()
            .expect("the top level is aye a function context")
    }

    /// The sentinel a `gie` inside a block expression throws tae reach the
    /// enclosing function, made on first use.
    fn block_return_sentinel(&mut self) -> String {
        let counter = &mut self.gie_counter;
        let ctx = self
            .fn_contexts
            .last_mut()
            .expect("the top level is aye a function context");
        ctx.sentinel
            .get_or_insert_with(|| {
                *counter += 1;
                format!("__gie_{}", counter)
            })
            .clone()
    }

    /// The `catch` that turns a function's `gie` sentinel back intae a return.
    fn block_return_catch(sentinel: &str) -> String {
        format!(
            "}} catch (__e) {{ if (__e === {0}) return {0}.v; throw __e; }}",
            sentinel
        )
    }

    /// A block expression as an IIFE. `in_block` is false only fer a lambda's
    /// ain block body, where a `gie` returns frae the IIFE (the lambda).
    fn compile_block_expr(&mut self, statements: &[Stmt], in_block: bool) {
        self.output.push_str("(() => {\n");
        self.indent += 1;
        self.push_scope();
        if in_block {
            self.fn_contexts
                .last_mut()
                .expect("the top level is aye a function context")
                .block_depth += 1;
        }
        for (i, stmt) in statements.iter().enumerate() {
            match stmt {
                Stmt::Expression { expr, .. } if i + 1 == statements.len() => {
                    self.emit_indent();
                    self.output.push_str("return (");
                    self.compile_expr(expr);
                    self.output.push_str(");\n");
                }
                _ => self.compile_stmt(stmt),
            }
        }
        if in_block {
            self.fn_contexts
                .last_mut()
                .expect("the top level is aye a function context")
                .block_depth -= 1;
        }
        self.pop_scope();
        self.indent -= 1;
        self.emit_indent();
        self.output.push_str("})()");
    }

    fn compile_pattern(&mut self, pattern: &Pattern, match_var: &str) {
        match pattern {
            Pattern::Literal(lit) => {
                // A number arm matches by value, so `whan 1` matches 1.0 too.
                let test = match lit {
                    Literal::Integer(n) => format!("__havers.eq({}, {}n)", match_var, n),
                    Literal::Float(f) => format!("__havers.eq({}, {})", match_var, f),
                    Literal::String(s) => format!("{} === {}", match_var, js_string(s)),
                    Literal::Bool(b) => format!("{} === {}", match_var, b),
                    Literal::Nil => format!("{} === null", match_var),
                };
                self.output.push_str(&test);
            }
            Pattern::Identifier(_) => {
                self.output.push_str("true"); // Always matches
            }
            Pattern::Wildcard => {
                self.output.push_str("true"); // Always matches
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                // Only an integer inside the bounds matches, as in the interpreter.
                self.output
                    .push_str(&format!("__havers.inRange({}, ", match_var));
                self.compile_expr(start);
                self.output.push_str(", ");
                self.compile_expr(end);
                self.output.push_str(&format!(", {})", inclusive));
            }
        }
    }

    /// Compile function parameters (bindin' them in the current scope), handling default values
    fn compile_params(&mut self, params: &[Param]) -> String {
        let mut result = Vec::new();
        for param in params {
            if let Some(default_expr) = &param.default {
                // Compile the default value
                let default_js = self.expr_to_string(default_expr);
                let js = self.bind_param(&param.name);
                result.push(format!("{} = {}", js, default_js));
            } else {
                result.push(self.bind_param(&param.name));
            }
        }
        result.join(", ")
    }

    /// Compile the object o' a `.member` access; `123.slice` is a JS syntax error,
    /// so numeric literals are parenthesised.
    fn compile_receiver(&mut self, object: &Expr) {
        if matches!(
            object,
            Expr::Literal {
                value: Literal::Integer(_) | Literal::Float(_),
                ..
            }
        ) {
            self.output.push('(');
            self.compile_expr(object);
            self.output.push(')');
        } else {
            self.compile_expr(object);
        }
    }

    /// Emit `helper(left, right[, line])`.
    fn compile_binary_helper(
        &mut self,
        helper: &str,
        left: &Expr,
        right: &Expr,
        line: Option<usize>,
    ) {
        self.output.push_str(helper);
        self.output.push('(');
        self.compile_expr(left);
        self.output.push_str(", ");
        self.compile_expr(right);
        if let Some(line) = line {
            self.output.push_str(&format!(", {}", line));
        }
        self.output.push(')');
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal { value, .. } => match value {
                // Integers are BigInts; a float is a JS number (Rust prints 2.0 as "2").
                Literal::Integer(n) => self.output.push_str(&format!("{}n", n)),
                Literal::Float(f) => self.output.push_str(&f.to_string()),
                Literal::String(s) => self.output.push_str(&js_string(s)),
                Literal::Bool(true) => self.output.push_str("true"),
                Literal::Bool(false) => self.output.push_str("false"),
                Literal::Nil => self.output.push_str("null"),
            },

            Expr::Variable { name, .. } => {
                let js = self.resolve(name);
                self.output.push_str(&js);
            }

            Expr::Assign { name, value, .. } => {
                let js = self.resolve(name);
                self.output.push_str(&format!("({} = ", js));
                self.compile_expr(value);
                self.output.push(')');
            }

            Expr::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let line = Some(span.line);
                match operator {
                    // Two string literals concatenate wi' plain `+`.
                    BinaryOp::Add if is_string_literal(left) && is_string_literal(right) => {
                        self.output.push('(');
                        self.compile_expr(left);
                        self.output.push_str(" + ");
                        self.compile_expr(right);
                        self.output.push(')');
                    }
                    // Arithmetic an' ordering go through the runtime's int/float helpers
                    // (integers are BigInts, floats JS numbers; DEC F5).
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Modulo
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual => {
                        let helper = match operator {
                            BinaryOp::Add => "__havers.add",
                            BinaryOp::Subtract => "__havers.sub",
                            BinaryOp::Multiply => "__havers.mul",
                            BinaryOp::Divide => "__havers.div",
                            BinaryOp::Modulo => "__havers.mod",
                            BinaryOp::Less => "__havers.lt",
                            BinaryOp::LessEqual => "__havers.le",
                            BinaryOp::Greater => "__havers.gt",
                            _ => "__havers.ge",
                        };
                        self.compile_binary_helper(helper, left, right, line);
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual
                        if !(is_primitive_operand(left) && is_primitive_operand(right)) =>
                    {
                        // An instance on either side may overload `==` / `!=`, so only a
                        // comparison o' two known primitives keeps the raw `===`.
                        let helper = if matches!(operator, BinaryOp::NotEqual) {
                            "__havers.neOp"
                        } else {
                            "__havers.eqOp"
                        };
                        self.compile_binary_helper(helper, left, right, None);
                    }
                    // Both sides are string or bool literals: JS `===` agrees.
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        self.output.push('(');
                        self.compile_expr(left);
                        if matches!(operator, BinaryOp::Equal) {
                            self.output.push_str(" === ");
                        } else {
                            self.output.push_str(" !== ");
                        }
                        self.compile_expr(right);
                        self.output.push(')');
                    }
                }
            }

            Expr::Unary {
                operator,
                operand,
                span,
            } => match operator {
                // A negated number literal cannae overflow, so it stays a literal.
                UnaryOp::Negate if is_number_literal(operand) => {
                    self.output.push_str("(-");
                    self.compile_expr(operand);
                    self.output.push(')');
                }
                UnaryOp::Negate => {
                    self.output.push_str("__havers.neg(");
                    self.compile_expr(operand);
                    self.output.push_str(&format!(", {})", span.line));
                }
                UnaryOp::Not => {
                    self.output.push_str("(!");
                    self.compile_condition(operand);
                    self.output.push(')');
                }
            },

            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => {
                if is_bool_expr(left) && is_bool_expr(right) {
                    self.output.push('(');
                    self.compile_expr(left);
                    let op_str = match operator {
                        LogicalOp::And => " && ",
                        LogicalOp::Or => " || ",
                    };
                    self.output.push_str(op_str);
                    self.compile_expr(right);
                    self.output.push(')');
                } else {
                    // Short-circuit on mdhavers truthiness, yieldin' the deciding operand.
                    let helper = match operator {
                        LogicalOp::And => "__havers.and(",
                        LogicalOp::Or => "__havers.or(",
                    };
                    self.output.push_str(helper);
                    self.compile_expr(left);
                    self.output.push_str(", () => (");
                    self.compile_expr(right);
                    self.output.push_str("))");
                }
            }

            Expr::Call {
                callee,
                arguments,
                span,
            } => {
                // A `kin`/`thing` name that is never rebound is constructed directly.
                // Ony ither callee is called plainly: classes are wrapped by
                // __havers.kin so callin' one wi'oot `new` still constructs it.
                if let Expr::Variable { name, .. } = &**callee {
                    if self.static_classes.contains(name) {
                        self.output.push_str("new ");
                    }
                }

                // A higher-order builtin gets its call line via `hof`, fer the interpreter's error text.
                let hof_line = match &**callee {
                    Expr::Variable { name, .. } if HOF_BUILTINS.contains(&name.as_str()) => {
                        Some(span.line)
                    }
                    _ => None,
                };
                if let Some(line) = hof_line {
                    self.output.push_str(&format!("__havers.hof({}, ", line));
                    self.compile_expr(callee);
                    self.output.push_str(", [");
                } else if let Expr::Get {
                    object,
                    property,
                    span: get_span,
                } = &**callee
                {
                    // A method call prefers the class's method tae an own field o' the
                    // same name (the runtime's callM), nae binding needed.
                    self.output.push_str("__havers.callM(");
                    self.compile_expr(object);
                    self.output
                        .push_str(&format!(", {}, {}", js_string(property), get_span.line));
                    for arg in arguments {
                        self.output.push_str(", ");
                        self.compile_expr(arg);
                    }
                    self.output.push(')');
                    return;
                } else {
                    self.compile_expr(callee);
                    self.output.push('(');
                }
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.compile_expr(arg);
                }
                self.output
                    .push_str(if hof_line.is_some() { "])" } else { ")" });
            }

            Expr::Get {
                object,
                property,
                span,
            } => {
                // A method value is bound tae its instance, an own field beats a
                // method, an' a missing field is an error (the runtime's getProp).
                self.output.push_str("__havers.getProp(");
                self.compile_expr(object);
                self.output
                    .push_str(&format!(", {}, {})", js_string(property), span.line));
            }

            Expr::Set {
                object,
                property,
                value,
                ..
            } => {
                self.output.push('(');
                self.compile_receiver(object);
                self.output.push('.');
                self.output.push_str(property);
                self.output.push_str(" = ");
                self.compile_expr(value);
                self.output.push(')');
            }

            Expr::Index {
                object,
                index,
                span,
            } => {
                // Negative indices, codepoint strings an' bounds errors, like the interpreter.
                self.compile_binary_helper("__havers.idx", object, index, Some(span.line));
            }

            Expr::IndexSet {
                object,
                index,
                value,
                span,
            } => {
                // Lists need an in-range integer index, like the interpreter.
                self.output.push_str("__havers.setIdx(");
                self.compile_expr(object);
                self.output.push_str(", ");
                self.compile_expr(index);
                self.output.push_str(", ");
                self.compile_expr(value);
                self.output.push_str(&format!(", {})", span.line));
            }

            Expr::Slice {
                object,
                start,
                end,
                step,
                span,
            } => {
                // `__havers.slice(obj, start, end, step, line)`; a missing part is null.
                self.output.push_str("__havers.slice(");
                self.compile_expr(object);
                for part in [start, end, step] {
                    self.output.push_str(", ");
                    match part {
                        Some(e) => self.compile_expr(e),
                        None => self.output.push_str("null"),
                    }
                }
                self.output.push_str(&format!(", {})", span.line));
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
                    match key {
                        // A string the runtime keeps as its ain property name.
                        Expr::Literal {
                            value: Literal::String(s),
                            ..
                        } if is_plain_dict_key(s) => self.output.push_str(&js_string(s)),
                        // Keys are expressions: `{k: 1}` uses the value o' k, typed
                        // by the runtime's key encodin' (MDH-BUG-FLU-00109).
                        _ => {
                            self.output.push_str("[__havers.dkey(");
                            self.compile_expr(key);
                            self.output.push_str(")]");
                        }
                    }
                    self.output.push_str(": ");
                    self.compile_expr(value);
                }
                self.output.push('}');
            }

            Expr::Range {
                start,
                end,
                inclusive,
                span,
            } => {
                self.output.push_str("__havers.rangeOf(");
                self.compile_expr(start);
                self.output.push_str(", ");
                self.compile_expr(end);
                self.output
                    .push_str(&format!(", {}, {})", inclusive, span.line));
            }

            Expr::Grouping { expr, .. } => {
                self.output.push('(');
                self.compile_expr(expr);
                self.output.push(')');
            }

            Expr::Lambda { params, body, .. } => {
                self.push_scope();
                let js_params: Vec<String> = params.iter().map(|p| self.bind_param(p)).collect();
                self.output.push('(');
                self.output.push_str(&js_params.join(", "));
                self.output.push_str(") => ");
                self.fn_contexts.push(FnContext::default());
                let body_start = self.output.len();
                if let Expr::BlockExpr { statements, .. } = &**body {
                    self.compile_block_expr(statements, false);
                } else {
                    // `=> {` would start a function body, no' an object - and no'
                    // only fer a bare dict: `|k| {"a": 1}[k]` starts wi' one too.
                    self.output.push('(');
                    self.compile_expr(body);
                    self.output.push(')');
                }
                if let Some(sentinel) = self.fn_contexts.pop().and_then(|ctx| ctx.sentinel) {
                    let body_js = self.output.split_off(body_start);
                    self.output.push_str(&format!(
                        "{{ const {0} = {{}}; try {{ return {1}; {2} }}",
                        sentinel,
                        body_js,
                        Self::block_return_catch(&sentinel)
                    ));
                }
                self.pop_scope();
            }

            Expr::Masel { .. } => {
                self.output.push_str(if self.nested_fn_depth > 0 {
                    "__masel"
                } else {
                    "this"
                });
            }

            Expr::Input { prompt, .. } => {
                self.output.push_str("__havers.speir(");
                self.compile_expr(prompt);
                self.output.push(')');
            }

            Expr::FString { parts, .. } => {
                // Compile to JavaScript template literal; text is fully escaped and
                // interpolations print like blether does.
                self.output.push('`');
                for part in parts {
                    match part {
                        FStringPart::Text(text) => {
                            self.output.push_str(&js_escape(text));
                        }
                        FStringPart::Expr(expr) => {
                            self.output.push_str("${__havers.show(");
                            self.compile_expr(expr);
                            self.output.push_str(")}");
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
                self.compile_condition(condition);
                self.output.push_str(" ? ");
                self.compile_expr(then_expr);
                self.output.push_str(" : ");
                self.compile_expr(else_expr);
                self.output.push(')');
            }
            Expr::BlockExpr { statements, .. } => {
                // Block expressions compile to an IIFE (immediately invoked function
                // expression). The last expression statement is the block's value
                // (MDH-BUG-FLU-00064); a `gie` inside returns frae the enclosing
                // function (MDH-BUG-FLU-00094).
                self.compile_block_expr(statements, true);
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
        assert!(result.contains("let x = 5n;"));
    }

    #[test]
    fn test_var_no_initializer() {
        let result = compile("ken x").unwrap();
        assert!(result.contains("let x = null;"));
    }

    #[test]
    fn test_expression_statement() {
        let result = compile("ken x = 1\nx + 2").unwrap();
        assert!(result.contains("__havers.add(x, 2n, 2);"));
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

    #[cfg(coverage)]
    #[test]
    fn compiler_branch_matrix_for_coverage() {
        let span = Span::new(1, 1);

        assert!(Compiler::is_tri_import_path("tri"));
        assert!(Compiler::is_tri_import_path("tri.js"));
        assert!(Compiler::is_tri_import_path("tri.braw"));
        assert!(!Compiler::is_tri_import_path("other"));

        let mut needs_tri = false;
        let if_stmt = Stmt::If {
            condition: Expr::Literal {
                value: Literal::Bool(true),
                span,
            },
            then_branch: Box::new(Stmt::Break { span }),
            else_branch: None,
            span,
        };
        Compiler::scan_stmt_for_runtime_requirements(&if_stmt, &mut needs_tri).unwrap();
        let if_stmt_with_else = Stmt::If {
            condition: Expr::Literal {
                value: Literal::Bool(false),
                span,
            },
            then_branch: Box::new(Stmt::Break { span }),
            else_branch: Some(Box::new(Stmt::Break { span })),
            span,
        };
        Compiler::scan_stmt_for_runtime_requirements(&if_stmt_with_else, &mut needs_tri).unwrap();

        let mut compiler = Compiler::new();

        compiler.compile_stmt(&Stmt::VarDecl {
            name: "x".to_string(),
            initializer: None,
            span,
        });
        compiler.compile_stmt(&Stmt::VarDecl {
            name: "y".to_string(),
            initializer: Some(Expr::Literal {
                value: Literal::Integer(1),
                span,
            }),
            span,
        });

        compiler.compile_stmt(&Stmt::If {
            condition: Expr::Literal {
                value: Literal::Bool(true),
                span,
            },
            then_branch: Box::new(Stmt::Break { span }),
            else_branch: Some(Box::new(Stmt::Break { span })),
            span,
        });
        compiler.compile_stmt(&Stmt::If {
            condition: Expr::Literal {
                value: Literal::Bool(false),
                span,
            },
            then_branch: Box::new(Stmt::Break { span }),
            else_branch: None,
            span,
        });

        compiler.compile_stmt(&Stmt::Return { value: None, span });
        compiler.compile_stmt(&Stmt::Return {
            value: Some(Expr::Literal {
                value: Literal::Integer(2),
                span,
            }),
            span,
        });

        compiler.compile_stmt(&Stmt::Class {
            name: "C".to_string(),
            superclass: Some("Base".to_string()),
            methods: vec![
                Stmt::Function {
                    name: "init".to_string(),
                    params: Vec::new(),
                    body: vec![Stmt::Return { value: None, span }],
                    span,
                },
                Stmt::Function {
                    name: "ping".to_string(),
                    params: Vec::new(),
                    body: vec![Stmt::Return {
                        value: Some(Expr::Literal {
                            value: Literal::Integer(3),
                            span,
                        }),
                        span,
                    }],
                    span,
                },
            ],
            span,
        });
        compiler.compile_stmt(&Stmt::Class {
            name: "D".to_string(),
            superclass: None,
            methods: Vec::new(),
            span,
        });

        compiler.compile_stmt(&Stmt::Import {
            path: "tri".to_string(),
            alias: Some("tri".to_string()),
            span,
        });
        compiler.compile_stmt(&Stmt::Import {
            path: "lib/util.js".to_string(),
            alias: None,
            span,
        });

        compiler.compile_stmt(&Stmt::Match {
            value: Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            arms: vec![
                MatchArm {
                    pattern: Pattern::Identifier("val".to_string()),
                    body: Stmt::Break { span },
                    span,
                },
                MatchArm {
                    pattern: Pattern::Literal(Literal::Integer(2)),
                    body: Stmt::Break { span },
                    span,
                },
            ],
            span,
        });
        compiler.compile_stmt(&Stmt::Match {
            value: Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            arms: Vec::new(),
            span,
        });

        compiler.compile_stmt(&Stmt::Assert {
            condition: Expr::Literal {
                value: Literal::Bool(false),
                span,
            },
            message: None,
            span,
        });
        compiler.compile_stmt(&Stmt::Assert {
            condition: Expr::Literal {
                value: Literal::Bool(false),
                span,
            },
            message: Some(Expr::Literal {
                value: Literal::String("nope".to_string()),
                span,
            }),
            span,
        });

        compiler.compile_stmt(&Stmt::Destructure {
            patterns: vec![
                DestructPattern::Variable("a".to_string()),
                DestructPattern::Ignore,
            ],
            value: Expr::List {
                elements: vec![
                    Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                    Expr::Literal {
                        value: Literal::Integer(2),
                        span,
                    },
                ],
                span,
            },
            span,
        });

        compiler.compile_expr(&Expr::Literal {
            value: Literal::Bool(true),
            span,
        });
        compiler.compile_expr(&Expr::Literal {
            value: Literal::Bool(false),
            span,
        });

        compiler.compile_expr(&Expr::Call {
            callee: Box::new(Expr::Variable {
                name: "foo".to_string(),
                span,
            }),
            arguments: vec![
                Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                },
                Expr::Literal {
                    value: Literal::Integer(2),
                    span,
                },
            ],
            span,
        });

        compiler.compile_expr(&Expr::Slice {
            object: Box::new(Expr::Variable {
                name: "xs".to_string(),
                span,
            }),
            start: None,
            end: None,
            step: Some(Box::new(Expr::Literal {
                value: Literal::Integer(2),
                span,
            })),
            span,
        });
        compiler.compile_expr(&Expr::Slice {
            object: Box::new(Expr::Variable {
                name: "xs".to_string(),
                span,
            }),
            start: Some(Box::new(Expr::Literal {
                value: Literal::Integer(1),
                span,
            })),
            end: Some(Box::new(Expr::Literal {
                value: Literal::Integer(3),
                span,
            })),
            step: Some(Box::new(Expr::Literal {
                value: Literal::Integer(1),
                span,
            })),
            span,
        });
        compiler.compile_expr(&Expr::Slice {
            object: Box::new(Expr::Variable {
                name: "xs".to_string(),
                span,
            }),
            start: Some(Box::new(Expr::Literal {
                value: Literal::Integer(0),
                span,
            })),
            end: None,
            step: None,
            span,
        });
        compiler.compile_expr(&Expr::Slice {
            object: Box::new(Expr::Variable {
                name: "xs".to_string(),
                span,
            }),
            start: None,
            end: Some(Box::new(Expr::Literal {
                value: Literal::Integer(2),
                span,
            })),
            step: None,
            span,
        });

        compiler.compile_expr(&Expr::List {
            elements: vec![
                Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                },
                Expr::Literal {
                    value: Literal::Integer(2),
                    span,
                },
            ],
            span,
        });

        compiler.compile_expr(&Expr::Range {
            start: Box::new(Expr::Literal {
                value: Literal::Integer(0),
                span,
            }),
            end: Box::new(Expr::Literal {
                value: Literal::Integer(2),
                span,
            }),
            inclusive: true,
            span,
        });
        compiler.compile_expr(&Expr::Range {
            start: Box::new(Expr::Literal {
                value: Literal::Integer(0),
                span,
            }),
            end: Box::new(Expr::Literal {
                value: Literal::Integer(2),
                span,
            }),
            inclusive: false,
            span,
        });

        compiler.compile_expr(&Expr::FString {
            parts: vec![FStringPart::Text("price $".to_string())],
            span,
        });

        compiler.compile_expr(&Expr::Dict {
            pairs: vec![
                (
                    Expr::Literal {
                        value: Literal::String("k".to_string()),
                        span,
                    },
                    Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                ),
                (
                    Expr::Literal {
                        value: Literal::String("v".to_string()),
                        span,
                    },
                    Expr::Literal {
                        value: Literal::Integer(2),
                        span,
                    },
                ),
            ],
            span,
        });

        let _ = HashMap::<String, String>::new(); // keep imports in-use under coverage.
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
        assert!(result.contains("for (let i of __havers.iter("));
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
        assert!(result.contains("let x = 1n;"));
        assert!(result.contains("let y = 2n;"));
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
        assert!(result.contains("init(name)"));
        assert!(result.contains("constructor(...args)"));
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
        assert!(result.contains("let m = __havers.foreign(require('math'))"));
    }

    #[test]
    fn test_import_tri_compile() {
        let result = compile("fetch \"tri\" tae tri").unwrap();
        assert!(result.contains("let tri = __havers_tri"));
    }

    #[test]
    fn test_import_tri_braw_compile() {
        let result = compile("fetch \"tri.braw\" tae tri").unwrap();
        assert!(result.contains("let tri = __havers_tri"));
    }

    #[test]
    fn test_tri_runtime_not_emitted_without_tri_import() {
        let result = compile("blether 1").unwrap();
        assert!(!result.contains("__havers_tri"));
        assert!(!result.contains("require('three')"));
    }

    #[test]
    fn test_tri_runtime_emitted_with_tri_import() {
        let result = compile("fetch \"tri\" tae tri").unwrap();
        assert!(result.contains("const __havers_tri = (() => {"));
        assert!(result.contains("require('three')"));
    }

    #[test]
    fn test_tri_runtime_emitted_for_nested_import() {
        let result = compile("dae f() { fetch \"tri\" tae tri }").unwrap();
        assert!(result.contains("const __havers_tri = (() => {"));
    }

    #[test]
    fn test_import_tri_requires_alias() {
        let err = compile("fetch \"tri\"").unwrap_err();
        assert!(err.to_string().contains("requires an alias"));
    }

    #[test]
    fn test_scan_tri_import_requires_alias_errors_propagate_through_nested_stmts_for_coverage() {
        let err = compile(
            r#"
dae f() {
    gin aye {
        whiles aye {
            fetch "tri"
        }
    }
}
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("requires an alias"));
    }

    #[test]
    fn test_scan_tri_import_requires_alias_errors_propagate_through_else_branch_for_coverage() {
        let err = compile(
            r#"
gin aye {
    blether 1
} ither {
    fetch "tri"
}
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("requires an alias"));
    }

    #[test]
    fn test_scan_tri_import_requires_alias_errors_propagate_through_for_loop_for_coverage() {
        let err = compile(
            r#"
fer i in 0..1 {
    fetch "tri"
}
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("requires an alias"));
    }

    #[test]
    fn test_scan_tri_import_requires_alias_errors_propagate_through_try_block_for_coverage() {
        let err = compile(
            r#"
hae_a_bash {
    fetch "tri"
} gin_it_gangs_wrang e {
    blether e
}
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("requires an alias"));
    }

    #[test]
    fn test_scan_tri_import_requires_alias_errors_propagate_through_class_try_catch_and_match_for_coverage(
    ) {
        let err = compile(
            r#"
kin C {
    dae m() {
        hae_a_bash {
            keek x {
                whan 1 -> blether "ok"
            }
        } gin_it_gangs_wrang e {
            keek x {
                whan 1 -> { fetch "tri" }
            }
        }
    }
}
"#,
        )
        .unwrap_err();
        let msg = err.to_string();
        #[cfg(not(coverage))]
        assert!(msg.contains("requires an alias"), "unexpected error: {msg}");
        #[cfg(coverage)]
        assert!(msg.contains("requires an alias"));
    }

    // ==================== Try-Catch Tests ====================

    #[test]
    fn test_try_catch_compile() {
        let result =
            compile("hae_a_bash { ken x = 1 } gin_it_gangs_wrang e { blether e }").unwrap();
        assert!(result.contains("try {"));
        assert!(result.contains("catch (__err)"));
        // `e` is also the maths builtin, so the catch binding shadows it under a fresh name.
        assert!(result.contains("let e$1 = __havers.caught(__err);"));
        assert!(result.contains("__havers.blether(e$1);"));
    }

    #[test]
    fn test_return_with_value_compile() {
        let result = compile("dae add() { gie 1 }").unwrap();
        assert!(result.contains("return 1n"));
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
    fn test_match_compile_empty_arms_does_not_emit_else_for_coverage() {
        let result = compile("keek x { }").unwrap();
        assert!(result.contains("__match_val_"));
        assert!(!result.contains("Nae match found"));
    }

    #[test]
    fn test_match_literal_patterns() {
        let result = compile(
            r#"keek x {
    whan "hello" -> blether "hi"
    whan 3.14 -> blether "pi"
    whan aye -> blether "true"
    whan nae -> blether "false"
    whan naething -> blether "nil"
}"#,
        )
        .unwrap();
        assert!(result.contains("=== \"hello\""));
        assert!(result.contains("__havers.eq(__match_val_0, 3.14)"));
        assert!(result.contains("=== true"));
        assert!(result.contains("=== false"));
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
        assert!(result.contains("let value ="));
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
        assert!(result.contains("__havers.inRange(__match_val_0, 1n, 10n, false)"));
    }

    // ==================== Assert Tests ====================

    #[test]
    fn test_assert_compile() {
        let result = compile("mak_siccar x > 0").unwrap();
        assert!(result.contains("if (!("));
        assert!(result.contains(
            "throw __havers.fail(\"Mak siccar failed at line 1! \" + \"Assertion failed\")"
        ));
    }

    #[test]
    fn test_assert_with_message() {
        let result = compile("mak_siccar x > 0, \"x must be positive\"").unwrap();
        assert!(result.contains("throw __havers.fail("));
        assert!(result.contains("__havers.show(\"x must be positive\")"));
    }

    // ==================== Destructuring Tests ====================

    #[test]
    fn test_destructure_compile() {
        let result = compile("ken [a, b] = [1, 2]").unwrap();
        assert!(result.contains("let [a, b] = "));
    }

    #[test]
    fn test_destructure_rest() {
        let result = compile("ken [first, ...rest] = [1, 2, 3]").unwrap();
        assert!(result.contains("let [first, ...rest] = "));
    }

    #[test]
    fn test_destructure_ignore() {
        let result = compile("ken [_, second, _] = [1, 2, 3]").unwrap();
        assert!(result.contains("let [, second, ] = "));
    }

    // ==================== Expression Tests ====================

    #[test]
    fn test_assignment_compile() {
        let result = compile("ken x = 1\nx = 42").unwrap();
        assert!(result.contains("(x = 42n)"));
    }

    #[test]
    fn test_unary_negate() {
        let result = compile("-42").unwrap();
        assert!(result.contains("(-42n)"));
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
        assert!(result.contains("foo(1n, 2n, 3n)"));
    }

    #[test]
    fn test_call_compile_get_callee_exercises_constructor_heuristic_skip_for_coverage() {
        let _ = std::hint::black_box(compile("foo.bar(1)\n"));
    }

    #[test]
    fn test_get_property() {
        let result = compile("obj.prop").unwrap();
        assert!(result.contains("obj.prop"));
    }

    #[test]
    fn test_set_property() {
        let result = compile("ken obj = {}\nobj.prop = 42").unwrap();
        assert!(result.contains("obj.prop = 42n"));
    }

    #[test]
    fn test_index_compile() {
        let result = compile("list[0]").unwrap();
        assert!(result.contains("__havers.idx(list, 0n, 1)"));
    }

    #[test]
    fn test_index_set_compile() {
        let result = compile("ken list = [1,2,3]\nlist[0] = 99").unwrap();
        assert!(result.contains("__havers.setIdx(list, 0n, 99n, 2)"));
    }

    #[test]
    fn test_slice_simple() {
        let result = compile("list[1:3]").unwrap();
        assert!(result.contains("__havers.slice(list, 1n, 3n, null, 1)"));
    }

    #[test]
    fn test_slice_with_step() {
        let result = compile("list[::2]").unwrap();
        assert!(result.contains("__havers.slice(list, null, null, 2n, 1)"));
    }

    #[test]
    fn test_slice_start_only() {
        let result = compile("list[1:]").unwrap();
        assert!(result.contains("__havers.slice(list, 1n, null, null, 1)"));
    }

    #[test]
    fn test_list_compile() {
        let result = compile("[1, 2, 3]").unwrap();
        assert!(result.contains("[1n, 2n, 3n]"));
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
        assert!(result.contains("__havers.rangeOf(0n, 10n, false, 1)"));
    }

    #[test]
    fn test_range_compile_inclusive() {
        let result = compile("0..=10").unwrap();
        assert!(result.contains("__havers.rangeOf(0n, 10n, true, 1)"));
    }

    #[test]
    fn test_grouping_compile() {
        let result = compile("(1 + 2) * 3").unwrap();
        assert!(result.contains("__havers.mul((__havers.add(1n, 2n, 1)), 3n, 1)"));
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
        assert!(result.contains("`Hello ${__havers.show(name)}!`"));
    }

    #[test]
    fn test_fstring_escapes() {
        let result = compile("f\"cost: $5\"").unwrap();
        assert!(result.contains("`cost: \\$5`"));
    }

    #[test]
    fn test_spread_compile() {
        let result = compile("[1, ...[2, 3]]").unwrap();
        assert!(result.contains("...[2n, 3n]"));
    }

    #[test]
    fn test_pipe_compile() {
        let result = compile("ken dbl = |x| x * 2\n5 |> dbl").unwrap();
        assert!(result.contains("dbl(5n)"));
    }

    #[test]
    fn test_ternary_compile() {
        let result = compile("ken x = gin aye than 1 ither 0").unwrap();
        assert!(result.contains("true ? 1n : 0n"));
    }

    #[test]
    fn test_more_stmt_and_expr_variants_for_coverage() {
        let result = compile(
            r#"
log_blether 1.5
log_holler naething
log_roar "err", 1 <= 2, 10 - 3
log_mutter "debug"
log_whisper "trace"
hurl "boom"

ken y = 10 / 2
ken z = 10 % 3
ken eq = 1 == 1
ken neq = 1 != 2
ken ge = 2 >= 1

Point(1, 2)

ken xs = [1, 2, 3, 4, 5, 6]
xs[1:5:2]
xs[:2]

ken s = f"a`b$c"

ken bx = || {
    ken t = 1
    gie t
}

keek 1 {
    whan 1 -> { blether "one" }
    whan _ -> blether "other"
}
"#,
        )
        .unwrap();

        assert!(result.contains("log_event(\"blether\""));
        assert!(result.contains("log_event(\"roar\""));
        assert!(result.contains("throw __havers.hurl(\"boom\", 7);"));
        // Point is no' a declared kin, so it is called plainly (nae capitalisation guess)
        assert!(!result.contains("new Point"));
        assert!(result.contains("Point(1n, 2n)"));
        assert!(result.contains("__havers.slice(xs, 1n, 5n, 2n, 18)"));
        assert!(result.contains("__havers.slice(xs, null, 2n, null, 19)"));
        assert!(result.contains("`a\\`b\\$c`"));
        assert!(result.contains("(() => {"));
    }

    #[test]
    fn test_compile_log_wheesht_from_ast_for_coverage() {
        use crate::ast::{Expr, Literal, LogLevel, Program, Span, Stmt};

        let span = Span::new(1, 1);
        let stmt = Stmt::Log {
            level: LogLevel::Wheesht,
            message: Expr::Literal {
                value: Literal::String("silent".to_string()),
                span,
            },
            extras: vec![],
            span,
        };

        let program = Program::new(vec![stmt]);
        let mut compiler = Compiler::new();
        let js = compiler.compile(&program).unwrap();

        assert!(!js.contains("log_event(\"wheesht\""));
    }

    #[test]
    fn test_compile_stmt_inline_non_block_body_for_coverage() {
        use crate::ast::{Expr, Literal, Program, Span, Stmt};

        let span = Span::new(1, 1);
        let then_branch = Stmt::Print {
            value: Expr::Literal {
                value: Literal::String("hi".to_string()),
                span,
            },
            span,
        };
        let stmt = Stmt::If {
            condition: Expr::Literal {
                value: Literal::Bool(true),
                span,
            },
            then_branch: Box::new(then_branch),
            else_branch: None,
            span,
        };

        let program = Program::new(vec![stmt]);
        let mut compiler = Compiler::new();
        let js = compiler.compile(&program).unwrap();

        assert!(js.contains("if (true)"));
        assert!(js.contains("blether(\"hi\")"));
    }

    #[test]
    fn test_class_non_function_method_is_skipped_for_coverage() {
        use crate::ast::{Program, Span, Stmt};

        let span = Span::new(1, 1);
        let program = Program::new(vec![Stmt::Class {
            name: "Foo".to_string(),
            superclass: None,
            methods: vec![Stmt::VarDecl {
                name: "x".to_string(),
                initializer: None,
                span,
            }],
            span,
        }]);

        let mut compiler = Compiler::new();
        let _ = std::hint::black_box(compiler.compile(&program));
    }

    #[test]
    fn test_compile_branchy_constructs_for_coverage() {
        let src = r#"
ken [a, b, ...rest] = [1, 2, 3]
ken xs = [1, 2, 3]
ken slice1 = xs[1:]
ken slice2 = xs[:2]
ken slice3 = xs[1::2]
ken slice4 = xs[:3:2]
ken range1 = 1..3
ken range2 = 1..=3
ken dict = {"a": 1, "b": 2}
blether sum(1, 2, 3)
keek aye {
  whan aye -> blether 1
  whan nae -> blether 0
  whan x -> blether 2
}
mak_siccar aye, "boom"
mak_siccar aye
ken s = f"price: $5"
"#;
        let result = compile(src).unwrap();
        assert!(result.contains("let [a, b, ...rest] ="));
        assert!(result.contains("__havers.slice(xs, 1n, null, null, 4)"));
        assert!(result.contains("__havers.rangeOf(1n, 3n, true, 9)"));
        assert!(result.contains("blether(sum(1n, 2n, 3n))"));
        assert!(result.contains("let x = __match_val_"));
        assert!(result.contains("throw __havers.fail(\"Mak siccar failed at line 17! \""));
        assert!(result.contains("price: \\$5"));
    }

    #[test]
    fn test_tri_import_path_variants_for_coverage() {
        assert!(Compiler::is_tri_import_path("tri"));
        assert!(Compiler::is_tri_import_path("tri.js"));
        assert!(Compiler::is_tri_import_path("tri.braw"));
        assert!(!Compiler::is_tri_import_path("triangles"));

        let result = compile("fetch \"tri.js\" tae tri").unwrap();
        assert!(result.contains("let tri = __havers_tri"));
    }

    #[test]
    fn test_compile_match_with_empty_arms_for_coverage() {
        use crate::ast::{Expr, Literal, Program, Span, Stmt};

        let span = Span::new(1, 1);
        let stmt = Stmt::Match {
            value: Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            arms: Vec::new(),
            span,
        };
        let program = Program::new(vec![stmt]);
        let mut compiler = Compiler::new();
        let js = compiler.compile(&program).unwrap();
        assert!(js.contains("__match_val_"));
        assert!(!js.contains("Nae match found"));
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
        assert!(result.contains("const __havers_audio ="));
        assert!(result.contains("len:"));
        assert!(result.contains("whit_kind:"));
        assert!(result.contains("blether:"));
        assert!(result.contains("soond_stairt"));
    }

    // ==================== Hygiene Tests (MDH-BUG-FLU-00037..00042) ====================

    #[test]
    fn test_js_escape_covers_every_string_form() {
        assert_eq!(
            js_escape("a\\b\"c'd`e${f}\n\r\t\u{0}\u{7f}\u{2028}\u{2029}é"),
            "a\\\\b\\\"c\\'d\\`e\\${f}\\n\\r\\t\\u0000\\u007f\\u2028\\u2029é"
        );
        assert_eq!(js_string("hi"), "\"hi\"");
    }

    #[test]
    fn test_js_ident_mangles_reserved_and_runtime_names() {
        for word in ["new", "class", "this", "var", "function", "let", "typeof"] {
            assert_eq!(js_ident(word), format!("{}$", word));
        }
        for word in ["delete", "default", "static", "arguments", "eval"] {
            assert_eq!(js_ident(word), format!("{}$", word));
        }
        assert_eq!(js_ident("__havers"), "__havers$");
        assert_eq!(js_ident("_private"), "_private");
        assert_eq!(js_ident("len"), "len");
    }

    #[test]
    fn test_module_binding_name_is_a_valid_identifier() {
        assert_eq!(module_binding_name("lib/my-mod.braw"), "my_mod");
        assert_eq!(module_binding_name("2fast.js"), "_2fast");
        assert_eq!(module_binding_name("dir\\x.y.braw"), "x_y");
        assert_eq!(module_binding_name(""), "_");
    }

    #[test]
    fn test_match_string_pattern_is_escaped() {
        let result = compile(
            r#"keek "x" {
    whan "\" || console.log(\"INJECTED\") || \"" -> blether "bad"
    whan _ -> blether "ok"
}"#,
        )
        .unwrap();
        assert!(result.contains(r#"=== "\" || console.log(\"INJECTED\") || \"""#));
        assert!(!result.contains(r#"=== "" || console.log"#));
    }

    #[test]
    fn test_import_path_is_escaped_and_binding_mangled() {
        let result = compile("fetch \"x'); console.log('INJECTED'); ('\"").unwrap();
        assert!(result.contains(r#"require('x\'); console.log(\'INJECTED\'); (\'')"#));
        let result = compile("fetch \"lib/my-mod\"").unwrap();
        assert!(result.contains("let my_mod = __havers.foreign(require('lib/my-mod'));"));
    }

    #[test]
    fn test_fstring_text_escapes_backslashes() {
        let result = compile(
            r#"ken who = "nae"
f"C:\\users\\{who}""#,
        )
        .unwrap();
        assert!(result.contains(r#"`C:\\users\\${__havers.show(who)}`"#));
    }

    #[test]
    fn test_reserved_identifiers_are_mangled() {
        let result = compile("ken new = 1\nken class = 2\ndae function(var) { gie var }\nblether function(new + class)").unwrap();
        assert!(result.contains("let new$ = 1n;"));
        assert!(result.contains("let class$ = 2n;"));
        assert!(result.contains("function function$(var$)"));
        assert!(!result.contains("let new ="));
    }

    #[test]
    fn test_user_code_shadows_runtime_helpers() {
        let result = compile("dae len(x) { gie 99 }\nken max = 5").unwrap();
        // Helpers are bound wi' `let` inside the user block, never as module-level consts
        assert!(result.contains("let { len, creel,"));
        assert!(result.contains(" whit_kind, "));
        assert!(!result.contains("const { len"));
        assert!(result.contains("len = function len(x) {"));
        assert!(result.contains("\n  max = 5n;"));
    }

    #[test]
    fn test_ken_redeclaration_and_loop_variable() {
        let result = compile("ken x = 1\nken x = x + 1\nfer i in 0..2 { i = i + 1 }").unwrap();
        assert!(result.contains("let x = 1n;"));
        assert!(result.contains("\n  x = __havers.add(x, 1n, 2);"));
        assert!(result.contains("for (let i of __havers.iter("));
    }

    #[test]
    fn test_inner_ken_shadowing_outer_name_is_renamed() {
        let result = compile("ken x = 1\ngin aye { ken x = x + 1\nblether x }").unwrap();
        assert!(result.contains("let x$1 = __havers.add(x, 1n, 2);"));
        assert!(result.contains("__havers.blether(x$1);"));
    }

    #[test]
    fn test_subclass_init_and_new_decision() {
        let result = compile(
            r#"kin Animal { dae init(name) { masel.name = name } }
kin Dog fae Animal { dae init(name) { masel.name = name } }
dae Square(n) { gie n * n }
ken d = Dog("Rex")
blether Square(3)"#,
        )
        .unwrap();
        assert!(result.contains("let Dog = __havers.kin(class Dog extends Animal {"));
        assert!(result.contains("new Dog(\"Rex\")"));
        assert!(result.contains("Square(3n)"));
        assert!(!result.contains("new Square"));
        // Only the root class has a constructor; it dispatches tae the derived init.
        assert_eq!(result.matches("constructor(...args)").count(), 1);
    }

    #[test]
    fn test_rebound_class_name_is_called_plainly() {
        let result = compile("kin A { }\nken A = |x| x\nblether A(1)").unwrap();
        assert!(!result.contains("new A("));
        assert!(result.contains("A(1n)"));
    }

    #[test]
    fn test_dict_literal_keys_are_expressions() {
        let result = compile("ken k = \"colour\"\nken d = {k: 1, \"a\": 2}").unwrap();
        assert!(result.contains("{[__havers.dkey(k)]: 1n, \"a\": 2n}"));
    }

    #[test]
    fn test_dict_literal_encodes_keys_js_would_reorder() {
        let result = compile("ken d = {\"b\": 1, \"1\": 2, \"__proto__\": 3, \"01\": 4}").unwrap();
        assert!(result.contains(
            "{\"b\": 1n, [__havers.dkey(\"1\")]: 2n, [__havers.dkey(\"__proto__\")]: 3n, \"01\": 4n}"
        ));
        assert!(is_plain_dict_key("") && is_plain_dict_key("-1") && is_plain_dict_key("1.5"));
        assert!(!is_plain_dict_key("0") && !is_plain_dict_key("\0x"));
    }

    #[test]
    fn test_truthiness_and_equality_lowering() {
        let result =
            compile("ken xs = []\ngin xs { blether 1 }\nblether xs == [1]\nblether 1 == 1")
                .unwrap();
        assert!(result.contains("if (__havers.truthy(xs))"));
        assert!(result.contains("__havers.eqOp(xs, [1n])"));
        assert!(result.contains("__havers.eqOp(1n, 1n)"));
    }

    #[test]
    fn test_literal_equality_routes_through_overload_helpers_mdh_bug_flu_00126() {
        let result = compile("ken x = 1\nblether x == \"a\"\nblether aye != x").unwrap();
        assert!(result.contains("__havers.eqOp(x, \"a\")"));
        assert!(result.contains("__havers.neOp(true, x)"));
        let both = compile("blether \"a\" == \"b\"\nblether aye != nae").unwrap();
        assert!(both.contains("(\"a\" === \"b\")"));
        assert!(both.contains("(true !== false)"));
    }

    #[test]
    fn test_numeric_literal_receiver_is_parenthesised() {
        let result = compile("blether 123[0:1]\nblether 1.5.x").unwrap();
        assert!(result.contains("__havers.slice(123n, 0n, 1n, null, 1)"));
        assert!(result.contains("__havers.getProp(1.5, \"x\", 2)"));
    }

    #[test]
    fn test_division_lowering() {
        let result = compile("ken a = 7\nblether 7 / 2\nblether a / 2\nblether a % 2").unwrap();
        assert!(result.contains("__havers.div(7n, 2n, 2)"));
        assert!(result.contains("__havers.div(a, 2n, 3)"));
        assert!(result.contains("__havers.mod(a, 2n, 4)"));
    }
}
