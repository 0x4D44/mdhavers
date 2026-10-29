#![cfg(all(feature = "cli", feature = "llvm"))]

use mdhavers::{parse, Interpreter};
use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

const MAX_OUTPUT: u64 = 1024 * 1024;

// Keep compiler configuration local to each child: changing MDH_RC in this test
// process would race the other tests running alongside it.
fn capture(
    mut command: Command,
    dir: &Path,
    label: &str,
    rc: Option<&str>,
) -> (ExitStatus, String, String) {
    let stdout = dir.join(format!("{label}.out"));
    let stderr = dir.join(format!("{label}.err"));
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(File::create(&stdout).unwrap()))
        .stderr(Stdio::from(File::create(&stderr).unwrap()))
        .env_remove("MDH_RC")
        .env_remove("MDH_RC_REPORT");
    if let Some(rc) = rc {
        command.env("MDH_RC", rc);
    }
    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt;
        if label == "native" {
            command.pre_exec(|| {
                let limit = libc::rlimit {
                    rlim_cur: MAX_OUTPUT as libc::rlim_t,
                    rlim_max: MAX_OUTPUT as libc::rlim_t,
                };
                if libc::setrlimit(libc::RLIMIT_FSIZE, &limit) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    let status = command.status().expect("run bounded child");
    let out = fs::read_to_string(stdout).unwrap();
    let err = fs::read_to_string(stderr).unwrap();
    assert!((out.len() + err.len()) as u64 <= MAX_OUTPUT, "output cap");
    (status, out, err)
}

fn assert_parity(source: &str, expected: &str, rc: Option<&str>) {
    let program = parse(source).expect("parse operator defaults");
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&program).expect("interpret defaults");
    assert_eq!(
        format!("{}\n", interpreter.get_output().join("\n")),
        expected,
        "interpreter stdout"
    );

    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join(".scratch");
    fs::create_dir_all(&scratch).unwrap();
    let dir = tempfile::Builder::new()
        .prefix("operator-defaults-")
        .tempdir_in(scratch)
        .unwrap();
    let input = dir.path().join("main.braw");
    let executable = dir.path().join("operator-defaults");
    fs::write(&input, source).unwrap();
    let mut compile = Command::new("deltic");
    compile
        .args(["timeout", "30"])
        .arg(env!("CARGO_BIN_EXE_mdhavers"))
        .arg("build")
        .arg(&input)
        .arg("-o")
        .arg(&executable);
    let (status, _, err) = capture(compile, dir.path(), "compile", rc);
    assert!(
        status.success(),
        "operator defaults must compile (MDH_RC={rc:?}, {status}): {err}"
    );
    let mut run = Command::new("deltic");
    run.args(["timeout", "30"]).arg(&executable);
    let (status, out, err) = capture(run, dir.path(), "native", rc);
    fs::remove_file(executable).expect("remove native fixture executable");
    assert!(
        status.success(),
        "native exit status (MDH_RC={rc:?}, {status}): {err}"
    );
    assert_eq!(out, expected, "native stdout (MDH_RC={rc:?})");
    assert_eq!(err, "", "native stderr (MDH_RC={rc:?})");
}

const REPRO: &str = r#"
kin Ops {
    dae __tak_awa__(that, extra = that + 1) { gie extra }
    dae ordinary(that, extra = that + 1) { gie extra }
}
blether Ops() - 1
ken o = Ops()
blether o - 1
blether o.ordinary(1)
blether Ops().ordinary(1)
dae ordinary(that, extra = that + 1) { gie extra }
blether ordinary(1)
"#;

fn arithmetic_chain() -> String {
    let operators = [
        ("__pit_thegither__", "+"),
        ("__tak_awa__", "-"),
        ("__times__", "*"),
        ("__pairt__", "/"),
        ("__lave__", "%"),
    ];
    let mut source = String::from("kin Parent {\n");
    for (method, _) in operators {
        source.push_str(&format!(
            "dae {method}(that, extra = that, total = [that, extra]) {{ gie total }}\n"
        ));
    }
    source.push_str("}\nkin Ops fae Parent {}\nken o = Ops()\n");
    for (_, operator) in operators {
        source.push_str(&format!(
            "blether Ops() {operator} 2\nblether o {operator} 2\n"
        ));
    }
    source
}

fn comparisons() -> String {
    let operators = [
        ("__same_as__", "=="),
        ("__differs_fae__", "!="),
        ("__wee_er__", "<"),
        ("__wee_er_or_same__", "<="),
        ("__muckle_er__", ">"),
        ("__muckle_er_or_same__", ">="),
    ];
    let mut source = String::from("kin Ops {\n");
    for (method, _) in operators {
        source.push_str(&format!(
            "dae {method}(that, extra = that + 1) {{ gie extra == 3 }}\n"
        ));
    }
    source.push_str("}\nken o = Ops()\n");
    for (_, operator) in operators {
        source.push_str(&format!(
            "blether Ops() {operator} 2\nblether o {operator} 2\n"
        ));
    }
    source
}

const OVERRIDE: &str = r#"
ken hits = 0
dae bump(that) { hits = hits + 1 gie that + 1 }
kin Ops { dae __tak_awa__(that, extra = bump(that), total = [that, extra]) { gie total } }
blether Ops() - 1
ken o = Ops()
blether o.__tak_awa__(1, 40)
blether Ops().__tak_awa__(1, 50)
blether hits
"#;

macro_rules! cases {
    ($repro:ident, $chain:ident, $compare:ident, $override:ident, $rc:expr) => {
        #[test]
        fn $repro() {
            assert_parity(REPRO, "2\n2\n2\n2\n2\n", $rc);
        }
        #[test]
        fn $chain() {
            assert_parity(&arithmetic_chain(), &"[2, 2]\n".repeat(10), $rc);
        }
        #[test]
        fn $compare() {
            assert_parity(&comparisons(), &"aye\n".repeat(12), $rc);
        }
        #[test]
        fn $override() {
            assert_parity(OVERRIDE, "[1, 2]\n[1, 40]\n[1, 50]\n1\n", $rc);
        }
    };
}

cases!(
    repro_rc_default,
    two_earlier_params_rc_default,
    comparisons_rc_default,
    overridden_rc_default,
    None
);
cases!(
    repro_rc_off,
    two_earlier_params_rc_off,
    comparisons_rc_off,
    overridden_rc_off,
    Some("0")
);
