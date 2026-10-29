#![cfg(feature = "llvm")]

//! MDH-BUG-FLU-00218: allocas emitted inside loop blocks accumulate on the native stack.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use mdhavers::{parse, LLVMCompiler};
use tempfile::Builder;

const ITERATIONS: u32 = 2_000_000;
const MAX_OUTPUT_BYTES: u64 = 1024 * 1024;
const RUN_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RcMode {
    Off,
    On,
}

impl RcMode {
    fn enabled(self) -> bool {
        matches!(self, Self::On)
    }

    fn env_value(self) -> &'static str {
        if self.enabled() {
            "1"
        } else {
            "0"
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Placement {
    TopLevel,
    Function,
}

struct NativeRun {
    pid: u32,
    code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
    output_too_large: bool,
}

fn scratch_dir() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let scratch = root.join(".scratch");
    fs::create_dir_all(&scratch).expect("create worktree .scratch directory");
    scratch
}

fn compile_and_run_loop(call: &str, placement: Placement, mode: RcMode, opt_level: u8) {
    let source = match placement {
        Placement::TopLevel => format!(
            "ken i = 0\nken value = 0\nwhiles i < {ITERATIONS} {{\n    value = {call}\n    i = i + 1\n}}\nblether \"done\"\n"
        ),
        Placement::Function => format!(
            "dae run() {{\n    ken i = 0\n    ken value = 0\n    whiles i < {ITERATIONS} {{\n        value = {call}\n        i = i + 1\n    }}\n}}\nrun()\nblether \"done\"\n"
        ),
    };
    let description = format!("placement={placement:?}, mode={mode:?}, call={call}");
    compile_and_run_source(&source, "done\n", mode, opt_level, &description);
}

fn compile_and_run_source(
    source: &str,
    expected: &str,
    mode: RcMode,
    opt_level: u8,
    description: &str,
) {
    let dir = Builder::new()
        .prefix("llvm-stack-alloca-")
        .tempdir_in(scratch_dir())
        .expect("create native test tempdir under .scratch");
    let source_path = dir.path().join("main.braw");
    fs::write(&source_path, source).expect("write stress source");
    let program = parse(source)
        .unwrap_or_else(|error| panic!("parse stress source ({description}): {error:?}\n{source}"));
    let executable = dir.path().join("stack_stress");
    LLVMCompiler::new()
        .with_refcounting(mode.enabled())
        .compile_to_native_with_source(&program, &executable, opt_level, Some(&source_path))
        .unwrap_or_else(|error| {
            panic!("compile stress source ({description}): {error:?}\n{source}")
        });

    let stdout_path = dir.path().join("stdout.log");
    let stderr_path = dir.path().join("stderr.log");
    let mut child = Command::new(&executable)
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("create stdout log"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("create stderr log"),
        ))
        .env("MDH_RC", mode.env_value())
        .env_remove("MDH_RC_NOFREE")
        .env_remove("MDH_RC_REPORT")
        .env_remove("MDH_RC_KINDS")
        .env_remove("MDH_RC_FREE_KINDS")
        .spawn()
        .expect("spawn native stress program");

    let pid = child.id();
    let started = Instant::now();
    let mut timed_out = false;
    let mut output_too_large = false;
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll native stress program") {
            break status;
        }
        let output_bytes = fs::metadata(&stdout_path).map_or(0, |metadata| metadata.len())
            + fs::metadata(&stderr_path).map_or(0, |metadata| metadata.len());
        if output_bytes > MAX_OUTPUT_BYTES {
            output_too_large = true;
            break terminate_child(&mut child, "output-limit");
        }
        if started.elapsed() >= RUN_TIMEOUT {
            timed_out = true;
            break terminate_child(&mut child, "timeout");
        }
        thread::sleep(Duration::from_millis(5));
    };

    let stdout = fs::read(&stdout_path).unwrap_or_default();
    let stderr = fs::read(&stderr_path).unwrap_or_default();
    output_too_large |= (stdout.len() + stderr.len()) as u64 > MAX_OUTPUT_BYTES;
    let run = NativeRun {
        pid,
        code: status.code(),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
        output_too_large,
    };

    assert!(
        !run.timed_out,
        "native loop exceeded 30 seconds; child pid={} ({description})\n{}",
        run.pid, run.stderr
    );
    assert!(
        !run.output_too_large,
        "native loop exceeded 1 MiB output ({description})"
    );
    assert_eq!(
        run.code,
        Some(0),
        "native loop failed ({description})\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(
        run.stdout, expected,
        "wrong output ({description})\nstderr:\n{}",
        run.stderr
    );
}

fn terminate_child(child: &mut Child, reason: &str) -> ExitStatus {
    let pid = child.id();
    let kill_error = child.kill().err();
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Ok(None) => panic!(
                "could not terminate {reason} child pid={pid} within 1 second; kill error: {kill_error:?}"
            ),
            Err(poll_error) => panic!(
                "could not reap {reason} child pid={pid}; kill error: {kill_error:?}; poll error: {poll_error}"
            ),
        }
    }
}

