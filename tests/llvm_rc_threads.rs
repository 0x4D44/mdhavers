#![cfg(all(feature = "cli", feature = "llvm", feature = "native"))]
//! MDH-BUG-FLU-00213: a thread's cached result lives until exit so repeated joins
//! remain valid, then its owning reference must be released before the RC report.

use std::fs;
use std::process::{Command, Stdio};

const REPRO: &str = r#"
ken message = "from-main" + ""
dae worker() { gie message }
blether thread_join(thread_spawn(worker, []))
"#;
const WORKER_STRING: &str = r#"
dae worker() { gie "from-worker" + "" }
blether thread_join(thread_spawn(worker, []))
"#;
const WORKER_LIST: &str = r#"
dae worker() { gie ["from-worker" + "", 7] }
blether thread_join(thread_spawn(worker, []))
"#;
const DISCARDED: &str = r#"
dae worker() { gie "discarded" + "" }
thread_join(thread_spawn(worker, []))
blether "done"
"#;

fn assert_native(source: &str, expected: &str, counted: bool, nofree: bool) {
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".scratch");
    fs::create_dir_all(&scratch).expect("create scratch");
    let dir = tempfile::tempdir_in(scratch).expect("temp dir");
    let source_path = dir.path().join("thread.braw");
    let exe = dir.path().join("thread_exe");
    fs::write(&source_path, source).expect("write source");
    let rc = if counted { "1" } else { "0" };
    let build = Command::new("deltic")
        .args(["timeout", "30"])
        .arg(env!("CARGO_BIN_EXE_mdhavers"))
        .arg("build")
        .arg(&source_path)
        .args(["-O", "2", "-o"])
        .arg(&exe)
        .env("MDH_RC", rc)
        .stdin(Stdio::null())
        .output()
        .expect("build native executable");
    assert!(build.status.success(), "build failed: {build:?}");
    let stdout_path = dir.path().join("stdout");
    let stderr_path = dir.path().join("stderr");
    let mut run = Command::new("deltic");
    run.args(["timeout", "30"])
        .arg(&exe)
        .env("MDH_RC", rc)
        .env("MDH_RC_REPORT", "1")
        .env("MDH_RC_NOFREE", if nofree { "1" } else { "0" })
        .env_remove("MDH_RC_KINDS")
        .env_remove("MDH_RC_FREE_KINDS")
        .stdin(Stdio::null())
        .stdout(fs::File::create(&stdout_path).expect("stdout file"))
        .stderr(fs::File::create(&stderr_path).expect("stderr file"));
    // RLIMIT_FSIZE caps each redirected output at 1 MiB.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            run.pre_exec(|| {
                let limit = libc::rlimit {
                    rlim_cur: 1024 * 1024,
                    rlim_max: 1024 * 1024,
                };
                if libc::setrlimit(libc::RLIMIT_FSIZE, &limit) == 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::last_os_error())
                }
            });
        }
    }
    let status = run.status().expect("run native executable");
    // Remove each built executable immediately after its run.
    fs::remove_file(exe).expect("remove executable");
    let stdout = fs::read_to_string(stdout_path).expect("read stdout");
    let stderr = fs::read_to_string(stderr_path).expect("read stderr");
    assert!(status.success(), "{status}: {stderr}");
    assert_eq!(
        stdout, expected,
        "native output (RC={rc}, NOFREE={nofree}): {stderr}"
    );
    let kinds = if counted { "1fe" } else { "0" };
    let freed = if counted && !nofree { "fe" } else { "0" };
    assert_eq!(
        stderr,
        format!(
            "mdhavers rc: live at exit: string=0 list=0 dict=0 closure=0 instance=0 bytes=0 \
         bound method=0 native=0 total=0 (counted kinds 0x{kinds}, freed kinds 0x{freed})\n"
        ),
        "cached thread result leaked (RC={rc}, NOFREE={nofree})"
    );
}

macro_rules! leak_case {
    ($free:ident, $nofree:ident, $source:expr, $expected:expr) => {
        #[test]
        fn $free() {
            assert_native($source, $expected, true, false);
        }
        #[test]
        fn $nofree() {
            assert_native($source, $expected, true, true);
        }
    };
}

leak_case!(main_string, main_string_nofree, REPRO, "from-main\n");
leak_case!(
    worker_string,
    worker_string_nofree,
    WORKER_STRING,
    "from-worker\n"
);
leak_case!(
    worker_list,
    worker_list_nofree,
    WORKER_LIST,
    "[from-worker, 7]\n"
);
leak_case!(discarded_join, discarded_join_nofree, DISCARDED, "done\n");

#[test]
fn repro_rc_off() {
    assert_native(REPRO, "from-main\n", false, false);
}

#[test]
fn immediate_result() {
    assert_native(
        "dae worker() { gie 7 }\nblether thread_join(thread_spawn(worker, []))",
        "7\n",
        true,
        false,
    );
}

leak_case!(
    repeated_join,
    repeated_join_nofree,
    r#"
dae worker() { gie ["cached" + "", 7] }
ken t = thread_spawn(worker, [])
ken first = thread_join(t)
shove(first, 8)
blether thread_join(t)
first = naething
blether thread_join(t)
"#,
    "[cached, 7, 8]\n[cached, 7, 8]\n"
);
