//! Main LLVM compiler interface
//!
//! Provides high-level API for compiling mdhavers to LLVM IR, object files,
//! and native executables.

use std::io::{self, IsTerminal, Write};
use std::path::Path;
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
        let use_color = enabled && term_ok && std::env::var_os("NO_COLOR").is_none();
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

/// LLVM Compiler for mdhavers
pub struct LLVMCompiler {
    // Configuration options
    opt_level: OptimizationLevel,
}

impl LLVMCompiler {
    #[inline]
    fn llvm_compile_error<E: std::fmt::Display>(e: E) -> HaversError {
        HaversError::CompileError(e.to_string())
    }

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
        Target::initialize_native(&InitializationConfig::default())
            .map_err(Self::llvm_compile_error)?;

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).map_err(Self::llvm_compile_error)?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                self.opt_level,
                RelocMode::PIC, // Use PIC for PIE executables
                CodeModel::Default,
            )
            .ok_or_else(|| HaversError::CompileError("Failed to create target machine".to_string()))?;

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
        target_machine
            .write_to_file(codegen.get_module(), FileType::Object, output_path)
            .map_err(Self::llvm_compile_error)?;

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

        status.update("Preparing runtime", StatusColor::Yellow);

        // Write embedded runtime to temp file for linking
        std::fs::File::create(&runtime_path)
            .and_then(|mut f| f.write_all(EMBEDDED_RUNTIME))
            .map_err(Self::llvm_compile_error)?;

        // Write embedded Rust runtime to temp file for linking
        std::fs::File::create(&runtime_rs_path)
            .and_then(|mut f| f.write_all(EMBEDDED_RUNTIME_RS))
            .map_err(Self::llvm_compile_error)?;

        // Write embedded GC stub to temp file for linking
        std::fs::File::create(&gc_stub_path)
            .and_then(|mut f| f.write_all(EMBEDDED_GC_STUB))
            .map_err(Self::llvm_compile_error)?;

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

        let link_status = Command::new("cc")
            .args(&link_args)
            .status()
            .map_err(Self::llvm_compile_error)?;

        // Clean up temp files
        let _ = std::fs::remove_file(&obj_path);
        let _ = std::fs::remove_file(&runtime_path);
        let _ = std::fs::remove_file(&runtime_rs_path);
        let _ = std::fs::remove_file(&gc_stub_path);

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
        // Verify the module first
        if let Err(e) = module.verify() {
            return Err(HaversError::CompileError(format!(
                "Module verification failed: {}",
                e.to_string()
            )));
        }

        let opt_level = self.opt_level;

        // Skip optimization if level is None
        if matches!(opt_level, OptimizationLevel::None) {
            return Ok(());
        }

        // Create function pass manager
        let fpm: PassManager<inkwell::values::FunctionValue> = PassManager::create(module);

        // Add passes based on optimization level
        if matches!(opt_level, OptimizationLevel::Less) {
            // -O1: Basic optimizations
            fpm.add_instruction_combining_pass();
            fpm.add_reassociate_pass();
            fpm.add_gvn_pass();
            fpm.add_cfg_simplification_pass();
            fpm.add_basic_alias_analysis_pass();
            fpm.add_promote_memory_to_register_pass();
        } else if matches!(opt_level, OptimizationLevel::Default) {
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
        } else if matches!(opt_level, OptimizationLevel::Aggressive) {
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
        assert!(ir.contains("i8 2") || ir.contains("insertvalue"));
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
        assert!(matches!(none.opt_level, OptimizationLevel::None));

        let less = LLVMCompiler::new().with_optimization(1);
        assert!(matches!(less.opt_level, OptimizationLevel::Less));

        let default = LLVMCompiler::new().with_optimization(2);
        assert!(matches!(default.opt_level, OptimizationLevel::Default));

        let aggressive = LLVMCompiler::new().with_optimization(3);
        assert!(matches!(
            aggressive.opt_level,
            OptimizationLevel::Aggressive
        ));
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
        assert!(matches!(err, HaversError::CompileError(_)));
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
        assert!(matches!(err, HaversError::CompileError(_)));
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
}