macro_rules! define_loop_cases {
    ($(($off_top:ident, $on_top:ident, $off_fn:ident, $on_fn:ident, $call:literal)),+ $(,)?) => {
        $(
            #[test]
            fn $off_top() {
                compile_and_run_loop($call, Placement::TopLevel, RcMode::Off, 2);
            }

            #[test]
            fn $on_top() {
                compile_and_run_loop($call, Placement::TopLevel, RcMode::On, 2);
            }

            #[test]
            fn $off_fn() {
                compile_and_run_loop($call, Placement::Function, RcMode::Off, 2);
            }

            #[test]
            fn $on_fn() {
                compile_and_run_loop($call, Placement::Function, RcMode::On, 2);
            }
        )+
    };
}

define_loop_cases!(
    (
        snooze_toplevel_rc_off,
        snooze_toplevel_rc_on,
        snooze_function_rc_off,
        snooze_function_rc_on,
        "snooze(0)"
    ),
    (
        noo_toplevel_rc_off,
        noo_toplevel_rc_on,
        noo_function_rc_off,
        noo_function_rc_on,
        "noo()"
    ),
    (
        tick_toplevel_rc_off,
        tick_toplevel_rc_on,
        tick_function_rc_off,
        tick_function_rc_on,
        "tick()"
    ),
    (
        bide_int_toplevel_rc_off,
        bide_int_toplevel_rc_on,
        bide_int_function_rc_off,
        bide_int_function_rc_on,
        "bide(0)"
    ),
    (
        bide_float_toplevel_rc_off,
        bide_float_toplevel_rc_on,
        bide_float_function_rc_off,
        bide_float_function_rc_on,
        "bide(0.0)"
    ),
    (
        sleep_toplevel_rc_off,
        sleep_toplevel_rc_on,
        sleep_function_rc_off,
        sleep_function_rc_on,
        "sleep(0)"
    ),
);

const FER_LOOP_BODY: &str = r#"
ken i = 0
ken total = 0
whiles i < 2000000 {
    fer n in 0..1 {
        fer ch in "x" {
            fer value in [1] { }
        }
    }
    fer j in 0..2 { total = total + j }
    i = i + 1
}
"#;

fn fer_stress_source(placement: Placement) -> String {
    match placement {
        Placement::TopLevel => format!("{FER_LOOP_BODY}\nblether total\n"),
        Placement::Function => {
            format!("dae run() {{\n{FER_LOOP_BODY}\n    gie total\n}}\nblether run()\n")
        }
    }
}

#[test]
fn nested_fer_toplevel_allocas_stay_bounded_rc_off() {
    compile_and_run_source(
        &fer_stress_source(Placement::TopLevel),
        "2000000\n",
        RcMode::Off,
        0,
        "top-level nested fer stress",
    );
}

#[test]
fn nested_fer_toplevel_allocas_stay_bounded_rc_on() {
    compile_and_run_source(
        &fer_stress_source(Placement::TopLevel),
        "2000000\n",
        RcMode::On,
        0,
        "top-level nested fer stress",
    );
}

#[test]
fn nested_fer_function_allocas_stay_bounded_rc_off() {
    compile_and_run_source(
        &fer_stress_source(Placement::Function),
        "2000000\n",
        RcMode::Off,
        0,
        "function nested fer stress",
    );
}

#[test]
fn nested_fer_function_allocas_stay_bounded_rc_on() {
    compile_and_run_source(
        &fer_stress_source(Placement::Function),
        "2000000\n",
        RcMode::On,
        0,
        "function nested fer stress",
    );
}

const ALLOCA_INVENTORY_SOURCE: &str = r#"
dae add(a, b) { gie a + b }
dae positive(x) { gie x > 0 }
dae consume(x) { gie x }

kin Box {
    dae init(v) { masel.v = v }
    dae bump() {
        masel.v = masel.v + 1
        gie masel.v
    }
}

