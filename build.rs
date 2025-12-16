use std::process::Command;

fn main() {
    // Tell cargo to rerun this script if the runtime source changes
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.c");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.h");
    println!("cargo:rerun-if-changed=runtime/gc_stub.c");

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
}
