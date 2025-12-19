use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

fn find_raylib_static(dir: &Path) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_raylib_static(&path) {
                return Some(found);
            }
        } else if path.file_name().and_then(|s| s.to_str()) == Some("libraylib.a") {
            return Some(path);
        }
    }
    None
}

fn main() {
    // `cargo llvm-cov` sets `cfg(coverage)`; register it so `unexpected_cfgs` doesn't warn.
    println!("cargo:rustc-check-cfg=cfg(coverage)");

    // Tell cargo to rerun this script if the runtime source changes
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.c");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.h");
    println!("cargo:rerun-if-changed=runtime/gc_stub.c");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/Cargo.toml");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/lib.rs");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime_rs/src/audio.rs");

    // Compile the main runtime
    let status = Command::new("gcc")
        .args([
            "-c",
            "-O2",
            "-fPIC",
            "runtime/mdh_runtime.c",
            "-o",
            "runtime/mdh_runtime.o",
        ])
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
        "build",
        "--manifest-path",
        "runtime/mdh_runtime_rs/Cargo.toml",
    ];
    if env::var("CARGO_FEATURE_AUDIO").is_ok() {
        cargo_args.push("--features");
        cargo_args.push("audio");
    }
    if profile == "release" {
        cargo_args.push("--release");
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

    // Copy raylib static library built for the runtime crate (needed for audio backend)
    if env::var("CARGO_FEATURE_AUDIO").is_ok() {
        let raylib_search_root = Path::new("runtime")
            .join("mdh_runtime_rs")
            .join("target")
            .join(&profile)
            .join("build");
        let raylib_src = find_raylib_static(&raylib_search_root).unwrap_or_else(|| {
            panic!(
                "Failed to find libraylib.a under {}",
                raylib_search_root.display()
            )
        });
        let raylib_out = Path::new("runtime").join("libraylib.a");
        fs::copy(&raylib_src, &raylib_out)
            .unwrap_or_else(|e| panic!("Failed to copy {}: {}", raylib_src.display(), e));
    }
}
