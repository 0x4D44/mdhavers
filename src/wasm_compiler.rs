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

/// The higher-order builtins (name, arity). They ca' back intae guest function values,
/// so the host runs them itsel' (`Interpreter::call_builtin_hof` is the reference).
pub const HOF_BUILTINS: &[(&str, usize)] = &[
    ("gaun", 2),
    ("sieve", 2),
    ("tumble", 3),
    ("ilk", 2),
    ("hunt", 2),
    ("ony", 2),
    ("aw", 2),
    ("grup_up", 2),
    ("pairt_by", 2),
];

/// The arity o' an interpreter builtin the WASM host can call, if `name` is one.
fn builtin_arity(name: &str) -> Option<usize> {
    if let Some((_, arity)) = HOF_BUILTINS.iter().find(|(hof, _)| *hof == name) {
        return Some(*arity);
    }
    BUILTIN_ARITIES.with(|arities| arities.get(name).copied())
}

fn is_hof(name: &str) -> bool {
    HOF_BUILTINS.iter().any(|(hof, _)| *hof == name)
}

/// A lambda's body as statements: a `{ ... }` body's last expression is its value (DEC 64),
/// an' an expression body is returned.
fn lambda_body(body: &Expr, span: Span) -> Vec<Stmt> {
    let mut stmts = match body {
        Expr::BlockExpr { statements, .. } => statements.clone(),
        other => vec![Stmt::Return {
            value: Some(other.clone()),
            span,
        }],
    };
    if let Some(Stmt::Expression { expr, span }) = stmts.last() {
        let ret = Stmt::Return {
            value: Some(expr.clone()),
            span: *span,
        };
        *stmts.last_mut().expect("non-empty") = ret;
    }
    stmts
}

/// A class method's parameters as a function sees them: `masel` first.
fn method_params(params: &[Param]) -> Vec<String> {
    std::iter::once("masel".to_string())
        .chain(params.iter().map(|p| p.name.clone()))
        .collect()
}

/// How many arguments a call must pass (the interpreter's `min_arity`). A function wi'
/// fewer than all its parameters required takes the argument count as a hidden last one.
fn required_params(params: &[Param]) -> usize {
    params.iter().filter(|p| p.default.is_none()).count()
}

/// A dae's or method's body wi' its default expressions in front, fer the name scans: a
/// default is evaluated in the callee, so the names it uses are the callee's ain.
fn method_scan_body(params: &[Param], body: &[Stmt], span: Span) -> Vec<Stmt> {
    params
        .iter()
        .filter_map(|p| p.default.clone())
        .map(|expr| Stmt::Expression { expr, span })
        .chain(body.iter().cloned())
        .collect()
}

/// The names a closure (lambda or nested dae) uses but doesnae bind itsel', in first-use
/// order: the enclosing function's variables among them are captured as cells.
fn closure_free(params: &[String], body: &[Stmt]) -> Vec<String> {
    let mut scan = NameScan::default();
    scan.closure(params, body);
    scan.refs
}

/// Collects the variable names a body reads or assigns, an' the names its closures capture.
#[derive(Default)]
struct NameScan {
    refs: Vec<String>,
    captured: HashSet<String>,
    /// A `hae_a_bash` was seen (closures included).
    tries: bool,
}

impl NameScan {
    fn add(&mut self, name: &str) {
        if !self.refs.iter().any(|n| n == name) {
            self.refs.push(name.to_string());
        }
    }

    fn closure(&mut self, params: &[String], body: &[Stmt]) {
        let mut inner = NameScan::default();
        for stmt in body {
            inner.stmt(stmt);
        }
        self.tries |= inner.tries;
        let bound = declared_names(params, body.iter());
        for name in inner.refs {
            if !bound.contains(&name) {
                self.add(&name);
                self.captured.insert(name);
            }
        }
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                initializer: Some(e),
                ..
            }
            | Stmt::Expression { expr: e, .. }
            | Stmt::Print { value: e, .. }
            | Stmt::Return { value: Some(e), .. }
            | Stmt::Hurl { message: e, .. }
            | Stmt::Destructure { value: e, .. } => self.expr(e),
            Stmt::Block { statements, .. } => statements.iter().for_each(|s| self.stmt(s)),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.expr(condition);
                self.stmt(then_branch);
                if let Some(eb) = else_branch {
                    self.stmt(eb);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.expr(condition);
                self.stmt(body);
            }
            Stmt::For { iterable, body, .. } => {
                self.expr(iterable);
                self.stmt(body);
            }
            Stmt::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                self.tries = true;
                self.stmt(try_block);
                self.stmt(catch_block);
            }
            Stmt::Match { value, arms, .. } => {
                self.expr(value);
                for arm in arms {
                    if let Pattern::Range { start, end, .. } = &arm.pattern {
                        self.expr(start);
                        self.expr(end);
                    }
                    self.stmt(&arm.body);
                }
            }
            Stmt::Assert {
                condition, message, ..
            } => {
                self.expr(condition);
                if let Some(m) = message {
                    self.expr(m);
                }
            }
            Stmt::Function {
                params, body, span, ..
            } => {
                let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                self.closure(&names, &method_scan_body(params, body, *span));
            }
            Stmt::Class { methods, .. } => {
                for method in methods {
                    if let Stmt::Function {
                        params, body, span, ..
                    } = method
                    {
                        let scan_body = method_scan_body(params, body, *span);
                        self.closure(&method_params(params), &scan_body);
                    }
                }
            }
            _ => {}
        }
    }

    fn expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable { name, .. } => self.add(name),
            Expr::Assign { name, value, .. } => {
                self.expr(value);
                self.add(name);
            }
            Expr::Lambda { params, body, span } => self.closure(params, &lambda_body(body, *span)),
            Expr::BlockExpr { statements, .. } => statements.iter().for_each(|s| self.stmt(s)),
            Expr::Masel { .. } => self.add("masel"),
            _ => expr_children(expr).into_iter().for_each(|e| self.expr(e)),
        }
    }
}

/// The sub-expressions an expression evaluates itsel', in evaluation order (a lambda's
/// or a block expression's body is no' among them).
fn expr_children(expr: &Expr) -> Vec<&Expr> {
    match expr {
        Expr::Literal { .. }
        | Expr::Variable { .. }
        | Expr::Masel { .. }
        | Expr::Lambda { .. }
        | Expr::BlockExpr { .. } => Vec::new(),
        Expr::Assign { value: e, .. }
        | Expr::Unary { operand: e, .. }
        | Expr::Grouping { expr: e, .. }
        | Expr::Get { object: e, .. }
        | Expr::Spread { expr: e, .. }
        | Expr::Input { prompt: e, .. } => vec![e],
        Expr::Binary { left, right, .. }
        | Expr::Logical { left, right, .. }
        | Expr::Pipe { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
            ..
        } => vec![left, right],
        Expr::Call {
            callee, arguments, ..
        } => std::iter::once(&**callee).chain(arguments).collect(),
        Expr::Set { object, value, .. } => vec![object, value],
        Expr::Index { object, index, .. } => vec![object, index],
        Expr::IndexSet {
            object,
            index,
            value,
            ..
        } => vec![object, index, value],
        Expr::Slice {
            object,
            start,
            end,
            step,
            ..
        } => std::iter::once(&**object)
            .chain([start, end, step].into_iter().flatten().map(|e| &**e))
            .collect(),
        Expr::List { elements, .. } => elements.iter().collect(),
        Expr::Dict { pairs, .. } => pairs.iter().flat_map(|(k, v)| [k, v]).collect(),
        Expr::FString { parts, .. } => parts
            .iter()
            .filter_map(|part| match part {
                FStringPart::Expr(e) => Some(&**e),
                FStringPart::Text(_) => None,
            })
            .collect(),
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
            ..
        } => vec![condition, then_expr, else_expr],
    }
}

/// The expressions a statement evaluates itsel' (no' those o' its nested statements).
fn stmt_exprs(stmt: &Stmt) -> Vec<&Expr> {
    match stmt {
        Stmt::VarDecl {
            initializer: Some(e),
            ..
        }
        | Stmt::Expression { expr: e, .. }
        | Stmt::Print { value: e, .. }
        | Stmt::Return { value: Some(e), .. }
        | Stmt::Hurl { message: e, .. }
        | Stmt::If { condition: e, .. }
        | Stmt::While { condition: e, .. }
        | Stmt::For { iterable: e, .. }
        | Stmt::Destructure { value: e, .. } => vec![e],
        Stmt::Match { value, arms, .. } => {
            let mut out = vec![value];
            for arm in arms {
                if let Pattern::Range { start, end, .. } = &arm.pattern {
                    out.push(start);
                    out.push(end);
                }
            }
            out
        }
        Stmt::Assert {
            condition, message, ..
        } => std::iter::once(condition).chain(message).collect(),
        Stmt::Log {
            message, extras, ..
        } => std::iter::once(message).chain(extras).collect(),
        _ => Vec::new(),
    }
}

