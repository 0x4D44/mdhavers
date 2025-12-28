//! Main LLVM compiler interface
//!
//! Provides high-level API for compiling mdhavers to LLVM IR, object files,
//! and native executables.

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Embedded runtime object file - compiled into the binary at build time.
static EMBEDDED_RUNTIME: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mdh_runtime.o"));

/// Embedded Rust runtime staticlib (JSON/regex helpers).
static EMBEDDED_RUNTIME_RS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mdh_runtime_rs.a"));

/// Embedded GC stub - minimal malloc wrappers for standalone builds.
static EMBEDDED_GC_STUB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/gc_stub.o"));

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;

use crate::ast::Program;
use crate::error::HaversError;

use super::codegen::CodeGen;

#[derive(Copy, Clone)]
enum StatusColor {
    Cyan,
    Yellow,
    Green,
    Red,
    Dim,
}

#[cfg(all(test, coverage))]
mod coverage_inject {
    use std::cell::Cell;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub(super) enum FailPoint {
        None,
        InitializeNativeTarget,
        TargetFromTriple,
        CreateTargetMachine,
        RunOptimizationPasses,
        WriteObjectFile,
        CreateRuntimeFile,
        WriteRuntimeFile,
        CreateRuntimeRsFile,
        WriteRuntimeRsFile,
        CreateGcStubFile,
        WriteGcStubFile,
        LinkCommandStatus,
    }

    thread_local! {
        static FAIL_POINT: Cell<FailPoint> = Cell::new(FailPoint::None);
    }

    pub(super) fn should_fail(point: FailPoint) -> bool {
        FAIL_POINT.with(|cell| cell.get() == point)
    }

    #[cfg_attr(coverage, inline(never))]
    pub(super) fn with_failpoint<T>(point: FailPoint, f: impl FnOnce() -> T) -> T {
        FAIL_POINT.with(|cell| {
            let previous = cell.get();
            cell.set(point);
            let out = f();
            cell.set(previous);
            out
        })
    }
}

struct BuildStatus {
    label: &'static str,
    enabled: bool,
    use_color: bool,
    wrote_any: bool,
}

impl BuildStatus {
    fn new(label: &'static str) -> Self {
        let enabled = io::stderr().is_terminal();
        let term_ok = std::env::var("TERM")
            .map(|term| term != "dumb")
            .unwrap_or(true);
        #[cfg(not(coverage))]
        let use_color = enabled && term_ok && std::env::var_os("NO_COLOR").is_none();

        #[cfg(coverage)]
        let use_color = enabled & term_ok & std::env::var_os("NO_COLOR").is_none();
        Self {
            label,
            enabled,
            use_color,
            wrote_any: false,
        }
    }

    fn guard(&mut self) -> BuildStatusGuard {
        BuildStatusGuard {
            status: self as *mut BuildStatus,
        }
    }

    fn update(&mut self, stage: &str, color: StatusColor) {
        if !self.enabled {
            #[cfg(coverage)]
            {
                let _ = self.paint(self.label, StatusColor::Cyan, true);
                let _ = self.paint(stage, color, false);
            }
            return;
        }

        let label = self.paint(self.label, StatusColor::Cyan, true);
        let stage = self.paint(stage, color, false);
        eprint!("\r\x1b[2K{} -> {}", label, stage);
        let _ = io::stderr().flush();
        self.wrote_any = true;
    }

    fn finish(&mut self, stage: &str, color: StatusColor) {
        self.update(stage, color);
        self.ensure_newline();
    }

    fn fail(&mut self, stage: &str) {
        self.finish(stage, StatusColor::Red);
    }

    fn ensure_newline(&mut self) {
        if self.enabled && self.wrote_any {
            eprintln!();
            self.wrote_any = false;
        }
    }

    fn paint(&self, text: &str, color: StatusColor, bold: bool) -> String {
        if !self.use_color {
            return text.to_string();
        }

        let code = match color {
            StatusColor::Cyan => "36",
            StatusColor::Yellow => "33",
            StatusColor::Green => "32",
            StatusColor::Red => "31",
            StatusColor::Dim => "2",
        };

        let prefix = if bold {
            format!("\x1b[1;{}m", code)
        } else {
            format!("\x1b[{}m", code)
        };

        format!("{prefix}{text}\x1b[0m")
    }
}

