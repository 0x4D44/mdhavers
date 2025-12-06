//! Main LLVM compiler interface
//!
//! Provides high-level API for compiling mdhavers to LLVM IR, object files,
//! and native executables.

use std::path::Path;
use std::process::Command;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;

use crate::ast::Program;
use crate::HaversError;

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
    pub fn compile_to_object(&self, program: &Program, output_path: &Path) -> Result<(), HaversError> {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "mdhavers_module");

        codegen.compile(program)?;

        // Initialize native target
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| HaversError::CompileError(format!("Failed to initialize target: {}", e)))?;

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| HaversError::CompileError(format!("Failed to get target: {}", e)))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                self.opt_level,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| HaversError::CompileError("Failed to create target machine".to_string()))?;

        // Run optimization passes
        self.run_optimization_passes(codegen.get_module())?;

        // Write object file
        target_machine
            .write_to_file(codegen.get_module(), FileType::Object, output_path)
            .map_err(|e| HaversError::CompileError(format!("Failed to write object file: {}", e)))?;

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

        // Link with runtime library using system linker
        let status = Command::new("cc")
            .args(&[
                obj_path.to_str().unwrap(),
                "-lmdh_runtime", // Our runtime library
                "-lgc",          // Boehm GC
                "-lm",           // Math library
                "-o",
                output_path.to_str().unwrap(),
            ])
            .status()
            .map_err(|e| HaversError::CompileError(format!("Failed to run linker: {}", e)))?;

        // Clean up object file
        let _ = std::fs::remove_file(&obj_path);

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
        // Verify the module
        if let Err(e) = module.verify() {
            return Err(HaversError::CompileError(format!(
                "Module verification failed: {}",
                e.to_string()
            )));
        }

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
        assert!(ir.contains("__mdh_make_int"));
        assert!(ir.contains("__mdh_blether"));
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

        assert!(ir.contains("br i1"));  // Conditional branch
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
