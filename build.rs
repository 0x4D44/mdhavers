use std::process::Command;
use std::{env, fs};

fn main() {
    if env::var("CARGO_FEATURE_AUDIO").is_ok() && env::var("CARGO_FEATURE_GRAPHICS").is_ok() {
        panic!(
            "Features `audio` and `graphics` both bundle miniaudio (via om-fork-miniaudio and raylib), \
             which causes duplicate symbols at link time. Build with only one enabled."
        );
    }

    // `cargo llvm-cov` sets `cfg(coverage)`; register it so `unexpected_cfgs` doesn't warn.
    println!("cargo:rustc-check-cfg=cfg(coverage)");

    // Tell cargo to rerun this script if the runtime source changes
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.c");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.h");
    println!("cargo:rerun-if-changed=runtime/gc_stub.c");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/Cargo.toml");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/lib.rs");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/audio.rs");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/tri_runtime.rs");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/tri_engine.rs");

    // Compile the main runtime
    let mut c_args = vec![
        "-c",
        "-O2",
        "-fPIC",
        "runtime/mdh_runtime.c",
        "-o",
        "runtime/mdh_runtime.o",
    ];
    if env::var("CARGO_FEATURE_GRAPHICS3D").is_ok() {
        c_args.push("-DMDH_TRI_RUST");
    }

    let status = Command::new("gcc")
        .args(&c_args)
        .status()
        .expect("Failed to run gcc");

    if !status.success() {
        panic!("Failed to compile runtime (mdh_runtime.c)");
    }

    // Compile the GC stub (needed for LLVM backend)
    let status = Command::new("gcc")
        .args([
            "-c",
            "-O2",
            "-fPIC",
            "runtime/gc_stub.c",
            "-o",
            "runtime/gc_stub.o",
        ])
        .status()
        .expect("Failed to run gcc");

    if !status.success() {
        panic!("Failed to compile runtime (gc_stub.c)");
    }

    // Build Rust runtime helpers (JSON + regex) as a staticlib.
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let mut cargo_args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        "runtime/mdh_runtime_rs/Cargo.toml".to_string(),
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
    let built_lib = format!("runtime/mdh_runtime_rs/target/{}/{}", profile, lib_name);
    let out_path = "runtime/mdh_runtime_rs.a";
    fs::copy(&built_lib, out_path)
        .unwrap_or_else(|e| panic!("Failed to copy {}: {}", built_lib, e));
}
