//! Engine and front-end unit fixtures must agree in the interpreter and LLVM native backend.

#![cfg(feature = "cli")]

use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "llvm")]
use std::sync::atomic::{AtomicUsize, Ordering};

const OUTPUT_LIMIT_PER_STREAM: usize = 512 * 1024;
#[cfg(feature = "llvm")]
const BUILD_TIMEOUT: Duration = Duration::from_secs(120);
// Debug-build interpreter runs on a loaded shared box take over a minute.
const RUN_TIMEOUT: Duration = Duration::from_secs(300);

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_braw_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", directory.display()));
    for entry in entries {
        let entry = entry.expect("read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_braw_files(&path, files);
        } else if path.extension() == Some(OsStr::new("braw")) {
            files.push(path);
        }
    }
}

#[derive(Debug)]
struct CapturedStream {
    bytes: Vec<u8>,
    truncated: bool,
}

#[derive(Debug)]
struct CapturedOutput {
    status: ExitStatus,
    stdout: CapturedStream,
    stderr: CapturedStream,
}

fn drain_bounded<R: Read>(mut reader: R) -> CapturedStream {
    let mut bytes = Vec::new();
    let mut truncated = false;
    let mut buffer = [0; 8192];
    loop {
        let count = reader.read(&mut buffer).expect("read subprocess output");
        if count == 0 {
            break;
        }
        let remaining = OUTPUT_LIMIT_PER_STREAM.saturating_sub(bytes.len());
        let keep = count.min(remaining);
        bytes.extend_from_slice(&buffer[..keep]);
        truncated |= keep < count;
    }
    CapturedStream { bytes, truncated }
}

fn join_reader(reader: thread::JoinHandle<CapturedStream>) -> CapturedStream {
    reader.join().expect("subprocess output reader panicked")
}

fn wait_bounded(mut child: Child, timeout: Duration, description: &str) -> CapturedOutput {
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let stdout_reader = thread::spawn(move || drain_bounded(stdout));
    let stderr_reader = thread::spawn(move || drain_bounded(stderr));
    let deadline = Instant::now() + timeout;

    let status = loop {
        if let Some(status) = child.try_wait().expect("poll subprocess") {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let status = child.wait().expect("reap timed out subprocess");
            let output = CapturedOutput {
                status,
                stdout: join_reader(stdout_reader),
                stderr: join_reader(stderr_reader),
            };
            panic!("{description} timed out after {timeout:?}: {output:?}");
        }
        thread::sleep(Duration::from_millis(10));
    };

    CapturedOutput {
        status,
        stdout: join_reader(stdout_reader),
        stderr: join_reader(stderr_reader),
    }
}

fn run_command(mut command: Command, timeout: Duration, description: &str) -> CapturedOutput {
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("start {description}: {error}"));
    let output = wait_bounded(child, timeout, description);
    assert!(
        !output.stdout.truncated && !output.stderr.truncated,
        "{description} exceeded the 1 MiB combined output limit: {output:?}"
    );
    output
}

fn assert_success(output: &CapturedOutput, description: &str) {
    assert!(
        output.status.success(),
        "{description} failed with {}: stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout.bytes),
        String::from_utf8_lossy(&output.stderr.bytes)
    );
}

fn assert_clean_stderr(output: &CapturedOutput, description: &str) {
    assert!(
        output.stderr.bytes.is_empty(),
        "{description} wrote to stderr: {}",
        String::from_utf8_lossy(&output.stderr.bytes)
    );
}

#[cfg(feature = "llvm")]
struct NativeExecutable(PathBuf);

#[cfg(feature = "llvm")]
impl NativeExecutable {
    fn build(source: &Path, rc: bool) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let repo = root();
        let scratch = repo.join(".scratch");
        std::fs::create_dir_all(&scratch).expect("create .scratch directory");
        let executable = scratch.join(format!(
            "bogle-unit-{}-{}{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::env::consts::EXE_SUFFIX
        ));
        let native = Self(executable);

        let mut command = Command::new(env!("CARGO_BIN_EXE_mdhavers"));
        command
            .current_dir(&repo)
            .env("MDHAVERS_HOME", &repo)
            .env("TMPDIR", &scratch)
            .env("MDH_RC", if rc { "1" } else { "0" })
            .arg("build")
            // Fixtures check semantics; avoid repeatedly optimizing the whole game.
            .arg("-O0")
            .arg(source)
            .arg("--output")
            .arg(&native.0);
        let output = run_command(command, BUILD_TIMEOUT, "Bogle unit native build");
        assert_success(&output, &format!("native build of {}", source.display()));
        native
    }

    fn run(&self, rc: bool) -> CapturedOutput {
        let repo = root();
        let mut command = Command::new(&self.0);
        command
            .current_dir(&repo)
            .env("MDHAVERS_HOME", &repo)
            .env("MDH_RC", if rc { "1" } else { "0" });
        run_command(command, RUN_TIMEOUT, "Bogle unit native fixture")
    }
}

