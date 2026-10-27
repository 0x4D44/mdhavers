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

/// Run `scripts/setup-deps.sh` as if on macOS: a stub `uname` says Darwin, and every
/// tool that could install anything is a stub that only appends its command line to a
/// log (the Linux package managers and `sudo` log too, so a wrong branch is seen and
/// nothing is ever installed). Returns (exit ok, stdout, the log).
fn setup_deps_on_darwin(args: &[&str]) -> (bool, String, String) {
    let home = tempfile::tempdir().unwrap();
    let stubs = home.path().join("stubs");
    let log = home.path().join("calls.log");
    write_exe(&stubs.join("uname"), "#!/bin/sh\necho Darwin\n");
    write_exe(
        &stubs.join("brew"),
        &format!(
            "#!/bin/sh\n[ \"$1\" = \"--prefix\" ] && {{ echo \"{h}/$2\"; exit 0; }}\necho \"brew $*\" >> \"{l}\"\n",
            h = home.path().display(),
            l = log.display()
        ),
    );
    for tool in ["xcode-select", "sudo", "apt-get", "dnf", "pacman", "zypper"] {
        write_exe(
            &stubs.join(tool),
            &format!("#!/bin/sh\necho \"{tool} $*\" >> \"{}\"\n", log.display()),
        );
    }
    let path = format!(
        "{}:{}",
        stubs.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new("bash")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/setup-deps.sh"))
        .args(args)
        .env("PATH", path)
        .stdin(Stdio::null())
        .output()
        .expect("run setup-deps.sh");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        std::fs::read_to_string(&log).unwrap_or_default(),
    )
}

/// MDH-BUG-FLU-00059: on Darwin the script must pick Homebrew and its package names,
/// never apt/sudo.
#[test]
fn setup_deps_on_darwin_uses_brew_and_homebrew_package_names() {
    let (ok, stdout, calls) = setup_deps_on_darwin(&["--all"]);
    assert!(ok, "setup-deps.sh failed:\n{stdout}\n{calls}");
    assert!(
        stdout.contains("Detected package manager: brew"),
        "{stdout}"
    );
    for want in [
        "brew install pkg-config",
        "brew install llvm@19 cmake zstd",
        "brew install cmake",
    ] {
        assert!(
            calls.lines().any(|l| l == want),
            "missing `{want}` in:\n{calls}"
        );
    }
    for bad in ["sudo", "apt-get", "dnf", "pacman", "zypper"] {
        assert!(
            !calls.lines().any(|l| l.starts_with(bad)),
            "Darwin run called {bad}:\n{calls}"
        );
    }
}
