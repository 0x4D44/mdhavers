use std::process::Command;

fn main() {
    // Tell cargo to rerun this script if the runtime source changes
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.c");
    println!("cargo:rerun-if-changed=runtime/mdh_runtime.h");

    // Compile the runtime
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
        panic!("Failed to compile runtime");
    }
}