#[cfg(feature = "llvm")]
impl Drop for NativeExecutable {
    fn drop(&mut self) {
        for path in [&self.0, &self.0.with_extension("o")] {
            if path.exists() {
                std::fs::remove_file(path)
                    .unwrap_or_else(|error| panic!("remove {}: {error}", path.display()));
            }
        }
    }
}

fn bogle_scan_braw_source(source: &str, relative: &str) -> Vec<String> {
    fn advance(bytes: &[u8], start: usize, end: usize, line: &mut usize, column: &mut usize) {
        for byte in &bytes[start..end] {
            if *byte == b'\n' {
                *line += 1;
                *column = 1;
            } else if byte & 0b1100_0000 != 0b1000_0000 {
                *column += 1;
            }
        }
    }

    fn identifier_start(byte: u8) -> bool {
        byte.is_ascii_alphabetic() || byte == b'_'
    }

    fn identifier_continue(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_'
    }

    fn call_follows(bytes: &[u8], mut index: usize) -> bool {
        loop {
            while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
                index += 1;
            }
            if bytes.get(index) == Some(&b'#') {
                while bytes.get(index).is_some_and(|byte| *byte != b'\n') {
                    index += 1;
                }
                continue;
            }
            if bytes.get(index) == Some(&b'/') && bytes.get(index + 1) == Some(&b'/') {
                while bytes.get(index).is_some_and(|byte| *byte != b'\n') {
                    index += 1;
                }
                continue;
            }
            return bytes.get(index) == Some(&b'(');
        }
    }

    const FLOAT_CALLS: &[&str] = &[
        "sqrt",
        "sin",
        "cos",
        "tan",
        "asin",
        "acos",
        "atan",
        "atan2",
        "hypot",
        "floor",
        "ceil",
        "round",
        "trunc",
        "tae_float",
        "log",
        "log2",
        "log10",
        "exp",
        "pow",
        "pooer",
        "degrees",
        "radians",
    ];
    const FLOAT_CONSTANTS: &[&str] = &["PI", "E", "TAU", "pi", "e", "tau"];
    const RANDOM_CALLS: &[&str] = &[
        "jammy",
        "shuffle",
        "dram",
        "random",
        "haver",
        "blooter",
        "slainte",
        "stooshie",
        "bampot_mode",
    ];
    const TIME_CALLS: &[&str] = &[
        "snooze",
        "bide",
        "the_noo",
        "braw_time",
        "braw_date",
        "noo",
        "tick",
        "stopwatch",
        "date_now",
        "timestamp",
        "timestamp_millis",
    ];

    let bytes = source.as_bytes();
    let mut findings = Vec::new();
    let mut index = 0;
    let mut line = 1;
    let mut column = 1;

    while index < bytes.len() {
        let start = index;
        let token_line = line;
        let token_column = column;
        match bytes[index] {
            b'#' => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                advance(bytes, start, index, &mut line, &mut column);
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                advance(bytes, start, index, &mut line, &mut column);
            }
            quote @ (b'\'' | b'"') => {
                index += 1;
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = (index + 2).min(bytes.len());
                    } else if bytes[index] == quote {
                        index += 1;
                        break;
                    } else {
                        index += 1;
                    }
                }
                advance(bytes, start, index, &mut line, &mut column);
            }
            byte if identifier_start(byte) => {
                index += 1;
                while index < bytes.len() && identifier_continue(bytes[index]) {
                    index += 1;
                }
                let name = &source[start..index];
                if FLOAT_CONSTANTS.contains(&name) {
                    findings.push(format!(
                        "{relative}:{token_line}:{token_column} float constant {name}"
                    ));
                }
                if call_follows(bytes, index)
                    && (name.starts_with("screen_")
                        || name.starts_with("key_")
                        || name.starts_with("mouse_")
                        || name.starts_with("mono_")
                        || name.starts_with("random_")
                        || FLOAT_CALLS.contains(&name)
                        || RANDOM_CALLS.contains(&name)
                        || TIME_CALLS.contains(&name))
                {
                    findings.push(format!(
                        "{relative}:{token_line}:{token_column} forbidden call {name}"
                    ));
                }
                advance(bytes, start, index, &mut line, &mut column);
            }
            byte if byte.is_ascii_digit() => {
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                let mut is_float = false;
                if bytes.get(index) == Some(&b'.')
                    && bytes.get(index + 1).is_some_and(u8::is_ascii_digit)
                {
                    is_float = true;
                    index += 1;
                    while index < bytes.len() && bytes[index].is_ascii_digit() {
                        index += 1;
                    }
                }
                if matches!(bytes.get(index), Some(b'e' | b'E')) {
                    let mut exponent_end = index + 1;
                    if matches!(bytes.get(exponent_end), Some(b'+' | b'-')) {
                        exponent_end += 1;
                    }
                    if bytes.get(exponent_end).is_some_and(u8::is_ascii_digit) {
                        is_float = true;
                        index = exponent_end + 1;
                        while index < bytes.len() && bytes[index].is_ascii_digit() {
                            index += 1;
                        }
                    }
                }
                if is_float {
                    findings.push(format!(
                        "{relative}:{token_line}:{token_column} float literal {}",
                        &source[start..index]
                    ));
                }
                advance(bytes, start, index, &mut line, &mut column);
            }
            _ => {
                index += 1;
                advance(bytes, start, index, &mut line, &mut column);
            }
        }
    }
    findings
}

