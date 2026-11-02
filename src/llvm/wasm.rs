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
//!   returned as several values. The Rust runtime (including the scalar value
//!   constructors in `runtime/mdh_runtime_core`) keeps the standard ABI, exports its
//!   `MdhValue` entries as `<name>__rsabi`, and is joined to C in both directions by
//!   `runtime/platform/wasm_abi.c`.
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
pub(crate) fn wasm_target_machine(
    opt_level: OptimizationLevel,
) -> Result<TargetMachine, HaversError> {
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
        .ok_or_else(|| {
            HaversError::CompileError("Failed to create the wasm32 target machine".into())
        })
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
                let arg: BasicValueEnum = call
                    .get_operand(i)
                    .and_then(|op| op.value())
                    .ok_or_else(|| {
                        HaversError::CompileError(format!("wasm32: odd call of {name}"))
                    })?;
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
        let new = module.get_function(helper).unwrap_or_else(|| {
            module.add_function(helper, old.get_type(), Some(Linkage::External))
        });
        old.replace_all_uses_with(new);
        unsafe { old.delete() };
    }
}

/// `snooze` (nanosleep) goes to the runtime's `__mdh_wasm_nanosleep`
/// (`runtime/platform/wasm_sleep.c`), which a program with a screen links in its
/// yielding form: the browser runs while the program sleeps.
fn redirect_sleep(module: &Module<'_>) {
    if let Some(old) = module.get_function("nanosleep") {
        let new = module
            .get_function("__mdh_wasm_nanosleep")
            .unwrap_or_else(|| {
                module.add_function(
                    "__mdh_wasm_nanosleep",
                    old.get_type(),
                    Some(Linkage::External),
                )
            });
        old.replace_all_uses_with(new);
        unsafe { old.delete() };
    }
}

/// Everything the browser build does to compiled code before it is optimised.
pub(crate) fn legalize(module: &Module<'_>) -> Result<(), HaversError> {
    legalize_size_t(module)?;
    redirect_mul_overflow(module);
    redirect_sleep(module);
    Ok(())
}

