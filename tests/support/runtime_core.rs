//! Link the production Rust value core into direct C runtime regression drivers.
//!
//! These drivers deliberately omit the higher-level Rust runtime (JSON, networking,
//! and its third-party native dependencies). Compile the actual dependency-free core
//! as an object so they still exercise its C ABI without duplicating its functions.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn build_object(directory: &Path) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let object = directory.join("mdh_runtime_core.o");
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc)
        .arg(root.join("runtime/mdh_runtime_core/src/lib.rs"))
        .args([
            "--edition=2021",
            "--crate-name=mdh_runtime_core",
            "--crate-type=rlib",
            "--emit=obj",
            "-C",
            "opt-level=2",
            "-C",
            "panic=abort",
            "-o",
        ])
        .arg(&object)
        .stdin(Stdio::null())
        .output()
        .expect("run rustc for the dependency-free runtime core");
    assert!(
        output.status.success(),
        "Rust runtime core build failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    object
}
