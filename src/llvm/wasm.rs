//! The browser build: `mdhavers build --target wasm32-emscripten`.
//!
//! Compiled code is emitted for `wasm32-unknown-emscripten` and linked with Emscripten's
//! `emcc` into its usual pair, `<out>.js` + `<out>.wasm`. Two things differ from the
//! native build:
//!
//! - **The C ABI.** Compiled code passes `MdhValue` (`{i8, i64}`) as a plain value. On
//!   wasm32 the standard C ABI passes such a struct by pointer, so the target machine
//!   here, and the C runtime when built for the browser, both use the multivalue ABI
//!   (`experimental-mv`), under which struct arguments are flattened and struct results
//!   returned as several values. The Rust runtime keeps the standard ABI and is reached
//!   through `runtime/platform/wasm_abi.c`.
//! - **`size_t` is 32 bits.** Codegen declares the C functions that take or return a
//!   `size_t` with `i64`; `legalize_size_t` narrows those declarations and their calls.
//!
//! The runtime archives are built for the browser on first use from the runtime sources
//! (`MDH_RUNTIME_DIR`, else this source tree's `runtime/`) into a cache
//! (`MDH_WASM_RUNTIME_DIR`, else `target/mdh-wasm-runtime` here), and rebuilt when
//! those sources or the Emscripten version change.

use std::collections::hash_map::DefaultHasher;
use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;

use inkwell::attributes::AttributeLoc;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{
    CodeModel, InitializationConfig, RelocMode, Target, TargetMachine, TargetMachineOptions,
    TargetTriple,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use inkwell::values::{
    AnyValueEnum, BasicMetadataValueEnum, BasicValue, BasicValueEnum, InstructionValue,
};
use inkwell::OptimizationLevel;

use crate::error::HaversError;

/// What `mdhavers build` compiles for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildTarget {
    /// The machine mdhavers runs on (the default).
    #[default]
    Native,
    /// The browser (and node): WebAssembly via Emscripten.
    Wasm32Emscripten,
}

impl BuildTarget {
    /// The `--target` names: `native`, or `wasm32-emscripten` (also the full
    /// `wasm32-unknown-emscripten`).
    pub fn parse(name: &str) -> Result<Self, String> {
        match name {
            "native" => Ok(BuildTarget::Native),
            "wasm32-emscripten" | "wasm32-unknown-emscripten" => Ok(BuildTarget::Wasm32Emscripten),
            other => Err(format!(
                "Dinnae ken the target '{other}': use native or wasm32-emscripten"
            )),
        }
    }
}

pub(crate) const WASM_TRIPLE: &str = "wasm32-unknown-emscripten";
/// The ABI both sides of compiled code use in the browser build (see the module docs).
const WASM_ABI: &str = "experimental-mv";
const WASM_FEATURES: &str =
    "+multivalue,+mutable-globals,+sign-ext,+nontrapping-fptoint,+bulk-memory,+exception-handling";
/// The same choice for clang, when the C runtime is built for the browser.
const EMCC_ABI_FLAGS: &[&str] = &[
    "-sSUPPORT_LONGJMP=wasm",
    "-mmultivalue",
    "-mnontrapping-fptoint",
    "-mbulk-memory",
    "-Xclang",
    "-target-abi",
    "-Xclang",
    WASM_ABI,
];

/// The wasm32 target machine compiled code is built with.
pub(crate) fn wasm_target_machine(opt_level: OptimizationLevel) -> Result<TargetMachine, HaversError> {
    Target::initialize_webassembly(&InitializationConfig::default());
    enable_wasm_sjlj();
    let triple = TargetTriple::create(WASM_TRIPLE);
    let target = Target::from_triple(&triple)
        .map_err(|e| HaversError::CompileError(format!("No wasm32 target in this LLVM: {e}")))?;
    let options = TargetMachineOptions::new()
        .set_cpu("generic")
        .set_features(WASM_FEATURES)
        .set_abi(WASM_ABI)
        .set_level(opt_level)
        .set_reloc_mode(RelocMode::Static)
        .set_code_model(CodeModel::Default);
    target
        .create_target_machine_from_options(&triple, options)
        .ok_or_else(|| HaversError::CompileError("Failed to create the wasm32 target machine".into()))
}

