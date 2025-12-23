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
use std::collections::{BTreeSet, HashSet};

const AUDIO_IMPORTS: &[(&str, &str)] = &[
    (
        "soond_stairt",
        "(import \"env\" \"soond_stairt\" (func $soond_stairt (result i64)))",
    ),
    (
        "soond_steek",
        "(import \"env\" \"soond_steek\" (func $soond_steek (result i64)))",
    ),
    (
        "soond_wheesht",
        "(import \"env\" \"soond_wheesht\" (func $soond_wheesht (param i64) (result i64)))",
    ),
    (
        "soond_luid",
        "(import \"env\" \"soond_luid\" (func $soond_luid (param i64) (result i64)))",
    ),
    (
        "soond_hou_luid",
        "(import \"env\" \"soond_hou_luid\" (func $soond_hou_luid (result i64)))",
    ),
    (
        "soond_haud_gang",
        "(import \"env\" \"soond_haud_gang\" (func $soond_haud_gang (result i64)))",
    ),
    (
        "soond_lade",
        "(import \"env\" \"soond_lade\" (func $soond_lade (param i64) (result i64)))",
    ),
    (
        "soond_spiel",
        "(import \"env\" \"soond_spiel\" (func $soond_spiel (param i64) (result i64)))",
    ),
    (
        "soond_haud",
        "(import \"env\" \"soond_haud\" (func $soond_haud (param i64) (result i64)))",
    ),
    (
        "soond_gae_on",
        "(import \"env\" \"soond_gae_on\" (func $soond_gae_on (param i64) (result i64)))",
    ),
    (
        "soond_stap",
        "(import \"env\" \"soond_stap\" (func $soond_stap (param i64) (result i64)))",
    ),
    (
        "soond_unlade",
        "(import \"env\" \"soond_unlade\" (func $soond_unlade (param i64) (result i64)))",
    ),
    (
        "soond_is_spielin",
        "(import \"env\" \"soond_is_spielin\" (func $soond_is_spielin (param i64) (result i64)))",
    ),
    (
        "soond_pit_luid",
        "(import \"env\" \"soond_pit_luid\" (func $soond_pit_luid (param i64 i64) (result i64)))",
    ),
    (
        "soond_pit_pan",
        "(import \"env\" \"soond_pit_pan\" (func $soond_pit_pan (param i64 i64) (result i64)))",
    ),
    (
        "soond_pit_tune",
        "(import \"env\" \"soond_pit_tune\" (func $soond_pit_tune (param i64 i64) (result i64)))",
    ),
    (
        "soond_pit_rin_roond",
        "(import \"env\" \"soond_pit_rin_roond\" (func $soond_pit_rin_roond (param i64 i64) (result i64)))",
    ),
    (
        "soond_ready",
        "(import \"env\" \"soond_ready\" (func $soond_ready (param i64) (result i64)))",
    ),
    (
        "muisic_lade",
        "(import \"env\" \"muisic_lade\" (func $muisic_lade (param i64) (result i64)))",
    ),
    (
        "muisic_spiel",
        "(import \"env\" \"muisic_spiel\" (func $muisic_spiel (param i64) (result i64)))",
    ),
    (
        "muisic_haud",
        "(import \"env\" \"muisic_haud\" (func $muisic_haud (param i64) (result i64)))",
    ),
    (
        "muisic_gae_on",
        "(import \"env\" \"muisic_gae_on\" (func $muisic_gae_on (param i64) (result i64)))",
    ),
    (
        "muisic_stap",
        "(import \"env\" \"muisic_stap\" (func $muisic_stap (param i64) (result i64)))",
    ),
    (
        "muisic_unlade",
        "(import \"env\" \"muisic_unlade\" (func $muisic_unlade (param i64) (result i64)))",
    ),
    (
        "muisic_is_spielin",
        "(import \"env\" \"muisic_is_spielin\" (func $muisic_is_spielin (param i64) (result i64)))",
    ),
    (
        "muisic_loup",
        "(import \"env\" \"muisic_loup\" (func $muisic_loup (param i64 i64) (result i64)))",
    ),
    (
        "muisic_hou_lang",
        "(import \"env\" \"muisic_hou_lang\" (func $muisic_hou_lang (param i64) (result i64)))",
    ),
    (
        "muisic_whaur",
        "(import \"env\" \"muisic_whaur\" (func $muisic_whaur (param i64) (result i64)))",
    ),
    (
        "muisic_pit_luid",
        "(import \"env\" \"muisic_pit_luid\" (func $muisic_pit_luid (param i64 i64) (result i64)))",
    ),
    (
        "muisic_pit_pan",
        "(import \"env\" \"muisic_pit_pan\" (func $muisic_pit_pan (param i64 i64) (result i64)))",
    ),
    (
        "muisic_pit_tune",
        "(import \"env\" \"muisic_pit_tune\" (func $muisic_pit_tune (param i64 i64) (result i64)))",
    ),
    (
        "muisic_pit_rin_roond",
        "(import \"env\" \"muisic_pit_rin_roond\" (func $muisic_pit_rin_roond (param i64 i64) (result i64)))",
    ),
    (
        "midi_lade",
        "(import \"env\" \"midi_lade\" (func $midi_lade (param i64 i64) (result i64)))",
    ),
    (
        "midi_spiel",
        "(import \"env\" \"midi_spiel\" (func $midi_spiel (param i64) (result i64)))",
    ),
    (
        "midi_haud",
        "(import \"env\" \"midi_haud\" (func $midi_haud (param i64) (result i64)))",
    ),
    (
        "midi_gae_on",
        "(import \"env\" \"midi_gae_on\" (func $midi_gae_on (param i64) (result i64)))",
    ),
    (
        "midi_stap",
        "(import \"env\" \"midi_stap\" (func $midi_stap (param i64) (result i64)))",
    ),
    (
        "midi_unlade",
        "(import \"env\" \"midi_unlade\" (func $midi_unlade (param i64) (result i64)))",
    ),
    (
        "midi_is_spielin",
        "(import \"env\" \"midi_is_spielin\" (func $midi_is_spielin (param i64) (result i64)))",
    ),
    (
        "midi_loup",
        "(import \"env\" \"midi_loup\" (func $midi_loup (param i64 i64) (result i64)))",
    ),
    (
        "midi_hou_lang",
        "(import \"env\" \"midi_hou_lang\" (func $midi_hou_lang (param i64) (result i64)))",
    ),
    (
        "midi_whaur",
        "(import \"env\" \"midi_whaur\" (func $midi_whaur (param i64) (result i64)))",
    ),
    (
        "midi_pit_luid",
        "(import \"env\" \"midi_pit_luid\" (func $midi_pit_luid (param i64 i64) (result i64)))",
    ),
    (
        "midi_pit_pan",
        "(import \"env\" \"midi_pit_pan\" (func $midi_pit_pan (param i64 i64) (result i64)))",
    ),
    (
        "midi_pit_rin_roond",
        "(import \"env\" \"midi_pit_rin_roond\" (func $midi_pit_rin_roond (param i64 i64) (result i64)))",
    ),
];