/// The stack guard of the browser build (natively: tripwire and probes, see
/// `CodeGen::emit_stack_probes`), so unbounded recursion raises "Stack's fair puggled!"
/// rather than trapping. Applied to optimised code. Two stacks can run out:
///
/// - **Linear memory's** (`-sSTACK_SIZE`): on entry each function compares its stack
///   pointer with `__mdh_wasm_stack_floor` (set by `__mdh_stack_guard_init`).
/// - **The engine's own**, which holds every wasm frame and is far smaller in practice
///   (V8: ~1 MB, a few thousand small frames, an uncatchable trap). Each function adds
///   an estimate of its engine frame to `__mdh_wasm_depth` on entry and takes it off
///   before each return; past `__mdh_wasm_depth_limit` it raises. A raise longjmps over
///   frames that never return, so the runtime puts back the count its handler saw
///   (`__mdh_try_push`).
///
/// Either way the function calls `__mdh_wasm_stack_overflow`, which raises the error.
/// The check goes in a new entry block, to which the old entry's static allocas move.
pub(crate) fn guard_stack(module: &Module<'_>) {
    use inkwell::values::InstructionOpcode;
    use inkwell::IntPredicate;
    let context = module.get_context();
    let builder = context.create_builder();
    let i32_type = context.i32_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let global = |name: &str| match module.get_global(name) {
        Some(g) => g,
        None => {
            let g = module.add_global(i32_type, None, name);
            g.set_linkage(Linkage::External);
            g
        }
    };
    let floor = global("__mdh_wasm_stack_floor").as_pointer_value();
    let depth = global("__mdh_wasm_depth").as_pointer_value();
    let depth_limit = global("__mdh_wasm_depth_limit").as_pointer_value();
    let stacksave = module.get_function("llvm.stacksave.p0").unwrap_or_else(|| {
        module.add_function("llvm.stacksave.p0", ptr_type.fn_type(&[], false), None)
    });
    let overflow = module
        .get_function("__mdh_wasm_stack_overflow")
        .unwrap_or_else(|| {
            module.add_function(
                "__mdh_wasm_stack_overflow",
                context.void_type().fn_type(&[], false),
                Some(Linkage::External),
            )
        });
    let noreturn = context.create_enum_attribute(
        inkwell::attributes::Attribute::get_named_enum_kind_id("noreturn"),
        0,
    );
    overflow.add_attribute(AttributeLoc::Function, noreturn);
    let functions: Vec<_> = module
        .get_functions()
        .filter(|f| f.count_basic_blocks() > 0)
        .collect();
    for function in functions {
        // The engine frame, estimated from the values the function computes.
        let mut values = 0u64;
        for block in function.get_basic_block_iter() {
            let mut instr = block.get_first_instruction();
            while let Some(current) = instr {
                instr = current.get_next_instruction();
                if !current.get_type().is_void_type() {
                    values += 1;
                }
            }
        }
        let frame = i32_type.const_int(FRAME_BASE + values / FRAME_VALUES_PER_BYTE, false);
        let old_entry = function.get_first_basic_block().expect("a body");
        for block in function.get_basic_block_iter() {
            let Some(ret) = block.get_terminator() else {
                continue;
            };
            if ret.get_opcode() != InstructionOpcode::Return {
                continue;
            }
            builder.position_before(&ret);
            let d = builder
                .build_load(i32_type, depth, "depth")
                .unwrap()
                .into_int_value();
            let d = builder.build_int_sub(d, frame, "depth_out").unwrap();
            builder.build_store(depth, d).unwrap();
        }
        let check = context.prepend_basic_block(old_entry, "stack_check");
        let overflowed = context.insert_basic_block_after(check, "stack_overflow");
        let mut allocas = Vec::new();
        let mut instr = old_entry.get_first_instruction();
        while let Some(current) = instr {
            instr = current.get_next_instruction();
            let static_size = current
                .get_operand(0)
                .and_then(|op| op.value())
                .is_some_and(|size| size.is_int_value() && size.into_int_value().is_const());
            if current.get_opcode() == InstructionOpcode::Alloca && static_size {
                allocas.push(current);
            }
        }
        builder.position_at_end(check);
        for alloca in allocas {
            alloca.remove_from_basic_block();
            builder.insert_instruction(&alloca, None);
        }
        let sp = builder
            .build_call(stacksave, &[], "sp")
            .unwrap()
            .try_as_basic_value()
            .basic()
            .unwrap()
            .into_pointer_value();
        let sp = builder.build_ptr_to_int(sp, i32_type, "sp_int").unwrap();
        let lowest = builder
            .build_load(i32_type, floor, "stack_floor")
            .unwrap()
            .into_int_value();
        let low = builder
            .build_int_compare(IntPredicate::ULT, sp, lowest, "stack_low")
            .unwrap();
        let d = builder
            .build_load(i32_type, depth, "depth")
            .unwrap()
            .into_int_value();
        let d = builder.build_int_add(d, frame, "depth_in").unwrap();
        builder.build_store(depth, d).unwrap();
        let max = builder
            .build_load(i32_type, depth_limit, "depth_limit")
            .unwrap()
            .into_int_value();
        let deep = builder
            .build_int_compare(IntPredicate::UGT, d, max, "too_deep")
            .unwrap();
        let out = builder.build_or(low, deep, "stack_out").unwrap();
        builder
            .build_conditional_branch(out, overflowed, old_entry)
            .unwrap();
        builder.position_at_end(overflowed);
        builder.build_call(overflow, &[], "").unwrap();
        builder.build_unreachable().unwrap();
    }
}

/// The estimate of a function's V8 frame (`guard_stack`), in bytes: a fixed part plus
/// a byte per two values. Most values are never live together (refcounting's cold
/// paths), so this tracks measured frames better than a slot per value: measured under
/// node 26 at 984 KiB, functions of 68, 967 and 2425 values overflowed at about 4000,
/// 2900 and 1000 frames (some 250, 340 and 980 bytes each); this estimate stops them at
/// about 2360, 880 and 450 with `__mdh_wasm_depth_limit`'s 640 KiB.
const FRAME_BASE: u64 = 256;
const FRAME_VALUES_PER_BYTE: u64 = 2;

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
        HaversError::CompileError(format!(
            "Cannae run {what} ({:?}): {e}",
            command.get_program()
        ))
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
    (
        "libmdh_screen.a",
        &["mdh_screen.c", "platform/screen_emscripten.c"],
    ),
    ("libmdh_sound.a", &["mdh_sound.c"]),
    ("libmdh_sleep.a", &["platform/wasm_sleep.c"]),
    ("libmdh_sleep_yield.a", &["platform/wasm_sleep.c"]),
];

