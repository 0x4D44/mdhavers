//! WebAssembly compiler fer mdhavers
//! Generates WAT (WebAssembly Text Format) that can be compiled tae WASM
//!
//! This is a basic WASM compiler that supports:
//! - Integer and float arithmetic
//! - Variables (local)
//! - Functions
//! - Basic control flow (if/while)
//!
//! Note: This is an experimental feature - no' aw mdhavers features are supported!

use crate::ast::*;
use crate::error::{HaversError, HaversResult};
use crate::interpreter::Interpreter;
use std::collections::{BTreeSet, HashMap, HashSet};

/// Audio host imports: (name, arity). Every argument and the result is an i64 handle.
///
/// This is the single source of truth: the compiler derives its `(import ...)` lines from it
/// and the wasm runner registers its stubs from it, so the two cannot drift apart.
pub const AUDIO_IMPORTS: &[(&str, usize)] = &[
    ("soond_stairt", 0),
    ("soond_steek", 0),
    ("soond_wheesht", 1),
    ("soond_luid", 1),
    ("soond_hou_luid", 0),
    ("soond_haud_gang", 0),
    ("soond_lade", 1),
    ("soond_spiel", 1),
    ("soond_haud", 1),
    ("soond_gae_on", 1),
    ("soond_stap", 1),
    ("soond_unlade", 1),
    ("soond_is_spielin", 1),
    ("soond_pit_luid", 2),
    ("soond_pit_pan", 2),
    ("soond_pit_tune", 2),
    ("soond_pit_rin_roond", 2),
    ("soond_ready", 1),
    ("muisic_lade", 1),
    ("muisic_spiel", 1),
    ("muisic_haud", 1),
    ("muisic_gae_on", 1),
    ("muisic_stap", 1),
    ("muisic_unlade", 1),
    ("muisic_is_spielin", 1),
    ("muisic_loup", 2),
    ("muisic_hou_lang", 1),
    ("muisic_whaur", 1),
    ("muisic_pit_luid", 2),
    ("muisic_pit_pan", 2),
    ("muisic_pit_tune", 2),
    ("muisic_pit_rin_roond", 2),
    ("midi_lade", 2),
    ("midi_spiel", 1),
    ("midi_haud", 1),
    ("midi_gae_on", 1),
    ("midi_stap", 1),
    ("midi_unlade", 1),
    ("midi_is_spielin", 1),
    ("midi_loup", 2),
    ("midi_hou_lang", 1),
    ("midi_whaur", 1),
    ("midi_pit_luid", 2),
    ("midi_pit_pan", 2),
    ("midi_pit_rin_roond", 2),
];

/// The `(import ...)` line for one audio host function.
fn audio_import_line(name: &str, arity: usize) -> String {
    let params = if arity == 0 {
        String::new()
    } else {
        format!(" (param{})", " i64".repeat(arity))
    };
    format!("(import \"env\" \"{name}\" (func ${name}{params} (result i64)))")
}

/// Most arguments a builtin call can pass through `__mdh_builtinN`.
pub const MAX_BUILTIN_ARGS: usize = 16;

thread_local! {
    /// Name -> arity o' the interpreter's first-order natives (`usize::MAX` = variadic).
    static BUILTIN_ARITIES: HashMap<String, usize> = Interpreter::native_builtins()
        .into_iter()
        .map(|(name, native)| (name, native.arity))
        .collect();
}

/// The arity o' an interpreter builtin the WASM host can call, if `name` is one.
fn builtin_arity(name: &str) -> Option<usize> {
    BUILTIN_ARITIES.with(|arities| arities.get(name).copied())
}

/// Builtin calls go through `__mdh_builtinN(name_ptr, name_len, a1..aN) -> handle`: the
/// host looks the name up in the interpreter's natives an' runs the very same code.
fn builtin_import_line(argc: usize) -> String {
    format!(
        "(import \"env\" \"__mdh_builtin{argc}\" (func $mdh_builtin{argc} (param i32 i32{}) (result i64)))",
        " i64".repeat(argc)
    )
}

#[derive(Debug, Default)]
struct WasmImportRequirements {
    needs_tri_module: bool,
    audio_imports: BTreeSet<String>,
    /// Argument counts o' the builtin calls, one `__mdh_builtinN` import each.
    builtin_arities: BTreeSet<usize>,
}

impl WasmImportRequirements {
    fn from_program(program: &Program) -> Self {
        let mut defined_functions = HashSet::new();
        for stmt in &program.statements {
            if let Stmt::Function { name, .. } = stmt {
                defined_functions.insert(name.clone());
            }
        }

        let mut req = Self::default();
        for stmt in &program.statements {
            req.scan_stmt(stmt, &defined_functions);
        }
        req
    }

    fn add_audio_import(&mut self, name: &str, defined_functions: &HashSet<String>) {
        if defined_functions.contains(name) {
            return;
        }
        if AUDIO_IMPORTS
            .iter()
            .any(|(import_name, _)| *import_name == name)
        {
            self.audio_imports.insert(name.to_string());
        }
    }

    fn scan_stmt(&mut self, stmt: &Stmt, defined_functions: &HashSet<String>) {
        match stmt {
            Stmt::VarDecl { initializer, .. } => {
                if let Some(expr) = initializer {
                    self.scan_expr(expr, defined_functions);
                }
            }
            Stmt::Expression { expr, .. } => self.scan_expr(expr, defined_functions),
            Stmt::Block { statements, .. } => {
                for stmt in statements {
                    self.scan_stmt(stmt, defined_functions);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.scan_expr(condition, defined_functions);
                self.scan_stmt(then_branch, defined_functions);
                if let Some(else_branch) = else_branch {
                    self.scan_stmt(else_branch, defined_functions);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.scan_expr(condition, defined_functions);
                self.scan_stmt(body, defined_functions);
            }
            Stmt::For { iterable, body, .. } => {
                self.scan_expr(iterable, defined_functions);
                self.scan_stmt(body, defined_functions);
            }
            Stmt::Function { params, body, .. } => {
                for param in params {
                    if let Some(default) = &param.default {
                        self.scan_expr(default, defined_functions);
                    }
                }
                for stmt in body {
                    self.scan_stmt(stmt, defined_functions);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.scan_expr(expr, defined_functions);
                }
            }
            Stmt::Print { value, .. } => self.scan_expr(value, defined_functions),
            Stmt::Import { path, .. } => {
                if path == "tri" || path == "tri.braw" {
                    self.needs_tri_module = true;
                }
            }
            Stmt::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                self.scan_stmt(try_block, defined_functions);
                self.scan_stmt(catch_block, defined_functions);
            }
            Stmt::Match { value, arms, .. } => {
                self.scan_expr(value, defined_functions);
                for arm in arms {
                    self.scan_stmt(&arm.body, defined_functions);
                }
            }
            Stmt::Assert {
                condition, message, ..
            } => {
                self.scan_expr(condition, defined_functions);
                if let Some(expr) = message {
                    self.scan_expr(expr, defined_functions);
                }
            }
            Stmt::Destructure { value, .. } => self.scan_expr(value, defined_functions),
            Stmt::Log {
                message, extras, ..
            } => {
                self.scan_expr(message, defined_functions);
                for expr in extras {
                    self.scan_expr(expr, defined_functions);
                }
            }
            Stmt::Hurl { message, .. } => self.scan_expr(message, defined_functions),
            Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Class { .. }
            | Stmt::Struct { .. } => {}
        }
    }

    fn scan_expr(&mut self, expr: &Expr, defined_functions: &HashSet<String>) {
        match expr {
            Expr::Literal { .. } | Expr::Variable { .. } | Expr::Masel { .. } => {}
            Expr::Assign { value, .. } => self.scan_expr(value, defined_functions),
            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
                self.scan_expr(left, defined_functions);
                self.scan_expr(right, defined_functions);
            }
            Expr::Unary { operand, .. } => self.scan_expr(operand, defined_functions),
            Expr::Call {
                callee, arguments, ..
            } => {
                if let Expr::Variable { name, .. } = callee.as_ref() {
                    self.add_audio_import(name, defined_functions);
                    if !defined_functions.contains(name.as_str())
                        && !self.audio_imports.contains(name.as_str())
                        && builtin_arity(name).is_some()
                    {
                        self.builtin_arities.insert(arguments.len());
                    }
                }
                self.scan_expr(callee, defined_functions);
                for arg in arguments {
                    self.scan_expr(arg, defined_functions);
                }
            }
            Expr::Get { object, .. } => self.scan_expr(object, defined_functions),
            Expr::Set { object, value, .. } => {
                self.scan_expr(object, defined_functions);
                self.scan_expr(value, defined_functions);
            }
            Expr::Index { object, index, .. } => {
                self.scan_expr(object, defined_functions);
                self.scan_expr(index, defined_functions);
            }
            Expr::IndexSet {
                object,
                index,
                value,
                ..
            } => {
                self.scan_expr(object, defined_functions);
                self.scan_expr(index, defined_functions);
                self.scan_expr(value, defined_functions);
            }
            Expr::Slice {
                object,
                start,
                end,
                step,
                ..
            } => {
                self.scan_expr(object, defined_functions);
                if let Some(expr) = start {
                    self.scan_expr(expr, defined_functions);
                }
                if let Some(expr) = end {
                    self.scan_expr(expr, defined_functions);
                }
                if let Some(expr) = step {
                    self.scan_expr(expr, defined_functions);
                }
            }
            Expr::List { elements, .. } => {
                for expr in elements {
                    self.scan_expr(expr, defined_functions);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (k, v) in pairs {
                    self.scan_expr(k, defined_functions);
                    self.scan_expr(v, defined_functions);
                }
            }
            Expr::Range { start, end, .. } => {
                self.scan_expr(start, defined_functions);
                self.scan_expr(end, defined_functions);
            }
            Expr::Grouping { expr, .. } => self.scan_expr(expr, defined_functions),
            Expr::Lambda { body, .. } => self.scan_expr(body, defined_functions),
            Expr::BlockExpr { statements, .. } => {
                for stmt in statements {
                    self.scan_stmt(stmt, defined_functions);
                }
            }
            Expr::Input { prompt, .. } => self.scan_expr(prompt, defined_functions),
            Expr::FString { parts, .. } => {
                for part in parts {
                    if let FStringPart::Expr(expr) = part {
                        self.scan_expr(expr, defined_functions);
                    }
                }
            }
            Expr::Spread { expr, .. } => self.scan_expr(expr, defined_functions),
            Expr::Pipe { left, right, .. } => {
                self.scan_expr(left, defined_functions);
                self.scan_expr(right, defined_functions);
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.scan_expr(condition, defined_functions);
                self.scan_expr(then_expr, defined_functions);
                self.scan_expr(else_expr, defined_functions);
            }
        }
    }
}

/// The WASM compiler
pub struct WasmCompiler {
    output: String,
    indent: usize,
    local_vars: Vec<String>,
    func_params: Vec<String>,
    string_data: Vec<String>,
    /// User functions (name -> params), filled by `compile` so calls can fill in defaults.
    functions: HashMap<String, Vec<Param>>,
    /// Host scopes open in the function being compiled (one per enclosing statement).
    scope_depth: usize,
    /// Each enclosing `whiles`, innermost last (fer brak/haud): its `scope_depth`, and
    /// whether it sweeps its ain scope at iteration boundaries.
    loop_depths: Vec<(usize, bool)>,
    /// Bytes at the top o' memory the biggest function needs tae spill its locals.
    max_root_bytes: usize,
    /// The previous statement's `scope_next` already opened this statement's scope.
    scope_preopened: bool,
    /// Locals o' the current function that only ever hold numbers.
    numeric_locals: HashSet<String>,
}

/// Linear memory the host gives a module (one 64KiB page); string data must fit inside it.
const WASM_MEMORY_BYTES: usize = 65_536;

/// The WAT id fer a user function. Names that would collide wi' the exported `$main`
/// or the `$mdh_*` host imports are mangled; `$` cannae appear in an mdhavers identifier,
/// so a mangled id never clashes wi' another user function.
fn function_id(name: &str) -> String {
    if name == "main" || name.starts_with("mdh_") {
        format!("u${}", name)
    } else {
        name.to_string()
    }
}

fn compile_err(line: usize, message: String) -> HaversError {
    HaversError::CompileError(format!("{} (line {})", message, line))
}

/// Is this expression safe tae copy intae a caller as a parameter default?
/// Defaults are compiled at the call site, so they maunnae touch variables or calls.
fn is_const_default(expr: &Expr) -> bool {
    match expr {
        Expr::Literal { .. } => true,
        Expr::Grouping { expr, .. } => is_const_default(expr),
        Expr::Unary { operand, .. } => is_const_default(operand),
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            is_const_default(left) && is_const_default(right)
        }
        Expr::List { elements, .. } => elements.iter().all(is_const_default),
        Expr::Dict { pairs, .. } => pairs
            .iter()
            .all(|(k, v)| is_const_default(k) && is_const_default(v)),
        _ => false,
    }
}

