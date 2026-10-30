//! Headless Bogle replay hashes must match the interpreter and both LLVM RC modes.

#![cfg(all(feature = "cli", feature = "llvm"))]

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const OUTPUT_LIMIT_PER_STREAM: usize = 512 * 1024;
const BUILD_TIMEOUT: Duration = Duration::from_secs(120);
// Debug-build interpreter runs on a loaded shared box take over a minute.
const RUN_TIMEOUT: Duration = Duration::from_secs(300);

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

struct NativeExecutable(PathBuf);

impl NativeExecutable {
    fn build(source: &Path, rc: bool) -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        use std::sync::atomic::Ordering;

        let repo = root();
        let scratch = repo.join(".scratch");
        std::fs::create_dir_all(&scratch).expect("create .scratch directory");
        let executable = scratch.join(format!(
            "bogle-parity-{}-{}{}",
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
            .arg("-O0")
            .arg(source)
            .arg("--output")
            .arg(&native.0);
        let output = run_command(command, BUILD_TIMEOUT, "Bogle parity native build");
        assert_success(&output, &format!("native build of {}", source.display()));
        native
    }

    fn run(&self, rc: bool, repo: &Path, map: &Path, replay: &Path) -> CapturedOutput {
        let mut command = Command::new(&self.0);
        configure_headless(&mut command, rc, repo, map, replay);
        run_command(command, RUN_TIMEOUT, "Bogle parity native replay")
    }

    fn run_positional_with_invalid_dimension_env(
        &self,
        repo: &Path,
        map: &Path,
        replay: &Path,
    ) -> CapturedOutput {
        let mut command = Command::new(&self.0);
        configure_headless(&mut command, false, repo, map, replay);
        command
            .env("BOGLE_W", "oops")
            .env("BOGLE_H", "oops")
            .env("BOGLE_EVERY", "oops")
            .arg(map)
            .arg(replay)
            .args(["80", "50", "35"]);
        run_command(
            command,
            RUN_TIMEOUT,
            "Bogle parity positional replay over invalid dimension env",
        )
    }
}

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

fn configure_headless(command: &mut Command, rc: bool, repo: &Path, map: &Path, replay: &Path) {
    command
        .current_dir(repo)
        .env("MDHAVERS_HOME", repo)
        .env("MDH_RC", if rc { "1" } else { "0" })
        .env("BOGLE_MAP", map)
        .env(
            "BOGLE_FRONTEND",
            if replay.file_stem() == Some(std::ffi::OsStr::new("frontend")) {
                "1"
            } else {
                "0"
            },
        )
        .env("BOGLE_REPLAY", replay)
        .env("BOGLE_W", "80")
        .env("BOGLE_H", "50")
        .env("BOGLE_EVERY", "35");
}

fn check_replay(map_name: &str, replay_name: &str, rows: usize, check_positional: bool) {
    let repo = root();
    let script = repo.join("games/bogle/shell/headless.braw");
    let map = repo.join(format!("games/bogle/data/{map_name}.map"));
    let replay = repo.join(format!("games/bogle/replays/{replay_name}.txt"));
    let expected_path = repo.join(format!("games/bogle/replays/{replay_name}.hashes"));
    for path in [&script, &map, &replay, &expected_path] {
        assert!(path.is_file(), "missing parity input {}", path.display());
    }
    let expected = std::fs::read(&expected_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", expected_path.display()));
    let expected_text = std::str::from_utf8(&expected)
        .unwrap_or_else(|error| panic!("{} is not UTF-8: {error}", expected_path.display()));
    assert_eq!(
        expected_text.lines().count(),
        rows,
        "{replay_name}.hashes must contain one row for each sampled frame"
    );

    let mut interpreter = Command::new(env!("CARGO_BIN_EXE_mdhavers"));
    configure_headless(&mut interpreter, false, &repo, &map, &replay);
    interpreter.arg("run").arg(&script);
    let interpreted = run_command(interpreter, RUN_TIMEOUT, "Bogle parity interpreter replay");
    assert_success(&interpreted, "interpreter replay");
    assert_clean_stderr(&interpreted, "interpreter replay");

    let mut native_outputs = Vec::new();
    let mut positional_output = None;
    for rc in [false, true] {
        let native = NativeExecutable::build(&script, rc);
        let output = native.run(rc, &repo, &map, &replay);
        let mode = if rc { "MDH_RC=1" } else { "MDH_RC=0" };
        assert_success(&output, &format!("native replay with {mode}"));
        assert_clean_stderr(&output, &format!("native replay with {mode}"));
        assert_eq!(
            output.stdout.bytes, interpreted.stdout.bytes,
            "native hashes with {mode} differ from interpreter hashes"
        );
        if !rc && check_positional {
            let positional = native.run_positional_with_invalid_dimension_env(&repo, &map, &replay);
            assert_success(
                &positional,
                "native replay with positional dimensions over invalid BOGLE_* dimension env",
            );
            assert_clean_stderr(&positional, "native positional replay");
            assert_eq!(
                positional.stdout.bytes, interpreted.stdout.bytes,
                "positional replay arguments did not take precedence over invalid dimension env"
            );
            positional_output = Some(positional.stdout.bytes);
        }
        native_outputs.push((rc, output.stdout.bytes));
    }
    // Compare backends before consulting the golden, so a rendering change can
    // only be blessed after all three executions agree.
    assert_eq!(
        interpreted.stdout.bytes, expected,
        "interpreter hashes differ from committed {replay_name}.hashes"
    );
    for (rc, output) in native_outputs {
        assert_eq!(
            output,
            expected,
            "native hashes with MDH_RC={} differ from committed {replay_name}.hashes",
            u8::from(rc)
        );
    }
    if let Some(output) = positional_output {
        assert_eq!(
            output, expected,
            "positional replay arguments did not take precedence over invalid dimension env"
        );
    }
}

#[test]
fn headless_replay_hashes_match_interpreter_and_native_rc_modes() {
    check_replay("test", "stage2", 10, true);
}

#[test]
fn textured_room_replay_matches_interpreter_and_native_rc_modes() {
    check_replay("stage3", "stage3", 2, false);
}

#[test]
fn frontend_frames_match_interpreter_and_native_rc_modes() {
    check_replay("e1m1", "frontend", 30, false);
}

// A full level route runs natively twice. The debug interpreter replays the
// first 140 tics to bound its cost; every native output must extend it.
fn check_playthrough(level: &str, rows: usize) {
    let repo = root();
    let script = repo.join("games/bogle/shell/headless.braw");
    let map = repo.join(format!("games/bogle/data/{level}.map"));
    let replay = repo.join(format!("games/bogle/replays/{level}.txt"));
    let expected = std::fs::read(repo.join(format!("games/bogle/replays/{level}.hashes")))
        .unwrap_or_else(|error| panic!("read full {level} frame hashes: {error}"));
    assert_eq!(
        expected.iter().filter(|byte| **byte == b'\n').count(),
        rows,
        "{level}.hashes must contain one row for each sampled frame"
    );
    let mut interpreter = Command::new(env!("CARGO_BIN_EXE_mdhavers"));
    configure_headless(&mut interpreter, false, &repo, &map, &replay);
    interpreter
        .env("BOGLE_EVERY", "140")
        .env("BOGLE_LIMIT", "140")
        .arg("run")
        .arg(&script);
    let prefix = run_command(interpreter, RUN_TIMEOUT, "level interpreter prefix");
    assert_success(&prefix, &format!("{level} interpreter prefix"));
    assert_clean_stderr(&prefix, &format!("{level} interpreter prefix"));
    assert_eq!(
        prefix
            .stdout
            .bytes
            .iter()
            .filter(|byte| **byte == b'\n')
            .count(),
        1
    );
    let mut outputs = Vec::new();
    for rc in [false, true] {
        let native = NativeExecutable::build(&script, rc);
        let mut command = Command::new(&native.0);
        configure_headless(&mut command, rc, &repo, &map, &replay);
        // Headless asserts exit reached alive, both keys collected and all four
        // weapons owned. The last hash includes the completed-level overlay.
        command
            .env("BOGLE_EVERY", "140")
            .env("BOGLE_REQUIRE_EXIT", "1");
        let output = run_command(command, RUN_TIMEOUT, "complete native playthrough");
        assert_success(&output, &format!("{level} complete native playthrough"));
        assert_clean_stderr(&output, &format!("{level} complete native playthrough"));
        assert!(
            output.stdout.bytes.starts_with(&prefix.stdout.bytes),
            "{level} MDH_RC={} diverged from interpreter prefix",
            u8::from(rc)
        );
        outputs.push(output.stdout.bytes);
    }
    assert_eq!(outputs[0], outputs[1], "{level} RC modes diverged");
    assert_eq!(
        outputs[0], expected,
        "{level} differs from committed frame hashes"
    );
}

#[test]
fn e1m1_playthrough_matches_native_rc_modes_and_interpreter_prefix() {
    check_playthrough("e1m1", 5);
}

// E1M2 exercises lifts, a switch, secret doors and a monster closet.
#[test]
fn e1m2_playthrough_matches_native_rc_modes_and_interpreter_prefix() {
    check_playthrough("e1m2", 38);
}