/// `hae_a_bash` is setjmp/longjmp. On wasm the backend must rewrite each setjmp site, with
/// wasm exception handling (as `emcc -sSUPPORT_LONGJMP=wasm` asks for with `-mllvm
/// -wasm-enable-sjlj`; Emscripten's older JS scheme cannot return several values). The
/// option is process-wide but only read by the WebAssembly backend.
fn enable_wasm_sjlj() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let args = [
            CString::new("mdhavers").unwrap(),
            CString::new("-wasm-enable-sjlj").unwrap(),
        ];
        let argv: Vec<*const std::os::raw::c_char> = args.iter().map(|a| a.as_ptr()).collect();
        let overview = CString::new("").unwrap();
        unsafe {
            inkwell::llvm_sys::support::LLVMParseCommandLineOptions(
                argv.len() as i32,
                argv.as_ptr(),
                overview.as_ptr(),
            );
        }
    });
}

/// C functions codegen declares with an `i64` where C has a `size_t`: which parameters
/// (by index), and whether the result is one.
const SIZE_T_ABI: &[(&str, &[usize], bool)] = &[
    ("strlen", &[], true),
    ("memcpy", &[2], false),
    ("strncmp", &[2], false),
    ("snprintf", &[1], false),
    ("qsort", &[1, 2], false),
    ("__mdh_buf_alloc", &[0], false),
    ("__mdh_buf_alloc_checked", &[0], false),
    ("__mdh_buf_realloc", &[1], false),
    ("__mdh_buf_realloc_checked", &[1], false),
    ("__mdh_obj_alloc", &[1], false),
    ("__mdh_obj_alloc_checked", &[1], false),
    ("__mdh_obj_realloc", &[1], false),
    ("__mdh_obj_realloc_checked", &[1], false),
];

/// Narrow each `size_t` of `SIZE_T_ABI` to wasm32's 32 bits: redeclare the function with
/// `i32` there, truncate those arguments at each call and zero-extend a `size_t` result.
/// The name stays, so the passes that look calls up by name still find them.
pub(crate) fn legalize_size_t(module: &Module<'_>) -> Result<(), HaversError> {
    let context = module.get_context();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let builder = context.create_builder();
    for &(name, params, result) in SIZE_T_ABI {
        let Some(old) = module.get_function(name) else {
            continue;
        };
        let old_type = old.get_type();
        let mut param_types: Vec<BasicMetadataTypeEnum> = old_type.get_param_types();
        for &i in params {
            param_types[i] = i32_type.into();
        }
        let new_type = match (result, old_type.get_return_type()) {
            (true, _) => i32_type.fn_type(&param_types, old_type.is_var_arg()),
            (false, Some(ret)) => ret.fn_type(&param_types, old_type.is_var_arg()),
            (false, None) => context
                .void_type()
                .fn_type(&param_types, old_type.is_var_arg()),
        };
        old.as_global_value().set_name(&format!("{name}.size_t64"));
        let new = module.add_function(name, new_type, Some(Linkage::External));
        for loc in [AttributeLoc::Function, AttributeLoc::Return] {
            if loc == AttributeLoc::Return && result {
                continue;
            }
            for attribute in old.attributes(loc) {
                new.add_attribute(loc, attribute);
            }
        }
        for i in 0..param_types.len() {
            if params.contains(&i) {
                continue;
            }
            let loc = AttributeLoc::Param(i as u32);
            for attribute in old.attributes(loc) {
                new.add_attribute(loc, attribute);
            }
        }

        let old_ptr = old.as_global_value().as_pointer_value();
        let mut users = Vec::new();
        let mut next = old_ptr.get_first_use();
        while let Some(u) = next {
            users.push(u.get_user());
            next = u.get_next_use();
        }
        for user in users {
            let call = user_instruction(user).ok_or_else(|| {
                HaversError::CompileError(format!("wasm32: unexpected use of {name}"))
            })?;
            let n = call.get_num_operands();
            let callee = call
                .get_operand(n - 1)
                .and_then(|op| op.value())
                .map(|v| v.as_value_ref());
            if callee != Some(old_ptr.as_value_ref()) {
                return Err(HaversError::CompileError(format!(
                    "wasm32: {name} is used other than by a call"
                )));
            }
            builder.position_before(&call);
            let mut args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(n as usize - 1);
            for i in 0..n - 1 {
                let arg: BasicValueEnum = call.get_operand(i).and_then(|op| op.value()).ok_or_else(
                    || HaversError::CompileError(format!("wasm32: odd call of {name}")),
                )?;
                if params.contains(&(i as usize)) {
                    let narrow = builder
                        .build_int_truncate(arg.into_int_value(), i32_type, "size_t")
                        .unwrap();
                    args.push(narrow.into());
                } else {
                    args.push(arg.into());
                }
            }
            let new_call = builder.build_call(new, &args, "").unwrap();
            if let Some(value) = new_call.try_as_basic_value().basic() {
                let replacement = if result {
                    builder
                        .build_int_z_extend(value.into_int_value(), i64_type, "size_t64")
                        .unwrap()
                        .as_instruction()
                } else {
                    value.as_instruction_value()
                };
                let replacement = replacement.ok_or_else(|| {
                    HaversError::CompileError(format!("wasm32: odd result of {name}"))
                })?;
                call.replace_all_uses_with(&replacement);
            }
            call.erase_from_basic_block();
        }
        unsafe { old.delete() };
    }
    Ok(())
}