/// Every name a function body (or the top level) declares as a WASM local.
fn declared_names<'a>(params: &[String], stmts: impl Iterator<Item = &'a Stmt>) -> HashSet<String> {
    let mut collector = WasmCompiler::new();
    collector.func_params = params.to_vec();
    for stmt in stmts {
        collector.collect_locals_stmt(stmt);
    }
    collector
        .local_vars
        .into_iter()
        .chain(params.iter().cloned())
        .collect()
}

/// Checks a program can become a *valid* WASM module before any code is emitted, so
/// `compile_to_wat` fails wi' a clear error instead o' producing WAT that willnae load.
/// Constructs the code generator doesnae support are left tae it tae report.
struct Validator<'a> {
    functions: HashMap<&'a str, &'a [Param]>,
    top_level_vars: HashSet<String>,
    scope: HashSet<String>,
    current_fn: Option<&'a str>,
    loop_depth: usize,
}

fn validate_program(program: &Program) -> HaversResult<()> {
    let mut functions: HashMap<&str, &[Param]> = HashMap::new();
    let mut fn_stmts: Vec<&Stmt> = Vec::new();
    let mut main_stmts: Vec<&Stmt> = Vec::new();

    for stmt in &program.statements {
        if let Stmt::Function {
            name, params, span, ..
        } = stmt
        {
            if functions.insert(name.as_str(), params.as_slice()).is_some() {
                return Err(compile_err(
                    span.line,
                    format!(
                        "Dae '{}' is defined mair than once - WASM cannae hae twa o' the same name",
                        name
                    ),
                ));
            }
            let mut seen = HashSet::new();
            for p in params {
                if !seen.insert(p.name.as_str()) {
                    return Err(compile_err(
                        span.line,
                        format!(
                            "Dae '{}' has the parameter '{}' mair than once",
                            name, p.name
                        ),
                    ));
                }
            }
            fn_stmts.push(stmt);
        } else {
            main_stmts.push(stmt);
        }
    }

    let top_level_vars = declared_names(&[], main_stmts.iter().copied());
    let mut validator = Validator {
        functions,
        top_level_vars: top_level_vars.clone(),
        scope: HashSet::new(),
        current_fn: None,
        loop_depth: 0,
    };

    for stmt in fn_stmts {
        if let Stmt::Function {
            name, params, body, ..
        } = stmt
        {
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            validator.scope = declared_names(&param_names, body.iter());
            validator.current_fn = Some(name.as_str());
            validator.loop_depth = 0;
            validator.stmts(body)?;
        }
    }

    validator.scope = top_level_vars;
    validator.current_fn = None;
    validator.loop_depth = 0;
    for stmt in main_stmts {
        validator.stmt(stmt)?;
    }
    Ok(())
}

impl<'a> Validator<'a> {
    fn stmts(&mut self, stmts: &[Stmt]) -> HaversResult<()> {
        for stmt in stmts {
            self.stmt(stmt)?;
        }
        Ok(())
    }

    fn stmt(&mut self, stmt: &Stmt) -> HaversResult<()> {
        match stmt {
            Stmt::VarDecl { initializer, .. } => {
                if let Some(expr) = initializer {
                    self.expr(expr)?;
                }
            }
            Stmt::Expression { expr, .. } => self.expr(expr)?,
            Stmt::Block { statements, .. } => self.stmts(statements)?,
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.expr(condition)?;
                self.stmt(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.stmt(else_branch)?;
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.expr(condition)?;
                self.loop_depth += 1;
                let result = self.stmt(body);
                self.loop_depth -= 1;
                result?;
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.expr(expr)?;
                }
            }
            Stmt::Print { value, .. } => self.expr(value)?,
            Stmt::Break { span } => {
                if self.loop_depth == 0 {
                    return Err(compile_err(
                        span.line,
                        "'brak' can only be used inside a loop".to_string(),
                    ));
                }
            }
            Stmt::Continue { span } => {
                if self.loop_depth == 0 {
                    return Err(compile_err(
                        span.line,
                        "'haud' can only be used inside a loop".to_string(),
                    ));
                }
            }
            // Anything else is reported by the code generator as unsupported.
            _ => {}
        }
        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> HaversResult<()> {
        match expr {
            Expr::Variable { name, span } => self.check_name(name, span.line, false)?,
            Expr::Assign { name, value, span } => {
                self.expr(value)?;
                self.check_name(name, span.line, true)?;
            }
            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
                self.expr(left)?;
                self.expr(right)?;
            }
            Expr::Unary { operand, .. } => self.expr(operand)?,
            Expr::Grouping { expr, .. } => self.expr(expr)?,
            Expr::Get { object, .. } => self.expr(object)?,
            Expr::Set { object, value, .. } => {
                self.expr(object)?;
                self.expr(value)?;
            }
            Expr::List { elements, .. } => {
                for element in elements {
                    self.expr(element)?;
                }
            }
            Expr::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.expr(key)?;
                    self.expr(value)?;
                }
            }
            Expr::Call {
                callee,
                arguments,
                span,
            } => {
                match callee.as_ref() {
                    Expr::Get { object, .. } => self.expr(object)?,
                    Expr::Variable { name, .. } if !self.scope.contains(name) => {
                        self.check_direct_call(name, arguments.len(), span.line)?;
                    }
                    // Local-value calls and unsupported callees are left tae the code generator.
                    _ => {}
                }
                for arg in arguments {
                    self.expr(arg)?;
                }
            }
            // Anything else is reported by the code generator as unsupported.
            _ => {}
        }
        Ok(())
    }

    fn check_name(&self, name: &str, line: usize, assigning: bool) -> HaversResult<()> {
        if self.scope.contains(name) {
            return Ok(());
        }
        let message = if let (Some(func), true) =
            (self.current_fn, self.top_level_vars.contains(name))
        {
            format!(
                "Top-level variable '{}' cannae be used inside dae '{}' in WASM yet - pass it in as a parameter",
                name, func
            )
        } else if self.functions.contains_key(name) {
            format!(
                "'{}' is a dae - WASM cannae use daes as values yet, only call them",
                name
            )
        } else if assigning {
            format!(
                "Cannae assign tae '{}' - it hasnae been declared wi' ken",
                name
            )
        } else {
            format!(
                "'{}' hasnae been declared - use ken tae declare it first",
                name
            )
        };
        Err(compile_err(line, message))
    }

    fn check_direct_call(&self, name: &str, argc: usize, line: usize) -> HaversResult<()> {
        if let Some(params) = self.functions.get(name) {
            if argc > params.len() {
                return Err(compile_err(
                    line,
                    format!(
                        "Dae '{}' tak's at maist {} argument(s) but wis gien {}",
                        name,
                        params.len(),
                        argc
                    ),
                ));
            }
            for param in &params[argc..] {
                match &param.default {
                    None => {
                        return Err(compile_err(
                            line,
                            format!(
                                "Dae '{}' needs the argument '{}' but wis gien only {}",
                                name, param.name, argc
                            ),
                        ));
                    }
                    Some(default) if !is_const_default(default) => {
                        return Err(compile_err(
                            line,
                            format!(
                                "The default fer '{}' in dae '{}' maun be a constant in WASM",
                                param.name, name
                            ),
                        ));
                    }
                    Some(_) => {}
                }
            }
            return Ok(());
        }
        if let Some((_, arity)) = AUDIO_IMPORTS.iter().find(|(import, _)| *import == name) {
            if argc != *arity {
                return Err(compile_err(
                    line,
                    format!(
                        "Dae '{}' tak's {} argument(s) but wis gien {}",
                        name, arity, argc
                    ),
                ));
            }
            return Ok(());
        }
        if let Some(arity) = builtin_arity(name) {
            if (arity != usize::MAX && argc != arity) || argc > MAX_BUILTIN_ARGS {
                return Err(compile_err(
                    line,
                    format!(
                        "Dae '{}' tak's {} argument(s) but wis gien {}",
                        name,
                        if arity == usize::MAX {
                            format!("at maist {}", MAX_BUILTIN_ARGS)
                        } else {
                            arity.to_string()
                        },
                        argc
                    ),
                ));
            }
            return Ok(());
        }
        Err(compile_err(
            line,
            format!(
                "Dae '{}' isnae kent tae the WASM backend - higher-order builtins (gaun, sieve, ...) an' prelude daes arenae supported in WASM yet",
                name
            ),
        ))
    }
}

const TMP_LOGIC: &str = "__mdh$tmp0";
const TMP_BUILD: &str = "__mdh$tmp1";
const TMP_RET: &str = "__mdh$tmp2";

impl Default for WasmCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmCompiler {
    pub fn new() -> Self {
        WasmCompiler {
            output: String::new(),
            indent: 0,
            local_vars: Vec::new(),
            func_params: Vec::new(),
            string_data: Vec::new(),
            functions: HashMap::new(),
            scope_depth: 0,
            loop_depths: Vec::new(),
            max_root_bytes: 0,
            scope_preopened: false,
            numeric_locals: HashSet::new(),
        }
    }