dae inventory() {
    ken xs = [0, 1, 2, 3]
    ken text = "a::b,,c"
    ken dict = {"a": 1, "b": 2}
    ken total = 0

    whiles total < 1 {
    blether tail(xs)
    blether tail(text)
    blether scran(xs, 0, 2)
    blether scran(text, 0, 2)
    blether sample(xs, 1)
    blether slap(xs, [4])
    blether slap(text, "!")
    blether zipwith(add, xs, xs)
    blether reverse(xs)
    blether reverse(text)
    blether sumaw(xs)
    blether wheesht(xs)
    blether wheesht(text)
    blether product(xs)
    blether upper(text)
    blether lower(text)
    blether is_upper("A")
    blether is_lower("a")
    blether is_alpha("a")
    blether is_alnum("1")
    blether is_digit("1")
    blether coont(text, "b")
    text = text + "!"
    blether bit_count(7)
    blether is_number("12")
    blether is_number("3.5")
    blether is_prime(7)
    blether gcd(12, 8)
    blether lcm(3, 4)
    blether zip(xs, xs)
    blether replicate(1, 3)
    blether avg(xs)
    blether factorial(5)
    blether repeat("x", 2)

    ken parts_empty = split("a,,b", ",")
    ken parts_single = split("abc", "")
    ken parts_multi = split(text, "::")
    blether join(parts_empty, "-")
    blether join(parts_single, "-")
    blether join(parts_multi, "-")

    blether gaun(xs, |x| x + 1)
    each(xs, |x| x)
    blether sieve(xs, |x| x > 1)
    blether pairt_by(xs, |x| x > 1)
    blether grup_up(xs, |x| x % 2)
    blether tumble(xs, 0, |acc, x| acc + x)
    blether aw(xs, |x| x >= 0)
    blether aw(xs)
    blether ony(xs, |x| x > 2)
    blether ony(xs)
    blether hunt(xs, |x| x > 2)
    blether find(xs, positive)
    blether find_index(xs, |x| x == 2)
    blether ilk(xs, |x| x + 1)
    blether ilkane(xs, |item, index| item + index)
    blether ilkane(dict, |key, value| value)
    blether fin(xs, |x| x > 2)
    blether find_first(xs, |x| x > 2)

    ken spread_list = [...xs, 4]
    ken spread_string = [...text]
    fer n in 0..2 { total = total + n }
    fer ch in text { total = total + len(chars(ch)) }
    fer item in xs { total = total + item }
    whiles total < 0 { total = total + 1 }

    ken now = noo()
    now = tick()
    snooze(0)
    bide(0)
    bide(0.0)
    sleep(0)
    total = total + 1
    }
    gie total
}

ken box = Box(1)
ken field = box.v
box.v = field + 1
blether box.bump()
blether inventory()
"#;

#[derive(Debug, Eq, PartialEq)]
struct AllocaSite {
    function: String,
    block: String,
    instruction: String,
}

fn function_name(header: &str) -> String {
    let symbol = header
        .split_once('@')
        .map(|(_, after)| after)
        .unwrap_or("<unknown>");
    let name = symbol.split(['(', ' ']).next().unwrap_or("<unknown>");
    name.trim_matches('"').to_string()
}

fn is_block_label(line: &str) -> Option<&str> {
    if line.starts_with(char::is_whitespace) {
        return None;
    }
    let (label, _) = line.trim().split_once(':')?;
    (!label.is_empty() && !label.contains(' ')).then_some(label)
}

fn is_alloca_instruction(line: &str) -> bool {
    let rhs = line
        .split_once('=')
        .map_or(line.trim(), |(_, rhs)| rhs.trim());
    rhs == "alloca" || rhs.starts_with("alloca ")
}

fn non_entry_allocas(ir: &str) -> Vec<AllocaSite> {
    let mut function: Option<String> = None;
    let mut entry_block: Option<String> = None;
    let mut current_block: Option<String> = None;
    let mut violations = Vec::new();

    for line in ir.lines() {
        let trimmed = line.trim();
        if function.is_none() {
            if trimmed.starts_with("define ") {
                function = Some(function_name(trimmed));
                entry_block = None;
                current_block = None;
            }
            continue;
        }

        if trimmed == "}" {
            function = None;
            entry_block = None;
            current_block = None;
            continue;
        }

        if let Some(label) = is_block_label(line) {
            current_block = Some(label.to_string());
            if entry_block.is_none() {
                entry_block = Some(label.to_string());
            }
            continue;
        }

        if is_alloca_instruction(trimmed) {
            let block = current_block
                .clone()
                .unwrap_or_else(|| "<entry-before-label>".to_string());
            if entry_block.as_ref().is_some_and(|entry| entry != &block) {
                violations.push(AllocaSite {
                    function: function.clone().unwrap_or_else(|| "<unknown>".to_string()),
                    block,
                    instruction: trimmed.to_string(),
                });
            }
        }
    }
    violations
}

fn assert_all_allocas_in_entry(mode: RcMode) {
    let program = parse(ALLOCA_INVENTORY_SOURCE)
        .unwrap_or_else(|error| panic!("parse alloca inventory source: {error:?}"));
    let ir = LLVMCompiler::new()
        .with_refcounting(mode.enabled())
        .compile_to_ir(&program)
        .unwrap_or_else(|error| {
            panic!("compile alloca inventory source to unoptimized IR: {error:?}")
        });
    let violations = non_entry_allocas(&ir);
    assert!(
        violations.is_empty(),
        "found allocas outside their function entry blocks with {mode:?}:\n{}",
        violations
            .iter()
            .map(|site| format!(
                "  function @{}, block {}: {}",
                site.function, site.block, site.instruction
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn every_emitted_alloca_is_in_its_function_entry_block_rc_off() {
    assert_all_allocas_in_entry(RcMode::Off);
}

#[test]
fn every_emitted_alloca_is_in_its_function_entry_block_rc_on() {
    assert_all_allocas_in_entry(RcMode::On);
}
