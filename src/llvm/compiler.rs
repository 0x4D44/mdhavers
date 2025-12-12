//! Main LLVM compiler interface
//!
//! Provides high-level API for compiling mdhavers to LLVM IR, object files,
//! and native executables.

use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Embedded runtime object file - compiled into the binary at build time
static EMBEDDED_RUNTIME: &[u8] = include_bytes!("../../runtime/mdh_runtime.o");

/// Embedded GC stub - minimal malloc wrappers for standalone builds
static EMBEDDED_GC_STUB: &[u8] = include_bytes!("../../runtime/gc_stub.o");

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
    pub fn compile_to_object(
        &self,
        program: &Program,
        output_path: &Path,
    ) -> Result<(), HaversError> {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "mdhavers_module");

        codegen.compile(program)?;

        // Initialize native target
        Target::initialize_native(&InitializationConfig::default()).map_err(|e| {
            HaversError::CompileError(format!("Failed to initialize target: {}", e))
        })?;

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| HaversError::CompileError(format!("Failed to get target: {}", e)))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                self.opt_level,
                RelocMode::PIC, // Use PIC for PIE executables
                CodeModel::Default,
            )
            .ok_or_else(|| {
                HaversError::CompileError("Failed to create target machine".to_string())
            })?;

        // Run optimization passes
        self.run_optimization_passes(codegen.get_module())?;

        // Write object file
        target_machine
            .write_to_file(codegen.get_module(), FileType::Object, output_path)
            .map_err(|e| {
                HaversError::CompileError(format!("Failed to write object file: {}", e))
            })?;

        Ok(())
    }

    /// Compile to native executable
    pub fn compile_to_native(
        &self,
        program: &Program,
        output_path: &Path,
        opt_level: u8,
    ) -> Result<(), HaversError> {
        // First compile to object file
        let obj_path = output_path.with_extension("o");
        let compiler = LLVMCompiler::new().with_optimization(opt_level);
        compiler.compile_to_object(program, &obj_path)?;

        // Generate unique temp file names using process ID and a counter
        // This avoids race conditions when tests run in parallel
        let unique_id = format!("{}_{:?}", std::process::id(), std::thread::current().id());
        let runtime_path = std::env::temp_dir().join(format!("mdh_runtime_{}.o", unique_id));
        let gc_stub_path = std::env::temp_dir().join(format!("mdh_gc_stub_{}.o", unique_id));

        // Write embedded runtime to temp file for linking
        std::fs::File::create(&runtime_path)
            .and_then(|mut f| f.write_all(EMBEDDED_RUNTIME))
            .map_err(|e| HaversError::CompileError(format!("Failed to write runtime: {}", e)))?;

        // Write embedded GC stub to temp file for linking
        std::fs::File::create(&gc_stub_path)
            .and_then(|mut f| f.write_all(EMBEDDED_GC_STUB))
            .map_err(|e| HaversError::CompileError(format!("Failed to write GC stub: {}", e)))?;

        // Link with system linker
        let status = Command::new("cc")
            .args([
                obj_path.to_str().unwrap(),
                runtime_path.to_str().unwrap(),
                gc_stub_path.to_str().unwrap(),
                "-lm", // Math library (for floor, ceil, etc.)
                "-o",
                output_path.to_str().unwrap(),
            ])
            .status()
            .map_err(|e| HaversError::CompileError(format!("Failed to run linker: {}", e)))?;

        // Clean up temp files
        let _ = std::fs::remove_file(&obj_path);
        let _ = std::fs::remove_file(&runtime_path);
        let _ = std::fs::remove_file(&gc_stub_path);

        if status.success() {
            Ok(())
        } else {
            Err(HaversError::CompileError(format!(
                "Linker failed with exit code: {:?}",
                status.code()
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

        // Skip optimization if level is None
        if matches!(self.opt_level, OptimizationLevel::None) {
            return Ok(());
        }

        // Create function pass manager
        let fpm: PassManager<inkwell::values::FunctionValue> = PassManager::create(module);

        // Add passes based on optimization level
        match self.opt_level {
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
            OptimizationLevel::None => {}
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
}
