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
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/Cargo.toml");
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
    let cc = env::var("CC").unwrap_or_else(|_| "gcc".to_string());

    // Compile the main runtime
    let runtime_obj = out_dir.join("mdh_runtime.o");
    let mut cmd = Command::new(&cc);
    cmd.args(["-c", "-O2", "-fPIC", "runtime/mdh_runtime.c", "-o"]);
    cmd.arg(&runtime_obj);
    if env::var("CARGO_FEATURE_GRAPHICS3D").is_ok() {
        cmd.arg("-DMDH_TRI_RUST");
    }

    let status = cmd.status().expect("Failed to run C compiler");

    if !status.success() {
        panic!("Failed to compile runtime (mdh_runtime.c)");
    }

    // Compile the GC stub (needed for LLVM backend)
    let gc_stub_obj = out_dir.join("gc_stub.o");
    let status = Command::new(&cc)
        .args(["-c", "-O2", "-fPIC", "runtime/gc_stub.c", "-o"])
        .arg(&gc_stub_obj)
        .status()
        .expect("Failed to run C compiler");

    if !status.success() {
        panic!("Failed to compile runtime (gc_stub.c)");
    }

    // Build Rust runtime helpers (JSON + regex) as a staticlib.
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let mut cargo_args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        "runtime/mdh_runtime_rs/Cargo.toml".to_string(),
        "--target".to_string(),
        target.clone(),
    ];
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
    if profile == "release" {
        cargo_args.push("--release".to_string());
    }

    let status = Command::new("cargo")
        .args(&cargo_args)
        .status()
        .expect("Failed to run cargo for mdh_runtime_rs");

    if !status.success() {
        panic!("Failed to compile Rust runtime (mdh_runtime_rs)");
    }

    let lib_name = "libmdh_runtime_rs.a";
    let built_lib = format!(
        "runtime/mdh_runtime_rs/target/{}/{}/{}",
        target, profile, lib_name
    );
    let out_path = out_dir.join("mdh_runtime_rs.a");
    fs::copy(&built_lib, out_path)
        .unwrap_or_else(|e| panic!("Failed to copy {}: {}", built_lib, e));
}
