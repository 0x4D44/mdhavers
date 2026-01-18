use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn main() {
    // `cargo llvm-cov` sets `cfg(coverage)`; register it so `unexpected_cfgs` doesn't warn.
    println!("cargo:rustc-check-cfg=cfg(coverage)");

    // Ensure feature/target changes rerun the build script.
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_LLVM");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_AUDIO");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_GRAPHICS3D");
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-env-changed=CC");

    // Tell cargo to rerun this script if the runtime source changes
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.c");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.h");
    println!("cargo:rerun-if-changed=runtime/gc_stub.c");
    println!("cargo:rerun-if-changed=runtime/platform/platform.h");
    println!("cargo:rerun-if-changed=runtime/platform/platform_unix.c");
    println!("cargo:rerun-if-changed=runtime/platform/platform_win32.c");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/Cargo.toml");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/Cargo.lock");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/lib.rs");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/audio.rs");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/tri_runtime.rs");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/tri_engine.rs");

    let llvm_enabled = env::var("CARGO_FEATURE_LLVM").is_ok();
    if !llvm_enabled {
        return;
    }

    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    if target.starts_with("wasm32") {
        panic!("The 'llvm' feature is not supported for target {target}");
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is required"));

    // Use cc crate for cross-platform C compilation (handles MSVC, GCC, Clang)
    let mut build = cc::Build::new();
    build
        .file("runtime/mdh_runtime.c")
        .include("runtime/platform")
        .opt_level(2)
        .warnings(false);

    // Add platform-specific implementation
    if target.contains("windows") {
        build.file("runtime/platform/platform_win32.c");
        // Link Windows libraries needed by platform layer
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=userenv");
    } else {
        build.file("runtime/platform/platform_unix.c");
        println!("cargo:rustc-link-lib=pthread");
    }

    if env::var("CARGO_FEATURE_GRAPHICS3D").is_ok() {
        build.define("MDH_TRI_RUST", None);
    }

    build.compile("mdh_runtime");

    // Compile the GC stub
    cc::Build::new()
        .file("runtime/gc_stub.c")
        .opt_level(2)
        .warnings(false)
        .compile("gc_stub");

    // Build Rust runtime helpers (JSON + regex) as a staticlib.
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let mut cargo_args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        "runtime/mdh_runtime_rs/Cargo.toml".to_string(),
        "--target".to_string(),
        target.clone(),
    ];
    let runtime_target_dir = out_dir.join("mdh_runtime_rs_target");
    cargo_args.push("--target-dir".to_string());
    cargo_args.push(runtime_target_dir.to_string_lossy().to_string());
    let mut features = Vec::new();
    if env::var("CARGO_FEATURE_AUDIO").is_ok() {
        features.push("audio");
    }
    if env::var("CARGO_FEATURE_GRAPHICS3D").is_ok() {
        features.push("graphics3d");
    }
    if !features.is_empty() {
        cargo_args.push("--features".to_string());
        cargo_args.push(features.join(","));
    }
    match profile.as_str() {
        "debug" => {}
        "release" => {
            cargo_args.push("--release".to_string());
        }
        other => {
            cargo_args.push("--profile".to_string());
            cargo_args.push(other.to_string());
        }
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .args(&cargo_args)
        .status()
        .expect("Failed to run cargo for mdh_runtime_rs");

    if !status.success() {
        panic!("Failed to compile Rust runtime (mdh_runtime_rs)");
    }

    // On Windows, the static lib is named differently
    let (lib_name, out_name) = if target.contains("windows") {
        ("mdh_runtime_rs.lib", "mdh_runtime_rs.lib")
    } else {
        ("libmdh_runtime_rs.a", "mdh_runtime_rs.a")
    };

    let built_lib = runtime_target_dir.join(&target).join(&profile).join(lib_name);
    let out_path = out_dir.join(out_name);
    fs::copy(&built_lib, &out_path)
        .unwrap_or_else(|e| panic!("Failed to copy {} to {}: {}", built_lib.display(), out_path.display(), e));
}