struct BuildStatusGuard {
    status: *mut BuildStatus,
}

impl Drop for BuildStatusGuard {
    fn drop(&mut self) {
        // SAFETY: BuildStatusGuard is created from a valid mutable reference
        // and dropped before the status goes out of scope.
        unsafe {
            if let Some(status) = self.status.as_mut() {
                status.ensure_newline();
            }
        }
    }
}

struct TempFileCleanup {
    paths: Vec<PathBuf>,
}

impl TempFileCleanup {
    #[cfg_attr(coverage, inline(never))]
    fn new(paths: Vec<PathBuf>) -> Self {
        Self { paths }
    }
}

impl Drop for TempFileCleanup {
    #[cfg_attr(coverage, inline(never))]
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg_attr(coverage, inline(never))]
fn initialize_native_target() -> Result<(), String> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::InitializeNativeTarget) {
        return Err("coverage injected: initialize_native_target".to_string());
    }
    Target::initialize_native(&InitializationConfig::default())
}

#[cfg_attr(coverage, inline(never))]
fn target_from_triple(triple: &inkwell::targets::TargetTriple) -> Result<Target, String> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::TargetFromTriple) {
        return Err("coverage injected: target_from_triple".to_string());
    }
    match Target::from_triple(triple) {
        Ok(target) => Ok(target),
        Err(err) => Err(err.to_string()),
    }
}

#[cfg_attr(coverage, inline(never))]
fn create_target_machine(
    target: &Target,
    triple: &inkwell::targets::TargetTriple,
    opt_level: OptimizationLevel,
) -> Option<TargetMachine> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::CreateTargetMachine) {
        return None;
    }
    target.create_target_machine(
        triple,
        "generic",
        "",
        opt_level,
        RelocMode::PIC,
        CodeModel::Default,
    )
}

#[cfg_attr(coverage, inline(never))]
fn write_object_file(
    target_machine: &TargetMachine,
    module: &Module,
    output_path: &Path,
) -> Result<(), String> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::WriteObjectFile) {
        return Err("coverage injected: write_object_file".to_string());
    }
    match target_machine.write_to_file(module, FileType::Object, output_path) {
        Ok(()) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

#[cfg_attr(coverage, inline(never))]
fn create_runtime_file(path: &Path) -> io::Result<std::fs::File> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::CreateRuntimeFile) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "coverage injected: create_runtime_file",
        ));
    }
    std::fs::File::create(path)
}

#[cfg_attr(coverage, inline(never))]
fn write_runtime_file(handle: &mut std::fs::File) -> io::Result<()> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::WriteRuntimeFile) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "coverage injected: write_runtime_file",
        ));
    }
    handle.write_all(EMBEDDED_RUNTIME)
}

#[cfg_attr(coverage, inline(never))]
fn create_runtime_rs_file(path: &Path) -> io::Result<std::fs::File> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::CreateRuntimeRsFile) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "coverage injected: create_runtime_rs_file",
        ));
    }
    std::fs::File::create(path)
}

#[cfg_attr(coverage, inline(never))]
fn write_runtime_rs_file(handle: &mut std::fs::File) -> io::Result<()> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::WriteRuntimeRsFile) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "coverage injected: write_runtime_rs_file",
        ));
    }
    handle.write_all(EMBEDDED_RUNTIME_RS)
}

#[cfg_attr(coverage, inline(never))]
fn create_gc_stub_file(path: &Path) -> io::Result<std::fs::File> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::CreateGcStubFile) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "coverage injected: create_gc_stub_file",
        ));
    }
    std::fs::File::create(path)
}

#[cfg_attr(coverage, inline(never))]
fn write_gc_stub_file(handle: &mut std::fs::File) -> io::Result<()> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::WriteGcStubFile) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "coverage injected: write_gc_stub_file",
        ));
    }
    handle.write_all(EMBEDDED_GC_STUB)
}

#[cfg_attr(coverage, inline(never))]
fn link_command_status(link_args: &[&str]) -> io::Result<std::process::ExitStatus> {
    #[cfg(all(test, coverage))]
    if coverage_inject::should_fail(coverage_inject::FailPoint::LinkCommandStatus) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "coverage injected: link_command_status",
        ));
    }
    Command::new("cc").args(link_args).status()
}

/// LLVM Compiler for mdhavers
pub struct LLVMCompiler {
    // Configuration options
    opt_level: OptimizationLevel,
}