fn bogle_engine_purity_violations(engine: &std::path::Path) -> Result<Vec<String>, String> {
    fn collect(
        directory: &std::path::Path,
        files: &mut Vec<std::path::PathBuf>,
    ) -> Result<(), String> {
        let entries = std::fs::read_dir(directory)
            .map_err(|error| format!("read {}: {error}", directory.display()))?;
        for entry in entries {
            let path = entry
                .map_err(|error| format!("read entry in {}: {error}", directory.display()))?
                .path();
            if path.is_dir() {
                collect(&path, files)?;
            } else if path
                .extension()
                .is_some_and(|extension| extension == "braw")
            {
                files.push(path);
            }
        }
        Ok(())
    }

    if !engine.is_dir() {
        return Err(format!(
            "missing Bogle engine directory: {}",
            engine.display()
        ));
    }
    let mut files = Vec::new();
    collect(engine, &mut files)?;
    files.sort();
    if files.is_empty() {
        return Err(format!("no .braw sources under {}", engine.display()));
    }

    let mut findings = Vec::new();
    for path in files {
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        let relative = path.strip_prefix(engine).unwrap_or(&path);
        findings.extend(bogle_scan_braw_source(
            &source,
            &relative.display().to_string(),
        ));
    }
    Ok(findings)
}

#[test]
fn engine_sources_stay_pure_integer_and_headless() {
    let engine = root().join("games/bogle/engine");
    let violations = bogle_engine_purity_violations(&engine).expect("scan Bogle engine sources");
    assert!(
        violations.is_empty(),
        "Bogle engine sources must avoid bridge/time calls, float values, and random builtins:\n{}",
        violations.join("\n")
    );
}

#[test]
fn engine_purity_scan_ignores_comments_and_strings_and_rejects_forbidden_calls() {
    let ignored = r#"
# screen_open(1) sin(0) 2.5
// mouse_dx() random_int(1, 2) 3.5
blether "floor(2.5) snooze(1)"
blether 'PI'
ken floor = 1
floor
"#;
    assert!(bogle_scan_braw_source(ignored, "ignored.braw").is_empty());

    let forbidden = "screen_shaw(fb, palette)\nmono_ns()\nsin(0)\nrandom_int(1, 2)\n1.5\nPI\n";
    let findings = bogle_scan_braw_source(forbidden, "mutant.braw");
    assert_eq!(
        findings.len(),
        6,
        "forbidden tokens were not all reported: {findings:?}"
    );
}

#[test]
fn frontend_state_and_render_sources_stay_pure_integer_and_headless() {
    for name in ["frontend", "onboarding"] {
        let path = root().join(format!("games/bogle/shell/{name}.braw"));
        let source = std::fs::read_to_string(&path).expect("read front-end source");
        let violations = bogle_scan_braw_source(&source, &path.display().to_string());
        assert!(
            violations.is_empty(),
            "front-end must be host-independent: {violations:?}"
        );
    }
}

#[test]
fn bogle_unit_fixtures_match_interpreter_and_native() {
    let repo = root();
    let test_dir = repo.join("games/bogle/tests");
    assert!(
        test_dir.is_dir(),
        "missing Bogle tests directory: {}",
        test_dir.display()
    );
    let mut fixtures = Vec::new();
    collect_braw_files(&test_dir, &mut fixtures);
    fixtures.sort();
    assert!(!fixtures.is_empty(), "no Bogle unit .braw fixtures found");

    for source in fixtures {
        let expected_path = source.with_extension("expected");
        let expected = std::fs::read(&expected_path).unwrap_or_else(|error| {
            panic!(
                "read expected output {} for {}: {error}",
                expected_path.display(),
                source.display()
            )
        });

        let mut interpreter = Command::new(env!("CARGO_BIN_EXE_mdhavers"));
        interpreter
            .current_dir(&repo)
            .env("MDHAVERS_HOME", &repo)
            .env("MDH_RC", "0")
            .arg("run")
            .arg(&source);
        let interpreted = run_command(interpreter, RUN_TIMEOUT, "Bogle unit interpreter fixture");
        assert_success(
            &interpreted,
            &format!("interpreter run of {}", source.display()),
        );
        assert_clean_stderr(
            &interpreted,
            &format!("interpreter run of {}", source.display()),
        );
        assert_eq!(
            interpreted.stdout.bytes,
            expected,
            "interpreter output mismatch for {}",
            source.display()
        );

        #[cfg(feature = "llvm")]
        {
            let native = NativeExecutable::build(&source, false);
            let output = native.run(false);
            assert_success(&output, &format!("native run of {}", source.display()));
            assert_clean_stderr(&output, &format!("native run of {}", source.display()));
            assert_eq!(
                output.stdout.bytes,
                expected,
                "native output mismatch for {}",
                source.display()
            );
        }
    }
}
