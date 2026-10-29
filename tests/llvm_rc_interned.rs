#![cfg(feature = "llvm")]
//! Round 7 Change D: RC-on string indexing returns process-lifetime interned characters.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use mdhavers::{parse, LLVMCompiler};
use tempfile::Builder;

const MAX_CAPTURE_BYTES: usize = 1024 * 1024;
const RUN_TIMEOUT: Duration = Duration::from_secs(30);
const TOO_BIG: &str = "Och! That string would be ower muckle tae fit in memory";

#[derive(Clone, Copy, Debug)]
enum RcMode {
    Off,
    On,
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
    output_too_large: bool,
}

/// Compile each case at -O2, with the child process bounded like the existing RC top-level
/// tests. The IR assertion makes every RC-on test prove it reaches the new runtime entrypoint
/// instead of passing against the legacy call by accident.
fn run_native(source: &str, mode: RcMode, extra_env: &[(&str, &str)]) -> Run {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let scratch = root.join(".scratch");
    fs::create_dir_all(&scratch).expect("create worktree scratch directory");
    let dir = Builder::new()
        .prefix("rc7-interned-")
        .tempdir_in(&scratch)
        .expect("create RC test tempdir under .scratch");
    let main_path = dir.path().join("main.braw");
    fs::write(&main_path, source).expect("write native test source");
    let program = parse(source).expect("parse native test source");
    let refcounting = matches!(mode, RcMode::On);

    if refcounting {
        let ir = LLVMCompiler::new()
            .with_refcounting(true)
            .compile_to_ir(&program)
            .expect("compile RC-on test source to IR");
        assert!(
            ir.contains("call { i8, i64 } @__mdh_char_at_interned("),
            "RC-on indexing did not call the interned-character helper:\n{ir}"
        );
    }

    let executable = dir.path().join("rc7_interned_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native_with_source(&program, &executable, 2, Some(&main_path))
        .expect("compile native RC test at -O2");

    let mut command = Command::new(&executable);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match mode {
        RcMode::On => {
            command
                .env("MDH_RC", "1")
                .env("MDH_RC_NOFREE", "1")
                .env("MDH_RC_REPORT", "1");
        }
        RcMode::Off => {
            command
                .env("MDH_RC", "0")
                .env_remove("MDH_RC_NOFREE")
                .env_remove("MDH_RC_REPORT");
        }
    }
    for (key, value) in extra_env {
        command.env(key, value);
    }
    let mut child = command.spawn().expect("spawn RC test program");

    let output_bytes = Arc::new(AtomicUsize::new(0));
    let too_large = Arc::new(AtomicBool::new(false));
    let stdout = drain_output(
        child.stdout.take().expect("stdout pipe"),
        Arc::clone(&output_bytes),
        Arc::clone(&too_large),
    );
    let stderr = drain_output(
        child.stderr.take().expect("stderr pipe"),
        output_bytes,
        Arc::clone(&too_large),
    );

    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if too_large.load(Ordering::Relaxed) {
            let _ = child.kill();
            break child.wait().expect("wait after output limit");
        }
        if let Some(status) = child.try_wait().expect("poll child") {
            break status;
        }
        if started.elapsed() >= RUN_TIMEOUT {
            timed_out = true;
            let _ = child.kill();
            break child.wait().expect("wait after timeout");
        }
        thread::sleep(Duration::from_millis(5));
    };

    let stdout = stdout.join().expect("collect stdout");
    let stderr = stderr.join().expect("collect stderr");
    let run = Run {
        code: status.code(),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
        output_too_large: too_large.load(Ordering::Relaxed),
    };
    drop(dir);
    run
}

fn drain_output<R: Read + Send + 'static>(
    mut reader: R,
    output_bytes: Arc<AtomicUsize>,
    too_large: Arc<AtomicBool>,
) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut captured = Vec::new();
        let mut buffer = [0; 8192];
        loop {
            let count = match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(count) => count,
            };
            let previous = output_bytes.fetch_add(count, Ordering::Relaxed);
            let keep = MAX_CAPTURE_BYTES.saturating_sub(previous).min(count);
            captured.extend_from_slice(&buffer[..keep]);
            if keep != count {
                too_large.store(true, Ordering::Relaxed);
            }
        }
        captured
    })
}

fn assert_completed(run: &Run, expected: &str) {
    assert!(!run.timed_out, "native RC test exceeded 30 seconds");
    assert!(
        !run.output_too_large,
        "native RC test exceeded 1 MiB output"
    );
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, expected, "{}", run.stderr);
}

fn assert_rc_report_zero(stderr: &str) {
    let report = stderr
        .lines()
        .find(|line| line.starts_with("mdhavers rc: live at exit:"))
        .unwrap_or_else(|| panic!("no RC report in stderr: {stderr}"));
    let total = report
        .split(" total=")
        .nth(1)
        .and_then(|tail| tail.split_whitespace().next())
        .unwrap_or_else(|| panic!("no total in RC report: {report}"));
    assert_eq!(total, "0", "{stderr}");
}

