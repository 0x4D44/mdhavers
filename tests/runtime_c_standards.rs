//! The C runtime must stay clean under strict C (MDH-BUG-FLU-00151): a declaration straight
//! after a `default:` or `case` label is only C23, so clang warns (-Wc23-extensions) and
//! older clang rejects it. This syntax-checks the runtime with that warning class fatal.

use std::path::Path;
use std::process::{Command, Stdio};

fn compiler() -> String {
    std::env::var("CC").unwrap_or_else(|_| "cc".to_string())
}

fn is_clang(cc: &str) -> bool {
    Command::new(cc)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("clang"))
        .unwrap_or(false)
}

#[test]
fn runtime_has_no_c23_extensions_under_clang() {
    let cc = compiler();
    if !is_clang(&cc) {
        eprintln!("skipping: {cc} is not clang (the C23 label warning is clang's)");
        return;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new(&cc)
        .current_dir(root)
        .args([
            "-std=c17",
            "-fsyntax-only",
            "-Wno-unknown-warning-option",
            // clang 15 spells it c2x, newer clang c23.
            "-Werror=c2x-extensions",
            "-Werror=c23-extensions",
            "-Iruntime",
            "-Iruntime/platform",
            "runtime/mdh_runtime.c",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run the C compiler");
    assert!(
        out.status.success(),
        "mdh_runtime.c uses a C23-only construct:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
