#![cfg(all(feature = "cli", feature = "llvm"))]

use mdhavers::{parse, LLVMCompiler};
use std::process::{Command, Output, Stdio};

const SOURCE: &str = "ken words = [upper(\"hullo\")]\nblether words[0]\n";

fn compile(mode: Option<&str>, emit_ir: bool) -> (Output, Option<String>) {
    let dir = tempfile::tempdir().expect("temp dir");
    let source = dir.path().join("rc.braw");
    let output = dir.path().join(if emit_ir { "rc.ll" } else { "rc" });
    std::fs::write(&source, SOURCE).expect("write source");
    let mut command = Command::new(env!("CARGO_BIN_EXE_mdhavers"));
    command
        .arg("build")
        .arg(&source)
        .arg("-o")
        .arg(&output)
        .stdin(Stdio::null())
        .env_remove("MDH_RC");
    if emit_ir {
        command.arg("--emit-llvm");
    }
    if let Some(mode) = mode {
        command.env("MDH_RC", mode);
    }
    let result = command.output().expect("run compiler");
    let ir = emit_ir
        .then(|| std::fs::read_to_string(output).ok())
        .flatten();
    (result, ir)
}

fn ir(mode: Option<&str>) -> String {
    let (output, ir) = compile(mode, true);
    assert!(output.status.success(), "{output:?}");
    ir.expect("emitted IR")
}

#[test]
fn unset_defaults_to_rc_on_and_one_keeps_working() {
    let default = ir(None);
    assert!(
        default.contains("call void @__mdh_rc_enable()"),
        "{default}"
    );
    assert_eq!(default, ir(Some("1")));
}

#[test]
fn zero_is_byte_identical_to_explicit_rc_off() {
    let explicit_off = LLVMCompiler::new()
        .with_refcounting(false)
        .compile_to_ir(&parse(SOURCE).expect("parse"))
        .expect("explicit off IR");
    let zero = ir(Some("0"));
    assert_eq!(zero, explicit_off);
    assert!(!zero.contains("call void @__mdh_rc_enable()"));
}

#[test]
fn off_in_any_case_is_byte_identical_to_explicit_rc_off() {
    let explicit_off = LLVMCompiler::new()
        .with_refcounting(false)
        .compile_to_ir(&parse(SOURCE).expect("parse"))
        .expect("explicit off IR");
    for mode in ["off", "OFF", "Off", "oFf"] {
        assert_eq!(ir(Some(mode)), explicit_off, "MDH_RC={mode}");
    }
}

#[test]
fn invalid_values_fail_clearly_for_ir_and_native_builds() {
    for mode in ["", "2", "on", "true", " 0", "off "] {
        for emit_ir in [true, false] {
            let (output, ir) = compile(Some(mode), emit_ir);
            assert!(!output.status.success(), "MDH_RC={mode:?}: {output:?}");
            assert!(ir.is_none());
            let error = String::from_utf8_lossy(&output.stderr);
            for accepted in ["MDH_RC", "1", "0", "off", "Dinnae"] {
                assert!(error.contains(accepted), "{error}");
            }
        }
    }
}