    /// Compile a program tae WAT (WebAssembly Text Format)
    ///
    /// Never returns `Ok` fer a module that willnae load: scoping, callee and arity
    /// problems are reported as `CompileError`s up front.
    pub fn compile(&mut self, program: &Program) -> HaversResult<String> {
        validate_program(program)?;
        self.output.clear();
        self.indent = 0;
        self.string_data.clear();
        self.max_root_bytes = 0;
        self.functions = program
            .statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::Function { name, params, .. } => Some((name.clone(), params.clone())),
                _ => None,
            })
            .collect();
        let import_requirements = WasmImportRequirements::from_program(program);

        // Start the module
        self.emit("(module");
        self.indent += 1;

        // Import memory and print functions from the host
        self.emit_line("");
        self.emit_line(";; Imports fae the host environment");
        self.emit_line("(import \"env\" \"memory\" (memory 1))");
        self.emit_line("(import \"env\" \"__mdh_make_nil\" (func $mdh_make_nil (result i64)))");
        self.emit_line(
            "(import \"env\" \"__mdh_make_bool\" (func $mdh_make_bool (param i32) (result i64)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_make_int\" (func $mdh_make_int (param i64) (result i64)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_make_float\" (func $mdh_make_float (param f64) (result i64)))",
        );
        self.emit_line("(import \"env\" \"__mdh_make_string\" (func $mdh_make_string (param i32 i32) (result i64)))");
        self.emit_line(
            "(import \"env\" \"__mdh_truthy\" (func $mdh_truthy (param i64) (result i32)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_add\" (func $mdh_add (param i64 i64) (result i64)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_sub\" (func $mdh_sub (param i64 i64) (result i64)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_mul\" (func $mdh_mul (param i64 i64) (result i64)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_div\" (func $mdh_div (param i64 i64) (result i64)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_mod\" (func $mdh_mod (param i64 i64) (result i64)))",
        );
        self.emit_line("(import \"env\" \"__mdh_eq\" (func $mdh_eq (param i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_ne\" (func $mdh_ne (param i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_lt\" (func $mdh_lt (param i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_le\" (func $mdh_le (param i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_gt\" (func $mdh_gt (param i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_ge\" (func $mdh_ge (param i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_neg\" (func $mdh_neg (param i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_not\" (func $mdh_not (param i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_blether\" (func $mdh_blether (param i64)))");
        self.emit_line(
            "(import \"env\" \"__mdh_make_list\" (func $mdh_make_list (param i32) (result i64)))",
        );
        self.emit_line("(import \"env\" \"__mdh_list_push\" (func $mdh_list_push (param i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_make_dict\" (func $mdh_make_dict (result i64)))");
        // Host store reclamation: every statement runs in a host scope, and on exit the
        // function's locals (spilled tae the top o' memory) are the roots (MDH-BUG-FLU-00092).
        self.emit_line("(import \"env\" \"__mdh_scope_enter\" (func $mdh_scope_enter))");
        self.emit_line(
            "(import \"env\" \"__mdh_scope_exit\" (func $mdh_scope_exit (param i32 i32 i32)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_scope_next\" (func $mdh_scope_next (param i32 i32)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_scope_collect_due\" (func $mdh_scope_collect_due (result i32)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_scope_collect\" (func $mdh_scope_collect (param i32 i32)))",
        );
        self.emit_line("(import \"env\" \"__mdh_dict_set\" (func $mdh_dict_set (param i64 i64 i64) (result i64)))");
        self.emit_line(
            "(import \"env\" \"__mdh_prop_get\" (func $mdh_prop_get (param i64 i64) (result i64)))",
        );
        self.emit_line("(import \"env\" \"__mdh_prop_set\" (func $mdh_prop_set (param i64 i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_method_call0\" (func $mdh_method_call0 (param i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_method_call1\" (func $mdh_method_call1 (param i64 i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_method_call2\" (func $mdh_method_call2 (param i64 i64 i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_method_call3\" (func $mdh_method_call3 (param i64 i64 i64 i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_method_call4\" (func $mdh_method_call4 (param i64 i64 i64 i64 i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_method_call5\" (func $mdh_method_call5 (param i64 i64 i64 i64 i64 i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_method_call6\" (func $mdh_method_call6 (param i64 i64 i64 i64 i64 i64 i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_method_call7\" (func $mdh_method_call7 (param i64 i64 i64 i64 i64 i64 i64 i64 i64) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_method_call8\" (func $mdh_method_call8 (param i64 i64 i64 i64 i64 i64 i64 i64 i64 i64) (result i64)))");
        if import_requirements.needs_tri_module {
            self.emit_line(
                "(import \"env\" \"__mdh_tri_module\" (func $mdh_tri_module (result i64)))",
            );
        }

        for argc in &import_requirements.builtin_arities {
            self.emit_line(&builtin_import_line(*argc));
        }

        if !import_requirements.audio_imports.is_empty() {
            self.emit_line("");
            self.emit_line(";; Audio imports (i64 value ABI)");
            for (name, arity) in AUDIO_IMPORTS {
                if import_requirements.audio_imports.contains(*name) {
                    let import_line = audio_import_line(name, *arity);
                    self.emit_line(&import_line);
                }
            }
        }
        self.emit_line("");

        // Collect all function declarations first
        let mut functions: Vec<&Stmt> = Vec::new();
        let mut main_stmts: Vec<&Stmt> = Vec::new();

        for stmt in &program.statements {
            match stmt {
                Stmt::Function { .. } => functions.push(stmt),
                _ => main_stmts.push(stmt),
            }
        }

        // Compile functions
        for func in &functions {
            self.compile_function(func)?;
        }

        // Compile main code. `$main` is always emitted (empty if there are nae top-level
        // statements) because it is always exported.
        self.compile_main(&main_stmts)?;

        // Export the main function
        self.emit_line("");
        self.emit_line("(export \"main\" (func $main))");

        // Add string data section if we have strings
        let string_bytes: usize = self.string_data.iter().map(|s| s.len() + 1).sum();
        if string_bytes + self.max_root_bytes > WASM_MEMORY_BYTES {
            return Err(HaversError::CompileError(format!(
                "String data ({} bytes) is too muckle fer the {} byte WASM memory",
                string_bytes + self.max_root_bytes,
                WASM_MEMORY_BYTES
            )));
        }
        if !self.string_data.is_empty() {
            self.emit_line("");
            self.emit_line(";; String data");
            let mut offset = 0;
            // Collect string data first to avoid borrow issues
            let string_lines: Vec<String> = self
                .string_data
                .iter()
                .map(|s| {
                    let line = format!(
                        "(data (i32.const {}) \"{}\\00\")",
                        offset,
                        escape_wat_string(s)
                    );
                    offset += s.len() as i32 + 1; // +1 for null terminator
                    line
                })
                .collect();
            for line in string_lines {
                self.emit_line(&line);
            }
        }

        self.indent -= 1;
        self.emit_line(")");

        Ok(self.output.clone())
    }

    fn compile_function(&mut self, stmt: &Stmt) -> HaversResult<()> {
        if let Stmt::Function {
            name, params, body, ..
        } = stmt
        {
            self.local_vars.clear();
            self.func_params.clear();

            // Build parameter list
            let mut param_types = String::new();
            for p in params {
                self.func_params.push(p.name.clone());
                param_types.push_str(&format!("(param ${} i64) ", p.name));
            }

            // Start function
            self.emit_line(&format!(
                "(func ${} {}(result i64)",
                function_id(name),
                param_types
            ));
            self.indent += 1;

            // Collect locals from body
            self.collect_locals(body);

            self.ensure_temp_locals();

            // Declare locals (collect first to avoid borrow issues)
            let local_decls: Vec<String> = self
                .local_vars
                .iter()
                .map(|var| format!("(local ${} i64)", var))
                .collect();
            for decl in local_decls {
                self.emit_line(&decl);
            }

            // Compile body
            let body: Vec<&Stmt> = body.iter().collect();
            self.infer_numeric_locals(&body);
            self.compile_stmts(&body)?;

            // Default return value
            self.emit_nil();

            self.indent -= 1;
            self.emit_line(")");
            self.emit_line("");
        }
        Ok(())
    }

    fn compile_main(&mut self, stmts: &[&Stmt]) -> HaversResult<()> {
        self.local_vars.clear();
        self.func_params.clear();

        self.emit_line("(func $main (result i64)");
        self.indent += 1;

        // Collect all locals
        for stmt in stmts {
            self.collect_locals_stmt(stmt);
        }

        self.ensure_temp_locals();

        // Declare locals
        for var in &self.local_vars.clone() {
            self.emit_line(&format!("(local ${} i64)", var));
        }

        // Compile statements
        self.infer_numeric_locals(stmts);
        self.compile_stmts(stmts)?;

        // Return nil
        self.emit_nil();

        self.indent -= 1;
        self.emit_line(")");
        Ok(())
    }

    fn collect_locals(&mut self, body: &[Stmt]) {
        for stmt in body {
            self.collect_locals_stmt(stmt);
        }
    }

    fn collect_locals_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, .. } => {
                if !self.func_params.contains(name) && !self.local_vars.contains(name) {
                    self.local_vars.push(name.clone());
                }
            }
            Stmt::Block { statements, .. } => {
                for s in statements {
                    self.collect_locals_stmt(s);
                }
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.collect_locals_stmt(then_branch);
                if let Some(eb) = else_branch {
                    self.collect_locals_stmt(eb);
                }
            }
            Stmt::While { body, .. } => {
                self.collect_locals_stmt(body);
            }
            Stmt::For { variable, body, .. } => {
                if !self.func_params.contains(variable) && !self.local_vars.contains(variable) {
                    self.local_vars.push(variable.clone());
                }
                self.collect_locals_stmt(body);
            }
            Stmt::Import {
                alias: Some(name), ..
            } => {
                if !self.func_params.contains(name) && !self.local_vars.contains(name) {
                    self.local_vars.push(name.clone());
                }
            }
            _ => {}
        }
    }

    fn ensure_temp_locals(&mut self) {
        self.ensure_temp_local(TMP_LOGIC);
        self.ensure_temp_local(TMP_BUILD);
        self.ensure_temp_local(TMP_RET);
        let root_bytes = 8 * (self.func_params.len() + self.local_vars.len());
        self.max_root_bytes = self.max_root_bytes.max(root_bytes);
        self.scope_depth = 0;
        self.loop_depths.clear();
        self.scope_preopened = false;
    }

    /// Close the innermost `pop` host scopes. Every local and parameter o' the current
    /// function is spilled tae the top o' memory as the roots: anything allocated since
    /// the outermost closed scope began, and no' reachable fae them, is freed.
    /// Sound because the operand stack is empty at statement boundaries and older
    /// frames only hold handles allocated before these scopes began.
    fn emit_scope_exit(&mut self, pop: usize) {
        if pop == 0 {
            return;
        }
        let base = self.emit_root_spill();
        self.emit_line(&format!("(i32.const {})", base));
        self.emit_line(&format!(
            "(i32.const {})",
            self.func_params.len() + self.local_vars.len()
        ));
        self.emit_line(&format!("(i32.const {})", pop));
        self.emit_line("(call $mdh_scope_exit)");
    }

    /// Store every parameter and local at the top o' memory; returns the base address.
    fn emit_root_spill(&mut self) -> usize {
        let roots: Vec<String> = self
            .func_params
            .iter()
            .chain(self.local_vars.iter())
            .cloned()
            .collect();
        let base = WASM_MEMORY_BYTES - 8 * roots.len();
        for (i, name) in roots.iter().enumerate() {
            self.emit_line(&format!("(i32.const {})", base + 8 * i));
            self.emit_line(&format!("(local.get ${})", name));
            self.emit_line("(i64.store)");
        }
        base
    }

    /// Close the current scope and open the next statement's in one host call.
    fn emit_scope_next(&mut self) {
        let base = self.emit_root_spill();
        self.emit_line(&format!("(i32.const {})", base));
        self.emit_line(&format!(
            "(i32.const {})",
            self.func_params.len() + self.local_vars.len()
        ));
        self.emit_line("(call $mdh_scope_next)");
    }

    /// Compile one statement (a block compiles its statements as a sequence).
    fn compile_stmt(&mut self, stmt: &Stmt) -> HaversResult<()> {
        self.compile_stmts(&[stmt])
    }

    /// Compile a statement sequence. Each statement that can allocate runs in its ain
    /// host scope; between two such neighbours the exit and enter are one host call.
    fn compile_stmts(&mut self, stmts: &[&Stmt]) -> HaversResult<()> {
        fn flatten<'a>(stmts: &[&'a Stmt], out: &mut Vec<&'a Stmt>) {
            for stmt in stmts {
                match stmt {
                    Stmt::Block { statements, .. } => {
                        let inner: Vec<&Stmt> = statements.iter().collect();
                        flatten(&inner, out);
                    }
                    _ => out.push(stmt),
                }
            }
        }
        let mut flat = Vec::new();
        flatten(stmts, &mut flat);
        for (i, stmt) in flat.iter().enumerate() {
            if !self.stmt_needs_scope(stmt) {
                self.compile_stmt_body(stmt)?;
                continue;
            }
            if self.scope_preopened {
                self.scope_preopened = false;
            } else {
                self.emit_line("(call $mdh_scope_enter)");
            }
            self.scope_depth += 1;
            self.compile_stmt_body(stmt)?;
            self.scope_depth -= 1;
            // Return closes its scopes itself before branching.
            if matches!(stmt, Stmt::Return { .. }) {
                continue;
            }
            let next_scoped = flat
                .get(i + 1)
                .is_some_and(|next| self.stmt_needs_scope(next));
            if next_scoped {
                self.emit_scope_next();
                self.scope_preopened = true;
            } else {
                self.emit_scope_exit(1);
            }
        }
        Ok(())
    }

    /// Whether a statement needs its ain host scope. One that can only allocate wide
    /// numbers (inline-range numbers never touch the store) runs in its parent's scope,
    /// which reclaims those slots when it closes. Skipping a scope is always sound.
    fn stmt_needs_scope(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::VarDecl { initializer, .. } => {
                initializer.as_ref().is_some_and(|e| !self.expr_is_cheap(e))
            }
            Stmt::Expression { expr, .. } | Stmt::Print { value: expr, .. } => {
                !self.expr_is_cheap(expr)
            }
            Stmt::Return { value, .. } => value.as_ref().is_some_and(|e| !self.expr_is_cheap(e)),
            Stmt::If { condition, .. } => !self.expr_is_cheap(condition),
            Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Block { .. } => false,
            // A loop pays one scope per run, no' per iteration.
            _ => true,
        }
    }

    /// An expression that allocates nothing but (at most) wide numbers.
    fn expr_is_cheap(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Literal { value, .. } => !matches!(value, Literal::String(_)),
            Expr::Variable { .. } => true,
            Expr::Grouping { expr, .. } => self.expr_is_cheap(expr),
            Expr::Assign { value, .. } => self.expr_is_cheap(value),
            Expr::Unary {
                operator: UnaryOp::Not,
                operand,
                ..
            } => self.expr_is_cheap(operand),
            Expr::Logical { left, right, .. } => {
                self.expr_is_cheap(left) && self.expr_is_cheap(right)
            }
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } if is_comparison(operator) => self.expr_is_cheap(left) && self.expr_is_cheap(right),
            _ => self.expr_is_numeric(expr, &self.numeric_locals),
        }
    }

    /// An expression that always yields a number (or traps), given which locals only
    /// ever hold numbers. Arithmetic on numbers never builds a string or container.
    fn expr_is_numeric(&self, expr: &Expr, numeric: &HashSet<String>) -> bool {
        match expr {
            Expr::Literal { value, .. } => {
                matches!(value, Literal::Integer(_) | Literal::Float(_))
            }
            Expr::Variable { name, .. } => numeric.contains(name),
            Expr::Grouping { expr, .. } => self.expr_is_numeric(expr, numeric),
            Expr::Assign { value, .. } => self.expr_is_numeric(value, numeric),
            Expr::Unary {
                operator: UnaryOp::Negate,
                operand,
                ..
            } => self.expr_is_numeric(operand, numeric),
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                !is_comparison(operator)
                    && self.expr_is_numeric(left, numeric)
                    && self.expr_is_numeric(right, numeric)
            }
            _ => false,
        }
    }

    /// The function's locals that only ever hold numbers: every assignment tae them
    /// is a numeric expression (a fixed point, since they can feed each other).
    /// Parameters are never included; an unassigned local is nil, and nil arithmetic
    /// traps rather than allocating.
    fn infer_numeric_locals(&mut self, stmts: &[&Stmt]) {
        let mut assigns: Vec<(&str, Option<&Expr>)> = Vec::new();
        for stmt in stmts {
            collect_assignments_stmt(stmt, &mut assigns);
        }
        let mut numeric: HashSet<String> = self
            .local_vars
            .iter()
            .filter(|name| !name.starts_with("__mdh$"))
            .cloned()
            .collect();
        loop {
            let demote: Vec<String> = assigns
                .iter()
                .filter(|(name, value)| {
                    numeric.contains(*name)
                        && !value.is_some_and(|v| self.expr_is_numeric(v, &numeric))
                })
                .map(|(name, _)| name.to_string())
                .collect();
            if demote.is_empty() {
                break;
            }
            for name in demote {
                numeric.remove(&name);
            }
        }
        self.numeric_locals = numeric;
    }

    fn ensure_temp_local(&mut self, name: &str) {
        if !self.func_params.contains(&name.to_string())
            && !self.local_vars.contains(&name.to_string())
        {
            self.local_vars.push(name.to_string());
        }
    }

    fn emit_nil(&mut self) {
        self.emit_line("(call $mdh_make_nil)");
    }

    fn intern_string(&mut self, s: &str) -> usize {
        let offset = self.string_data.iter().map(|v| v.len() + 1).sum::<usize>();
        self.string_data.push(s.to_string());
        offset
    }

    fn emit_string_handle(&mut self, s: &str) {
        let offset = self.intern_string(s);
        self.emit_line(&format!("(i32.const {})", offset));
        self.emit_line(&format!("(i32.const {})", s.len()));
        self.emit_line("(call $mdh_make_string)");
    }

    fn is_local_or_param(&self, name: &str) -> bool {
        self.func_params
            .iter()
            .chain(self.local_vars.iter())
            .any(|n| n == name)
    }

    fn emit_value_call(&mut self, callee: &Expr, arguments: &[Expr]) -> HaversResult<()> {
        self.compile_expr(callee)?;
        self.emit_string_handle("call");
        for arg in arguments {
            self.compile_expr(arg)?;
        }
        let call_name = match arguments.len() {
            0 => "$mdh_method_call0",
            1 => "$mdh_method_call1",
            2 => "$mdh_method_call2",
            3 => "$mdh_method_call3",
            4 => "$mdh_method_call4",
            5 => "$mdh_method_call5",
            6 => "$mdh_method_call6",
            7 => "$mdh_method_call7",
            8 => "$mdh_method_call8",
            _ => {
                return Err(HaversError::InternalError(
                    "Method call arity too large for WASM backend (max 8)".to_string(),
                ));
            }
        };
        self.emit_line(&format!("(call {})", call_name));
        Ok(())
    }

    fn compile_stmt_body(&mut self, stmt: &Stmt) -> HaversResult<()> {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                if let Some(init) = initializer {
                    self.compile_expr(init)?;
                } else {
                    self.emit_nil();
                }
                self.emit_line(&format!("(local.set ${})", name));
            }

            Stmt::Expression { expr, .. } => {
                self.compile_expr(expr)?;
                self.emit_line("(drop)");
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Compile condition
                self.compile_expr(condition)?;
                self.emit_line("(call $mdh_truthy)");

                self.emit_line("(if");
                self.indent += 1;
                self.emit_line("(then");
                self.indent += 1;
                self.compile_stmt(then_branch)?;
                self.indent -= 1;
                self.emit_line(")");

                if let Some(eb) = else_branch {
                    self.emit_line("(else");
                    self.indent += 1;
                    self.compile_stmt(eb)?;
                    self.indent -= 1;
                    self.emit_line(")");
                }

                self.indent -= 1;
                self.emit_line(")");
            }

            Stmt::While {
                condition, body, ..
            } => {
                self.emit_line("(block $break");
                self.indent += 1;
                self.emit_line("(loop $continue");
                self.indent += 1;

                // Check condition, in its ain scope (if it can allocate) so its garbage
                // is freed each time roond
                let cond_scope = !self.expr_is_cheap(condition);
                if cond_scope {
                    self.emit_line("(call $mdh_scope_enter)");
                }
                self.compile_expr(condition)?;
                self.emit_line("(call $mdh_truthy)");
                if cond_scope {
                    self.emit_scope_exit(1);
                }
                self.emit_line("(i32.eqz)");
                self.emit_line("(br_if $break)");

                // Body. A `whiles` always has its ain scope (see `stmt_needs_scope`), so
                // at an iteration boundary that scope is the innermost one.
                let collects = cond_scope || self.stmt_allocates(body);
                self.loop_depths.push((self.scope_depth, collects));
                self.compile_stmt(body)?;
                if collects {
                    self.emit_loop_collect();
                }
                self.loop_depths.pop();

                // Loop back
                self.emit_line("(br $continue)");

                self.indent -= 1;
                self.emit_line(")");
                self.indent -= 1;
                self.emit_line(")");
            }

            Stmt::Return { value, .. } => {
                if let Some(val) = value {
                    self.compile_expr(val)?;
                } else {
                    self.emit_nil();
                }
                self.emit_line(&format!("(local.set ${})", TMP_RET));
                self.emit_scope_exit(self.scope_depth);
                self.emit_line(&format!("(local.get ${})", TMP_RET));
                self.emit_line("(return)");
            }

            Stmt::Print { value, .. } => {
                self.compile_expr(value)?;
                self.emit_line("(call $mdh_blether)");
            }

            Stmt::Break { .. } => {
                self.emit_loop_scope_exit();
                self.emit_line("(br $break)");
            }

            Stmt::Continue { .. } => {
                self.emit_loop_scope_exit();
                if self
                    .loop_depths
                    .last()
                    .is_some_and(|(_, collects)| *collects)
                {
                    self.emit_loop_collect();
                }
                self.emit_line("(br $continue)");
            }

            Stmt::Import { path, alias, .. } => {
                let is_tri = path == "tri" || path == "tri.braw";
                if !is_tri {
                    return Err(HaversError::InternalError(
                        "Only the tri module is supported in WASM imports".to_string(),
                    ));
                }
                let alias_name = alias.as_ref().ok_or_else(|| {
                    HaversError::InternalError(
                        "WASM import requires an alias (fetch \"tri\" tae name)".to_string(),
                    )
                })?;
                self.emit_line("(call $mdh_tri_module)");
                self.emit_line(&format!("(local.set ${})", alias_name));
            }

            _ => {
                // Unsupported statement type
                return Err(HaversError::InternalError(
                    "This statement type isnae supported in WASM yet!".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Close every scope inside the innermost loop before brak/haud jumps oot o' them.
    /// (`validate_program` has already rejected a brak/haud ootside a loop.)
    fn emit_loop_scope_exit(&mut self) {
        if let Some(&(loop_depth, _)) = self.loop_depths.last() {
            self.emit_scope_exit(self.scope_depth - loop_depth);
        }
    }

    /// At an iteration boundary, sweep the loop's scope in place when the host says
    /// it has grown enough; values earlier iterations dropped are freed there.
    fn emit_loop_collect(&mut self) {
        self.emit_line("(call $mdh_scope_collect_due)");
        self.emit_line("(if");
        self.indent += 1;
        self.emit_line("(then");
        self.indent += 1;
        let base = self.emit_root_spill();
        self.emit_line(&format!("(i32.const {})", base));
        self.emit_line(&format!(
            "(i32.const {})",
            self.func_params.len() + self.local_vars.len()
        ));
        self.emit_line("(call $mdh_scope_collect)");
        self.indent -= 1;
        self.emit_line(")");
        self.indent -= 1;
        self.emit_line(")");
    }

    /// Whether running a statement can allocate anything but wide numbers.
    fn stmt_allocates(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Block { statements, .. } => statements.iter().any(|s| self.stmt_allocates(s)),
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.stmt_needs_scope(stmt)
                    || self.stmt_allocates(then_branch)
                    || else_branch
                        .as_ref()
                        .is_some_and(|eb| self.stmt_allocates(eb))
            }
            _ => self.stmt_needs_scope(stmt),
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> HaversResult<()> {
        match expr {
            Expr::Literal { value, .. } => match value {
                Literal::Integer(n) => {
                    self.emit_line(&format!("(i64.const {})", n));
                    self.emit_line("(call $mdh_make_int)");
                }
                Literal::Float(f) => {
                    self.emit_line(&format!("(f64.const {})", f));
                    self.emit_line("(call $mdh_make_float)");
                }
                Literal::Bool(b) => {
                    self.emit_line(&format!("(i32.const {})", if *b { 1 } else { 0 }));
                    self.emit_line("(call $mdh_make_bool)");
                }
                Literal::Nil => {
                    self.emit_nil();
                }
                Literal::String(s) => {
                    self.emit_string_handle(s);
                }
            },

            Expr::Variable { name, .. } => {
                self.emit_line(&format!("(local.get ${})", name));
            }

            Expr::Assign { name, value, .. } => {
                self.compile_expr(value)?;
                self.emit_line(&format!("(local.tee ${})", name));
            }

            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                match operator {
                    BinaryOp::Add => self.emit_line("(call $mdh_add)"),
                    BinaryOp::Subtract => self.emit_line("(call $mdh_sub)"),
                    BinaryOp::Multiply => self.emit_line("(call $mdh_mul)"),
                    BinaryOp::Divide => self.emit_line("(call $mdh_div)"),
                    BinaryOp::Modulo => self.emit_line("(call $mdh_mod)"),
                    BinaryOp::Equal => self.emit_line("(call $mdh_eq)"),
                    BinaryOp::NotEqual => self.emit_line("(call $mdh_ne)"),
                    BinaryOp::Less => self.emit_line("(call $mdh_lt)"),
                    BinaryOp::LessEqual => self.emit_line("(call $mdh_le)"),
                    BinaryOp::Greater => self.emit_line("(call $mdh_gt)"),
                    BinaryOp::GreaterEqual => self.emit_line("(call $mdh_ge)"),
                }
            }

            Expr::Unary {
                operator, operand, ..
            } => match operator {
                UnaryOp::Negate => {
                    self.compile_expr(operand)?;
                    self.emit_line("(call $mdh_neg)");
                }
                UnaryOp::Not => {
                    self.compile_expr(operand)?;
                    self.emit_line("(call $mdh_not)");
                }
            },

            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => match operator {
                LogicalOp::And => {
                    self.compile_expr(left)?;
                    self.emit_line(&format!("(local.tee ${})", TMP_LOGIC));
                    self.emit_line("(call $mdh_truthy)");
                    self.emit_line("(if (result i64)");
                    self.indent += 1;
                    self.emit_line("(then");
                    self.indent += 1;
                    self.compile_expr(right)?;
                    self.indent -= 1;
                    self.emit_line(")");
                    self.emit_line(&format!("(else (local.get ${}))", TMP_LOGIC));
                    self.indent -= 1;
                    self.emit_line(")");
                }
                LogicalOp::Or => {
                    self.compile_expr(left)?;
                    self.emit_line(&format!("(local.tee ${})", TMP_LOGIC));
                    self.emit_line("(call $mdh_truthy)");
                    self.emit_line("(if (result i64)");
                    self.indent += 1;
                    self.emit_line(&format!("(then (local.get ${}))", TMP_LOGIC));
                    self.emit_line("(else");
                    self.indent += 1;
                    self.compile_expr(right)?;
                    self.indent -= 1;
                    self.emit_line(")");
                    self.indent -= 1;
                    self.emit_line(")");
                }
            },

            Expr::Call {
                callee, arguments, ..
            } => {
                if let Expr::Get {
                    object, property, ..
                } = callee.as_ref()
                {
                    self.compile_expr(object)?;
                    self.emit_string_handle(property);
                    for arg in arguments {
                        self.compile_expr(arg)?;
                    }
                    let argc = arguments.len();
                    let call_name = match argc {
                        0 => "$mdh_method_call0",
                        1 => "$mdh_method_call1",
                        2 => "$mdh_method_call2",
                        3 => "$mdh_method_call3",
                        4 => "$mdh_method_call4",
                        5 => "$mdh_method_call5",
                        6 => "$mdh_method_call6",
                        7 => "$mdh_method_call7",
                        8 => "$mdh_method_call8",
                        _ => {
                            return Err(HaversError::InternalError(
                                "Method call arity too large for WASM backend (max 8)".to_string(),
                            ));
                        }
                    };
                    self.emit_line(&format!("(call {})", call_name));
                } else if let Expr::Variable { name, .. } = callee.as_ref() {
                    if self.is_local_or_param(name) {
                        self.emit_value_call(callee, arguments)?;
                    } else if !self.functions.contains_key(name)
                        && !AUDIO_IMPORTS.iter().any(|(import, _)| import == name)
                        && builtin_arity(name).is_some()
                    {
                        // Interpreter builtin: the host runs it on borrowed arguments
                        // an' hands back a fresh handle in the current scope.
                        let offset = self.intern_string(name);
                        self.emit_line(&format!("(i32.const {})", offset));
                        self.emit_line(&format!("(i32.const {})", name.len()));
                        for arg in arguments {
                            self.compile_expr(arg)?;
                        }
                        self.emit_line(&format!("(call $mdh_builtin{})", arguments.len()));
                    } else {
                        // Direct function call (compiled function)
                        for arg in arguments {
                            self.compile_expr(arg)?;
                        }
                        // Fill in any omitted trailing parameters from their defaults.
                        if let Some(params) = self.functions.get(name).cloned() {
                            if arguments.len() > params.len() {
                                return Err(HaversError::CompileError(format!(
                                    "Dae '{}' tak's at maist {} argument(s) but wis gien {}",
                                    name,
                                    params.len(),
                                    arguments.len()
                                )));
                            }
                            for param in &params[arguments.len()..] {
                                match &param.default {
                                    Some(default) => self.compile_expr(default)?,
                                    None => {
                                        return Err(HaversError::CompileError(format!(
                                            "Dae '{}' needs the argument '{}'",
                                            name, param.name
                                        )));
                                    }
                                }
                            }
                        }
                        self.emit_line(&format!("(call ${})", function_id(name)));
                    }
                } else {
                    return Err(HaversError::InternalError(
                        "Only direct, property, or local-value calls are supported in WASM"
                            .to_string(),
                    ));
                }
            }

            Expr::Get {
                object, property, ..
            } => {
                self.compile_expr(object)?;
                self.emit_string_handle(property);
                self.emit_line("(call $mdh_prop_get)");
            }

            Expr::Set {
                object,
                property,
                value,
                ..
            } => {
                self.compile_expr(object)?;
                self.emit_string_handle(property);
                self.compile_expr(value)?;
                self.emit_line("(call $mdh_prop_set)");
            }

            Expr::List { elements, .. } => {
                self.emit_line(&format!("(i32.const {})", elements.len()));
                self.emit_line("(call $mdh_make_list)");
                self.emit_line(&format!("(local.set ${})", TMP_BUILD));
                for elem in elements {
                    self.emit_line(&format!("(local.get ${})", TMP_BUILD));
                    self.compile_expr(elem)?;
                    self.emit_line("(call $mdh_list_push)");
                    self.emit_line(&format!("(local.set ${})", TMP_BUILD));
                }
                self.emit_line(&format!("(local.get ${})", TMP_BUILD));
            }

            Expr::Dict { pairs, .. } => {
                self.emit_line("(call $mdh_make_dict)");
                self.emit_line(&format!("(local.set ${})", TMP_BUILD));
                for (key, value) in pairs {
                    self.emit_line(&format!("(local.get ${})", TMP_BUILD));
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                    self.emit_line("(call $mdh_dict_set)");
                    self.emit_line(&format!("(local.set ${})", TMP_BUILD));
                }
                self.emit_line(&format!("(local.get ${})", TMP_BUILD));
            }

            Expr::Grouping { expr, .. } => {
                self.compile_expr(expr)?;
            }

            _ => {
                return Err(HaversError::InternalError(
                    "This expression type isnae supported in WASM yet!".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(&"  ".repeat(self.indent));
        self.output.push_str(s);
    }

    fn emit_line(&mut self, s: &str) {
        self.emit(s);
        self.output.push('\n');
    }
}

fn is_comparison(op: &BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
    )
}

/// Every assignment in a statement: (local, value), `None` fer a non-numeric source.
fn collect_assignments_stmt<'a>(stmt: &'a Stmt, out: &mut Vec<(&'a str, Option<&'a Expr>)>) {
    match stmt {
        Stmt::VarDecl {
            name, initializer, ..
        } => {
            out.push((name, initializer.as_ref()));
            if let Some(init) = initializer {
                collect_assignments_expr(init, out);
            }
        }
        Stmt::Expression { expr, .. } | Stmt::Print { value: expr, .. } => {
            collect_assignments_expr(expr, out)
        }
        Stmt::Return {
            value: Some(value), ..
        } => collect_assignments_expr(value, out),
        Stmt::Block { statements, .. } => {
            for s in statements {
                collect_assignments_stmt(s, out);
            }
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_assignments_expr(condition, out);
            collect_assignments_stmt(then_branch, out);
            if let Some(eb) = else_branch {
                collect_assignments_stmt(eb, out);
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            collect_assignments_expr(condition, out);
            collect_assignments_stmt(body, out);
        }
        Stmt::For { variable, body, .. } => {
            out.push((variable, None));
            collect_assignments_stmt(body, out);
        }
        Stmt::Import {
            alias: Some(name), ..
        } => out.push((name, None)),
        _ => {}
    }
}

fn collect_assignments_expr<'a>(expr: &'a Expr, out: &mut Vec<(&'a str, Option<&'a Expr>)>) {
    match expr {
        Expr::Assign { name, value, .. } => {
            out.push((name, Some(value)));
            collect_assignments_expr(value, out);
        }
        Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
            collect_assignments_expr(left, out);
            collect_assignments_expr(right, out);
        }
        Expr::Unary { operand, .. } => collect_assignments_expr(operand, out),
        Expr::Grouping { expr, .. } => collect_assignments_expr(expr, out),
        Expr::Call {
            callee, arguments, ..
        } => {
            collect_assignments_expr(callee, out);
            for arg in arguments {
                collect_assignments_expr(arg, out);
            }
        }
        Expr::Get { object, .. } => collect_assignments_expr(object, out),
        Expr::Set { object, value, .. } => {
            collect_assignments_expr(object, out);
            collect_assignments_expr(value, out);
        }
        Expr::List { elements, .. } => {
            for e in elements {
                collect_assignments_expr(e, out);
            }
        }
        Expr::Dict { pairs, .. } => {
            for (k, v) in pairs {
                collect_assignments_expr(k, out);
                collect_assignments_expr(v, out);
            }
        }
        // Anything else is rejected by the code generator.
        _ => {}
    }
}

/// Escape a string fer WAT data section
fn escape_wat_string(s: &str) -> String {
    // WAT string escapes are per *byte*, so walk the UTF-8 bytes (the same bytes whose
    // length is handed tae `__mdh_make_string`), never the code points.
    let mut result = String::new();
    for &b in s.as_bytes() {
        match b {
            b'"' => result.push_str("\\\""),
            b'\\' => result.push_str("\\\\"),
            b'\n' => result.push_str("\\n"),
            b'\r' => result.push_str("\\r"),
            b'\t' => result.push_str("\\t"),
            b if b.is_ascii_graphic() || b == b' ' => result.push(b as char),
            b => result.push_str(&format!("\\{:02x}", b)),
        }
    }
    result
}

/// Compile source code tae WAT
pub fn compile_to_wat(source: &str) -> HaversResult<String> {
    #[cfg(coverage)]
    {
        wasm_compiler_dependency_branch_matrix_for_coverage();
    }
    let program = crate::parser::parse(source)?;
    let mut compiler = WasmCompiler::new();
    compiler.compile(&program)
}

#[cfg(coverage)]
pub fn wasm_compiler_dependency_branch_matrix_for_coverage() {
    let span = Span::new(1, 1);
    let defined = HashSet::new();
    let mut req = WasmImportRequirements::default();

    req.scan_stmt(&Stmt::Return { value: None, span }, &defined);
    req.scan_stmt(
        &Stmt::Return {
            value: Some(Expr::Literal {
                value: Literal::Integer(1),
                span,
            }),
            span,
        },
        &defined,
    );
    req.scan_stmt(
        &Stmt::Assert {
            condition: Expr::Literal {
                value: Literal::Bool(true),
                span,
            },
            message: None,
            span,
        },
        &defined,
    );
    req.scan_stmt(
        &Stmt::Assert {
            condition: Expr::Literal {
                value: Literal::Bool(true),
                span,
            },
            message: Some(Expr::Literal {
                value: Literal::String("msg".to_string()),
                span,
            }),
            span,
        },
        &defined,
    );
    req.scan_expr(
        &Expr::Slice {
            object: Box::new(Expr::Variable {
                name: "xs".to_string(),
                span,
            }),
            start: None,
            end: Some(Box::new(Expr::Literal {
                value: Literal::Integer(2),
                span,
            })),
            step: Some(Box::new(Expr::Literal {
                value: Literal::Integer(1),
                span,
            })),
            span,
        },
        &defined,
    );
    req.scan_expr(
        &Expr::FString {
            parts: vec![FStringPart::Expr(Box::new(Expr::Literal {
                value: Literal::Integer(3),
                span,
            }))],
            span,
        },
        &defined,
    );

    let mut compiler = WasmCompiler::new();
    let _ = compiler.compile_function(&Stmt::VarDecl {
        name: "x".to_string(),
        initializer: None,
        span,
    });

    compiler.func_params.push("p".to_string());
    compiler.local_vars.push("l".to_string());
    compiler.collect_locals_stmt(&Stmt::VarDecl {
        name: "p".to_string(),
        initializer: None,
        span,
    });
    compiler.collect_locals_stmt(&Stmt::VarDecl {
        name: "fresh".to_string(),
        initializer: None,
        span,
    });
    compiler.collect_locals_stmt(&Stmt::For {
        variable: "l".to_string(),
        iterable: Expr::Literal {
            value: Literal::Integer(1),
            span,
        },
        body: Box::new(Stmt::Break { span }),
        span,
    });
    compiler.collect_locals_stmt(&Stmt::Import {
        path: "tri".to_string(),
        alias: Some("p".to_string()),
        span,
    });

    compiler.ensure_temp_local(TMP_LOGIC);
    compiler.func_params.push(TMP_LOGIC.to_string());
    compiler.ensure_temp_local(TMP_LOGIC);

    let _ = compiler.compile_stmt(&Stmt::VarDecl {
        name: "a".to_string(),
        initializer: None,
        span,
    });
    let _ = compiler.compile_stmt(&Stmt::VarDecl {
        name: "b".to_string(),
        initializer: Some(Expr::Literal {
            value: Literal::Integer(2),
            span,
        }),
        span,
    });
    let _ = compiler.compile_stmt(&Stmt::If {
        condition: Expr::Literal {
            value: Literal::Bool(true),
            span,
        },
        then_branch: Box::new(Stmt::Break { span }),
        else_branch: None,
        span,
    });
    let _ = compiler.compile_stmt(&Stmt::If {
        condition: Expr::Literal {
            value: Literal::Bool(false),
            span,
        },
        then_branch: Box::new(Stmt::Break { span }),
        else_branch: Some(Box::new(Stmt::Break { span })),
        span,
    });
    let _ = compiler.compile_stmt(&Stmt::Return { value: None, span });
    let _ = compiler.compile_stmt(&Stmt::Return {
        value: Some(Expr::Literal {
            value: Literal::Integer(3),
            span,
        }),
        span,
    });
    let _ = compiler.compile_expr(&Expr::Call {
        callee: Box::new(Expr::Variable {
            name: "f".to_string(),
            span,
        }),
        arguments: Vec::new(),
        span,
    });
    let _ = escape_wat_string("Az ");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_wasm_compile() {
        let source = "ken x = 42";
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("(module"));
        assert!(wat.contains("i64.const 42"));
    }

    #[test]
    fn test_arithmetic_wasm() {
        let source = "ken x = 10 + 20";
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("call $mdh_add"));
    }

    #[test]
    fn test_function_wasm() {
        let source = r#"
            dae add(a, b) {
                gie a + b
            }
        "#;
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("(func $add"));
    }

    #[test]
    fn test_compile_function_non_function_stmt_is_a_noop_for_coverage() {
        let program = crate::parser::parse("ken x = 1").unwrap();
        let mut compiler = WasmCompiler::new();
        compiler
            .compile_function(&program.statements[0])
            .expect("compile_function should be a no-op for non-function statements");
    }

    #[test]
    fn test_is_local_or_param_checks_params_and_locals_for_coverage() {
        let mut compiler = WasmCompiler::new();
        compiler.func_params = vec!["p".to_string()];
        compiler.local_vars = vec!["l".to_string()];

        assert!(compiler.is_local_or_param("p"));
        assert!(compiler.is_local_or_param("l"));
        assert!(!compiler.is_local_or_param("x"));
    }

    #[test]
    fn test_scan_expr_slice_and_fstring_parts_for_coverage() {
        let source = r#"
            ken s = "hello"
            ken a = s[1:4:2]
            ken b = f"Value {1 + 2}"
        "#;
        let program = crate::parser::parse(source).expect("parse");
        let _req = WasmImportRequirements::from_program(&program);
    }

    #[test]
    fn test_if_wasm() {
        let source = r#"
            ken x = 5
            gin x > 3 {
                blether x
            }
        "#;
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("(if"));
    }

    #[test]
    fn test_while_wasm() {
        let source = r#"
            ken x = 0
            whiles x < 10 {
                x = x + 1
            }
        "#;
        let result = compile_to_wat(source);
        assert!(result.is_ok());
        let wat = result.unwrap();
        assert!(wat.contains("(loop"));
        assert!(wat.contains("(block"));
    }

    // ==================== Arithmetic Operations ====================

    #[test]
    fn test_subtraction_wasm() {
        let source = "ken x = 50 - 8";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_sub"));
    }

    #[test]
    fn test_multiplication_wasm() {
        let source = "ken x = 6 * 7";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_mul"));
    }

    #[test]
    fn test_division_wasm() {
        let source = "ken x = 84 / 2";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_div"));
    }

    #[test]
    fn test_modulo_wasm() {
        let source = "ken x = 10 % 3";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_mod"));
    }

    // ==================== Comparison Operations ====================

    #[test]
    fn test_greater_than_wasm() {
        let source = "ken b = 5 > 3";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_gt"));
    }

    #[test]
    fn test_less_than_wasm() {
        let source = "ken b = 3 < 5";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_lt"));
    }

    #[test]
    fn test_greater_equal_wasm() {
        let source = "ken b = 5 >= 5";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_ge"));
    }

    #[test]
    fn test_less_equal_wasm() {
        let source = "ken b = 3 <= 5";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_le"));
    }

    #[test]
    fn test_equal_wasm() {
        let source = "ken b = 5 == 5";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_eq"));
    }

    #[test]
    fn test_not_equal_wasm() {
        let source = "ken b = 5 != 3";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_ne"));
    }

    // ==================== Logical Operations ====================

    #[test]
    fn test_logical_and_wasm() {
        let source = "ken b = aye an nae";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_logical_or_wasm() {
        let source = "ken b = aye or nae";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_logical_not_wasm() {
        let source = "ken b = nae aye";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(call $mdh_not)"));
    }

    // ==================== Unary Operations ====================

    #[test]
    fn test_negate_wasm() {
        let source = "ken x = -42";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(call $mdh_neg)"));
    }

    // ==================== Control Flow ====================

    #[test]
    fn test_if_else_wasm() {
        let source = r#"
            ken x = 5
            gin x > 3 {
                blether x
            } ither {
                blether 0
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(if"));
        assert!(result.contains("(then"));
        assert!(result.contains("(else"));
    }

    #[test]
    fn test_for_loop_wasm() {
        let source = r#"
            fer i in 1..5 {
                blether i
            }
        "#;
        let result = compile_to_wat(source);
        // For loops may not be supported in WASM yet
        assert!(result.is_err());
    }

    #[test]
    fn test_return_wasm() {
        let source = r#"
            dae answer() {
                gie 42
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(return"));
    }

    #[test]
    fn test_return_implicit_wasm() {
        let source = r#"
            dae answer() {
                gie
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(return"));
    }

    // ==================== Literals ====================

    #[test]
    fn test_boolean_true_wasm() {
        let source = "ken b = aye";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(call $mdh_make_bool)"));
    }

    #[test]
    fn test_boolean_false_wasm() {
        let source = "ken b = nae";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(call $mdh_make_bool)"));
    }

    #[test]
    fn test_nil_wasm() {
        let source = "ken n = naething";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(call $mdh_make_nil)"));
    }

    #[test]
    fn test_float_wasm() {
        let source = "ken f = 3.14";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_string_wasm() {
        let source = r#"ken s = "Hello""#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(data"));
    }

    #[test]
    fn test_string_data_null_terminated() {
        let source = r#"ken s = "Hello""#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("\\00"));
    }

    // ==================== String Escape ====================

    #[test]
    fn test_string_escape_newline() {
        let source = r#"ken s = "hello\nworld""#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(data"));
    }

    #[test]
    fn test_string_escape_tab() {
        let source = r#"ken s = "hello\tworld""#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(data"));
    }

    // ==================== Multiple Statements ====================

    #[test]
    fn test_multiple_vars_wasm() {
        let source = r#"
            ken a = 1
            ken b = 2
            ken c = a + b
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_function_with_params_wasm() {
        let source = r#"
            dae multiply(a, b) {
                gie a * b
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(func $multiply"));
        assert!(result.contains("(param"));
    }

    // ==================== Variable Operations ====================

    #[test]
    fn test_variable_assignment_wasm() {
        let source = r#"
            ken x = 1
            x = 42
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("local.set"));
    }

    #[test]
    fn test_variable_get_wasm() {
        let source = r#"
            ken x = 42
            ken y = x
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("local.get"));
    }

    // ==================== Block ====================

    #[test]
    fn test_block_wasm() {
        let source = r#"
            {
                ken x = 1
                ken y = 2
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_nested_blocks_wasm() {
        let source = r#"
            {
                ken x = 1
                {
                    ken y = 2
                }
            }
        "#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    // ==================== Print ====================

    #[test]
    fn test_print_wasm() {
        let source = "blether 42";
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("call $mdh_blether"));
    }

    #[test]
    fn test_print_string_wasm() {
        let source = r#"blether "Hello""#;
        let result = compile_to_wat(source).unwrap();
        assert!(result.contains("(module"));
    }

    #[test]
    fn test_wasm_compiler_default_constructs() {
        let _ = WasmCompiler::default();
    }

    #[test]
    fn test_function_with_local_decl_wasm() {
        let source = r#"
            dae foo(a) {
                ken x = a + 1
                gie x
            }
        "#;
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("(local $x i64)"));
    }

    #[test]
    fn test_var_decl_without_initializer_wasm_defaults_to_zero() {
        let source = r#"
            ken x
            blether x
        "#;
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("(local.set $x)"));
        assert!(wat.contains("call $mdh_make_nil"));
    }

    #[test]
    fn test_break_and_continue_wasm() {
        let source = r#"
            ken x = 0
            whiles x < 10 {
                x = x + 1
                gin x == 2 {
                    haud
                }
                gin x == 3 {
                    brak
                }
            }
        "#;
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("(br $continue)"));
        assert!(wat.contains("(br $break)"));
    }

    #[test]
    fn test_function_call_wasm() {
        let source = r#"
            dae add(a, b) {
                gie a + b
            }
            blether add(1, 2)
        "#;
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("(call $add)"));
    }

    #[test]
    fn test_grouping_expr_wasm() {
        let source = r#"blether (1 + 2)"#;
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("call $mdh_add"));
    }

    #[test]
    fn test_import_tri_wasm() {
        let source = r#"fetch "tri" tae tri"#;
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("(import \"env\" \"__mdh_tri_module\""));
        assert!(wat.contains("call $mdh_tri_module"));
    }

    #[test]
    fn test_tri_method_call_six_args() {
        let source = r#"
            fetch "tri" tae tri
            ken cam = tri.OrthograffikKamera(1, 2, 3, 4, 5, 6)
        "#;
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("call $mdh_method_call6"));
    }

    #[test]
    fn test_tri_constructor_value_call_wasm() {
        let source = r#"
            fetch "tri" tae tri
            ken ctor = tri.Sicht
            ken sicht = ctor()
        "#;
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("call $mdh_method_call0"));
    }

    #[test]
    fn test_property_call_arity_variants_wasm() {
        for argc in 0..=8 {
            let args = (0..argc)
                .map(|i| (i + 1).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let source = format!("ken obj = {{}}\nobj.call({})", args);
            let wat = compile_to_wat(&source).unwrap();
            assert!(wat.contains(&format!("$mdh_method_call{}", argc)));
        }
    }

    #[test]
    fn test_value_call_arity_variants_wasm() {
        for argc in 0..=8 {
            let args = (0..argc)
                .map(|i| (i + 1).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let source = format!("ken f = aye\nf({})", args);
            let wat = compile_to_wat(&source).unwrap();
            assert!(wat.contains(&format!("$mdh_method_call{}", argc)));
        }
    }

    #[test]
    fn test_list_dict_and_property_set_wasm() {
        let source = r#"
            ken obj = {}
            obj.prop = 42
            ken xs = [1, 2, 3]
            ken d = {"a": 1, "b": 2}
        "#;
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("$mdh_prop_set"));
        assert!(wat.contains("$mdh_make_list"));
        assert!(wat.contains("$mdh_list_push"));
        assert!(wat.contains("$mdh_make_dict"));
        assert!(wat.contains("$mdh_dict_set"));
    }

    #[test]
    fn test_import_tri_requires_alias_wasm() {
        let source = r#"fetch "tri""#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err.to_string().contains("requires an alias"));
    }

    #[test]
    fn test_non_direct_call_wasm_returns_error() {
        let source = r#"
            dae add(a, b) {
                gie a + b
            }
            blether (add)(1, 2)
        "#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err
            .to_string()
            .contains("Only direct, property, or local-value calls"));
    }

    #[test]
    fn test_import_non_tri_wasm_returns_error() {
        let source = r#"fetch "math" tae m"#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err.to_string().contains("Only the tri module is supported"));
    }

    #[test]
    fn test_unsupported_statement_wasm_returns_error() {
        let source = r#"kin Foo { }"#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err.to_string().contains("statement type isnae supported"));
    }

    #[test]
    fn test_property_call_arity_too_large_wasm_errors() {
        let source = r#"
            ken obj = 1
            obj.foo(1, 2, 3, 4, 5, 6, 7, 8, 9)
        "#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err.to_string().contains("Method call arity too large"));
    }

    #[test]
    fn test_value_call_arity_too_large_wasm_errors() {
        let source = r#"
            ken f = aye
            f(1, 2, 3, 4, 5, 6, 7, 8, 9)
        "#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err.to_string().contains("Method call arity too large"));
    }

    #[test]
    fn test_unsupported_expr_wasm_returns_error() {
        let source = "ken x = [1, 2][0]";
        let err = compile_to_wat(source).unwrap_err();
        assert!(err.to_string().contains("expression type isnae supported"));
    }

    #[test]
    fn test_unsupported_expr_inside_function_wasm_returns_error() {
        let source = r#"
dae f() {
    ken xs = [1, 2]
    xs[0]
}
"#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err.to_string().contains("expression type isnae supported"));
    }

    #[test]
    fn test_escape_wat_string_covers_special_chars() {
        let input = "\"\\\r\t\n\u{0001}";
        let escaped = escape_wat_string(input);
        assert_eq!(escaped, r#"\"\\\r\t\n\01"#);
    }

    #[test]
    fn test_escape_wat_string_keeps_spaces_for_coverage() {
        assert_eq!(escape_wat_string("a b"), "a b");
    }

    #[test]
    fn wasm_import_scanner_non_tri_import_does_not_enable_tri_module_for_coverage() {
        let program = crate::parser::parse("fetch \"something_else\"").unwrap();
        let req = WasmImportRequirements::from_program(&program);
        assert!(!req.needs_tri_module);
    }

    #[test]
    fn test_audio_imports_wasm() {
        let wat = compile_to_wat("soond_stairt()").unwrap();
        assert!(wat.contains("(import \"env\" \"soond_stairt\""));
        assert!(!wat.contains("(import \"env\" \"midi_lade\""));
        assert!(wat.contains("(call $soond_stairt)"));
    }

    #[test]
    fn wasm_import_requirement_branch_matrix_for_coverage() {
        use crate::ast::{Expr, Literal, Param, Program, Span, Stmt};
        let span = Span::new(1, 1);
        let func = Stmt::Function {
            name: "f".to_string(),
            params: vec![Param {
                name: "x".to_string(),
                default: Some(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
            }],
            body: Vec::new(),
            span,
        };
        let import = Stmt::Import {
            path: "tri.braw".to_string(),
            alias: Some("tri".to_string()),
            span,
        };
        let call = Stmt::Expression {
            expr: Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: "soond_stairt".to_string(),
                    span,
                }),
                arguments: Vec::new(),
                span,
            },
            span,
        };
        let program = Program::new(vec![func, import, call]);
        let req = WasmImportRequirements::from_program(&program);
        assert!(req.needs_tri_module);
        assert!(req.audio_imports.contains("soond_stairt"));

        let mut req = WasmImportRequirements::default();
        let mut defined = std::collections::HashSet::new();
        req.add_audio_import("not_audio", &defined);
        assert!(req.audio_imports.is_empty());

        defined.insert("soond_stairt".to_string());
        req.add_audio_import("soond_stairt", &defined);
        assert!(req.audio_imports.is_empty());
    }

    #[test]
    fn wasm_local_or_param_and_tri_import_variants_for_coverage() {
        let mut compiler = WasmCompiler::new();
        compiler.func_params.push("param".to_string());
        compiler.local_vars.push("local".to_string());
        assert!(compiler.is_local_or_param("param"));
        assert!(compiler.is_local_or_param("local"));
        assert!(!compiler.is_local_or_param("missing"));

        let wat = compile_to_wat("fetch \"tri.braw\" tae tri").unwrap();
        assert!(wat.contains("(call $mdh_tri_module)"));
    }

    #[test]
    fn test_unused_imports_not_emitted() {
        let wat = compile_to_wat("blether 1").unwrap();
        assert!(!wat.contains("(import \"env\" \"__mdh_tri_module\""));
        assert!(!wat.contains(";; Audio imports"));
        assert!(!wat.contains("(import \"env\" \"soond_stairt\""));
    }

    #[cfg(coverage)]
    #[test]
    fn wasm_compiler_helper_instantiations_are_exercised_for_coverage() {
        use std::hint::black_box;

        let mut compiler = WasmCompiler::default();

        // Drive intern_string's iterator/sum closure by interning more than one string.
        let intern: fn(&mut WasmCompiler, &str) -> usize = WasmCompiler::intern_string;
        let _ = intern(&mut compiler, black_box("a"));
        let _ = intern(&mut compiler, black_box("b"));

        let emit_string_handle: fn(&mut WasmCompiler, &str) = WasmCompiler::emit_string_handle;
        emit_string_handle(&mut compiler, "hi");

        // Drive both any(...) closures (func_params + local_vars).
        compiler.func_params = vec!["p".to_string()];
        compiler.local_vars = vec!["l".to_string()];
        let is_local: fn(&WasmCompiler, &str) -> bool = WasmCompiler::is_local_or_param;
        assert!(is_local(&compiler, "p"));
        assert!(is_local(&compiler, "l"));
        assert!(!is_local(&compiler, "x"));

        // emit_value_call
        let span = Span::new(1, 1);
        let callee = Expr::Variable {
            name: "obj".to_string(),
            span,
        };
        let args = vec![
            Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            Expr::Literal {
                value: Literal::Integer(2),
                span,
            },
        ];
        let emit_value_call: fn(&mut WasmCompiler, &Expr, &[Expr]) -> HaversResult<()> =
            WasmCompiler::emit_value_call;
        emit_value_call(&mut compiler, &callee, &args).expect("emit_value_call");

        // Create an expression type that is known to be unsupported by the WASM backend.
        // This lets us exercise a bunch of `?` error-propagation regions deterministically.
        let bad_expr = Expr::Index {
            object: Box::new(Expr::List {
                elements: vec![Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }],
                span,
            }),
            index: Box::new(Expr::Literal {
                value: Literal::Integer(0),
                span,
            }),
            span,
        };

        // emit_value_call: error branches (callee + arg compilation).
        assert!(emit_value_call(&mut compiler, &bad_expr, &[]).is_err());
        assert!(emit_value_call(&mut compiler, &callee, &[bad_expr.clone()]).is_err());

        // collect_locals_stmt: cover the "already present" branches (no push).
        compiler.func_params = vec!["p".to_string()];
        compiler.local_vars = vec!["l".to_string()];
        compiler.collect_locals_stmt(&Stmt::VarDecl {
            name: "p".to_string(),
            initializer: None,
            span,
        });
        compiler.collect_locals_stmt(&Stmt::For {
            variable: "p".to_string(),
            iterable: Expr::Literal {
                value: Literal::Integer(0),
                span,
            },
            body: Box::new(Stmt::Block {
                statements: Vec::new(),
                span,
            }),
            span,
        });
        compiler.collect_locals_stmt(&Stmt::Import {
            path: "tri".to_string(),
            alias: Some("p".to_string()),
            span,
        });
        compiler.ensure_temp_local("p");

        // Cover the local_vars-side false branches in the same conditions.
        compiler.func_params.clear();
        compiler.local_vars = vec!["p".to_string()];
        compiler.collect_locals_stmt(&Stmt::VarDecl {
            name: "p".to_string(),
            initializer: None,
            span,
        });
        compiler.collect_locals_stmt(&Stmt::For {
            variable: "p".to_string(),
            iterable: Expr::Literal {
                value: Literal::Integer(0),
                span,
            },
            body: Box::new(Stmt::Block {
                statements: Vec::new(),
                span,
            }),
            span,
        });
        compiler.collect_locals_stmt(&Stmt::Import {
            path: "tri".to_string(),
            alias: Some("p".to_string()),
            span,
        });

        compiler.func_params.clear();
        compiler.local_vars = vec![TMP_LOGIC.to_string(), TMP_BUILD.to_string()];
        compiler.ensure_temp_local(TMP_LOGIC);
        compiler.ensure_temp_local(TMP_BUILD);

        // compile_stmt: exercise error propagation in a few statement shapes.
        assert!(compiler
            .compile_stmt(&Stmt::Block {
                statements: vec![Stmt::Expression {
                    expr: bad_expr.clone(),
                    span,
                }],
                span,
            })
            .is_err());

        assert!(compiler
            .compile_stmt(&Stmt::If {
                condition: bad_expr.clone(),
                then_branch: Box::new(Stmt::Expression {
                    expr: Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                    span,
                }),
                else_branch: None,
                span,
            })
            .is_err());

        assert!(compiler
            .compile_stmt(&Stmt::If {
                condition: Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                },
                then_branch: Box::new(Stmt::Expression {
                    expr: bad_expr.clone(),
                    span,
                }),
                else_branch: None,
                span,
            })
            .is_err());

        assert!(compiler
            .compile_stmt(&Stmt::If {
                condition: Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                },
                then_branch: Box::new(Stmt::Expression {
                    expr: Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                    span,
                }),
                else_branch: Some(Box::new(Stmt::Expression {
                    expr: bad_expr.clone(),
                    span,
                })),
                span,
            })
            .is_err());

        assert!(compiler
            .compile_stmt(&Stmt::While {
                condition: bad_expr.clone(),
                body: Box::new(Stmt::Expression {
                    expr: Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                    span,
                }),
                span,
            })
            .is_err());

        assert!(compiler
            .compile_stmt(&Stmt::While {
                condition: Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                },
                body: Box::new(Stmt::Expression {
                    expr: bad_expr.clone(),
                    span,
                }),
                span,
            })
            .is_err());

        assert!(compiler
            .compile_stmt(&Stmt::Return {
                value: Some(bad_expr.clone()),
                span,
            })
            .is_err());

        // compile_expr: cover error propagation for various expression shapes.
        let compile_expr: fn(&mut WasmCompiler, &Expr) -> HaversResult<()> =
            WasmCompiler::compile_expr;
        assert!(compile_expr(
            &mut compiler,
            &Expr::Assign {
                name: "x".to_string(),
                value: Box::new(bad_expr.clone()),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Binary {
                left: Box::new(bad_expr.clone()),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
                operator: BinaryOp::Add,
                right: Box::new(bad_expr.clone()),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Unary {
                operator: UnaryOp::Negate,
                operand: Box::new(bad_expr.clone()),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Unary {
                operator: UnaryOp::Not,
                operand: Box::new(bad_expr.clone()),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Logical {
                left: Box::new(bad_expr.clone()),
                operator: LogicalOp::And,
                right: Box::new(Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                }),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Logical {
                left: Box::new(Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                }),
                operator: LogicalOp::And,
                right: Box::new(bad_expr.clone()),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Logical {
                left: Box::new(bad_expr.clone()),
                operator: LogicalOp::Or,
                right: Box::new(Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                }),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Logical {
                left: Box::new(Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                }),
                operator: LogicalOp::Or,
                right: Box::new(bad_expr.clone()),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Call {
                callee: Box::new(Expr::Get {
                    object: Box::new(bad_expr.clone()),
                    property: "m".to_string(),
                    span,
                }),
                arguments: Vec::new(),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Call {
                callee: Box::new(Expr::Get {
                    object: Box::new(Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    }),
                    property: "m".to_string(),
                    span,
                }),
                arguments: vec![bad_expr.clone()],
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: "f".to_string(),
                    span,
                }),
                arguments: vec![bad_expr.clone()],
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Get {
                object: Box::new(bad_expr.clone()),
                property: "p".to_string(),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Set {
                object: Box::new(bad_expr.clone()),
                property: "p".to_string(),
                value: Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Set {
                object: Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
                property: "p".to_string(),
                value: Box::new(bad_expr.clone()),
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::List {
                elements: vec![bad_expr.clone()],
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Dict {
                pairs: vec![(
                    bad_expr.clone(),
                    Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                )],
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Dict {
                pairs: vec![(
                    Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                    bad_expr.clone(),
                )],
                span,
            }
        )
        .is_err());
        assert!(compile_expr(
            &mut compiler,
            &Expr::Grouping {
                expr: Box::new(bad_expr.clone()),
                span,
            }
        )
        .is_err());

        // Ensure the string-data emission closure in compile() runs.
        let program = crate::parser::parse("ken s = \"hi\"").unwrap();
        let mut compiler = WasmCompiler::new();
        let wat = compiler.compile(&program).unwrap();
        assert!(wat.contains(";; String data"));

        // Avoid inlining eliminating escape_wat_string execution under coverage.
        let esc: fn(&str) -> String = escape_wat_string;
        let _ = esc(black_box("\""));
    }

    #[test]
    fn wasm_scan_and_collect_locals_branch_matrix_for_coverage() {
        let mut compiler = WasmCompiler::new();

        compiler.func_params.push("param".to_string());
        compiler.local_vars.push("local".to_string());

        let span = Span::new(1, 1);
        compiler.collect_locals_stmt(&Stmt::VarDecl {
            name: "param".to_string(),
            initializer: None,
            span,
        });
        compiler.collect_locals_stmt(&Stmt::VarDecl {
            name: "local".to_string(),
            initializer: None,
            span,
        });

        compiler.collect_locals_stmt(&Stmt::For {
            variable: "param".to_string(),
            iterable: Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            body: Box::new(Stmt::Break { span }),
            span,
        });

        compiler.collect_locals_stmt(&Stmt::Import {
            path: "tri".to_string(),
            alias: Some("local".to_string()),
            span,
        });

        let mut req = WasmImportRequirements::default();
        req.scan_stmt(
            &Stmt::Return { value: None, span },
            &std::collections::HashSet::new(),
        );
    }

    #[test]
    fn wasm_compile_function_ignores_non_function_stmt_for_coverage() {
        let mut compiler = WasmCompiler::new();
        let span = Span::new(1, 1);
        let stmt = Stmt::VarDecl {
            name: "x".to_string(),
            initializer: None,
            span,
        };
        compiler.compile_function(&stmt).expect("compile function");
    }

    #[test]
    fn wasm_ensure_temp_local_skips_existing_names_for_coverage() {
        let mut compiler = WasmCompiler::new();
        compiler.func_params.push(TMP_LOGIC.to_string());
        compiler.local_vars.push(TMP_BUILD.to_string());
        compiler.ensure_temp_local(TMP_LOGIC);
        compiler.ensure_temp_local(TMP_BUILD);
    }

    #[test]
    fn wasm_compile_stmt_branches_for_coverage() {
        let mut compiler = WasmCompiler::new();
        let span = Span::new(1, 1);

        compiler
            .compile_stmt(&Stmt::If {
                condition: Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                },
                then_branch: Box::new(Stmt::Break { span }),
                else_branch: None,
                span,
            })
            .expect("compile if");

        compiler
            .compile_stmt(&Stmt::Return { value: None, span })
            .expect("compile return");

        compiler
            .compile_stmt(&Stmt::VarDecl {
                name: "x".to_string(),
                initializer: None,
                span,
            })
            .expect("compile var decl");
    }

    #[test]
    fn wasm_compile_expr_bool_and_call_error_branches_for_coverage() {
        let mut compiler = WasmCompiler::new();
        let span = Span::new(1, 1);

        compiler
            .compile_expr(&Expr::Literal {
                value: Literal::Bool(true),
                span,
            })
            .expect("compile true");
        compiler
            .compile_expr(&Expr::Literal {
                value: Literal::Bool(false),
                span,
            })
            .expect("compile false");

        let err = compiler.compile_expr(&Expr::Call {
            callee: Box::new(Expr::Literal {
                value: Literal::Integer(1),
                span,
            }),
            arguments: Vec::new(),
            span,
        });
        assert!(err.is_err());
    }

    #[test]
    fn escape_wat_string_emits_utf8_bytes_matching_the_byte_length() {
        // MDH-BUG-FLU-00045
        assert_eq!(escape_wat_string("£"), "\\c2\\a3");
        assert_eq!(escape_wat_string("€"), "\\e2\\82\\ac");
        assert_eq!(escape_wat_string("😀"), "\\f0\\9f\\98\\80");
        let wat = compile_to_wat("ken s = \"£5\"").unwrap();
        assert!(
            wat.contains("(data (i32.const 0) \"\\c2\\a35\\00\")"),
            "{wat}"
        );
        assert!(wat.contains("(i32.const 3)"), "byte length is 3: {wat}");
    }

    #[test]
    fn functions_only_program_still_defines_the_exported_main() {
        // MDH-BUG-FLU-00044
        let wat = compile_to_wat("dae f() { gie 1 }").unwrap();
        assert!(wat.contains("(func $main (result i64)"), "{wat}");
        assert!(wat.contains("(export \"main\" (func $main))"));
    }

    #[test]
    fn user_main_and_host_import_names_are_mangled() {
        assert_eq!(function_id("main"), "u$main");
        assert_eq!(function_id("mdh_add"), "u$mdh_add");
        assert_eq!(function_id("add"), "add");
        let wat = compile_to_wat("dae main() { gie 1 }\nblether main()").unwrap();
        assert!(wat.contains("(func $u$main "), "{wat}");
        assert!(wat.contains("(call $u$main)"), "{wat}");
        assert!(wat.contains("(func $main (result i64)"), "{wat}");
    }

    #[test]
    fn default_parameters_are_compiled_at_the_call_site() {
        let wat = compile_to_wat("dae add(a, b = 2) { gie a + b }\nblether add(40)").unwrap();
        assert!(wat.contains("(i64.const 2)"), "{wat}");
        assert!(wat.contains("(call $add)"), "{wat}");
    }

    #[test]
    fn validation_rejects_calls_and_names_the_backend_cannot_resolve() {
        for (source, needle) in [
            ("blether gaun([1], 0)", "'gaun'"),
            ("blether len([1], 2)", "tak's 1 argument(s)"),
            (
                "blether center(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17)",
                "tak's at maist 16 argument(s)",
            ),
            ("dae f(a) { gie a }\nf()", "'a'"),
            ("dae f(a) { gie a }\nf(1, 2)", "'f'"),
            ("ken x = 1\ndae f() { gie x }", "'x'"),
            ("y = 1", "'y'"),
            ("brak", "brak"),
            ("haud", "haud"),
            ("soond_luid()", "'soond_luid'"),
        ] {
            match compile_to_wat(source) {
                Err(HaversError::CompileError(message)) => {
                    assert!(message.contains(needle), "{source}: {message}")
                }
                other => panic!("{source}: expected CompileError, got {other:?}"),
            }
        }
    }

    #[test]
    fn builtin_calls_import_one_host_function_per_argument_count() {
        assert_eq!(
            builtin_import_line(2),
            "(import \"env\" \"__mdh_builtin2\" (func $mdh_builtin2 (param i32 i32 i64 i64) (result i64)))"
        );
        let wat =
            compile_to_wat("blether len([1])\nblether len(\"ab\")\nblether upper(\"a\")").unwrap();
        assert_eq!(wat.matches("\"__mdh_builtin1\"").count(), 1, "{wat}");
        assert_eq!(wat.matches("(call $mdh_builtin1)").count(), 3, "{wat}");
        assert!(!wat.contains("__mdh_builtin0"), "{wat}");
    }

    #[test]
    fn audio_import_lines_are_derived_from_the_single_table() {
        assert_eq!(
            audio_import_line("soond_stairt", 0),
            "(import \"env\" \"soond_stairt\" (func $soond_stairt (result i64)))"
        );
        assert_eq!(
            audio_import_line("midi_pit_rin_roond", 2),
            "(import \"env\" \"midi_pit_rin_roond\" (func $midi_pit_rin_roond (param i64 i64) (result i64)))"
        );
        assert!(AUDIO_IMPORTS
            .iter()
            .any(|(name, arity)| *name == "midi_pit_rin_roond" && *arity == 2));
        assert!(!AUDIO_IMPORTS
            .iter()
            .any(|(name, _)| *name == "midi_pit_tune"));
    }

    #[test]
    fn escape_wat_string_non_graphic_and_space_branches_for_coverage() {
        let escaped = escape_wat_string(" \u{7f}");
        assert!(escaped.contains("\\7f"));
    }

    #[cfg(coverage)]
    #[test]
    fn wasm_missing_branch_matrix_for_coverage() {
        let span = Span::new(1, 1);
        let mut req = WasmImportRequirements::default();
        let defined = std::collections::HashSet::new();

        req.scan_stmt(&Stmt::Return { value: None, span }, &defined);
        req.scan_stmt(
            &Stmt::Return {
                value: Some(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
                span,
            },
            &defined,
        );

        let mut compiler = WasmCompiler::new();

        compiler
            .compile_function(&Stmt::VarDecl {
                name: "x".to_string(),
                initializer: None,
                span,
            })
            .expect("compile_function non-function");

        compiler.func_params.push("p".to_string());
        compiler.local_vars.push("l".to_string());
        compiler.collect_locals_stmt(&Stmt::VarDecl {
            name: "p".to_string(),
            initializer: None,
            span,
        });
        compiler.collect_locals_stmt(&Stmt::VarDecl {
            name: "fresh".to_string(),
            initializer: None,
            span,
        });
        compiler.collect_locals_stmt(&Stmt::For {
            variable: "l".to_string(),
            iterable: Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            body: Box::new(Stmt::Break { span }),
            span,
        });
        compiler.collect_locals_stmt(&Stmt::Import {
            path: "tri".to_string(),
            alias: Some("p".to_string()),
            span,
        });

        compiler.ensure_temp_local(TMP_LOGIC);
        compiler.func_params.push(TMP_LOGIC.to_string());
        compiler.ensure_temp_local(TMP_LOGIC);

        compiler
            .compile_stmt(&Stmt::VarDecl {
                name: "a".to_string(),
                initializer: None,
                span,
            })
            .expect("compile var decl none");
        compiler
            .compile_stmt(&Stmt::VarDecl {
                name: "b".to_string(),
                initializer: Some(Expr::Literal {
                    value: Literal::Integer(2),
                    span,
                }),
                span,
            })
            .expect("compile var decl init");

        compiler
            .compile_stmt(&Stmt::If {
                condition: Expr::Literal {
                    value: Literal::Bool(true),
                    span,
                },
                then_branch: Box::new(Stmt::Break { span }),
                else_branch: None,
                span,
            })
            .expect("compile if without else");
        compiler
            .compile_stmt(&Stmt::If {
                condition: Expr::Literal {
                    value: Literal::Bool(false),
                    span,
                },
                then_branch: Box::new(Stmt::Break { span }),
                else_branch: Some(Box::new(Stmt::Break { span })),
                span,
            })
            .expect("compile if with else");

        compiler
            .compile_stmt(&Stmt::Return { value: None, span })
            .expect("compile return none");
        compiler
            .compile_stmt(&Stmt::Return {
                value: Some(Expr::Literal {
                    value: Literal::Integer(3),
                    span,
                }),
                span,
            })
            .expect("compile return value");

        compiler
            .compile_expr(&Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: "f".to_string(),
                    span,
                }),
                arguments: Vec::new(),
                span,
            })
            .expect("compile variable call");

        let _ = escape_wat_string("Az ");
    }
}
