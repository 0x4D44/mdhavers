#![cfg(unix)]
//! MDH-BUG-FLU-00059: the Makefile's LLVM/zstd detection only knew Linux paths, so
//! macOS always reported LLVM disabled. `make status` is run with `UNAME_S=Darwin`
//! and a stub `brew`, so the Darwin branch is checked on any host with `make`.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

fn make_available() -> bool {
    Command::new("make")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn write_exe(path: &Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// `make status` with a stub `brew` whose `--prefix <formula>` prints `<home>/<formula>`.
fn status(with_llvm: bool, with_zstd: bool) -> String {
    let home = tempfile::tempdir().unwrap();
    let stubs = home.path().join("stubs");
    write_exe(
        &stubs.join("brew"),
        &format!(
            "#!/bin/sh\n[ \"$1\" = \"--prefix\" ] && echo \"{}/$2\"\n",
            home.path().display()
        ),
    );
    if with_llvm {
        write_exe(
            &home.path().join("llvm@19/bin/llvm-config"),
            "#!/bin/sh\necho /fake\n",
        );
    }
    if with_zstd {
        std::fs::create_dir_all(home.path().join("zstd/lib")).unwrap();
        std::fs::write(home.path().join("zstd/lib/libzstd.dylib"), "").unwrap();
    }
    let path = format!(
        "{}:{}",
        stubs.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new("make")
        .args([
            "-s",
            "-C",
            env!("CARGO_MANIFEST_DIR"),
            "UNAME_S=Darwin",
            "status",
        ])
        .env("PATH", path)
        .stdin(Stdio::null())
        .output()
        .expect("run make");
    assert!(
        out.status.success(),
        "make status failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn make_status_finds_homebrew_llvm_and_zstd_on_darwin() {
    if !make_available() {
        return;
    }
    let out = status(true, true);
    assert!(out.contains("libzstd: yes"), "{out}");
    assert!(out.contains("LLVM Feature: enabled"), "{out}");
}

#[test]
fn make_status_on_darwin_reports_missing_pieces() {
    if !make_available() {
        return;
    }
    let out = status(true, false);
    assert!(out.contains("libzstd: no"), "{out}");
    assert!(
        out.contains("LLVM Feature: disabled (missing libzstd)"),
        "{out}"
    );
    let out = status(false, true);
    assert!(
        out.contains("LLVM Feature: disabled (LLVM not found)"),
        "{out}"
    );
}
