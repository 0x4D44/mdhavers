//! Exercise the migrated symbols through a real C caller, independently of LLVM and
//! the higher-level Rust runtime. The standalone fixture can also be cross-linked.

use mdh_runtime_core::*;
use std::mem::{align_of, offset_of, size_of};
use std::path::Path;
use std::process::{Command, Stdio};

#[path = "support/runtime_core.rs"]
mod runtime_core;

#[test]
fn c_and_rust_agree_on_native_value_abi_and_bits() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().expect("ABI test directory");
    let object = runtime_core::build_object(dir.path());
    let driver = root.join("tests/fixtures/runtime_core_abi.c");
    let exe = dir
        .path()
        .join(if cfg!(windows) { "abi.exe" } else { "abi" });
    let compiler = std::env::var_os("CC").unwrap_or_else(|| {
        if cfg!(target_env = "msvc") {
            "cl".into()
        } else {
            "cc".into()
        }
    });
    let mut build = Command::new(compiler);
    build.current_dir(dir.path()).stdin(Stdio::null());
    if cfg!(target_env = "msvc") {
        // Run from a Visual Studio developer environment. clang-cl also accepts
        // these arguments when selected by CC. Object and executable remain in tmp.
        build
            .args(["/nologo", "/std:c17", "/W4", "/WX", "/O2"])
            .arg(format!("/I{}", root.join("runtime").display()))
            .arg(&driver)
            .arg(format!("/Fe{}", exe.display()))
            .arg("/link")
            .arg(&object);
    } else {
        build
            .args(["-std=c17", "-Wall", "-Wextra", "-Werror", "-O2"])
            .arg("-I")
            .arg(root.join("runtime"))
            .arg(&driver)
            .arg(&object)
            .arg("-o")
            .arg(&exe);
    }
    let output = build.output().expect("run C ABI compiler");
    assert!(
        output.status.success(),
        "C/Rust ABI link failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new(&exe)
        .stdin(Stdio::null())
        .output()
        .expect("run C ABI caller");
    assert!(
        run.status.success(),
        "C/Rust value ABI failed:\n{}\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let tags = [
        MDH_TAG_NIL,
        MDH_TAG_BOOL,
        MDH_TAG_INT,
        MDH_TAG_FLOAT,
        MDH_TAG_STRING,
        MDH_TAG_LIST,
        MDH_TAG_DICT,
        MDH_TAG_FUNCTION,
        MDH_TAG_CLASS,
        MDH_TAG_INSTANCE,
        MDH_TAG_RANGE,
        MDH_TAG_SET,
        MDH_TAG_CLOSURE,
        MDH_TAG_BYTES,
        MDH_TAG_NATIVE,
    ]
    .map(|tag| tag.to_string())
    .join(" ");
    let expected = format!(
        "value {} {} {} {}\nresult {} {} {} {} {}\ntags {tags}\noperations ok\n",
        size_of::<MdhValue>(),
        align_of::<MdhValue>(),
        offset_of!(MdhValue, tag),
        offset_of!(MdhValue, data),
        size_of::<MdhRsResult>(),
        align_of::<MdhRsResult>(),
        offset_of!(MdhRsResult, ok),
        offset_of!(MdhRsResult, value),
        offset_of!(MdhRsResult, error),
    );
    assert_eq!(
        String::from_utf8(run.stdout)
            .expect("ASCII output")
            .replace("\r\n", "\n"),
        expected
    );
}

/// The compiler is the third producer/consumer of this representation. This checks
/// its native memory layout; it does not establish aggregate argument/return ABI
/// lowering for a new target (which needs an executable C/Rust/LLVM call probe).
#[cfg(feature = "llvm")]
#[test]
fn llvm_and_rust_agree_on_native_value_layout_and_tags() {
    use inkwell::context::Context;
    use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
    use inkwell::OptimizationLevel;
    use mdhavers::llvm::types::{MdhTypes, ValueTag};

    for (tag, expected) in [
        (ValueTag::Nil, MDH_TAG_NIL),
        (ValueTag::Bool, MDH_TAG_BOOL),
        (ValueTag::Int, MDH_TAG_INT),
        (ValueTag::Float, MDH_TAG_FLOAT),
        (ValueTag::String, MDH_TAG_STRING),
        (ValueTag::List, MDH_TAG_LIST),
        (ValueTag::Dict, MDH_TAG_DICT),
        (ValueTag::Function, MDH_TAG_FUNCTION),
        (ValueTag::Class, MDH_TAG_CLASS),
        (ValueTag::Instance, MDH_TAG_INSTANCE),
        (ValueTag::Range, MDH_TAG_RANGE),
        (ValueTag::Set, MDH_TAG_SET),
        (ValueTag::Closure, MDH_TAG_CLOSURE),
        (ValueTag::Bytes, MDH_TAG_BYTES),
        (ValueTag::NativeObject, MDH_TAG_NATIVE),
    ] {
        assert_eq!(tag.as_u8(), expected, "LLVM tag {tag:?}");
    }

    Target::initialize_native(&InitializationConfig::default()).expect("native LLVM target");
    let triple = TargetMachine::get_default_triple();
    let machine = Target::from_triple(&triple)
        .expect("host target")
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .expect("host target machine");
    let context = Context::create();
    let value_type = MdhTypes::new(&context).value_type;
    let data = machine.get_target_data();
    assert_eq!(data.get_abi_size(&value_type), size_of::<MdhValue>() as u64);
    assert_eq!(
        data.get_abi_alignment(&value_type),
        align_of::<MdhValue>() as u32
    );
    assert_eq!(
        data.offset_of_element(&value_type, 0),
        Some(offset_of!(MdhValue, tag) as u64)
    );
    assert_eq!(
        data.offset_of_element(&value_type, 1),
        Some(offset_of!(MdhValue, data) as u64)
    );
}