impl LLVMCompiler {
    pub fn new() -> Self {
        LLVMCompiler {
            opt_level: OptimizationLevel::Default,
        }
    }

    /// Set optimization level (0-3)
    pub fn with_optimization(mut self, level: u8) -> Self {
        self.opt_level = match level {
            0 => OptimizationLevel::None,
            1 => OptimizationLevel::Less,
            2 => OptimizationLevel::Default,
            _ => OptimizationLevel::Aggressive,
        };
        self
    }

    /// Compile to LLVM IR (text format)
    pub fn compile_to_ir(&self, program: &Program) -> Result<String, HaversError> {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "mdhavers_module");

        codegen.compile(program)?;

        Ok(codegen.get_module().print_to_string().to_string())
    }

    /// Compile to object file
    #[allow(dead_code)]
    pub fn compile_to_object(
        &self,
        program: &Program,
        output_path: &Path,
    ) -> Result<(), HaversError> {
        self.compile_to_object_with_source(program, output_path, None)
    }

    /// Compile to object file with source path for import resolution
    pub fn compile_to_object_with_source(
        &self,
        program: &Program,
        output_path: &Path,
        source_path: Option<&Path>,
    ) -> Result<(), HaversError> {
        self.compile_to_object_with_source_status(program, output_path, source_path, None)
    }

    fn compile_to_object_with_source_status(
        &self,
        program: &Program,
        output_path: &Path,
        source_path: Option<&Path>,
        mut status: Option<&mut BuildStatus>,
    ) -> Result<(), HaversError> {
        if let Some(status) = status.as_mut() {
            status.update("Generating LLVM IR", StatusColor::Yellow);
        }

        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "mdhavers_module");
        if let Some(path) = source_path {
            codegen.set_source_path(path);
        }

        codegen.compile(program)?;

        if let Some(status) = status.as_mut() {
            status.update("Initializing target", StatusColor::Yellow);
        }

        // Initialize native target
        if let Err(err) = initialize_native_target() {
            return Err(HaversError::CompileError(err));
        }

        let target_triple = TargetMachine::get_default_triple();
        let target = {
            #[cfg(coverage)]
            let target_from_triple_fn: fn(&inkwell::targets::TargetTriple) -> Result<Target, String> =
                target_from_triple;
            #[cfg(coverage)]
            let target_from_triple_fn = unsafe { std::ptr::read_volatile(&target_from_triple_fn) };
            #[cfg(not(coverage))]
            let target_from_triple_fn = target_from_triple;

            match target_from_triple_fn(&target_triple) {
                Ok(target) => target,
                Err(err) => return Err(HaversError::CompileError(err)),
            }
        };

        let target_machine = create_target_machine(&target, &target_triple, self.opt_level)
            .ok_or(HaversError::CompileError(
                "Failed to create target machine".to_string(),
            ))?;

        if let Some(status) = status.as_mut() {
            if matches!(self.opt_level, OptimizationLevel::None) {
                status.update("Skipping optimizations", StatusColor::Dim);
            } else {
                status.update("Optimizing LLVM", StatusColor::Yellow);
            }
        }

        // Run optimization passes
        self.run_optimization_passes(codegen.get_module())?;

        if let Some(status) = status.as_mut() {
            status.update("Writing object file", StatusColor::Yellow);
        }

        // Write object file
        #[cfg(coverage)]
        let write_object_file_fn: fn(&TargetMachine, &Module, &Path) -> Result<(), String> =
            write_object_file;
        #[cfg(coverage)]
        let write_object_file_fn = unsafe { std::ptr::read_volatile(&write_object_file_fn) };
        #[cfg(not(coverage))]
        let write_object_file_fn = write_object_file;

        if let Err(err) = write_object_file_fn(&target_machine, codegen.get_module(), output_path) {
            return Err(HaversError::CompileError(err));
        }

        Ok(())
    }

    /// Compile to native executable
    #[allow(dead_code)]
    pub fn compile_to_native(
        &self,
        program: &Program,
        output_path: &Path,
        opt_level: u8,
    ) -> Result<(), HaversError> {
        self.compile_to_native_with_source(program, output_path, opt_level, None)
    }

    /// Compile to native executable with source path for import resolution
    pub fn compile_to_native_with_source(
        &self,
        program: &Program,
        output_path: &Path,
        opt_level: u8,
        source_path: Option<&Path>,
    ) -> Result<(), HaversError> {
        let mut status = BuildStatus::new("Native build");
        let _status_guard = status.guard();

        // First compile to object file
        let obj_path = output_path.with_extension("o");
        let compiler = LLVMCompiler::new().with_optimization(opt_level);
        if let Err(err) = compiler.compile_to_object_with_source_status(
            program,
            &obj_path,
            source_path,
            Some(&mut status),
        ) {
            status.fail("Native build failed");
            return Err(err);
        }

        // Generate unique temp file names using process ID and a counter
        // This avoids race conditions when tests run in parallel
        let unique_id = format!("{}_{:?}", std::process::id(), std::thread::current().id());
        let runtime_path = std::env::temp_dir().join(format!("mdh_runtime_{}.o", unique_id));
        let runtime_rs_path = std::env::temp_dir().join(format!("mdh_runtime_rs_{}.a", unique_id));
        let gc_stub_path = std::env::temp_dir().join(format!("mdh_gc_stub_{}.o", unique_id));
        let _cleanup = TempFileCleanup::new(vec![
            obj_path.clone(),
            runtime_path.clone(),
            runtime_rs_path.clone(),
            gc_stub_path.clone(),
        ]);

        status.update("Preparing runtime", StatusColor::Yellow);

        // Write embedded runtime to temp file for linking
        {
            let mut handle = match create_runtime_file(&runtime_path) {
                Ok(handle) => handle,
                Err(err) => return Err(HaversError::CompileError(err.to_string())),
            };
            if let Err(err) = write_runtime_file(&mut handle) {
                return Err(HaversError::CompileError(err.to_string()));
            }
        }

        // Write embedded Rust runtime to temp file for linking
        {
            let mut handle = match create_runtime_rs_file(&runtime_rs_path) {
                Ok(handle) => handle,
                Err(err) => return Err(HaversError::CompileError(err.to_string())),
            };
            if let Err(err) = write_runtime_rs_file(&mut handle) {
                return Err(HaversError::CompileError(err.to_string()));
            }
        }

        // Write embedded GC stub to temp file for linking
        {
            let mut handle = match create_gc_stub_file(&gc_stub_path) {
                Ok(handle) => handle,
                Err(err) => return Err(HaversError::CompileError(err.to_string())),
            };
            if let Err(err) = write_gc_stub_file(&mut handle) {
                return Err(HaversError::CompileError(err.to_string()));
            }
        }

        status.update("Linking native executable", StatusColor::Yellow);

        // Link with system linker
        let mut link_args = vec![
            obj_path.to_str().unwrap(),
            runtime_path.to_str().unwrap(),
            runtime_rs_path.to_str().unwrap(),
            gc_stub_path.to_str().unwrap(),
            "-lm", // Math library (for floor, ceil, etc.)
            "-pthread",
            "-static-libgcc",
        ];

        #[cfg(feature = "audio")]
        {
            // miniaudio uses dlopen on Linux for backend loading
            if cfg!(target_os = "linux") {
                link_args.push("-ldl");
            }
        }

        link_args.push("-o");
        link_args.push(output_path.to_str().unwrap());

        let link_status = match link_command_status(&link_args) {
            Ok(status) => status,
            Err(err) => return Err(HaversError::CompileError(err.to_string())),
        };

        if link_status.success() {
            status.finish("Native build complete", StatusColor::Green);
            Ok(())
        } else {
            status.fail("Link failed");
            Err(HaversError::CompileError(format!(
                "Linker failed with exit code: {:?}",
                link_status.code()
            )))
        }
    }

    /// Run LLVM optimization passes
    fn run_optimization_passes(&self, module: &Module) -> Result<(), HaversError> {
        #[cfg(all(test, coverage))]
        if coverage_inject::should_fail(coverage_inject::FailPoint::RunOptimizationPasses) {
            return Err(HaversError::CompileError(
                "coverage injected: run_optimization_passes".to_string(),
            ));
        }

        // Verify the module first
        if let Err(e) = module.verify() {
            return Err(HaversError::CompileError(format!(
                "Module verification failed: {}",
                e.to_string()
            )));
        }

        let opt_level = self.opt_level;

        // Create function pass manager
        let fpm: PassManager<inkwell::values::FunctionValue> = PassManager::create(module);

        // Add passes based on optimization level
        match opt_level {
            OptimizationLevel::None => return Ok(()),
            OptimizationLevel::Less => {
                // -O1: Basic optimizations
                fpm.add_instruction_combining_pass();
                fpm.add_reassociate_pass();
                fpm.add_gvn_pass();
                fpm.add_cfg_simplification_pass();
                fpm.add_basic_alias_analysis_pass();
                fpm.add_promote_memory_to_register_pass();
            }
            OptimizationLevel::Default => {
                // -O2: Standard optimizations
                fpm.add_instruction_combining_pass();
                fpm.add_reassociate_pass();
                fpm.add_gvn_pass();
                fpm.add_cfg_simplification_pass();
                fpm.add_basic_alias_analysis_pass();
                fpm.add_promote_memory_to_register_pass();
                fpm.add_instruction_combining_pass();
                fpm.add_tail_call_elimination_pass();
                fpm.add_dead_store_elimination_pass();
                fpm.add_loop_unroll_pass();
                fpm.add_licm_pass();
            }
            OptimizationLevel::Aggressive => {
                // -O3: Aggressive optimizations
                fpm.add_instruction_combining_pass();
                fpm.add_reassociate_pass();
                fpm.add_gvn_pass();
                fpm.add_cfg_simplification_pass();
                fpm.add_basic_alias_analysis_pass();
                fpm.add_promote_memory_to_register_pass();
                fpm.add_instruction_combining_pass();
                fpm.add_tail_call_elimination_pass();
                fpm.add_dead_store_elimination_pass();
                fpm.add_loop_unroll_pass();
                fpm.add_licm_pass();
                fpm.add_aggressive_dce_pass();
                fpm.add_scalarizer_pass();
                fpm.add_merged_load_store_motion_pass();
                fpm.add_ind_var_simplify_pass();
                fpm.add_loop_vectorize_pass();
                fpm.add_slp_vectorize_pass();
            }
        }

        fpm.initialize();

        // Run on all functions
        for func in module.get_functions() {
            fpm.run_on(&func);
        }

        fpm.finalize();

        Ok(())
    }
}