/// Point compiled code's 64-bit overflow-checked multiplies at the runtime's own
/// (`runtime/platform/wasm_abi.c`): LLVM would check them with a 128-bit multiply, whose
/// library routine returns through memory where the multivalue ABI expects two results.
/// Run before and after optimisation (the optimiser may form new ones).
pub(crate) fn redirect_mul_overflow(module: &Module<'_>) {
    for (intrinsic, helper) in [
        ("llvm.smul.with.overflow.i64", "__mdh_wasm_smul_overflow"),
        ("llvm.umul.with.overflow.i64", "__mdh_wasm_umul_overflow"),
    ] {
        let Some(old) = module.get_function(intrinsic) else {
            continue;
        };
        let new = module
            .get_function(helper)
            .unwrap_or_else(|| module.add_function(helper, old.get_type(), Some(Linkage::External)));
        old.replace_all_uses_with(new);
        unsafe { old.delete() };
    }
}

/// Everything the browser build does to compiled code before it is optimised.
pub(crate) fn legalize(module: &Module<'_>) -> Result<(), HaversError> {
    legalize_size_t(module)?;
    redirect_mul_overflow(module);
    Ok(())
}

fn user_instruction(user: AnyValueEnum<'_>) -> Option<InstructionValue<'_>> {
    match user {
        AnyValueEnum::InstructionValue(i) => Some(i),
        AnyValueEnum::IntValue(v) => v.as_instruction(),
        AnyValueEnum::PointerValue(v) => v.as_instruction(),
        AnyValueEnum::FloatValue(v) => v.as_instruction(),
        AnyValueEnum::StructValue(v) => v.as_instruction_value(),
        _ => None,
    }
}

use inkwell::values::AsValueRef;

// ---------------------------------------------------------------------------------------
// Toolchain: emcc, the runtime archives built for wasm, and the link.

fn emcc() -> String {
    std::env::var("EMCC").unwrap_or_else(|_| "emcc".to_string())
}

fn emar() -> String {
    std::env::var("EMAR").unwrap_or_else(|_| "emar".to_string())
}

/// Whether `emcc` can be run (the browser build needs it).
pub fn emcc_available() -> bool {
    Command::new(emcc())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn runtime_source_dir() -> PathBuf {
    std::env::var_os("MDH_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime"))
}

fn runtime_cache_dir() -> PathBuf {
    std::env::var_os("MDH_WASM_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("target/mdh-wasm-runtime"))
}

fn run(command: &mut Command, what: &str) -> Result<(), HaversError> {
    let output = command.output().map_err(|e| {
        HaversError::CompileError(format!("Cannae run {what} ({:?}): {e}", command.get_program()))
    })?;
    // wasm-ld only warns of a call whose signature differs from the callee's, and makes
    // it a trap: here that is a broken ABI bridge, so it fails the build.
    let mismatch = String::from_utf8_lossy(&output.stderr).contains("signature mismatch");
    if output.status.success() && !mismatch {
        return Ok(());
    }
    Err(HaversError::CompileError(format!(
        "{what} failed ({}):\n{}{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}

/// One C archive of the browser runtime: its name and sources (relative to `runtime/`).
const C_ARCHIVES: &[(&str, &[&str])] = &[
    (
        "libmdh_runtime.a",
        &[
            "mdh_runtime.c",
            "platform/platform_unix.c",
            "platform/wasm_abi.c",
            "platform/wasm_mul.c",
        ],
    ),
    ("libgc_stub.a", &["gc_stub.c"]),
    ("libmdh_screen.a", &["mdh_screen.c", "platform/screen_none.c"]),
    ("libmdh_sound.a", &["mdh_sound.c"]),
];

/// Everything the C archives are built from, for the cache stamp.
fn c_runtime_stamp(src: &Path) -> Result<String, HaversError> {
    let mut hasher = DefaultHasher::new();
    let version = Command::new(emcc())
        .arg("--version")
        .output()
        .map_err(|e| HaversError::CompileError(format!("Cannae run emcc: {e}")))?;
    version.stdout.hash(&mut hasher);
    EMCC_ABI_FLAGS.hash(&mut hasher);
    let mut files: Vec<PathBuf> = Vec::new();
    for dir in [src.to_path_buf(), src.join("platform")] {
        let entries = std::fs::read_dir(&dir).map_err(|e| {
            HaversError::CompileError(format!("Cannae read the runtime at {}: {e}", dir.display()))
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            if matches!(path.extension().and_then(|e| e.to_str()), Some("c" | "h")) {
                files.push(path);
            }
        }
    }
    files.sort();
    for file in files {
        file.file_name().hash(&mut hasher);
        std::fs::read(&file)
            .map_err(|e| HaversError::CompileError(format!("{}: {e}", file.display())))?
            .hash(&mut hasher);
    }
    Ok(format!("{:016x}", hasher.finish()))
}

/// The runtime archives for the browser, built (or rebuilt) as needed.
pub(crate) struct WasmRuntime {
    dir: PathBuf,
    rust: PathBuf,
}

pub(crate) fn ensure_wasm_runtime() -> Result<WasmRuntime, HaversError> {
    let src = runtime_source_dir();
    let dir = runtime_cache_dir();
    std::fs::create_dir_all(&dir).map_err(|e| {
        HaversError::CompileError(format!("Cannae make {}: {e}", dir.display()))
    })?;

    let stamp_path = dir.join("c.stamp");
    let stamp = c_runtime_stamp(&src)?;
    let fresh = std::fs::read_to_string(&stamp_path).ok().as_deref() == Some(stamp.as_str())
        && C_ARCHIVES.iter().all(|(lib, _)| dir.join(lib).exists());
    if !fresh {
        let obj_dir = dir.join("obj");
        std::fs::create_dir_all(&obj_dir)
            .map_err(|e| HaversError::CompileError(format!("{}: {e}", obj_dir.display())))?;
        for (lib, sources) in C_ARCHIVES {
            let mut objects = Vec::new();
            for source in *sources {
                let stem = source.replace('/', "_").replace(".c", ".o");
                let object = obj_dir.join(stem);
                let mut cc = Command::new(emcc());
                cc.args(["-O2", "-c", "-sSUPPORT_LONGJMP=wasm"]);
                // wasm_mul.c alone keeps the standard ABI (see the file).
                if !source.ends_with("wasm_mul.c") {
                    cc.args(EMCC_ABI_FLAGS);
                }
                cc
                    .arg("-I")
                    .arg(src.join("platform"))
                    .arg("-I")
                    .arg(&src)
                    .arg(src.join(source))
                    .arg("-o")
                    .arg(&object);
                run(&mut cc, &format!("emcc {source}"))?;
                objects.push(object);
            }
            let archive = dir.join(lib);
            let _ = std::fs::remove_file(&archive);
            run(
                Command::new(emar()).arg("rcs").arg(&archive).args(&objects),
                &format!("emar {lib}"),
            )?;
        }
        std::fs::write(&stamp_path, &stamp)
            .map_err(|e| HaversError::CompileError(format!("{}: {e}", stamp_path.display())))?;
    }

    // The Rust runtime: cargo knows when it is stale.
    let rust_target = dir.join("rs");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    run(
        Command::new(cargo)
            .args(["build", "--release", "--quiet", "--manifest-path"])
            .arg(src.join("mdh_runtime_rs/Cargo.toml"))
            .args(["--target", WASM_TRIPLE, "--target-dir"])
            .arg(&rust_target),
        "cargo build of the Rust runtime for wasm32",
    )?;
    let rust = rust_target.join(WASM_TRIPLE).join("release/libmdh_runtime_rs.a");
    Ok(WasmRuntime { dir, rust })
}

/// Read in node, the environment reaches the program as it does natively (Emscripten
/// otherwise gives it a fixed, made-up one).
const NODE_ENV_PRE_JS: &str = "\
Module.preRun = Module.preRun || [];
Module.preRun.push(function () {
  if (typeof process !== 'undefined' && process.env) {
    for (var name in process.env) ENV[name] = process.env[name];
  }
});
";

/// Link `object` and the browser runtime into `output` (`.js`, beside it the `.wasm`).
/// `extra` goes to emcc as it stands (e.g. `--preload-file dir`).
pub(crate) fn link_wasm(
    object: &Path,
    output: &Path,
    uses_screen: bool,
    uses_sound: bool,
    opt_level: u8,
    extra: &[String],
) -> Result<(), HaversError> {
    let runtime = ensure_wasm_runtime()?;
    let pre_js = output.with_extension("pre.js");
    std::fs::write(&pre_js, NODE_ENV_PRE_JS)
        .map_err(|e| HaversError::CompileError(format!("{}: {e}", pre_js.display())))?;
    let mut link = Command::new(emcc());
    link.arg(object)
        .arg(runtime.dir.join("libmdh_runtime.a"))
        .arg(runtime.dir.join("libgc_stub.a"));
    if uses_screen {
        link.arg(runtime.dir.join("libmdh_screen.a"));
    }
    if uses_sound {
        link.arg(runtime.dir.join("libmdh_sound.a"));
    }
    link.arg(&runtime.rust)
        .arg(format!("-O{}", opt_level.min(3)))
        .args(EMCC_ABI_FLAGS)
        .args([
            // The Rust runtime unwinds its panics with wasm exceptions.
            "-fwasm-exceptions",
            "-sALLOW_MEMORY_GROWTH=1",
            // Addresses stay below 2^31: C widens a pointer into MdhValue's i64 through
            // a signed 32-bit intptr_t, compiled code through an unsigned one, and the
            // two agree only there.
            "-sMAXIMUM_MEMORY=2GB",
            "-sSTACK_SIZE=8MB",
            "-sEXIT_RUNTIME=1",
        ])
        .arg("--pre-js")
        .arg(&pre_js)
        .args(extra)
        .arg("-o")
        .arg(output);
    let result = run(&mut link, "emcc link");
    let _ = std::fs::remove_file(&pre_js);
    result
}