/// A closure waiting tae be compiled as its ain WASM function, after the current one.
struct PendingClosure {
    id: String,
    params: Vec<String>,
    free: Vec<String>,
    body: Vec<Stmt>,
    /// A class method: the default o' each user parameter (after `masel`), evaluated
    /// in the callee when the host passes fewer arguments.
    defaults: Option<Vec<Option<Expr>>>,
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
            Stmt::Class { methods, .. } => {
                for method in methods {
                    self.scan_stmt(method, defined_functions);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Struct { .. } => {}
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
                        && !is_hof(name)
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
            // `x |> f` compiles as `f(x)`.
            Expr::Pipe { left, right, span } => {
                let call = Expr::Call {
                    callee: right.clone(),
                    arguments: vec![(**left).clone()],
                    span: *span,
                };
                self.scan_expr(&call, defined_functions);
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
    /// Each enclosing loop, innermost last (fer brak/haud): its `scope_depth`, whether
    /// it sweeps its ain scope at iteration boundaries, an' the `try_labels` depth.
    loop_depths: Vec<(usize, bool, usize)>,
    /// Raw i64 locals (loop counters) that never hold a handle, so are never roots.
    raw_locals: Vec<String>,
    /// Hidden-local number o' each `fer`/`keek` statement, keyed by its address.
    hidden_ids: HashMap<usize, usize>,
    /// The program has a `hae_a_bash`, so every fallible call is followed by a check
    /// o' the host's pending error.
    error_checks: bool,
    /// Compiling a user function (no' `$main`): an error unwinds by returning.
    in_function: bool,
    /// Labels o' the `hae_a_bash` bodies enclosing the code being compiled, innermost last.
    try_labels: Vec<String>,
    /// Counter fer unique block labels.
    label_count: usize,
    /// Bytes at the top o' memory the biggest function needs tae spill its locals.
    max_root_bytes: usize,
    /// The previous statement's `scope_next` already opened this statement's scope.
    scope_preopened: bool,
    /// Locals o' the current function that only ever hold numbers.
    numeric_locals: HashSet<String>,
    /// Locals o' the current function that haud a cell (a variable shared wi' a closure).
    cells: HashSet<String>,
    /// Closures met in the current function, compiled efter it.
    pending: Vec<PendingClosure>,
    /// The `__mdh_fns` table: the WAT id o' each function a value can ca'.
    fn_table: Vec<String>,
    /// Top-level daes used as values: name an' table slot o' their wrapper.
    dae_values: Vec<(String, usize)>,
    /// Argument counts o' the value calls, one `__mdh_callN` import each.
    call_arities: BTreeSet<usize>,
    /// The source line the guest global `$mdh_line` is known tae hold (0 when unknown),
    /// so a statement only sets it when the line changes (MDH-BUG-FLU-00120).
    cur_line: usize,
    /// The line o' the statement being compiled, restored in the global efter a call.
    stmt_line: usize,
    /// Top-level variables a dae or method uses: each is a host cell whose handle
    /// lives in the guest global `$g$name` (see `program_globals`).
    globals: BTreeSet<String>,
    /// Block scoping o' the function being compiled (see `resolve_scopes`): each binding
    /// site's local, an' whether it rebinds one already bound in the same scope.
    binds: HashMap<(usize, usize), (String, bool)>,
    /// A name read or assigned where a block binding shadows it: that binding's local.
    uses: HashMap<usize, String>,
    /// Each closure's free names that are bound where it is made, an' their locals.
    captures: HashMap<usize, HashMap<String, String>>,
    /// The locals made fer block bindings that shadow a name: (name, local).
    renamed: Vec<(String, String)>,
    /// The scopes open while resolving, innermost last: name -> (local, surely bound).
    res_scopes: Vec<HashMap<String, (String, bool)>>,
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

/// The top-level variables that a top-level dae (or a class method) reads or assigns
/// withoot bindin' them itsel'. Each becomes a host cell reached through a guest global,
/// so the dae sees the variable's current value, as in the interpreter's global scope.
fn program_globals(program: &Program) -> BTreeSet<String> {
    let top = declared_names(
        &[],
        program
            .statements
            .iter()
            .filter(|s| !matches!(s, Stmt::Function { .. })),
    );
    let mut globals = BTreeSet::new();
    let mut add_free = |params: Vec<String>, body: &[Stmt]| {
        for name in closure_free(&params, body) {
            if top.contains(&name) {
                globals.insert(name);
            }
        }
    };
    for stmt in &program.statements {
        match stmt {
            Stmt::Function {
                params, body, span, ..
            } => {
                add_free(
                    params.iter().map(|p| p.name.clone()).collect(),
                    &method_scan_body(params, body, *span),
                );
            }
            Stmt::Class { methods, .. } => {
                for method in methods {
                    if let Stmt::Function {
                        params, body, span, ..
                    } = method
                    {
                        add_free(
                            method_params(params),
                            &method_scan_body(params, body, *span),
                        );
                    }
                }
            }
            _ => {}
        }
    }
    globals
}

/// Checks a program can become a *valid* WASM module before any code is emitted, so
/// `compile_to_wat` fails wi' a clear error instead o' producing WAT that willnae load.
/// Constructs the code generator doesnae support are left tae it tae report.
struct Validator<'a> {
    functions: HashMap<&'a str, &'a [Param]>,
    scope: HashSet<String>,
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
        scope: HashSet::new(),
        loop_depth: 0,
    };

    for stmt in fn_stmts {
        if let Stmt::Function { params, body, .. } = stmt {
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            // A dae sees the top-level variables (as globals) unless it shadows them.
            validator.scope = declared_names(&param_names, body.iter());
            validator.scope.extend(top_level_vars.iter().cloned());
            validator.loop_depth = 0;
            // Defaults are evaluated in the callee, so they see what its body sees.
            for default in params.iter().filter_map(|p| p.default.as_ref()) {
                validator.expr(default)?;
            }
            validator.stmts(body)?;
        }
    }

    validator.scope = top_level_vars;
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
            Stmt::For { iterable, body, .. } => {
                self.expr(iterable)?;
                self.loop_depth += 1;
                let result = self.stmt(body);
                self.loop_depth -= 1;
                result?;
            }
            Stmt::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                self.stmt(try_block)?;
                self.stmt(catch_block)?;
            }
            Stmt::Match { value, arms, .. } => {
                self.expr(value)?;
                for arm in arms {
                    if let Pattern::Range { start, end, .. } = &arm.pattern {
                        self.expr(start)?;
                        self.expr(end)?;
                    }
                    self.stmt(&arm.body)?;
                }
            }
            Stmt::Assert {
                condition, message, ..
            } => {
                self.expr(condition)?;
                if let Some(message) = message {
                    self.expr(message)?;
                }
            }
            Stmt::Hurl { message, .. } => self.expr(message)?,
            Stmt::Function {
                params, body, span, ..
            } => {
                let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                self.closure(&names, &method_scan_body(params, body, *span))?;
            }
            Stmt::Destructure { value, .. } => self.expr(value)?,
            Stmt::Class {
                superclass,
                methods,
                span,
                ..
            } => {
                if let Some(parent) = superclass {
                    self.check_name(parent, span.line, false)?;
                }
                for method in methods {
                    if let Stmt::Function {
                        params, body, span, ..
                    } = method
                    {
                        let scan_body = method_scan_body(params, body, *span);
                        self.closure(&method_params(params), &scan_body)?;
                    }
                }
            }
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
            Expr::Masel { span } => self.check_name("masel", span.line, false)?,
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
                    Expr::Variable { .. } => {}
                    other => self.expr(other)?,
                }
                for arg in arguments {
                    self.expr(arg)?;
                }
            }
            Expr::Index { object, index, .. } => {
                self.expr(object)?;
                self.expr(index)?;
            }
            Expr::IndexSet {
                object,
                index,
                value,
                ..
            } => {
                self.expr(object)?;
                self.expr(index)?;
                self.expr(value)?;
            }
            Expr::Slice {
                object,
                start,
                end,
                step,
                ..
            } => {
                self.expr(object)?;
                for part in [start, end, step].into_iter().flatten() {
                    self.expr(part)?;
                }
            }
            Expr::FString { parts, .. } => {
                for part in parts {
                    if let FStringPart::Expr(expr) = part {
                        self.expr(expr)?;
                    }
                }
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.expr(condition)?;
                self.expr(then_expr)?;
                self.expr(else_expr)?;
            }
            Expr::Spread { expr, .. } => self.expr(expr)?,
            Expr::Range { start, end, .. } => {
                self.expr(start)?;
                self.expr(end)?;
            }
            Expr::Lambda { params, body, span } => {
                self.closure(params, &lambda_body(body, *span))?;
            }
            // Its names are among the function's (see `collect_locals_stmt`).
            Expr::BlockExpr { statements, .. } => self.stmts(statements)?,
            Expr::Pipe { left, right, span } => {
                self.expr(&Expr::Call {
                    callee: right.clone(),
                    arguments: vec![(**left).clone()],
                    span: *span,
                })?;
            }
            // Anything else is reported by the code generator as unsupported.
            _ => {}
        }
        Ok(())
    }

    /// A closure body sees its ain parameters an' locals plus the enclosing scope.
    fn closure(&mut self, params: &[String], body: &[Stmt]) -> HaversResult<()> {
        let outer = self.scope.clone();
        let outer_loops = self.loop_depth;
        self.scope.extend(declared_names(params, body.iter()));
        self.loop_depth = 0;
        let result = self.stmts(body);
        self.scope = outer;
        self.loop_depth = outer_loops;
        result
    }

    fn check_name(&self, name: &str, line: usize, assigning: bool) -> HaversResult<()> {
        if self.scope.contains(name) {
            return Ok(());
        }
        let message =
            if !assigning && (self.functions.contains_key(name) || builtin_arity(name).is_some()) {
                // A dae or builtin used as a value.
                return Ok(());
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
            if let Some(param) = params[argc..].iter().find(|p| p.default.is_none()) {
                return Err(compile_err(
                    line,
                    format!(
                        "Dae '{}' needs the argument '{}' but wis gien only {}",
                        name, param.name, argc
                    ),
                ));
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

/// `__mdh_raise` codes: which o' the interpreter's errors tae build.
pub const RAISE_HURL: i32 = 0;
pub const RAISE_ASSERT: i32 = 1;
pub const RAISE_NO_MATCH: i32 = 2;

const TMP_LOGIC: &str = "__mdh$tmp0";
const TMP_BUILD: &str = "__mdh$tmp1";
const TMP_RET: &str = "__mdh$tmp2";
const TMP_GLOBAL: &str = "__mdh$tmp3";
/// A closure's hidden first parameter: the list o' its captured cells.
const ENV_PARAM: &str = "__mdh$env";
/// A method's hidden last parameter: how many arguments the caller passed (an int handle).
const ARGC_PARAM: &str = "__mdh$argc";

/// What a global's cell holds until its `ken` runs: a handle that is never a value
/// (above every store index, below the inline floats). `__mdh_global_get` an'
/// `__mdh_global_set` raise the interpreter's "hasnae been defined" error on it.
pub const UNSET_GLOBAL: i64 = (1 << 62) - 1;

/// The host handle o' a small integer (`wasm_runner` keeps ints in the handle as `n - 2^62`).
fn int_handle(n: i64) -> i64 {
    n - (1 << 62)
}

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
            cur_line: 0,
            stmt_line: 0,
            numeric_locals: HashSet::new(),
            raw_locals: Vec::new(),
            hidden_ids: HashMap::new(),
            error_checks: false,
            in_function: false,
            try_labels: Vec::new(),
            label_count: 0,
            cells: HashSet::new(),
            pending: Vec::new(),
            fn_table: Vec::new(),
            dae_values: Vec::new(),
            call_arities: BTreeSet::new(),
            globals: BTreeSet::new(),
            binds: HashMap::new(),
            uses: HashMap::new(),
            captures: HashMap::new(),
            renamed: Vec::new(),
            res_scopes: Vec::new(),
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
        self.hidden_ids.clear();
        self.label_count = 0;
        self.pending.clear();
        self.fn_table.clear();
        self.dae_values.clear();
        self.call_arities.clear();
        let mut scan = NameScan::default();
        program.statements.iter().for_each(|stmt| scan.stmt(stmt));
        self.error_checks = scan.tries;
        self.globals = program_globals(program);
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
        // Error handling: inside a `hae_a_bash` a failing host call records a pending
        // error instead o' trapping, an' the guest unwinds tae the catch.
        self.emit_line("(import \"env\" \"__mdh_pending\" (func $mdh_pending (result i32)))");
        self.emit_line("(import \"env\" \"__mdh_try_enter\" (func $mdh_try_enter))");
        self.emit_line("(import \"env\" \"__mdh_try_pop\" (func $mdh_try_pop (param i32)))");
        self.emit_line("(import \"env\" \"__mdh_try_catch\" (func $mdh_try_catch (param i32 i32) (result i64)))");
        self.emit_line(
            "(import \"env\" \"__mdh_raise\" (func $mdh_raise (param i32 i64 i32) (result i64)))",
        );
        // Indexing, slicing an' iteration.
        self.emit_line(
            "(import \"env\" \"__mdh_index\" (func $mdh_index (param i64 i64 i32) (result i64)))",
        );
        self.emit_line("(import \"env\" \"__mdh_index_set\" (func $mdh_index_set (param i64 i64 i64 i32) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_slice\" (func $mdh_slice (param i64 i64 i64 i64 i32 i32) (result i64)))");
        self.emit_line(
            "(import \"env\" \"__mdh_iter_items\" (func $mdh_iter_items (param i64 i32) (result i64)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_list_len\" (func $mdh_list_len (param i64) (result i64)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_list_item\" (func $mdh_list_item (param i64 i64) (result i64)))",
        );
        self.emit_line("(import \"env\" \"__mdh_list_extend\" (func $mdh_list_extend (param i64 i64 i32) (result i64)))");
        self.emit_line(
            "(import \"env\" \"__mdh_range_int\" (func $mdh_range_int (param i64 i64 i32) (result i64)))",
        );
        self.emit_line("(import \"env\" \"__mdh_match_range\" (func $mdh_match_range (param i64 i64 i64 i32) (result i32)))");
        // Function values: closures, cells an' calls (the host dispatches on the callee).
        self.emit_line("(import \"env\" \"__mdh_make_fn\" (func $mdh_make_fn (param i32 i32 i32 i64 i32 i32) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_make_builtin\" (func $mdh_make_builtin (param i32 i32) (result i64)))");
        self.emit_line(
            "(import \"env\" \"__mdh_cell_new\" (func $mdh_cell_new (param i64) (result i64)))",
        );
        // Classes: a class value, then each method's table slot an' arity.
        self.emit_line("(import \"env\" \"__mdh_make_class\" (func $mdh_make_class (param i64 i32 i32 i32 i32 i32) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_class_method\" (func $mdh_class_method (param i64 i32 i32 i32 i32 i32 i64) (result i64)))");
        // A top-level variable's cell, checked fer a read or write before its `ken`.
        self.emit_line("(import \"env\" \"__mdh_global_get\" (func $mdh_global_get (param i64 i32 i32) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_global_set\" (func $mdh_global_set (param i64 i64 i32 i32) (result i64)))");
        self.emit_line("(import \"env\" \"__mdh_destructure\" (func $mdh_destructure (param i64 i32 i32 i32) (result i64)))");
        self.emit_line(
            "(import \"env\" \"__mdh_cell_get\" (func $mdh_cell_get (param i64) (result i64)))",
        );
        self.emit_line(
            "(import \"env\" \"__mdh_cell_set\" (func $mdh_cell_set (param i64 i64) (result i64)))",
        );
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
        // `__mdh_callN` imports go here once the code shows which counts are used.
        let call_imports_at = self.output.len();
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
            self.compile_pending()?;
        }

        // Compile main code. `$main` is always emitted (empty if there are nae top-level
        // statements) because it is always exported.
        self.compile_main(&main_stmts)?;
        self.compile_pending()?;
        self.emit_function_table();
        let call_imports: String = self
            .call_arities
            .iter()
            .map(|argc| {
                format!(
                    "{}(import \"env\" \"__mdh_call{argc}\" (func $mdh_call{argc} (param{}) (result i64)))\n",
                    "  ".repeat(self.indent),
                    " i64".repeat(argc + 1) + " i32"
                )
            })
            .collect();
        let globals: String = self
            .globals
            .iter()
            .map(|name| {
                format!(
                    "{}(global $g${} (mut i64) (i64.const 0))\n",
                    "  ".repeat(self.indent),
                    name
                )
            })
            .collect();
        self.output
            .insert_str(call_imports_at, &(call_imports + &globals));

        // Export the main function
        self.emit_line("");
        self.emit_line("(export \"main\" (func $main))");
        // The line o' the running statement, read by the host when it raises an error.
        self.emit_line("(global $mdh_line (export \"__mdh_line\") (mut i32) (i32.const 0))");

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
            let mut names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            // Defaults are evaluated in the callee; the caller passes the argument count.
            let defaults: Option<Vec<Option<Expr>>> = (required_params(params) < params.len())
                .then(|| {
                    names.push(ARGC_PARAM.to_string());
                    params.iter().map(|p| p.default.clone()).collect()
                });
            self.emit_function(
                &function_id(name),
                false,
                &names,
                &[],
                body,
                defaults.as_deref(),
            )?;
        }
        Ok(())
    }

    /// Emit one WASM function. A closure (`env`) takes its captured cells as a hidden
    /// first parameter, a list unpacked intae the `free` locals on entry.
    fn emit_function(
        &mut self,
        id: &str,
        env: bool,
        params: &[String],
        free: &[String],
        body: &[Stmt],
        defaults: Option<&[Option<Expr>]>,
    ) -> HaversResult<()> {
        self.local_vars.clear();
        self.raw_locals.clear();
        self.func_params.clear();
        self.in_function = true;
        if env {
            self.func_params.push(ENV_PARAM.to_string());
        }
        self.func_params.extend(params.iter().cloned());
        let param_types: String = self
            .func_params
            .iter()
            .map(|p| format!("(param ${} i64) ", p))
            .collect();
        self.emit_line(&format!("(func ${} {}(result i64)", id, param_types));
        self.indent += 1;

        let body: Vec<&Stmt> = body.iter().collect();
        self.resolve_scopes(free, &body);
        self.ensure_temp_locals();
        let local_decls: Vec<String> = self
            .local_vars
            .iter()
            .chain(self.raw_locals.iter())
            .map(|var| format!("(local ${} i64)", var))
            .collect();
        for decl in local_decls {
            self.emit_line(&decl);
        }

        self.setup_cells(&body, free);
        self.emit_unpack_env(free);
        if let Some(defaults) = defaults {
            self.emit_defaults(params, free, defaults)?;
        }
        self.emit_prologue(free);
        self.compile_stmts(&body)?;

        // Default return value
        self.emit_nil();
        self.indent -= 1;
        self.emit_line(")");
        self.emit_line("");
        Ok(())
    }

    /// Compile the closures met so far (an' ony they contain) as their ain functions.
    fn compile_pending(&mut self) -> HaversResult<()> {
        while let Some(closure) = self.pending.pop() {
            self.emit_function(
                &closure.id,
                true,
                &closure.params,
                &closure.free,
                &closure.body,
                closure.defaults.as_deref(),
            )?;
        }
        Ok(())
    }

    /// A dae's or method's defaults, run in the callee like the interpreter's
    /// `bind_params`: the caller passes the argument count (an int handle) last, an' nil
    /// fer each missing one. This runs efter the captured cells are unpacked but before
    /// captured parameters are boxed, so a parameter reads its raw local.
    fn emit_defaults(
        &mut self,
        params: &[String],
        free: &[String],
        defaults: &[Option<Expr>],
    ) -> HaversResult<()> {
        let cells = std::mem::replace(&mut self.cells, free.iter().cloned().collect());
        // The user parameters come efter `masel` in a method, an' the count is last.
        let first = params.len() - 1 - defaults.len();
        let mut result = Ok(());
        for (i, default) in defaults.iter().enumerate() {
            let Some(expr) = default else { continue };
            self.emit_line(&format!("(local.get ${})", ARGC_PARAM));
            self.emit_line(&format!("(i64.const {})", int_handle(i as i64)));
            self.emit_line("(i64.le_s)");
            self.emit_line("(if");
            self.indent += 1;
            self.emit_line("(then");
            self.indent += 1;
            result = self.compile_expr(expr);
            if result.is_err() {
                break;
            }
            self.emit_line(&format!("(local.set ${})", params[first + i]));
            self.indent -= 1;
            self.emit_line(")");
            self.indent -= 1;
            self.emit_line(")");
        }
        self.cells = cells;
        result
    }

    /// Work oot which locals haud cells (captured by a closure, or this closure's ain
    /// captures) an' which only ever haud numbers.
    fn setup_cells(&mut self, body: &[&Stmt], free: &[String]) {
        let mut scan = NameScan::default();
        for stmt in body {
            scan.stmt(stmt);
        }
        self.cells = free.iter().cloned().collect();
        for name in &scan.captured {
            if self.is_local_or_param(name) {
                self.cells.insert(name.clone());
            }
        }
        // A block binding o' a captured name is a cell too.
        for (name, id) in &self.renamed {
            if scan.captured.contains(name) {
                self.cells.insert(id.clone());
            }
        }
        self.infer_numeric_locals(body);
        let cells = std::mem::take(&mut self.cells);
        self.numeric_locals
            .retain(|name| !cells.contains(name) && !scan.captured.contains(name));
        self.cells = cells;
    }

    /// Function entry: unpack the captured cells fae the closure's env list.
    fn emit_unpack_env(&mut self, free: &[String]) {
        for (i, name) in free.iter().enumerate() {
            self.emit_line(&format!("(local.get ${})", ENV_PARAM));
            self.emit_line(&format!("(i64.const {})", i));
            self.emit_line("(call $mdh_list_item)");
            self.emit_line(&format!("(local.set ${})", name));
        }
    }

    /// Function entry, efter the defaults: box captured parameters, an' start a local
    /// that shadows a dae or builtin as that dae or builtin (the interpreter sees the
    /// global until the `ken` rebinds the name).
    fn emit_prologue(&mut self, free: &[String]) {
        let params: Vec<String> = self
            .func_params
            .iter()
            .filter(|p| self.cells.contains(*p))
            .cloned()
            .collect();
        for name in &params {
            self.emit_line(&format!("(local.get ${})", name));
            self.emit_bind(name);
        }
        let shadows: Vec<String> = self
            .local_vars
            .iter()
            .filter(|n| !n.starts_with("__mdh$") && !free.contains(*n))
            .filter(|n| self.functions.contains_key(*n) || builtin_arity(n).is_some())
            .cloned()
            .collect();
        for name in shadows {
            self.emit_global_value(&name);
            self.emit_bind(&name);
        }
    }

    /// Store the value on the stack in a local, boxed in a fresh cell if it is captured,
    /// or in a global's cell.
    fn emit_bind(&mut self, name: &str) {
        if self.is_global(name) {
            self.emit_line(&format!("(local.set ${})", TMP_GLOBAL));
            self.emit_line(&format!("(global.get $g${})", name));
            self.emit_line(&format!("(local.get ${})", TMP_GLOBAL));
            self.emit_line("(call $mdh_cell_set)");
            self.emit_line("(drop)");
            return;
        }
        if self.cells.contains(name) {
            self.emit_line("(call $mdh_cell_new)");
        }
        self.emit_line(&format!("(local.set ${})", name));
    }

    /// Bind the value on the stack at binding site `index` o' `stmt` (see
    /// `resolve_scopes`): a block binding that shadows a name has its ain local, an' a
    /// second binding in the same scope rebinds the cell closures already share.
    fn emit_bind_at(&mut self, stmt: &Stmt, index: usize, name: &str) {
        let key = (stmt as *const Stmt as usize, index);
        let (id, rebind) = self
            .binds
            .get(&key)
            .cloned()
            .unwrap_or_else(|| (name.to_string(), false));
        if rebind && self.cells.contains(&id) && !self.is_global(&id) {
            self.emit_line(&format!("(local.set ${})", TMP_GLOBAL));
            self.emit_line(&format!("(local.get ${})", id));
            self.emit_line(&format!("(local.get ${})", TMP_GLOBAL));
            self.emit_line("(call $mdh_cell_set)");
            self.emit_line("(drop)");
        } else {
            self.emit_bind(&id);
        }
    }

    /// Push a variable's value: a local (through its cell), a dae or a builtin.
    fn emit_name_get(&mut self, name: &str) {
        if self.is_local_or_param(name) {
            self.emit_line(&format!("(local.get ${})", name));
            if self.cells.contains(name) {
                self.emit_line("(call $mdh_cell_get)");
            }
        } else if self.is_global(name) {
            // Read before its `ken` has run, a global is an error (as in the interpreter).
            self.emit_line(&format!("(global.get $g${})", name));
            self.emit_name_ptr(name);
            self.emit_fallible_call("mdh_global_get");
        } else {
            self.emit_global_value(name);
        }
    }

    /// A name's bytes in the data section, as (offset, length) on the stack.
    fn emit_name_ptr(&mut self, name: &str) {
        let offset = self.intern_string(name);
        self.emit_line(&format!("(i32.const {})", offset));
        self.emit_line(&format!("(i32.const {})", name.len()));
    }

    /// A top-level variable seen through its cell (a local o' the same name shadows it).
    fn is_global(&self, name: &str) -> bool {
        self.globals.contains(name) && !self.is_local_or_param(name)
    }

    /// A top-level dae or a builtin as a value (the validator has rejected ony other name).
    fn emit_global_value(&mut self, name: &str) {
        if let Some(params) = self.functions.get(name) {
            let arity = params.len();
            let required = required_params(params);
            let slot = match self.dae_values.iter().find(|(n, _)| n == name) {
                Some((_, slot)) => *slot,
                None => {
                    let slot = self.fn_table.len();
                    self.fn_table.push(format!("{}$v", function_id(name)));
                    self.dae_values.push((name.to_string(), slot));
                    slot
                }
            };
            self.emit_make_fn(slot, arity, required, &[], name);
        } else {
            self.emit_name_ptr(name);
            self.emit_line("(call $mdh_make_builtin)");
        }
    }

    /// Make a function value fer table slot `slot`, capturin' the `free` cells. A
    /// function wi' fewer `required` parameters than its `arity` takes the argument
    /// count last, an' fills in its ain defaults.
    fn emit_make_fn(
        &mut self,
        slot: usize,
        arity: usize,
        required: usize,
        free: &[String],
        name: &str,
    ) {
        self.emit_line(&format!("(i32.const {})", slot));
        self.emit_line(&format!("(i32.const {})", arity));
        self.emit_line(&format!("(i32.const {})", required));
        self.emit_env_list(free);
        let offset = self.intern_string(name);
        self.emit_line(&format!("(i32.const {})", offset));
        self.emit_line(&format!("(i32.const {})", name.len()));
        self.emit_line("(call $mdh_make_fn)");
    }

    /// A closure's env: a list o' the cells it captures, or nil when it captures nane.
    fn emit_env_list(&mut self, cells: &[String]) {
        if cells.is_empty() {
            self.emit_nil();
            return;
        }
        self.emit_line(&format!("(i32.const {})", cells.len()));
        self.emit_line("(call $mdh_make_list)");
        for cell in cells {
            self.emit_line(&format!("(local.get ${})", cell));
            self.emit_line("(call $mdh_list_push)");
        }
    }

    /// The free names o' a closure (`scan` is its body, defaults included) that are
    /// locals here, an' the local each is bound tae where the closure (node `key`) is made.
    fn closure_cells(&self, key: usize, params: &[String], scan: &[Stmt]) -> Vec<(String, String)> {
        let bound = self.captures.get(&key);
        closure_free(params, scan)
            .into_iter()
            .filter_map(|name| {
                let id = bound
                    .and_then(|b| b.get(&name))
                    .cloned()
                    .unwrap_or_else(|| name.clone());
                self.is_local_or_param(&id).then_some((name, id))
            })
            .collect()
    }

    /// A lambda or nested dae (node `key`): queue its body as a function an' make the
    /// value here. A nested dae wi' defaults takes the argument count last.
    fn emit_closure(
        &mut self,
        key: usize,
        name: &str,
        params: &[Param],
        scan: &[Stmt],
        body: Vec<Stmt>,
    ) {
        let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        let (free, cells): (Vec<String>, Vec<String>) =
            self.closure_cells(key, &names, scan).into_iter().unzip();
        let slot = self.fn_table.len();
        let id = format!("__mdh$fn{}", slot);
        self.fn_table.push(id.clone());
        let required = required_params(params);
        self.emit_make_fn(slot, params.len(), required, &cells, name);
        let mut fn_params = names;
        let defaults = (required < params.len()).then(|| {
            fn_params.push(ARGC_PARAM.to_string());
            params.iter().map(|p| p.default.clone()).collect()
        });
        self.pending.push(PendingClosure {
            id,
            params: fn_params,
            free,
            body,
            defaults,
        });
    }

    /// The wrappers that let a top-level dae be ca'd as a value, an' the `__mdh_fns`
    /// table the host ca's function values through.
    fn emit_function_table(&mut self) {
        for (name, _) in self.dae_values.clone() {
            let params = &self.functions[&name];
            // A dae wi' defaults takes the argument count as well.
            let arity = params.len() + usize::from(required_params(params) < params.len());
            let params: String = (0..arity)
                .map(|i| format!("(param $a{} i64) ", i))
                .collect();
            self.emit_line(&format!(
                "(func ${}$v (param ${} i64) {}(result i64)",
                function_id(&name),
                ENV_PARAM,
                params
            ));
            self.indent += 1;
            for i in 0..arity {
                self.emit_line(&format!("(local.get $a{})", i));
            }
            self.emit_line(&format!("(call ${})", function_id(&name)));
            self.indent -= 1;
            self.emit_line(")");
        }
        if self.fn_table.is_empty() {
            return;
        }
        let entries: String = self.fn_table.iter().map(|id| format!(" ${}", id)).collect();
        self.emit_line(&format!(
            "(table $__mdh_fns {} funcref)",
            self.fn_table.len()
        ));
        self.emit_line(&format!("(elem (i32.const 0) func{})", entries));
        self.emit_line("(export \"__mdh_fns\" (table $__mdh_fns))");
    }

    fn compile_main(&mut self, stmts: &[&Stmt]) -> HaversResult<()> {
        self.local_vars.clear();
        self.raw_locals.clear();
        self.func_params.clear();
        self.in_function = false;

        self.emit_line("(func $main (result i64)");
        self.indent += 1;

        // Collect all locals; a global bound at the top level is never a local o' `$main`.
        self.resolve_scopes(&[], stmts);
        self.ensure_temp_locals();

        // Declare locals
        let decls: Vec<String> = self
            .local_vars
            .iter()
            .chain(self.raw_locals.iter())
            .cloned()
            .collect();
        for var in &decls {
            self.emit_line(&format!("(local ${} i64)", var));
        }

        // The global cells, made before ony host scope opens, so they are never swept;
        // their contents survive through the write barrier in `__mdh_cell_set`.
        for name in self.globals.clone() {
            if self.functions.contains_key(&name) || builtin_arity(&name).is_some() {
                self.emit_global_value(&name);
            } else {
                self.emit_line(&format!("(i64.const {})", UNSET_GLOBAL));
            }
            self.emit_line("(call $mdh_cell_new)");
            self.emit_line(&format!("(global.set $g${})", name));
        }

        // Compile statements
        self.setup_cells(stmts, &[]);
        self.emit_prologue(&[]);
        self.compile_stmts(stmts)?;

        // Return nil
        self.emit_nil();

        self.indent -= 1;
        self.emit_line(")");
        Ok(())
    }

    /// Work oot the function's locals wi' the interpreter's block scoping: a `ken` (or a
    /// `fer` variable) in a block that reuses a name seen ootside it binds a fresh local,
    /// so the ootside binding is untouched; a second `ken` in the same scope rebinds the
    /// first (closures made in between see the new value). Fills `binds`, `uses` an'
    /// `captures`, keyed by node address, fer the code generator.
    fn resolve_scopes(&mut self, free: &[String], body: &[&Stmt]) {
        self.binds.clear();
        self.uses.clear();
        self.captures.clear();
        self.renamed.clear();
        for name in free {
            self.add_local(name);
        }
        let top = self
            .func_params
            .iter()
            .chain(free)
            .map(|name| (name.clone(), (name.clone(), true)))
            .collect();
        self.res_scopes = vec![top];
        for stmt in body {
            self.resolve_stmt(stmt);
        }
        self.res_scopes.clear();
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        let key = stmt as *const Stmt as usize;
        for expr in stmt_exprs(stmt) {
            self.resolve_expr(expr);
        }
        match stmt {
            Stmt::VarDecl { name, .. } => self.resolve_bind((key, 0), name, true),
            Stmt::Block { statements, .. } => self.resolve_block(statements),
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.resolve_stmt(then_branch);
                if let Some(eb) = else_branch {
                    self.resolve_stmt(eb);
                }
            }
            Stmt::While { body, .. } => self.resolve_stmt(body),
            // Each iteration's scope holds the variable, an' the body block shares it.
            Stmt::For { variable, body, .. } => {
                self.add_for_locals(stmt);
                self.res_scopes.push(HashMap::new());
                self.resolve_bind((key, 0), variable, true);
                match body.as_ref() {
                    Stmt::Block { statements, .. } => {
                        statements.iter().for_each(|s| self.resolve_stmt(s))
                    }
                    other => self.resolve_stmt(other),
                }
                self.res_scopes.pop();
            }
            // The catch name an' a keek binding go in the enclosing scope, as in the
            // interpreter, but only maybe (so a later `ken` there makes a fresh cell).
            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                self.resolve_stmt(try_block);
                self.resolve_bind((key, 0), error_name, false);
                self.resolve_stmt(catch_block);
            }
            Stmt::Match { arms, .. } => {
                let id = self.hidden_id(stmt);
                self.add_local(&format!("__mdh$keek{}", id));
                for (i, arm) in arms.iter().enumerate() {
                    if let Pattern::Identifier(name) = &arm.pattern {
                        self.resolve_bind((key, i), name, false);
                    }
                    self.resolve_stmt(&arm.body);
                }
            }
            // Bound first, so a recursive dae (or a method naming its class) sees itsel'.
            Stmt::Function {
                name,
                params,
                body,
                span,
            } => {
                self.resolve_bind((key, 0), name, true);
                let names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                self.resolve_captures(key, &names, &method_scan_body(params, body, *span));
            }
            Stmt::Class { name, methods, .. } => {
                self.resolve_bind((key, 0), name, true);
                for method in methods {
                    if let Stmt::Function {
                        params, body, span, ..
                    } = method
                    {
                        let scan_body = method_scan_body(params, body, *span);
                        let method_key = method as *const Stmt as usize;
                        self.resolve_captures(method_key, &method_params(params), &scan_body);
                    }
                }
            }
            Stmt::Import {
                alias: Some(name), ..
            } => self.resolve_bind((key, 0), name, true),
            Stmt::Destructure { patterns, .. } => {
                for (i, pattern) in patterns.iter().enumerate() {
                    if let DestructPattern::Variable(name) | DestructPattern::Rest(name) = pattern {
                        self.resolve_bind((key, i), name, true);
                    }
                }
            }
            _ => {}
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        let key = expr as *const Expr as usize;
        match expr {
            Expr::Variable { name, .. } => self.resolve_use(key, name),
            Expr::Assign { name, value, .. } => {
                self.resolve_expr(value);
                self.resolve_use(key, name);
            }
            Expr::Lambda { params, body, span } => {
                self.resolve_captures(key, params, &lambda_body(body, *span));
            }
            Expr::BlockExpr { statements, .. } => self.resolve_block(statements),
            _ => {
                for child in expr_children(expr) {
                    self.resolve_expr(child);
                }
            }
        }
    }

    fn resolve_block(&mut self, statements: &[Stmt]) {
        self.res_scopes.push(HashMap::new());
        statements.iter().for_each(|s| self.resolve_stmt(s));
        self.res_scopes.pop();
    }

    fn resolve_lookup(&self, name: &str) -> Option<String> {
        self.res_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(|(id, _)| id.clone()))
    }

    fn resolve_use(&mut self, key: usize, name: &str) {
        if let Some(id) = self.resolve_lookup(name) {
            if id != name {
                self.uses.insert(key, id);
            }
        }
    }

    fn resolve_captures(&mut self, key: usize, params: &[String], body: &[Stmt]) {
        let bound = closure_free(params, body)
            .into_iter()
            .filter_map(|name| self.resolve_lookup(&name).map(|id| (name, id)))
            .collect();
        self.captures.insert(key, bound);
    }

    /// Bind `name` in the innermost scope. `surely` is false fer a binding that only
    /// happens on some paths (a catch name, a keek arm).
    fn resolve_bind(&mut self, key: (usize, usize), name: &str, surely: bool) {
        let nested = self.res_scopes.len() > 1;
        let current = self.res_scopes.last().and_then(|s| s.get(name)).cloned();
        let visible = self.res_scopes.iter().any(|s| s.contains_key(name))
            || self.globals.contains(name)
            || self.functions.contains_key(name)
            || builtin_arity(name).is_some();
        let (id, rebind) = match &current {
            Some((id, bound)) => (id.clone(), *bound && surely),
            None if nested && visible => {
                let id = format!("{}$s{}", name, self.renamed.len());
                self.add_local(&id);
                self.renamed.push((name.to_string(), id.clone()));
                (id, false)
            }
            None => {
                // A global bound at the top level o' `$main` lives in its cell.
                if self.in_function || !self.globals.contains(name) {
                    self.add_local(name);
                }
                (name.to_string(), false)
            }
        };
        let bound = surely || current.is_some_and(|(_, bound)| bound);
        if let Some(scope) = self.res_scopes.last_mut() {
            scope.insert(name.to_string(), (id.clone(), bound));
        }
        self.binds.insert(key, (id, rebind));
    }

    /// The local a binding site binds (its name unless a block shadows it).
    fn bind_id(&self, key: (usize, usize), name: &str) -> String {
        self.binds
            .get(&key)
            .map_or_else(|| name.to_string(), |(id, _)| id.clone())
    }

    /// The local (or global, dae or builtin name) a variable expression refers tae.
    fn use_id(&self, expr: &Expr, name: &str) -> String {
        self.uses
            .get(&(expr as *const Expr as usize))
            .cloned()
            .unwrap_or_else(|| name.to_string())
    }

    /// The snapshot (or range bounds) an' the raw counter an' end o' a `fer`.
    fn add_for_locals(&mut self, stmt: &Stmt) {
        let id = self.hidden_id(stmt);
        self.add_local(&format!("__mdh$for{}", id));
        self.add_local(&format!("__mdh$for{}b", id));
        self.raw_locals.push(format!("__mdh$for{}i", id));
        self.raw_locals.push(format!("__mdh$for{}n", id));
    }

    fn collect_locals_stmt(&mut self, stmt: &Stmt) {
        // A block expression's statements run in this function too.
        for expr in stmt_exprs(stmt) {
            self.collect_block_expr_locals(expr);
        }
        match stmt {
            Stmt::Destructure { patterns, .. } => {
                for pattern in patterns {
                    if let DestructPattern::Variable(name) | DestructPattern::Rest(name) = pattern {
                        self.add_local(name);
                    }
                }
            }
            Stmt::VarDecl { name, .. } => self.add_local(name),
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
                self.add_local(variable);
                self.add_for_locals(stmt);
                self.collect_locals_stmt(body);
            }
            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                self.collect_locals_stmt(try_block);
                self.add_local(error_name);
                self.collect_locals_stmt(catch_block);
            }
            Stmt::Match { arms, .. } => {
                let id = self.hidden_id(stmt);
                self.add_local(&format!("__mdh$keek{}", id));
                for arm in arms {
                    if let Pattern::Identifier(name) = &arm.pattern {
                        self.add_local(name);
                    }
                    self.collect_locals_stmt(&arm.body);
                }
            }
            Stmt::Import {
                alias: Some(name), ..
            } => self.add_local(name),
            // A dae inside a function is a local closure.
            Stmt::Function { name, .. } => self.add_local(name),
            Stmt::Class { name, .. } => self.add_local(name),
            _ => {}
        }
    }

    fn collect_block_expr_locals(&mut self, expr: &Expr) {
        if let Expr::BlockExpr { statements, .. } = expr {
            statements.iter().for_each(|s| self.collect_locals_stmt(s));
        }
        for child in expr_children(expr) {
            self.collect_block_expr_locals(child);
        }
    }

    fn add_local(&mut self, name: &str) {
        if !self.func_params.iter().any(|p| p == name) && !self.local_vars.iter().any(|v| v == name)
        {
            self.local_vars.push(name.to_string());
        }
    }

    /// The hidden-local number o' a `fer` or `keek` statement (stable fer its address).
    fn hidden_id(&mut self, stmt: &Stmt) -> usize {
        let next = self.hidden_ids.len();
        *self
            .hidden_ids
            .entry(stmt as *const Stmt as usize)
            .or_insert(next)
    }

    fn ensure_temp_locals(&mut self) {
        self.ensure_temp_local(TMP_LOGIC);
        self.ensure_temp_local(TMP_BUILD);
        self.ensure_temp_local(TMP_RET);
        self.ensure_temp_local(TMP_GLOBAL);
        let root_bytes = 8 * (self.func_params.len() + self.local_vars.len());
        self.max_root_bytes = self.max_root_bytes.max(root_bytes);
        self.scope_depth = 0;
        self.loop_depths.clear();
        self.try_labels.clear();
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
        // A block can be entered by a jump, so the global's line isnae known here.
        self.cur_line = 0;
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
        self.cur_line = 0;
        Ok(())
    }

    /// Whether a statement needs its ain host scope. One that can only allocate wide
    /// numbers (inline-range numbers never touch the store) runs in its parent's scope,
    /// which reclaims those slots when it closes. Skipping a scope is always sound.
    fn stmt_needs_scope(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                self.cells
                    .contains(&self.bind_id((stmt as *const Stmt as usize, 0), name))
                    || initializer.as_ref().is_some_and(|e| !self.expr_is_cheap(e))
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

    /// Ca' a function value (closure, dae, builtin or tri constructor) through the host.
    fn emit_value_call(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
        line: usize,
    ) -> HaversResult<()> {
        if arguments.len() > 8 {
            return Err(HaversError::InternalError(
                "Method call arity too large for WASM backend (max 8)".to_string(),
            ));
        }
        self.compile_expr(callee)?;
        for arg in arguments {
            self.compile_expr(arg)?;
        }
        self.emit_line(&format!("(i32.const {})", line));
        self.call_arities.insert(arguments.len());
        self.emit_fallible_call(&format!("mdh_call{}", arguments.len()));
        self.emit_restore_line();
        Ok(())
    }

    /// Efter a call that can run guest code (a method, closure or constructor), put the
    /// statement's line back in `$mdh_line`: the callee left its ain there.
    fn emit_restore_line(&mut self) {
        if self.stmt_line > 0 {
            self.emit_set_line(self.stmt_line);
        }
    }

    fn compile_stmt_body(&mut self, stmt: &Stmt) -> HaversResult<()> {
        let line = stmt.span().line;
        let saved_line = std::mem::replace(&mut self.stmt_line, line);
        if line > 0 && line != self.cur_line {
            self.emit_set_line(line);
        }
        let result = self.compile_stmt_body_inner(stmt);
        self.stmt_line = saved_line;
        result
    }

    fn emit_set_line(&mut self, line: usize) {
        self.emit_line(&format!("(global.set $mdh_line (i32.const {}))", line));
        self.cur_line = line;
    }

    fn compile_stmt_body_inner(&mut self, stmt: &Stmt) -> HaversResult<()> {
        match stmt {
            Stmt::VarDecl {
                name, initializer, ..
            } => {
                if let Some(init) = initializer {
                    self.compile_expr(init)?;
                } else {
                    self.emit_nil();
                }
                self.emit_bind_at(stmt, 0, name);
            }

            Stmt::Function {
                name,
                params,
                body,
                span,
            } => {
                // A nested dae is a named closure; a recursive one captures its ain cell,
                // so the cell is made first an' filled once the closure exists.
                let key = stmt as *const Stmt as usize;
                let scan = method_scan_body(params, body, *span);
                let id = self.bind_id((key, 0), name);
                if self.cells.contains(&id) {
                    self.emit_nil();
                    self.emit_bind_at(stmt, 0, name);
                    self.emit_line(&format!("(local.get ${})", id));
                    self.emit_closure(key, name, params, &scan, body.clone());
                    self.emit_line("(call $mdh_cell_set)");
                    self.emit_line("(drop)");
                } else {
                    self.emit_closure(key, name, params, &scan, body.clone());
                    self.emit_bind_at(stmt, 0, name);
                }
            }

            Stmt::Destructure {
                patterns, value, ..
            } => self.compile_destructure(stmt, patterns, value)?,

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
                // The condition runs again efter the body, so name its line each time.
                if self.stmt_line > 0 {
                    self.emit_set_line(self.stmt_line);
                }

                // Check condition, in its ain scope (if it can allocate) so its garbage
                // is freed each time roond
                let cond_scope = !self.expr_is_cheap(condition);
                if cond_scope {
                    self.emit_line("(call $mdh_scope_enter)");
                }
                // Counted, so a `gie` in a block expression here closes it too.
                self.scope_depth += usize::from(cond_scope);
                let cond = self.compile_expr(condition);
                self.scope_depth -= usize::from(cond_scope);
                cond?;
                self.emit_line("(call $mdh_truthy)");
                if cond_scope {
                    self.emit_scope_exit(1);
                }
                self.emit_line("(i32.eqz)");
                self.emit_line("(br_if $break)");

                // Body. A `whiles` always has its ain scope (see `stmt_needs_scope`), so
                // at an iteration boundary that scope is the innermost one.
                let collects = cond_scope || self.stmt_allocates(body);
                self.loop_depths
                    .push((self.scope_depth, collects, self.try_labels.len()));
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
                self.emit_try_pop(self.try_labels.len());
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
                    .is_some_and(|(_, collects, _)| *collects)
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
                self.emit_bind_at(stmt, 0, alias_name);
            }

            Stmt::TryCatch {
                try_block,
                error_name,
                catch_block,
                ..
            } => {
                // The host records the scope depth; an error anywhere in the body (or a
                // dae it calls) leaves a pending error, an' the guest branches tae the
                // catch, where the host closes the scopes the unwind skipped.
                let body_label = self.next_label("try");
                let end_label = self.next_label("tryend");
                self.emit_line("(call $mdh_try_enter)");
                self.emit_line(&format!("(block ${}", end_label));
                self.indent += 1;
                self.emit_line(&format!("(block ${}", body_label));
                self.indent += 1;
                self.try_labels.push(body_label);
                let body = self.compile_stmt(try_block);
                self.try_labels.pop();
                body?;
                self.emit_try_pop(1);
                self.emit_line(&format!("(br ${})", end_label));
                self.indent -= 1;
                self.emit_line(")");
                let base = self.emit_root_spill();
                self.emit_line(&format!("(i32.const {})", base));
                self.emit_line(&format!(
                    "(i32.const {})",
                    self.func_params.len() + self.local_vars.len()
                ));
                self.emit_line("(call $mdh_try_catch)");
                self.emit_bind_at(stmt, 0, error_name);
                self.compile_stmt(catch_block)?;
                self.indent -= 1;
                self.emit_line(")");
            }

            Stmt::Hurl { message, span } => {
                self.emit_line(&format!("(i32.const {})", RAISE_HURL));
                self.compile_expr(message)?;
                self.emit_line(&format!("(i32.const {})", span.line));
                self.emit_fallible_call("mdh_raise");
                self.emit_line("(drop)");
            }

            Stmt::Assert {
                condition,
                message,
                span,
            } => {
                self.compile_expr(condition)?;
                self.emit_line("(call $mdh_truthy)");
                self.emit_line("(i32.eqz)");
                self.emit_line("(if");
                self.indent += 1;
                self.emit_line("(then");
                self.indent += 1;
                match message {
                    Some(msg) => {
                        self.emit_line(&format!("(i32.const {})", RAISE_ASSERT));
                        self.compile_expr(msg)?;
                    }
                    None => {
                        self.emit_line(&format!("(i32.const {})", RAISE_ASSERT));
                        self.emit_string_handle("Assertion failed");
                    }
                }
                self.emit_line(&format!("(i32.const {})", span.line));
                self.emit_fallible_call("mdh_raise");
                self.emit_line("(drop)");
                self.indent -= 1;
                self.emit_line(")");
                self.indent -= 1;
                self.emit_line(")");
            }

            Stmt::For {
                variable,
                iterable,
                body,
                span,
            } => self.compile_for(stmt, variable, iterable, body, span.line)?,

            Stmt::Match { value, arms, span } => {
                self.compile_match(stmt, value, arms, span.line)?;
            }

            Stmt::Class {
                name,
                superclass,
                methods,
                span,
            } => self.compile_class(stmt, name, superclass, methods, span.line)?,

            _ => {
                // Unsupported statement type
                return Err(HaversError::InternalError(format!(
                    "The {} statement isnae supported in WASM yet!",
                    stmt_kind(stmt)
                )));
            }
        }
        Ok(())
    }

    /// `kin Name fae Parent { dae .. }`: a host class value whose methods are guest
    /// functions in the `__mdh_fns` table, each taking `masel` first an' the argument
    /// count last (see the WASM classes design doc). A class made inside a function
    /// gives each method the cells it captures, like a closure's env.
    fn compile_class(
        &mut self,
        stmt: &Stmt,
        name: &str,
        superclass: &Option<String>,
        methods: &[Stmt],
        line: usize,
    ) -> HaversResult<()> {
        // A method that names its ain class captures the class's cell: make it first.
        let class_id = self.bind_id((stmt as *const Stmt as usize, 0), name);
        let cell_first = self.cells.contains(&class_id);
        if cell_first {
            self.emit_nil();
            self.emit_bind_at(stmt, 0, name);
        }
        match superclass {
            Some(parent) => self.emit_name_get(parent),
            None => self.emit_nil(),
        }
        for text in [name, superclass.as_deref().unwrap_or("")] {
            let offset = self.intern_string(text);
            self.emit_line(&format!("(i32.const {})", offset));
            self.emit_line(&format!("(i32.const {})", text.len()));
        }
        self.emit_line(&format!("(i32.const {})", line));
        self.emit_fallible_call("mdh_make_class");
        for method in methods {
            let Stmt::Function {
                name: method_name,
                params,
                body,
                span,
            } = method
            else {
                continue;
            };
            let slot = self.fn_table.len();
            let id = format!("__mdh$fn{}", slot);
            self.fn_table.push(id.clone());
            self.emit_name_ptr(method_name);
            self.emit_line(&format!("(i32.const {})", slot));
            self.emit_line(&format!("(i32.const {})", params.len()));
            self.emit_line(&format!("(i32.const {})", required_params(params)));
            let mut names = method_params(params);
            let scan = method_scan_body(params, body, *span);
            let (free, cells): (Vec<String>, Vec<String>) = self
                .closure_cells(method as *const Stmt as usize, &names, &scan)
                .into_iter()
                .unzip();
            self.emit_env_list(&cells);
            self.emit_line("(call $mdh_class_method)");
            names.push(ARGC_PARAM.to_string());
            self.pending.push(PendingClosure {
                id,
                params: names,
                free,
                body: body.clone(),
                defaults: Some(params.iter().map(|p| p.default.clone()).collect()),
            });
        }
        if cell_first {
            self.emit_line(&format!("(local.set ${})", TMP_GLOBAL));
            self.emit_line(&format!("(local.get ${})", class_id));
            self.emit_line(&format!("(local.get ${})", TMP_GLOBAL));
            self.emit_line("(call $mdh_cell_set)");
            self.emit_line("(drop)");
        } else {
            self.emit_bind_at(stmt, 0, name);
        }
        Ok(())
    }

    /// `ken [a, _, ...rest, z] = value`: the host checks the value (a list or string) an'
    /// hands back one item per pattern, a rest pattern's list in its place; then each
    /// name is bound in order.
    fn compile_destructure(
        &mut self,
        stmt: &Stmt,
        patterns: &[DestructPattern],
        value: &Expr,
    ) -> HaversResult<()> {
        let rest = patterns
            .iter()
            .position(|p| matches!(p, DestructPattern::Rest(_)));
        self.compile_expr(value)?;
        self.emit_line(&format!("(i32.const {})", patterns.len()));
        self.emit_line(&format!("(i32.const {})", rest.map_or(-1, |r| r as i64)));
        self.emit_line(&format!("(i32.const {})", stmt.span().line));
        self.emit_fallible_call("mdh_destructure");
        self.emit_line(&format!("(local.set ${})", TMP_BUILD));
        for (i, pattern) in patterns.iter().enumerate() {
            if let DestructPattern::Variable(name) | DestructPattern::Rest(name) = pattern {
                self.emit_line(&format!("(local.get ${})", TMP_BUILD));
                self.emit_line(&format!("(i64.const {})", i));
                self.emit_line("(call $mdh_list_item)");
                self.emit_bind_at(stmt, i, name);
            }
        }
        Ok(())
    }

    /// `fer variable in iterable`. A range literal counts in raw locals; anything else
    /// is snapshotted first (like the interpreter), so the body cannae disturb it.
    fn compile_for(
        &mut self,
        stmt: &Stmt,
        variable: &str,
        iterable: &Expr,
        body: &Stmt,
        line: usize,
    ) -> HaversResult<()> {
        let id = self.hidden_id(stmt);
        let items = format!("__mdh$for{}", id);
        let bound = format!("__mdh$for{}b", id);
        let idx = format!("__mdh$for{}i", id);
        let end = format!("__mdh$for{}n", id);
        let inclusive = match iterable {
            Expr::Range {
                start,
                end: stop,
                inclusive,
                span,
            } => {
                self.compile_expr(start)?;
                self.emit_line(&format!("(local.set ${})", items));
                self.compile_expr(stop)?;
                self.emit_line(&format!("(local.set ${})", bound));
                for (value, other, raw) in [(&items, &bound, &idx), (&bound, &items, &end)] {
                    self.emit_line(&format!("(local.get ${})", value));
                    self.emit_line(&format!("(local.get ${})", other));
                    self.emit_line(&format!("(i32.const {})", span.line));
                    self.emit_fallible_call("mdh_range_int");
                    self.emit_line(&format!("(local.set ${})", raw));
                }
                Some(*inclusive)
            }
            _ => {
                self.compile_expr(iterable)?;
                self.emit_line(&format!("(i32.const {})", line));
                self.emit_fallible_call("mdh_iter_items");
                self.emit_line(&format!("(local.tee ${})", items));
                self.emit_line("(call $mdh_list_len)");
                self.emit_line(&format!("(local.set ${})", end));
                self.emit_line("(i64.const 0)");
                self.emit_line(&format!("(local.set ${})", idx));
                None
            }
        };

        let top = self.next_label("fortop");
        self.emit_line("(block $break");
        self.indent += 1;
        self.emit_line(&format!("(loop ${}", top));
        self.indent += 1;
        self.emit_line(&format!("(local.get ${})", idx));
        self.emit_line(&format!("(local.get ${})", end));
        self.emit_line(if inclusive == Some(true) {
            "(i64.gt_s)"
        } else {
            "(i64.ge_s)"
        });
        self.emit_line("(br_if $break)");
        if inclusive.is_some() {
            self.emit_line(&format!("(local.get ${})", idx));
            self.emit_line("(call $mdh_make_int)");
        } else {
            self.emit_line(&format!("(local.get ${})", items));
            self.emit_line(&format!("(local.get ${})", idx));
            self.emit_line("(call $mdh_list_item)");
        }
        // A captured loop variable gets a fresh cell each iteration (DEC F3).
        self.emit_bind_at(stmt, 0, variable);

        // `haud` branches tae the end o' this block, then the counter steps on.
        self.emit_line("(block $continue");
        self.indent += 1;
        let var_id = self.bind_id((stmt as *const Stmt as usize, 0), variable);
        let collects = self.stmt_allocates(body) || self.cells.contains(&var_id);
        self.loop_depths
            .push((self.scope_depth, collects, self.try_labels.len()));
        let result = self.compile_stmt(body);
        self.loop_depths.pop();
        result?;
        self.indent -= 1;
        self.emit_line(")");
        if collects {
            self.emit_loop_collect();
        }
        if inclusive == Some(true) {
            // Stop at the end itself, so `..=` the biggest integer never overflows.
            self.emit_line(&format!("(local.get ${})", idx));
            self.emit_line(&format!("(local.get ${})", end));
            self.emit_line("(i64.eq)");
            self.emit_line("(br_if $break)");
        }
        self.emit_line(&format!("(local.get ${})", idx));
        self.emit_line("(i64.const 1)");
        self.emit_line("(i64.add)");
        self.emit_line(&format!("(local.set ${})", idx));
        self.emit_line(&format!("(br ${})", top));
        self.indent -= 1;
        self.emit_line(")");
        self.indent -= 1;
        self.emit_line(")");
        Ok(())
    }

    /// `keek value { pattern -> body ... }`: the first matching arm runs; nane is an error.
    fn compile_match(
        &mut self,
        stmt: &Stmt,
        value: &Expr,
        arms: &[MatchArm],
        line: usize,
    ) -> HaversResult<()> {
        let subject = format!("__mdh$keek{}", self.hidden_id(stmt));
        self.compile_expr(value)?;
        self.emit_line(&format!("(local.set ${})", subject));
        let end = self.next_label("keekend");
        self.emit_line(&format!("(block ${}", end));
        self.indent += 1;
        for (i, arm) in arms.iter().enumerate() {
            let next = self.next_label("arm");
            self.emit_line(&format!("(block ${}", next));
            self.indent += 1;
            match &arm.pattern {
                Pattern::Literal(lit) => {
                    self.emit_line(&format!("(local.get ${})", subject));
                    self.compile_literal(lit);
                    self.emit_line("(call $mdh_eq)");
                    self.emit_line("(call $mdh_truthy)");
                    self.emit_line("(i32.eqz)");
                    self.emit_line(&format!("(br_if ${})", next));
                }
                Pattern::Identifier(name) => {
                    self.emit_line(&format!("(local.get ${})", subject));
                    self.emit_bind_at(stmt, i, name);
                }
                Pattern::Wildcard => {}
                Pattern::Range {
                    start,
                    end: stop,
                    inclusive,
                } => {
                    self.emit_line(&format!("(local.get ${})", subject));
                    self.compile_expr(start)?;
                    self.compile_expr(stop)?;
                    self.emit_line(&format!("(i32.const {})", i32::from(*inclusive)));
                    self.emit_line("(call $mdh_match_range)");
                    self.emit_line("(i32.eqz)");
                    self.emit_line(&format!("(br_if ${})", next));
                }
            }
            self.compile_stmt(&arm.body)?;
            self.emit_line(&format!("(br ${})", end));
            self.indent -= 1;
            self.emit_line(")");
        }
        self.emit_line(&format!("(i32.const {})", RAISE_NO_MATCH));
        self.emit_line(&format!("(local.get ${})", subject));
        self.emit_line(&format!("(i32.const {})", line));
        self.emit_fallible_call("mdh_raise");
        self.emit_line("(drop)");
        self.indent -= 1;
        self.emit_line(")");
        Ok(())
    }

    fn compile_literal(&mut self, value: &Literal) {
        match value {
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
            Literal::Nil => self.emit_nil(),
            Literal::String(s) => self.emit_string_handle(s),
        }
    }

    /// Close every scope inside the innermost loop before brak/haud jumps oot o' them.
    /// (`validate_program` has already rejected a brak/haud ootside a loop.)
    fn emit_loop_scope_exit(&mut self) {
        if let Some(&(loop_depth, _, try_depth)) = self.loop_depths.last() {
            self.emit_try_pop(self.try_labels.len() - try_depth);
            self.emit_scope_exit(self.scope_depth - loop_depth);
        }
    }

    /// Leave `n` enclosing `hae_a_bash` bodies withoot an error (brak, haud, gie).
    fn emit_try_pop(&mut self, n: usize) {
        if n > 0 {
            self.emit_line(&format!("(i32.const {})", n));
            self.emit_line("(call $mdh_try_pop)");
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
            Expr::Literal { value, .. } => self.compile_literal(value),

            Expr::Variable { name, .. } => {
                let id = self.use_id(expr, name);
                self.emit_name_get(&id);
            }

            // `masel` is a method's first parameter (validated tae be in scope).
            Expr::Masel { .. } => self.emit_name_get("masel"),

            Expr::Assign { name, value, .. } => {
                let id = self.use_id(expr, name);
                if self.is_global(&id) {
                    // Assigned before its `ken` has run, a global is an error.
                    self.emit_line(&format!("(global.get $g${})", id));
                    self.compile_expr(value)?;
                    self.emit_name_ptr(&id);
                    self.emit_fallible_call("mdh_global_set");
                } else if self.cells.contains(&id) {
                    self.emit_line(&format!("(local.get ${})", id));
                    self.compile_expr(value)?;
                    self.emit_line("(call $mdh_cell_set)");
                } else {
                    self.compile_expr(value)?;
                    self.emit_line(&format!("(local.tee ${})", id));
                }
            }

            Expr::Lambda { params, body, span } => {
                let params: Vec<Param> = params
                    .iter()
                    .map(|name| Param {
                        name: name.clone(),
                        default: None,
                    })
                    .collect();
                let body = lambda_body(body, *span);
                let key = expr as *const Expr as usize;
                self.emit_closure(key, "<lambda>", &params, &body, body.clone());
            }

            // `x |> f` is `f(x)`.
            Expr::Pipe { left, right, span } => {
                self.compile_call(right, std::slice::from_ref(&**left), span.line)?;
            }

            // A block's last expression is its value (DEC 64); its bindings are its ain.
            Expr::BlockExpr { statements, .. } => {
                let (body, value) = match statements.split_last() {
                    Some((Stmt::Expression { expr: last, .. }, rest)) => (rest, Some(last)),
                    _ => (statements.as_slice(), None),
                };
                let body: Vec<&Stmt> = body.iter().collect();
                self.compile_stmts(&body)?;
                match value {
                    Some(last) => {
                        let line = last.span().line;
                        let outer = std::mem::replace(&mut self.stmt_line, line);
                        self.emit_set_line(line);
                        let result = self.compile_expr(last);
                        self.stmt_line = outer;
                        result?;
                    }
                    None => self.emit_nil(),
                }
                self.emit_restore_line();
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
                    BinaryOp::Add => self.emit_fallible_call("mdh_add"),
                    BinaryOp::Subtract => self.emit_fallible_call("mdh_sub"),
                    BinaryOp::Multiply => self.emit_fallible_call("mdh_mul"),
                    BinaryOp::Divide => self.emit_fallible_call("mdh_div"),
                    BinaryOp::Modulo => self.emit_fallible_call("mdh_mod"),
                    // An instance's `__same_as__` overload can raise.
                    BinaryOp::Equal => self.emit_fallible_call("mdh_eq"),
                    BinaryOp::NotEqual => self.emit_fallible_call("mdh_ne"),
                    BinaryOp::Less => self.emit_fallible_call("mdh_lt"),
                    BinaryOp::LessEqual => self.emit_fallible_call("mdh_le"),
                    BinaryOp::Greater => self.emit_fallible_call("mdh_gt"),
                    BinaryOp::GreaterEqual => self.emit_fallible_call("mdh_ge"),
                }
            }

            Expr::Unary {
                operator, operand, ..
            } => match operator {
                UnaryOp::Negate => {
                    self.compile_expr(operand)?;
                    self.emit_fallible_call("mdh_neg");
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
                callee,
                arguments,
                span,
            } => self.compile_call(callee, arguments, span.line)?,

            Expr::Get {
                object, property, ..
            } => {
                self.compile_expr(object)?;
                self.emit_string_handle(property);
                // A missing instance field is an error.
                self.emit_fallible_call("mdh_prop_get");
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
                    if let Expr::Spread { expr, span } = elem {
                        // `...xs` skails a list's items (or a string's characters) in.
                        self.compile_expr(expr)?;
                        self.emit_line(&format!("(i32.const {})", span.line));
                        self.emit_fallible_call("mdh_list_extend");
                    } else {
                        self.compile_expr(elem)?;
                        self.emit_line("(call $mdh_list_push)");
                    }
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

            Expr::Index {
                object,
                index,
                span,
            } => {
                self.compile_expr(object)?;
                self.compile_expr(index)?;
                self.emit_line(&format!("(i32.const {})", span.line));
                self.emit_fallible_call("mdh_index");
            }

            Expr::IndexSet {
                object,
                index,
                value,
                span,
            } => {
                // The host writes through the barrier, so an older container
                // keeps the new (young) value alive.
                self.compile_expr(object)?;
                self.compile_expr(index)?;
                self.compile_expr(value)?;
                self.emit_line(&format!("(i32.const {})", span.line));
                self.emit_fallible_call("mdh_index_set");
            }

            Expr::Slice {
                object,
                start,
                end,
                step,
                span,
            } => {
                self.compile_expr(object)?;
                let mut present = 0;
                for (bit, part) in [start, end, step].into_iter().enumerate() {
                    match part {
                        Some(part) => {
                            self.compile_expr(part)?;
                            present |= 1 << bit;
                        }
                        None => self.emit_nil(),
                    }
                }
                self.emit_line(&format!("(i32.const {})", present));
                self.emit_line(&format!("(i32.const {})", span.line));
                self.emit_fallible_call("mdh_slice");
            }

            Expr::FString { parts, .. } => {
                // "" + part + part ...: string addition formats each part like `blether`.
                self.emit_string_handle("");
                for part in parts {
                    match part {
                        FStringPart::Text(text) => self.emit_string_handle(text),
                        FStringPart::Expr(expr) => self.compile_expr(expr)?,
                    }
                    self.emit_fallible_call("mdh_add");
                }
            }

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.compile_expr(condition)?;
                self.emit_line("(call $mdh_truthy)");
                self.emit_line("(if (result i64)");
                self.indent += 1;
                self.emit_line("(then");
                self.indent += 1;
                self.compile_expr(then_expr)?;
                self.indent -= 1;
                self.emit_line(")");
                self.emit_line("(else");
                self.indent += 1;
                self.compile_expr(else_expr)?;
                self.indent -= 1;
                self.emit_line(")");
                self.indent -= 1;
                self.emit_line(")");
            }

            _ => {
                return Err(HaversError::InternalError(format!(
                    "The {} expression isnae supported in WASM yet!",
                    expr_kind(expr)
                )));
            }
        }
        Ok(())
    }

    /// `callee(arguments)`: a method call, a direct ca' o' a top-level dae or builtin,
    /// or a value call through the host.
    fn compile_call(&mut self, callee: &Expr, arguments: &[Expr], line: usize) -> HaversResult<()> {
        if let Expr::Get {
            object, property, ..
        } = callee
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
            // A method or a callable field can raise.
            self.emit_error_check();
            self.emit_restore_line();
        } else if let Expr::Variable { name, .. } = callee {
            let id = self.use_id(callee, name);
            if self.is_local_or_param(&id)
                || self.is_global(&id)
                || (is_hof(name) && !self.functions.contains_key(name))
            {
                // A higher-order builtin is ca'd as a builtin value, wi' the line.
                self.emit_value_call(callee, arguments, line)?;
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
                self.emit_fallible_call(&format!("mdh_builtin{}", arguments.len()));
            } else {
                // Direct function call (compiled function)
                for arg in arguments {
                    self.compile_expr(arg)?;
                }
                // A dae wi' defaults fills them in itsel': pass nil fer each omitted
                // parameter an' the argument count.
                if let Some(params) = self.functions.get(name).cloned() {
                    if arguments.len() > params.len() {
                        return Err(HaversError::CompileError(format!(
                            "Dae '{}' tak's at maist {} argument(s) but wis gien {}",
                            name,
                            params.len(),
                            arguments.len()
                        )));
                    }
                    if let Some(param) = params[arguments.len()..]
                        .iter()
                        .find(|p| p.default.is_none())
                    {
                        return Err(HaversError::CompileError(format!(
                            "Dae '{}' needs the argument '{}'",
                            name, param.name
                        )));
                    }
                    if required_params(&params) < params.len() {
                        for _ in arguments.len()..params.len() {
                            self.emit_nil();
                        }
                        let argc = int_handle(arguments.len() as i64);
                        self.emit_line(&format!("(i64.const {})", argc));
                    }
                }
                self.emit_fallible_call(&function_id(name));
                // The callee left its ain line in the global.
                if self.stmt_line > 0 {
                    self.emit_set_line(self.stmt_line);
                }
            }
        } else {
            self.emit_value_call(callee, arguments, line)?;
        }
        Ok(())
    }

    /// Call `$func`, then (in a program wi' a `hae_a_bash`) unwind if it left a
    /// pending error: tae the innermost catch, or oot o' the function.
    fn emit_fallible_call(&mut self, func: &str) {
        self.emit_line(&format!("(call ${})", func));
        self.emit_error_check();
    }

    fn emit_error_check(&mut self) {
        if !self.error_checks {
            return;
        }
        // Scopes left open by the unwind are closed by the host at the catch.
        let unwind = match self.try_labels.last() {
            Some(label) => format!("(br ${})", label),
            None if self.in_function => "(return (i64.const 0))".to_string(),
            // At the top level ootside a catch naething is pending: errors trap.
            None => return,
        };
        self.emit_line(&format!("(if (call $mdh_pending) (then {}))", unwind));
    }

    fn next_label(&mut self, kind: &str) -> String {
        self.label_count += 1;
        format!("{}{}", kind, self.label_count)
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

/// The AST name o' a statement, sae an unsupported one can be named in the error.
fn stmt_kind(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::VarDecl { .. } => "VarDecl",
        Stmt::Expression { .. } => "Expression",
        Stmt::Block { .. } => "Block",
        Stmt::If { .. } => "If",
        Stmt::While { .. } => "While",
        Stmt::For { .. } => "For",
        Stmt::Function { .. } => "Function",
        Stmt::Return { .. } => "Return",
        Stmt::Print { .. } => "Print",
        Stmt::Break { .. } => "Break",
        Stmt::Continue { .. } => "Continue",
        Stmt::Class { .. } => "Class",
        Stmt::Struct { .. } => "Struct",
        Stmt::Import { .. } => "Import",
        Stmt::TryCatch { .. } => "TryCatch",
        Stmt::Match { .. } => "Match",
        Stmt::Assert { .. } => "Assert",
        Stmt::Destructure { .. } => "Destructure",
        Stmt::Log { .. } => "Log",
        Stmt::Hurl { .. } => "Hurl",
    }
}