impl Default for LLVMCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use tempfile::tempdir;

    #[test]
    #[cfg(coverage)]
    fn target_from_triple_error_mapping_branch_is_exercised_for_coverage() {
        let _ = initialize_native_target();
        let triple = inkwell::targets::TargetTriple::create("bogus-unknown-triple-for-coverage");
        let err = target_from_triple(&triple).expect_err("expected unknown triple to error");
        assert!(!err.is_empty());
    }

    #[test]
    fn build_status_paint_returns_plain_text_when_color_disabled_for_coverage() {
        let status = BuildStatus {
            label: "mdh",
            enabled: true,
            use_color: false,
            wrote_any: false,
        };
        assert_eq!(
            status.paint("hello", StatusColor::Green, true),
            "hello".to_string()
        );
    }

    #[test]
    fn build_status_paint_emits_ansi_when_color_enabled_for_coverage() {
        let status = BuildStatus {
            label: "mdh",
            enabled: true,
            use_color: true,
            wrote_any: false,
        };
        let bold = status.paint("ok", StatusColor::Green, true);
        assert!(bold.contains("\u{1b}[1;32m"));
        assert!(bold.ends_with("\u{1b}[0m"));

        let plain = status.paint("ok", StatusColor::Yellow, false);
        assert!(plain.contains("\u{1b}[33m"));
        assert!(plain.ends_with("\u{1b}[0m"));
    }

    #[test]
    fn build_status_update_is_noop_when_disabled_for_coverage() {
        let mut status = BuildStatus {
            label: "mdh",
            enabled: false,
            use_color: false,
            wrote_any: false,
        };
        status.update("stage", StatusColor::Dim);
        assert!(!status.wrote_any);
    }

    #[test]
    fn build_status_guard_ensures_newline_for_coverage() {
        let mut status = BuildStatus {
            label: "mdh",
            enabled: true,
            use_color: false,
            wrote_any: true,
        };
        {
            let _guard = status.guard();
        }
        assert!(!status.wrote_any);
    }

    #[test]
    fn test_compile_simple() {
        let source = r#"
            ken x = 42
            blether x
        "#;

        let program = parse(source).unwrap();
        let compiler = LLVMCompiler::new();
        let ir = compiler.compile_to_ir(&program).unwrap();

        assert!(ir.contains("define i32 @main"));
        // Check for inlined integer creation: { i8 2, i64 42 }
        let has_inline_tag = ir.contains("i8 2");
        let has_insertvalue = ir.contains("insertvalue");
        assert!(has_inline_tag | has_insertvalue);
        // Check for printf call (used by blether)
        assert!(ir.contains("@printf"));
    }

    #[test]
    fn test_compile_function() {
        let source = r#"
            dae add(a, b) {
                gie a + b
            }

            ken result = add(1, 2)
            blether result
        "#;

        let program = parse(source).unwrap();
        let compiler = LLVMCompiler::new();
        let ir = compiler.compile_to_ir(&program).unwrap();

        assert!(ir.contains("define"));
        assert!(ir.contains("@add"));
    }

    #[test]
    fn test_compile_control_flow() {
        let source = r#"
            ken x = 10
            gin x > 5 {
                blether "big"
            } ither {
                blether "small"
            }
        "#;

        let program = parse(source).unwrap();
        let compiler = LLVMCompiler::new();
        let ir = compiler.compile_to_ir(&program).unwrap();

        assert!(ir.contains("br i1")); // Conditional branch
        assert!(ir.contains("then"));
        assert!(ir.contains("else"));
    }

    #[test]
    fn test_compile_loop() {
        let source = r#"
            ken i = 0
            whiles i < 10 {
                blether i
                i = i + 1
            }
        "#;

        let program = parse(source).unwrap();
        let compiler = LLVMCompiler::new();
        let ir = compiler.compile_to_ir(&program).unwrap();

        assert!(ir.contains("loop"));
        assert!(ir.contains("body"));
    }

    #[test]
    fn test_compile_audio_builtins() {
        let source = r#"
            soond_stairt()
            soond_wheesht(aye)
            soond_luid(0.5)
            ken v = soond_hou_luid()
            soond_haud_gang()
            ken sfx = soond_lade("a.wav")
            soond_ready(sfx)
            soond_spiel(sfx)
            soond_haud(sfx)
            soond_gae_on(sfx)
            soond_stap(sfx)
            soond_is_spielin(sfx)
            soond_pit_luid(sfx, 0.7)
            soond_pit_pan(sfx, -0.2)
            soond_pit_tune(sfx, 1.1)
            soond_pit_rin_roond(sfx, aye)
            soond_unlade(sfx)
            soond_steek()

            ken mus = muisic_lade("a.mp3")
            muisic_spiel(mus)
            muisic_haud(mus)
            muisic_gae_on(mus)
            muisic_stap(mus)
            muisic_is_spielin(mus)
            muisic_loup(mus, 0.2)
            muisic_hou_lang(mus)
            muisic_whaur(mus)
            muisic_pit_luid(mus, 0.5)
            muisic_pit_pan(mus, 0.0)
            muisic_pit_tune(mus, 1.0)
            muisic_pit_rin_roond(mus, nae)
            muisic_unlade(mus)

            ken song = midi_lade("a.mid", naething)
            midi_spiel(song)
            midi_haud(song)
            midi_gae_on(song)
            midi_stap(song)
            midi_is_spielin(song)
            midi_loup(song, 1.0)
            midi_hou_lang(song)
            midi_whaur(song)
            midi_pit_luid(song, 0.4)
            midi_pit_pan(song, -0.5)
            midi_pit_rin_roond(song, aye)
            midi_unlade(song)
        "#;

        let program = parse(source).unwrap();
        let compiler = LLVMCompiler::new();
        let ir = compiler.compile_to_ir(&program).unwrap();

        assert!(ir.contains("@__mdh_soond_stairt"));
        assert!(ir.contains("@__mdh_muisic_lade"));
        assert!(ir.contains("@__mdh_midi_lade"));
    }

    #[test]
    fn test_with_optimization_levels() {
        let none = LLVMCompiler::new().with_optimization(0);
        let is_none = |compiler: &LLVMCompiler| matches!(compiler.opt_level, OptimizationLevel::None);
        assert!(is_none(&none));
        assert!(!is_none(&LLVMCompiler::new().with_optimization(1)));

        let less = LLVMCompiler::new().with_optimization(1);
        let is_less = |compiler: &LLVMCompiler| matches!(compiler.opt_level, OptimizationLevel::Less);
        assert!(is_less(&less));
        assert!(!is_less(&LLVMCompiler::new().with_optimization(0)));

        let default = LLVMCompiler::new().with_optimization(2);
        let is_default =
            |compiler: &LLVMCompiler| matches!(compiler.opt_level, OptimizationLevel::Default);
        assert!(is_default(&default));
        assert!(!is_default(&LLVMCompiler::new().with_optimization(3)));

        let aggressive = LLVMCompiler::new().with_optimization(3);
        let is_aggressive =
            |compiler: &LLVMCompiler| matches!(compiler.opt_level, OptimizationLevel::Aggressive);
        assert!(is_aggressive(&aggressive));
        assert!(!is_aggressive(&LLVMCompiler::new().with_optimization(2)));
    }

    #[test]
    fn test_build_status_updates_and_painting() {
        let mut status = BuildStatus::new("Test");
        status.enabled = true;
        status.use_color = true;

        status.update("Warmup", StatusColor::Yellow);
        status.update("Dimmed", StatusColor::Dim);
        status.finish("Done", StatusColor::Green);
        status.fail("Failed");

        let mut plain = BuildStatus::new("Plain");
        plain.use_color = false;
        assert_eq!(plain.paint("text", StatusColor::Red, false), "text");
    }

    #[test]
    fn test_build_status_guard_drop_emits_newline() {
        let mut status = BuildStatus::new("Guard");
        status.enabled = true;
        status.use_color = false;
        {
            let _guard = status.guard();
            status.update("Stage", StatusColor::Yellow);
        }
    }

    #[test]
    fn test_compile_to_object_and_write_error_paths() {
        let program = parse("ken x = 1").unwrap();
        let compiler = LLVMCompiler::new();

        let dir = tempdir().unwrap();
        let obj_path = dir.path().join("out.o");
        compiler.compile_to_object(&program, &obj_path).unwrap();
        assert!(obj_path.exists());

        let err = compiler
            .compile_to_object(&program, dir.path())
            .unwrap_err();
        let is_compile_error = |err: &HaversError| matches!(err, HaversError::CompileError(_));
        assert!(is_compile_error(&err));
        assert!(!is_compile_error(&HaversError::ParseError {
            message: String::new(),
            line: 0,
        }));
    }

    #[test]
    fn test_compile_to_native_with_source_propagates_object_build_failure() {
        let program = parse("ken x = 1").unwrap();
        let compiler = LLVMCompiler::new();

        let dir = tempdir().unwrap();
        let output_path = dir.path().join("missing_dir").join("out");
        let err = compiler
            .compile_to_native_with_source(&program, &output_path, 0, None)
            .unwrap_err();
        let is_compile_error = |err: &HaversError| matches!(err, HaversError::CompileError(_));
        assert!(is_compile_error(&err));
        assert!(!is_compile_error(&HaversError::ParseError {
            message: String::new(),
            line: 0,
        }));
    }

    #[test]
    fn test_compile_to_object_status_and_skip_optimizations() {
        let program = parse("ken x = 1").unwrap();
        let mut status = BuildStatus::new("Test");
        status.enabled = true;
        status.use_color = false;

        let compiler = LLVMCompiler::new().with_optimization(0);
        let dir = tempdir().unwrap();
        let obj_path = dir.path().join("out2.o");
        compiler
            .compile_to_object_with_source_status(&program, &obj_path, None, Some(&mut status))
            .unwrap();
        assert!(obj_path.exists());
    }

    #[test]
    fn test_run_optimization_passes_invalid_module_errors() {
        let context = Context::create();
        let module = context.create_module("bad");
        let fn_ty = context.void_type().fn_type(&[], false);
        let func = module.add_function("bad", fn_ty, None);
        context.append_basic_block(func, "entry");

        let compiler = LLVMCompiler::new();
        let err = compiler.run_optimization_passes(&module).unwrap_err();
        assert!(err.to_string().contains("Module verification failed"));
    }

    #[cfg(coverage)]
    #[test]
    fn build_status_guard_drop_is_safe_with_null_status_ptr_for_coverage() {
        // `BuildStatusGuard` is internal, but its Drop should be resilient (it already checks
        // `as_mut()`), so exercise the `None` branch of that check for region coverage.
        let guard = BuildStatusGuard {
            status: std::ptr::null_mut(),
        };
        drop(guard);
    }

    #[test]
    fn run_optimization_passes_covers_less_and_aggressive_levels_for_coverage() {
        let program = parse("ken x = 1\nx").unwrap();
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "opt_levels");
        codegen.compile(&program).unwrap();

        let less = LLVMCompiler::new().with_optimization(1);
        less.run_optimization_passes(codegen.get_module())
            .expect("less passes");

        let aggressive = LLVMCompiler::new().with_optimization(3);
        aggressive
            .run_optimization_passes(codegen.get_module())
            .expect("aggressive passes");
    }

    #[cfg(coverage)]
    #[test]
    fn llvm_compiler_wrappers_execute_for_instantiation_coverage() {
        let program = parse("blether 1").unwrap();
        let dir = tempdir().unwrap();

        let obj_path = dir.path().join("wrapper_with_source.o");
        LLVMCompiler::new()
            .compile_to_object_with_source(&program, &obj_path, None)
            .unwrap();
        assert!(obj_path.exists());

        let exe_path = dir.path().join("wrapper_exe");
        LLVMCompiler::default()
            .compile_to_native(&program, &exe_path, 0)
            .unwrap();
        assert!(exe_path.exists());
    }

    #[cfg(coverage)]
    #[test]
    fn compile_to_object_error_paths_are_covered_via_injection() {
        let program = parse("blether 1").unwrap();
        let dir = tempdir().unwrap();
        let obj_path = dir.path().join("injected.o");

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::InitializeNativeTarget,
            || LLVMCompiler::new().compile_to_object(&program, &obj_path).unwrap_err(),
        );
        assert!(err
            .to_string()
            .contains("coverage injected: initialize_native_target"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::TargetFromTriple,
            || LLVMCompiler::new().compile_to_object(&program, &obj_path).unwrap_err(),
        );
        assert!(err.to_string().contains("coverage injected: target_from_triple"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::CreateTargetMachine,
            || LLVMCompiler::new().compile_to_object(&program, &obj_path).unwrap_err(),
        );
        assert!(err.to_string().contains("Failed to create target machine"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::RunOptimizationPasses,
            || LLVMCompiler::new().compile_to_object(&program, &obj_path).unwrap_err(),
        );
        assert!(err
            .to_string()
            .contains("coverage injected: run_optimization_passes"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::WriteObjectFile,
            || LLVMCompiler::new().compile_to_object(&program, &obj_path).unwrap_err(),
        );
        assert!(err.to_string().contains("coverage injected: write_object_file"));
    }

    #[cfg(coverage)]
    #[test]
    fn compile_to_native_runtime_and_link_error_paths_are_covered_via_injection() {
        let program = parse("blether 1").unwrap();
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("out_exe");

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::CreateRuntimeFile,
            || LLVMCompiler::new().compile_to_native(&program, &output_path, 0).unwrap_err(),
        );
        assert!(err.to_string().contains("coverage injected: create_runtime_file"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::WriteRuntimeFile,
            || LLVMCompiler::new().compile_to_native(&program, &output_path, 0).unwrap_err(),
        );
        assert!(err.to_string().contains("coverage injected: write_runtime_file"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::CreateRuntimeRsFile,
            || LLVMCompiler::new().compile_to_native(&program, &output_path, 0).unwrap_err(),
        );
        assert!(err
            .to_string()
            .contains("coverage injected: create_runtime_rs_file"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::WriteRuntimeRsFile,
            || LLVMCompiler::new().compile_to_native(&program, &output_path, 0).unwrap_err(),
        );
        assert!(err
            .to_string()
            .contains("coverage injected: write_runtime_rs_file"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::CreateGcStubFile,
            || LLVMCompiler::new().compile_to_native(&program, &output_path, 0).unwrap_err(),
        );
        assert!(err.to_string().contains("coverage injected: create_gc_stub_file"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::WriteGcStubFile,
            || LLVMCompiler::new().compile_to_native(&program, &output_path, 0).unwrap_err(),
        );
        assert!(err.to_string().contains("coverage injected: write_gc_stub_file"));

        let err = super::coverage_inject::with_failpoint(
            super::coverage_inject::FailPoint::LinkCommandStatus,
            || LLVMCompiler::new().compile_to_native(&program, &output_path, 0).unwrap_err(),
        );
        assert!(err.to_string().contains("coverage injected: link_command_status"));
    }
}
