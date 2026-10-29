#![cfg(all(feature = "cli", feature = "llvm"))]

use std::fs;
use std::process::{Command, Stdio};

use tempfile::Builder;

fn assert_counted(source: &str, expected: &str) {
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".scratch");
    fs::create_dir_all(&scratch).expect("scratch directory");
    let dir = Builder::new()
        .prefix("rc-destructure-")
        .tempdir_in(scratch)
        .expect("temporary directory");
    let input = dir.path().join("main.braw");
    let exe = dir.path().join("main");
    fs::write(&input, source).expect("write program");
    let compiled = Command::new("deltic")
        .args(["timeout", "30", env!("CARGO_BIN_EXE_mdhavers"), "build"])
        .arg(&input)
        .args(["-O2", "-o"])
        .arg(&exe)
        .env("MDH_RC", "1")
        .stdin(Stdio::null())
        .output()
        .expect("compile program");
    assert!(compiled.status.success(), "{compiled:?}");

    for nofree in ["0", "1"] {
        let output = Command::new("deltic")
            .args(["timeout", "30"])
            .arg(&exe)
            .env("MDH_RC_REPORT", "1")
            .env("MDH_RC_NOFREE", nofree)
            .env_remove("MDH_RC_KINDS")
            .env_remove("MDH_RC_FREE_KINDS")
            .stdin(Stdio::null())
            .output()
            .expect("run program");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "NOFREE={nofree}: {stderr}");
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            expected,
            "{stderr}"
        );
        assert!(
            stderr.contains("live at exit: string=0 list=0 dict=0 closure=0 instance=0 bytes=0 bound method=0 native=0 total=0 "),
            "NOFREE={nofree}: {stderr}"
        );
    }
}

#[test]
fn string_rest_ascii_owns_its_characters() {
    assert_counted(
        "ken [first, ...mid, last] = \"hello\"\nblether first\nblether len(mid)\nblether last\n",
        "h\n3\no\n",
    );
}

#[test]
fn string_rest_unicode_owns_its_characters() {
    assert_counted(
        "ken [first, ...mid, last] = \"hełło\"\nblether first\nblether len(mid)\nblether mid[0]\nblether last\n",
        "h\n3\ne\no\n",
    );
}

#[test]
fn string_rest_with_two_trailing_names_owns_its_characters() {
    assert_counted(
        "ken [first, ...mid, penultimate, last] = \"hello\"\nblether first\nblether len(mid)\nblether penultimate\nblether last\n",
        "h\n2\nl\no\n",
    );
}

#[test]
fn string_without_rest_releases_its_characters() {
    assert_counted(
        "ken [first, second, last] = \"abc\"\nblether first\nblether second\nblether last\n",
        "a\nb\nc\n",
    );
}

#[test]
fn string_rest_with_trailing_ignore_owns_its_characters() {
    assert_counted(
        "ken [first, ...mid, _] = \"hello\"\nblether first\nblether len(mid)\nblether mid[0]\n",
        "h\n3\ne\n",
    );
}