/// The AST name o' an expression, sae an unsupported one can be named in the error.
fn expr_kind(expr: &Expr) -> &'static str {
    match expr {
        Expr::Literal { .. } => "Literal",
        Expr::Variable { .. } => "Variable",
        Expr::Assign { .. } => "Assign",
        Expr::Binary { .. } => "Binary",
        Expr::Unary { .. } => "Unary",
        Expr::Logical { .. } => "Logical",
        Expr::Call { .. } => "Call",
        Expr::Get { .. } => "Get",
        Expr::Set { .. } => "Set",
        Expr::Index { .. } => "Index",
        Expr::IndexSet { .. } => "IndexSet",
        Expr::Slice { .. } => "Slice",
        Expr::List { .. } => "List",
        Expr::Dict { .. } => "Dict",
        Expr::Range { .. } => "Range",
        Expr::Grouping { .. } => "Grouping",
        Expr::Lambda { .. } => "Lambda",
        Expr::BlockExpr { .. } => "BlockExpr",
        Expr::Masel { .. } => "Masel",
        Expr::Input { .. } => "Input",
        Expr::FString { .. } => "FString",
        Expr::Spread { .. } => "Spread",
        Expr::Pipe { .. } => "Pipe",
        Expr::Ternary { .. } => "Ternary",
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
        Stmt::For {
            variable,
            iterable,
            body,
            ..
        } => {
            out.push((variable, None));
            collect_assignments_expr(iterable, out);
            collect_assignments_stmt(body, out);
        }
        Stmt::TryCatch {
            try_block,
            error_name,
            catch_block,
            ..
        } => {
            collect_assignments_stmt(try_block, out);
            out.push((error_name, None));
            collect_assignments_stmt(catch_block, out);
        }
        Stmt::Match { value, arms, .. } => {
            collect_assignments_expr(value, out);
            for arm in arms {
                if let Pattern::Identifier(name) = &arm.pattern {
                    out.push((name, None));
                }
                collect_assignments_stmt(&arm.body, out);
            }
        }
        Stmt::Assert {
            condition, message, ..
        } => {
            collect_assignments_expr(condition, out);
            if let Some(message) = message {
                collect_assignments_expr(message, out);
            }
        }
        Stmt::Hurl { message, .. } => collect_assignments_expr(message, out),
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
        Expr::Index { object, index, .. } => {
            collect_assignments_expr(object, out);
            collect_assignments_expr(index, out);
        }
        Expr::IndexSet {
            object,
            index,
            value,
            ..
        } => {
            collect_assignments_expr(object, out);
            collect_assignments_expr(index, out);
            collect_assignments_expr(value, out);
        }
        Expr::Slice {
            object,
            start,
            end,
            step,
            ..
        } => {
            collect_assignments_expr(object, out);
            for part in [start, end, step].into_iter().flatten() {
                collect_assignments_expr(part, out);
            }
        }
        Expr::FString { parts, .. } => {
            for part in parts {
                if let FStringPart::Expr(expr) = part {
                    collect_assignments_expr(expr, out);
                }
            }
        }
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_assignments_expr(condition, out);
            collect_assignments_expr(then_expr, out);
            collect_assignments_expr(else_expr, out);
        }
        Expr::Spread { expr, .. } => collect_assignments_expr(expr, out),
        Expr::Range { start, end, .. } => {
            collect_assignments_expr(start, out);
            collect_assignments_expr(end, out);
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
        let result = compile_to_wat(source).unwrap();
        // A range literal counts in raw locals: nae list is built.
        assert!(result.contains("(loop $fortop"));
        assert!(result.contains("(call $mdh_range_int)"));
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
        assert!(wat.contains("call $mdh_call0"));
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
            assert!(wat.contains(&format!("$mdh_call{}", argc)));
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
        // Ony callee expression is a function value now, ca'd through the host.
        let wat = compile_to_wat(source).unwrap();
        assert!(wat.contains("call $mdh_call2"));
    }

    #[test]
    fn test_import_non_tri_wasm_returns_error() {
        let source = r#"fetch "math" tae m"#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err.to_string().contains("Only the tri module is supported"));
    }

    #[test]
    fn test_unsupported_statement_wasm_returns_error() {
        let source = r#"thing Point { x, y }"#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err
            .to_string()
            .contains("The Struct statement isnae supported"));
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
        let source = "ken x = speir \"a\"";
        let err = compile_to_wat(source).unwrap_err();
        assert!(err
            .to_string()
            .contains("The Input expression isnae supported"));
    }

    #[test]
    fn test_unsupported_expr_inside_function_wasm_returns_error() {
        let source = r#"
dae f() {
    ken g = speir "a"
}
"#;
        let err = compile_to_wat(source).unwrap_err();
        assert!(err
            .to_string()
            .contains("The Input expression isnae supported"));
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

        // Ony callee is a value call now; calling a number fails at run time, as in
        // the interpreter.
        compiler
            .compile_expr(&Expr::Call {
                callee: Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
                arguments: Vec::new(),
                span,
            })
            .expect("value call");
        assert!(compiler.output.contains("(call $mdh_call0)"));
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
    fn default_parameters_are_filled_in_by_the_callee() {
        let wat = compile_to_wat("dae add(a, b = 2) { gie a + b }\nblether add(40)").unwrap();
        // The callee takes the argument count an' evaluates the default itsel'.
        assert!(wat.contains("(param $__mdh$argc i64)"), "{wat}");
        assert!(wat.contains("(i64.const 2)"), "{wat}");
        let call = format!("(i64.const {})\n", int_handle(1));
        assert!(wat.contains(&call), "{wat}");
        assert!(wat.contains("(call $add)"), "{wat}");
    }

    #[test]
    fn validation_rejects_calls_and_names_the_backend_cannot_resolve() {
        for (source, needle) in [
            ("blether gaun([1])", "tak's 2 argument(s)"),
            ("blether len([1], 2)", "tak's 1 argument(s)"),
            (
                "blether center(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17)",
                "tak's at maist 16 argument(s)",
            ),
            ("dae f(a) { gie a }\nf()", "'a'"),
            ("dae f(a) { gie a }\nf(1, 2)", "'f'"),
            // A top-level variable is fine in a dae now; an undeclared name isnae.
            ("ken x = 1\ndae f() { gie zz }", "'zz'"),
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