#[test]
fn d1_indexed_character_append_does_not_mutate_the_source_character() {
    let source = include_str!("golden/rc7_interned_char_append.braw");
    let expected = include_str!("golden/rc7_interned_char_append.expected");
    let run = run_native(source, RcMode::On, &[]);
    assert_completed(&run, expected);
    assert_rc_report_zero(&run.stderr);
}

#[test]
fn d3_interned_characters_cover_unicode_page_boundaries() {
    let source = r#"
dae show_codepoint(cp) {
    ken s = chr(cp)
    blether ord(s[0])
}
show_codepoint(255)
show_codepoint(256)
show_codepoint(65535)
show_codepoint(65536)
show_codepoint(1114111)
"#;
    let run = run_native(source, RcMode::On, &[]);
    assert_completed(&run, "255\n256\n65535\n65536\n1114111\n");
    assert_rc_report_zero(&run.stderr);
}

#[test]
fn d4_interned_index_reads_match_legacy_unicode_and_error_results() {
    let source = r#"ken word = "é☃😀"
blether ord(word[0])
blether ord(word[1])
blether ord(word[2])
blether ord(word[-1])
blether ord(word[-3])
hae_a_bash {
    blether word[3]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether word[-4]
} gin_it_gangs_wrang e {
    blether e
}
ken number = 7
hae_a_bash {
    blether number[0]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether word[aye]
} gin_it_gangs_wrang e {
    blether e
}
"#;
    let legacy = run_native(source, RcMode::Off, &[]);
    assert_completed(
        &legacy,
        "233\n9731\n128512\n128512\n233\nHoachin'! Index 3 is oot o' bounds (size is 3) at line 8\nHoachin'! Index -4 is oot o' bounds (size is 3) at line 13\nOch! Type error in 'index': got integer and integer\nOch! Type error in 'index': got string and bool\n",
    );

    let interned = run_native(source, RcMode::On, &[]);
    assert_completed(&interned, &legacy.stdout);
}

#[test]
fn d4_interned_helper_uses_the_shared_decoder_and_static_error_fallbacks() {
    let runtime =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/mdh_runtime.c"))
            .expect("read runtime source");
    let legacy = c_function_body(&runtime, "MdhValue __mdh_char_at(");
    let interned = c_function_body(&runtime, "MdhValue __mdh_char_at_interned(");

    assert!(
        legacy.contains("__mdh_char_at_codepoint("),
        "legacy helper does not use the shared codepoint decoder"
    );
    assert!(
        interned.contains("__mdh_char_at_codepoint("),
        "interned helper does not use the shared codepoint decoder"
    );
    assert!(
        runtime.contains("static int __mdh_char_at_codepoint("),
        "the shared character-at decoder is missing"
    );
    assert!(
        interned.contains("return __mdh_intern_empty();"),
        "interned helper must return the static empty string after an error"
    );
    assert!(
        !interned.contains("__mdh_make_string("),
        "interned helper must not return a freshly allocated fallback string"
    );
}

fn c_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing C function {signature}"));
    let tail = &source[start..];
    let end = tail
        .find("\n}\n")
        .unwrap_or_else(|| panic!("no closing brace for C function {signature}"));
    &tail[..end + 2]
}

const D5_SOURCE: &str = r#"ken s = "λ"
hae_a_bash {
    blether s[0]
} gin_it_gangs_wrang e {
    blether e
}
blether ord(s[0])
"#;

fn assert_d5_intern_allocation_failure_retries(kind: &str) {
    let run = run_native(
        D5_SOURCE,
        RcMode::On,
        &[("MDH_TEST_FAIL_INTERN_ALLOC", kind)],
    );
    assert_completed(&run, &format!("{TOO_BIG}\n955\n"));
    assert_rc_report_zero(&run.stderr);
    assert!(
        run.stderr
            .lines()
            .any(|line| line == "mdhavers intern: live blocks=2"),
        "page and character entry should be the only live intern blocks: {}",
        run.stderr
    );
}

#[cfg(debug_assertions)]
#[test]
fn d5_page_allocation_failure_is_caught_and_retry_publishes_a_complete_character() {
    assert_d5_intern_allocation_failure_retries("page");
}

#[cfg(debug_assertions)]
#[test]
fn d5_entry_allocation_failure_is_caught_and_retry_publishes_a_complete_character() {
    assert_d5_intern_allocation_failure_retries("entry");
}

#[test]
fn d6_a_million_list_round_trips_keep_the_interned_character_usable_under_nofree() {
    let source = r#"
dae round_trip(c) {
    ken holder = [naething]
    ken i = 0
    whiles i < 1000000 {
        holder[0] = c
        c = holder[0]
        holder[0] = naething
        i = i + 1
    }
    gie c
}
ken text = "A"
ken c = round_trip(text[0])
blether c
blether text[0]
"#;
    let run = run_native(source, RcMode::On, &[]);
    assert_completed(&run, "A\nA\n");
    assert_rc_report_zero(&run.stderr);
}