#[derive(Debug, Default)]
struct WasmImportRequirements {
    needs_tri_module: bool,
    audio_imports: BTreeSet<String>,
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
}

const TMP_LOGIC: &str = "__mdh$tmp0";
const TMP_BUILD: &str = "__mdh$tmp1";

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
        }
    }

    /// Compile a program tae WAT (WebAssembly Text Format)
    pub fn compile(&mut self, program: &Program) -> HaversResult<String> {
        self.output.clear();
        self.string_data.clear();
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

        if !import_requirements.audio_imports.is_empty() {
            self.emit_line("");
            self.emit_line(";; Audio imports (i64 value ABI)");
            for (name, import_line) in AUDIO_IMPORTS {
                if import_requirements.audio_imports.contains(*name) {
                    self.emit_line(import_line);
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

        // Compile main code as start function
        if !main_stmts.is_empty() {
            self.compile_main(&main_stmts)?;
        }

        // Export the main function
        self.emit_line("");
        self.emit_line("(export \"main\" (func $main))");

        // Add string data section if we have strings
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
            self.emit_line(&format!("(func ${} {}(result i64)", name, param_types));
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
            for s in body {
                self.compile_stmt(s)?;
            }

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
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }

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
        self.func_params.iter().any(|n| n == name) || self.local_vars.iter().any(|n| n == name)
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

    fn compile_stmt(&mut self, stmt: &Stmt) -> HaversResult<()> {
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

            Stmt::Block { statements, .. } => {
                for s in statements {
                    self.compile_stmt(s)?;
                }
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

                // Check condition
                self.compile_expr(condition)?;
                self.emit_line("(call $mdh_truthy)");
                self.emit_line("(i32.eqz)");
                self.emit_line("(br_if $break)");

                // Body
                self.compile_stmt(body)?;

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
                self.emit_line("(return)");
            }

            Stmt::Print { value, .. } => {
                self.compile_expr(value)?;
                self.emit_line("(call $mdh_blether)");
            }

            Stmt::Break { .. } => {
                self.emit_line("(br $break)");
            }

            Stmt::Continue { .. } => {
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
                    } else {
                        // Direct function call (compiled function)
                        for arg in arguments {
                            self.compile_expr(arg)?;
                        }
                        self.emit_line(&format!("(call ${})", name));
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
                self.emit_line(&format!("(local.tee ${})", TMP_BUILD));
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
                self.emit_line(&format!("(local.tee ${})", TMP_BUILD));
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

/// Escape a string fer WAT data section
fn escape_wat_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_ascii_graphic() || c == ' ' => result.push(c),
            c => result.push_str(&format!("\\{:02x}", c as u32)),
        }
    }
    result
}

/// Compile source code tae WAT
pub fn compile_to_wat(source: &str) -> HaversResult<String> {
    let program = crate::parser::parse(source)?;
    let mut compiler = WasmCompiler::new();
    compiler.compile(&program)
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
            dae add(a, b) {
                gie a + b
            }
            ken f = add
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
    fn test_escape_wat_string_covers_special_chars() {
        let input = "\"\\\r\t\n\u{0001}";
        let escaped = escape_wat_string(input);
        assert_eq!(escaped, r#"\"\\\r\t\n\01"#);
    }

    #[test]
    fn test_audio_imports_wasm() {
        let wat = compile_to_wat("soond_stairt()").unwrap();
        assert!(wat.contains("(import \"env\" \"soond_stairt\""));
        assert!(!wat.contains("(import \"env\" \"midi_lade\""));
        assert!(wat.contains("(call $soond_stairt)"));
    }

    #[test]
    fn test_unused_imports_not_emitted() {
        let wat = compile_to_wat("blether 1").unwrap();
        assert!(!wat.contains("(import \"env\" \"__mdh_tri_module\""));
        assert!(!wat.contains(";; Audio imports"));
        assert!(!wat.contains("(import \"env\" \"soond_stairt\""));
    }
}