/// Whether an archive's sources are built to yield to the page (see wasm_sleep.c).
fn yields(lib: &str) -> bool {
    lib == "libmdh_sleep_yield.a"
}

/// emcc's arguments for one runtime source, bar its include paths, input and output.
fn c_compile_args(source: &str, yields: bool) -> Vec<&'static str> {
    let mut args = vec!["-O2", "-c", "-sSUPPORT_LONGJMP=wasm"];
    // wasm_mul.c alone keeps the standard ABI (see the file).
    if !source.ends_with("wasm_mul.c") {
        args.extend_from_slice(EMCC_ABI_FLAGS);
    }
    if yields {
        args.push("-DMDH_WEB_YIELD");
    }
    args
}

/// Everything the C archives are built from, for the cache stamp.
fn c_runtime_stamp(src: &Path) -> Result<String, HaversError> {
    let version = Command::new(emcc())
        .arg("--version")
        .output()
        .map_err(|e| HaversError::CompileError(format!("Cannae run emcc: {e}")))?;
    c_sources_stamp(&version.stdout, src, C_ARCHIVES)
}

/// The stamp of `archives` built from `src` by this emcc: its version, each archive's
/// sources and their flags, and every runtime `.c` and `.h`.
fn c_sources_stamp(
    emcc_version: &[u8],
    src: &Path,
    archives: &[(&str, &[&str])],
) -> Result<String, HaversError> {
    let mut hasher = DefaultHasher::new();
    emcc_version.hash(&mut hasher);
    for (lib, sources) in archives {
        lib.hash(&mut hasher);
        for source in *sources {
            source.hash(&mut hasher);
            c_compile_args(source, yields(lib)).hash(&mut hasher);
        }
    }
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
    std::fs::create_dir_all(&dir)
        .map_err(|e| HaversError::CompileError(format!("Cannae make {}: {e}", dir.display())))?;

    // One build at a time in this process (tests build in parallel); across processes
    // each archive is written aside and renamed into place, so a reader sees an old
    // archive or a new one, never a missing or half-written one.
    static BUILD: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _one_at_a_time = BUILD.lock().unwrap_or_else(|e| e.into_inner());

    let stamp_path = dir.join("c.stamp");
    let stamp = c_runtime_stamp(&src)?;
    let fresh = std::fs::read_to_string(&stamp_path).ok().as_deref() == Some(stamp.as_str())
        && C_ARCHIVES.iter().all(|(lib, _)| dir.join(lib).exists());
    if !fresh {
        let obj_dir = dir.join(format!("obj-{}", std::process::id()));
        std::fs::create_dir_all(&obj_dir)
            .map_err(|e| HaversError::CompileError(format!("{}: {e}", obj_dir.display())))?;
        for (lib, sources) in C_ARCHIVES {
            let mut objects = Vec::new();
            for source in *sources {
                let yields = yields(lib);
                let stem = source
                    .replace('/', "_")
                    .replace(".c", if yields { "_yield.o" } else { ".o" });
                let object = obj_dir.join(stem);
                let mut cc = Command::new(emcc());
                cc.args(c_compile_args(source, yields));
                cc.arg("-I")
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
            let aside = dir.join(format!("{lib}.{}.tmp", std::process::id()));
            let _ = std::fs::remove_file(&aside);
            run(
                Command::new(emar()).arg("rcs").arg(&aside).args(&objects),
                &format!("emar {lib}"),
            )?;
            std::fs::rename(&aside, &archive)
                .map_err(|e| HaversError::CompileError(format!("{}: {e}", archive.display())))?;
        }
        let _ = std::fs::remove_dir_all(&obj_dir);
        std::fs::write(&stamp_path, &stamp)
            .map_err(|e| HaversError::CompileError(format!("{}: {e}", stamp_path.display())))?;
    }

    // The Rust runtime: cargo knows when it is stale.
    let rust_target = dir.join("rs");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut build = Command::new(cargo);
    build
        .args(["build", "--release", "--quiet", "--manifest-path"])
        .arg(src.join("mdh_runtime_rs/Cargo.toml"))
        .args(["--target", WASM_TRIPLE, "--target-dir"])
        .arg(&rust_target);
    for (name, _) in std::env::vars_os() {
        if name.to_str().is_some_and(is_parent_build_setting) {
            build.env_remove(&name);
        }
    }
    run(&mut build, "cargo build of the Rust runtime for wasm32")?;
    let rust = rust_target
        .join(WASM_TRIPLE)
        .join("release/libmdh_runtime_rs.a");
    Ok(WasmRuntime { dir, rust })
}

/// Whether an environment variable is a build setting of a cargo this one runs under
/// (cargo-llvm-cov's instrumenting flags and profile file, a compiler wrapper, flags
/// meant for the host): the wasm32 runtime's build must not inherit it.
fn is_parent_build_setting(name: &str) -> bool {
    matches!(
        name,
        "RUSTFLAGS"
            | "CARGO_ENCODED_RUSTFLAGS"
            | "CARGO_BUILD_RUSTFLAGS"
            | "RUSTDOCFLAGS"
            | "CARGO_ENCODED_RUSTDOCFLAGS"
            | "CARGO_BUILD_RUSTDOCFLAGS"
            | "RUSTC_WRAPPER"
            | "RUSTC_WORKSPACE_WRAPPER"
            | "CARGO_BUILD_RUSTC_WRAPPER"
            | "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"
            | "CARGO_TARGET_DIR"
            | "CARGO_BUILD_TARGET_DIR"
            | "LLVM_PROFILE_FILE"
    ) || name.starts_with("CARGO_LLVM_COV")
        || name.starts_with("__CARGO_LLVM_COV")
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

/// The emcc settings among `extra`, in order: `JSPI` for `-sJSPI` or `-s JSPI`.
fn emcc_settings(extra: &[String]) -> impl Iterator<Item = &str> {
    extra
        .iter()
        .enumerate()
        .filter_map(|(i, arg)| match arg.strip_prefix("-s") {
            Some("") => extra.get(i + 1).map(String::as_str),
            Some(rest) => Some(rest),
            None => None,
        })
}

/// Whether the emcc arguments turn on JSPI (`-sJSPI`, `-sJSPI=1`, `-s JSPI`).
fn wants_jspi(extra: &[String]) -> bool {
    let mut setting = false;
    for value in emcc_settings(extra) {
        match value {
            "JSPI" | "JSPI=1" => setting = true,
            "JSPI=0" => setting = false,
            _ => {}
        }
    }
    setting
}

/// The browser build's memory limit (its -sMAXIMUM_MEMORY): see link_wasm.
const WASM_MEMORY_LIMIT: u64 = 2 << 30;

/// An emcc memory size (`4096`, `512MB`, `2GB`) in bytes.
fn memory_size(text: &str) -> Option<u64> {
    let digits = text.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    let unit: u64 = match text[digits.len()..].to_ascii_uppercase().as_str() {
        "" | "B" => 1,
        "KB" => 1 << 10,
        "MB" => 1 << 20,
        "GB" => 1 << 30,
        "TB" => 1 << 40,
        _ => return None,
    };
    digits.parse::<u64>().ok()?.checked_mul(unit)
}

/// A user's memory setting may lower the limit, never raise it: above 2GB an address
/// would no longer fit the runtime's signed 32-bit intptr_t.
fn check_memory_settings(extra: &[String]) -> Result<(), HaversError> {
    for value in emcc_settings(extra) {
        for name in ["MAXIMUM_MEMORY", "INITIAL_MEMORY", "TOTAL_MEMORY"] {
            let Some(size) = value.strip_prefix(name).and_then(|v| v.strip_prefix('=')) else {
                continue;
            };
            if !memory_size(size).is_some_and(|bytes| bytes <= WASM_MEMORY_LIMIT) {
                return Err(HaversError::CompileError(format!(
                    "-s{name}={size} is ower muckle for the browser: its memory stops at 2GB, \
                     so ask for 2GB or less"
                )));
            }
        }
    }
    Ok(())
}

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
    check_memory_settings(extra)?;
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
    // In a page snooze cannot block: frames come from screen_loop. A program built with
    // JSPI (`--emcc-arg=-sJSPI`) may snooze instead, its stack suspended while the page
    // runs (see wasm_sleep.c). ASYNCIFY could not: it supports neither wasm exceptions
    // nor the multivalue ABI.
    link.arg(runtime.dir.join(if wants_jspi(extra) {
        "libmdh_sleep_yield.a"
    } else {
        "libmdh_sleep.a"
    }));
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
            // two agree only there. A lower limit in `extra` overrides it; a higher one
            // is refused (check_memory_settings).
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn jspi_is_opt_in_through_the_emcc_arguments() {
        assert!(!wants_jspi(&args(&[])));
        assert!(!wants_jspi(&args(&["--preload-file", "data"])));
        assert!(wants_jspi(&args(&["-sJSPI"])));
        assert!(wants_jspi(&args(&["-sJSPI=1"])));
        assert!(wants_jspi(&args(&["-s", "JSPI"])));
        assert!(!wants_jspi(&args(&["-sJSPI", "-sJSPI=0"])));
        assert!(!wants_jspi(&args(&["-sJSPI_EXPORTS=main"])));
    }

    #[test]
    fn memory_settings_may_lower_the_limit_but_not_raise_it() {
        assert_eq!(memory_size("4096"), Some(4096));
        assert_eq!(memory_size("512MB"), Some(512 << 20));
        assert_eq!(memory_size("2gb"), Some(2 << 30));
        assert_eq!(memory_size("1TB"), Some(1 << 40));
        assert_eq!(memory_size("2XB"), None);
        assert_eq!(memory_size("-1"), None);
        assert_eq!(memory_size("99999999999TB"), None);
        assert!(check_memory_settings(&args(&["-sMAXIMUM_MEMORY=512MB"])).is_ok());
        assert!(check_memory_settings(&args(&["-sMAXIMUM_MEMORY=2GB"])).is_ok());
        assert!(check_memory_settings(&args(&["-sALLOW_MEMORY_GROWTH=1"])).is_ok());
        for bad in [
            &["-sMAXIMUM_MEMORY=4GB"][..],
            &["-s", "MAXIMUM_MEMORY=2049MB"],
            &["-sINITIAL_MEMORY=3GB"],
            &["-sTOTAL_MEMORY=3GB"],
            &["-sMAXIMUM_MEMORY=-1"],
        ] {
            let err = check_memory_settings(&args(bad)).unwrap_err().to_string();
            assert!(err.contains("ower muckle"), "{bad:?}: {err}");
        }
    }

    /// The stamp covers which sources each archive has and how each is compiled, not
    /// only the sources' text: a change to either rebuilds the cache.
    #[test]
    fn the_runtime_stamp_covers_the_archives_and_their_flags() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("platform")).unwrap();
        std::fs::write(dir.path().join("a.c"), "int a;\n").unwrap();
        let stamp =
            |plan: &[(&str, &[&str])]| c_sources_stamp(b"emcc 1", dir.path(), plan).unwrap();
        let plain = stamp(&[("liba.a", &["a.c"])]);
        assert_eq!(plain, stamp(&[("liba.a", &["a.c"])]));
        // The same source built to yield (-DMDH_WEB_YIELD).
        assert_ne!(plain, stamp(&[("libmdh_sleep_yield.a", &["a.c"])]));
        // A source moved to another archive.
        assert_ne!(
            stamp(&[("liba.a", &["a.c", "b.c"]), ("libb.a", &[])]),
            stamp(&[("liba.a", &["a.c"]), ("libb.a", &["b.c"])])
        );
        // Another emcc.
        assert_ne!(
            plain,
            c_sources_stamp(b"emcc 2", dir.path(), &[("liba.a", &["a.c"])]).unwrap()
        );
        // The flags differ by source: wasm_mul.c keeps the standard ABI.
        assert_ne!(
            c_compile_args("platform/wasm_mul.c", false),
            c_compile_args("a.c", false)
        );
        assert!(c_compile_args("a.c", true).contains(&"-DMDH_WEB_YIELD"));
    }
}
