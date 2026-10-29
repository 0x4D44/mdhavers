#![cfg(feature = "llvm")]
//! RC7 Change C: borrowed local slots must remain safe across mutation, calls, captures,
//! exits, and the R5 summary downgrade. Runtime cases run with checking and reporting.

use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::{Builder, TempDir};

const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

struct NativeProgram {
    _dir: TempDir,
    executable: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    output_too_large: bool,
}

fn compile_ir(source: &str) -> String {
    LLVMCompiler::new()
        .with_refcounting(true)
        .compile_to_ir(&parse(source).expect("parse test source"))
        .expect("compile source to RC-on LLVM IR")
}

#[test]
fn owning_loop_releases_preserve_inline_policy() {
    let ir = compile_ir(
        "dae sum(xs) {\nken s = 0\nken i = 0\nwhiles i < len(xs) {\ns = s + xs[i]\ni = i + 1\n}\ngie s\n}\nblether sum([1, 2, 3])\n",
    );
    let body = function_body(&ir, "__mdh_fn_sum");
    let releases: Vec<_> = body
        .lines()
        .filter_map(|line| {
            let (name, rhs) = line.trim().split_once(" = ")?;
            (name.starts_with("%rc_old") || name.starts_with("%rc_slot_done"))
                .then_some((name, rhs))
                .filter(|(_, rhs)| rhs.starts_with("load { i8, i64 }, ptr %s,"))
                .map(|(name, _)| name)
        })
        .collect();
    assert!(releases.len() >= 3, "initial, loop and epilogue s releases");
    for old in releases {
        assert!(
            !body.contains(&format!("@__mdh_release_fast({{ i8, i64 }} {old})")),
            "owning s release {old} lost its inline policy"
        );
        let field = |index| {
            body.lines()
                .find_map(|line| {
                    let (name, rhs) = line.trim().split_once(" = ")?;
                    (rhs == format!("extractvalue {{ i8, i64 }} {old}, {index}")).then_some(name)
                })
                .unwrap_or_else(|| panic!("inline s release {old} needs field {index}"))
        };
        let data = field(1);
        let tag = field(0);
        assert!(
            body.contains(&format!("@__mdh_rc_release_obj_fast(i64 {data})")),
            "inline s release {old} must release its own data"
        );
        let tag64 = body
            .lines()
            .find_map(|line| {
                let (name, rhs) = line.trim().split_once(" = ")?;
                (rhs == format!("zext i8 {tag} to i64")).then_some(name)
            })
            .expect("inline release extends its own tag");
        let shift = body
            .lines()
            .find_map(|line| {
                let (name, rhs) = line.trim().split_once(" = ")?;
                (rhs == format!("and i64 {tag64}, 63")).then_some(name)
            })
            .expect("inline release masks its own tag");
        assert!(
            body.contains(&format!("icmp ugt i64 {shift}, 3")),
            "owning s release {old} needs the threadable immediate guard"
        );
    }
}

fn compile_native(source: &str) -> NativeProgram {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join(".scratch");
    fs::create_dir_all(&scratch).expect("create worktree scratch directory");
    let dir = Builder::new()
        .prefix("rc7-borrowed-slots-")
        .tempdir_in(&scratch)
        .expect("create RC test tempdir under .scratch");
    let source_path = dir.path().join("main.braw");
    let executable = dir.path().join("rc7_borrowed_exe");
    let stdout_path = dir.path().join("stdout.log");
    let stderr_path = dir.path().join("stderr.log");
    fs::write(&source_path, source).expect("write native test source");
    LLVMCompiler::new()
        .with_refcounting(true)
        .compile_to_native_with_source(
            &parse(source).expect("parse native test source"),
            &executable,
            2,
            Some(&source_path),
        )
        .expect("compile native RC test at -O2");
    NativeProgram {
        _dir: dir,
        executable,
        stdout_path,
        stderr_path,
    }
}

fn run(program: &NativeProgram, nofree: bool) -> Run {
    let stdout = File::create(&program.stdout_path).expect("create capped stdout capture");
    let stderr = File::create(&program.stderr_path).expect("create capped stderr capture");
    let mut command = Command::new("deltic");
    command
        .arg("timeout")
        .arg("30")
        .arg(&program.executable)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .env("MDH_RC", "1")
        .env("MDH_RC_REPORT", "1")
        .env_remove("MDH_RC_NOFREE")
        .env_remove("MDH_RC_KINDS")
        .env_remove("MDH_RC_FREE_KINDS");
    if nofree {
        command.env("MDH_RC_NOFREE", "1");
    }
    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt;

        command.pre_exec(|| {
            let limit = libc::rlimit {
                rlim_cur: (MAX_OUTPUT_BYTES + 1) as libc::rlim_t,
                rlim_max: (MAX_OUTPUT_BYTES + 1) as libc::rlim_t,
            };
            if libc::setrlimit(libc::RLIMIT_FSIZE, &limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let no_core = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            if libc::setrlimit(libc::RLIMIT_CORE, &no_core) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let status = command
        .status()
        .expect("run native RC test with deltic timeout");
    let stdout = fs::read(&program.stdout_path).expect("read capped stdout capture");
    let stderr = fs::read(&program.stderr_path).expect("read capped stderr capture");
    let output_too_large = stdout.len().saturating_add(stderr.len()) > MAX_OUTPUT_BYTES;
    Run {
        code: status.code(),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        output_too_large,
    }
}

fn live_total(stderr: &str) -> i64 {
    let report = stderr
        .lines()
        .find(|line| line.starts_with("mdhavers rc: live at exit:"))
        .unwrap_or_else(|| panic!("missing RC report:\n{stderr}"));
    report
        .split(" total=")
        .nth(1)
        .and_then(|tail| tail.split_whitespace().next())
        .unwrap_or_else(|| panic!("missing total in RC report: {report}"))
        .parse()
        .expect("parse live total")
}

fn live_kind(stderr: &str, kind: &str) -> i64 {
    let report = stderr
        .lines()
        .find(|line| line.starts_with("mdhavers rc: live at exit:"))
        .unwrap_or_else(|| panic!("missing RC report:\n{stderr}"));
    let key = format!(" {kind}=");
    let start = report
        .find(&key)
        .unwrap_or_else(|| panic!("missing {kind} in RC report: {report}"))
        + key.len();
    report[start..]
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .expect("parse live kind")
}

fn assert_exact_run(source: &str, expected: &str) {
    assert_exact_run_with_total(source, expected, 0);
}

fn assert_exact_run_with_total(source: &str, expected: &str, total: i64) {
    for nofree in [false, true] {
        let program = compile_native(source);
        let result = run(&program, nofree);
        assert!(
            !result.output_too_large,
            "native RC test exceeded 1 MiB output (NOFREE={nofree})"
        );
        assert_eq!(result.code, Some(0), "NOFREE={nofree}: {}", result.stderr);
        assert_eq!(
            result.stdout, expected,
            "NOFREE={nofree}: {}",
            result.stderr
        );
        assert_eq!(
            live_total(&result.stderr),
            total,
            "NOFREE={nofree}: {}",
            result.stderr
        );
    }
}

/// Caught hurls leave exactly the thread's current error string in the runtime slot. Pin
/// that one accepted reference; the program's own objects must all be released.
fn assert_exact_run_with_one_caught_error(source: &str, expected: &str) {
    for nofree in [false, true] {
        let program = compile_native(source);
        let result = run(&program, nofree);
        assert!(
            !result.output_too_large,
            "native RC test exceeded 1 MiB output (NOFREE={nofree})"
        );
        assert_eq!(result.code, Some(0), "NOFREE={nofree}: {}", result.stderr);
        assert_eq!(
            result.stdout, expected,
            "NOFREE={nofree}: {}",
            result.stderr
        );
        assert_eq!(
            live_total(&result.stderr),
            1,
            "NOFREE={nofree}: {}",
            result.stderr
        );
        assert_eq!(live_kind(&result.stderr, "string"), 1, "{}", result.stderr);
    }
}

fn function_body<'a>(ir: &'a str, symbol: &str) -> &'a str {
    let marker = format!("@{symbol}(");
    let definition = ir
        .split("\ndefine ")
        .find(|part| {
            part.lines()
                .next()
                .is_some_and(|line| line.contains(&marker))
        })
        .unwrap_or_else(|| panic!("missing LLVM function {symbol}"));
    definition
        .split_once('{')
        .expect("LLVM function opening brace")
        .1
        .split_once("\n}")
        .expect("LLVM function closing brace")
        .0
}

fn llvm_value_tokens(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut values = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] != b'%' {
            at += 1;
            continue;
        }
        let start = at;
        at += 1;
        if at < bytes.len() && bytes[at] == b'"' {
            at += 1;
            while at < bytes.len() && bytes[at] != b'"' {
                at += 1;
            }
            if at < bytes.len() {
                at += 1;
            }
        } else {
            while at < bytes.len()
                && (bytes[at].is_ascii_alphanumeric()
                    || matches!(bytes[at], b'_' | b'.' | b'$' | b'-'))
            {
                at += 1;
            }
        }
        if at > start + 1 {
            values.push(text[start..at].to_string());
        }
    }
    values
}

fn llvm_name_matches_slot(value: &str, slot: &str) -> bool {
    let name = value.trim_start_matches('%').trim_matches('"');
    if name == slot {
        return true;
    }
    name.strip_prefix(slot).is_some_and(|suffix| {
        let suffix = suffix.strip_prefix('.').unwrap_or(suffix);
        !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit())
    })
}

/// Recover a local's alloca pointer from either its named alloca or a load whose result
/// preserves the source slot name. LLVM drops alloca names when the line-note pass rebuilds
/// the entry block, so a named slot load is the useful link back to its alloca.
fn slot_pointers(body: &str, slot: &str) -> HashSet<String> {
    let lines: Vec<_> = body.lines().map(str::trim).collect();
    let mut pointers = HashSet::new();

    for line in &lines {
        let Some((lhs, rhs)) = line.split_once(" = ") else {
            continue;
        };
        if (rhs.starts_with("alloca ") || rhs.contains(" = alloca "))
            && llvm_name_matches_slot(lhs, slot)
        {
            pointers.insert(lhs.to_string());
        }
    }

    for line in &lines {
        let Some((lhs, rhs)) = line.split_once(" = ") else {
            continue;
        };
        if llvm_name_matches_slot(lhs, slot) && rhs.starts_with("load ") {
            if let Some((_, pointer_and_attrs)) = rhs.rsplit_once(", ptr ") {
                if let Some(pointer) = llvm_value_tokens(pointer_and_attrs).first() {
                    pointers.insert(pointer.clone());
                }
            }
        }
    }

    assert!(
        !pointers.is_empty(),
        "no alloca or named slot load for local {slot} in:\n{body}"
    );
    pointers
}

/// Return RC calls whose argument is dataflow-derived from a load of `slot`. This checks
/// the slot's provenance rather than counting every retain/release in its function.
fn rc_calls_for_slot(body: &str, slot: &str) -> Vec<String> {
    let slot_ptrs = slot_pointers(body, slot);
    let mut instructions = Vec::new();
    let mut pending = String::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.contains(" = ") && trimmed.starts_with('%') {
            if !pending.is_empty() {
                instructions.push(std::mem::take(&mut pending));
            }
            pending.push_str(trimmed);
        } else if !pending.is_empty() && (trimmed.starts_with('[') || pending.contains(" phi ")) {
            pending.push(' ');
            pending.push_str(trimmed);
            if pending.matches('[').count() <= pending.matches(']').count() {
                instructions.push(std::mem::take(&mut pending));
            }
        } else if !pending.is_empty() {
            instructions.push(std::mem::take(&mut pending));
        }
    }
    if !pending.is_empty() {
        instructions.push(pending);
    }

    let mut derived = HashSet::new();
    for line in body.lines().map(str::trim) {
        if let Some((value, destination)) = line
            .strip_prefix("store ")
            .and_then(|store| store.rsplit_once(", ptr "))
        {
            if llvm_value_tokens(destination)
                .first()
                .is_some_and(|pointer| slot_ptrs.contains(pointer))
            {
                if let Some(source) = llvm_value_tokens(value).last() {
                    derived.insert(source.clone());
                }
            }
        }
        if let Some((lhs, rhs)) = line.split_once(" = ") {
            if rhs.starts_with("load ") {
                if let Some((_, pointer_and_attrs)) = rhs.rsplit_once(", ptr ") {
                    if llvm_value_tokens(pointer_and_attrs)
                        .first()
                        .is_some_and(|pointer| slot_ptrs.contains(pointer))
                    {
                        derived.insert(lhs.to_string());
                    }
                }
            }
        }
    }

    loop {
        let mut changed = false;
        for instruction in &instructions {
            let Some((lhs, rhs)) = instruction.split_once(" = ") else {
                continue;
            };
            if derived.contains(lhs) && (rhs.starts_with("phi ") || rhs.starts_with("select ")) {
                for incoming in llvm_value_tokens(rhs) {
                    changed |= derived.insert(incoming);
                }
            }
            // An explicit compiled +1 return owns its result independently of
            // the called alias. Do not flood that result's consumers.
            if rhs.starts_with("call ")
                && ["@__mdh_fn_", "@__lambda_", "@__mdh_late_call_"]
                    .iter()
                    .any(|prefix| rhs.contains(prefix))
            {
                continue;
            }
            if ![
                "extractvalue ",
                "insertvalue ",
                "phi ",
                "select ",
                "inttoptr ",
                "ptrtoint ",
                "bitcast ",
                "freeze ",
            ]
            .iter()
            .any(|prefix| rhs.starts_with(prefix))
            {
                continue;
            }
            let depends_on_slot = llvm_value_tokens(rhs)
                .iter()
                .any(|token| derived.contains(token));
            if depends_on_slot && derived.insert(lhs.to_string()) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    body.lines()
        .filter(|line| {
            let line = line.trim();
            let is_rc_call = line.contains("call ")
                && (line.contains("@__mdh_retain")
                    || line.contains("@__mdh_rc_retain")
                    || line.contains("@__mdh_release")
                    || line.contains("@__mdh_rc_release")
                    || line.contains("@__mdh_rc_borrow_release"));
            is_rc_call
                && llvm_value_tokens(line)
                    .iter()
                    .any(|token| derived.contains(token))
        })
        .map(|line| line.trim().to_string())
        .collect()
}

fn assert_slot_has_no_retain(body: &str, slot: &str) {
    let retains: Vec<_> = rc_calls_for_slot(body, slot)
        .into_iter()
        .filter(|call| call.contains("@__mdh_retain") || call.contains("@__mdh_rc_retain"))
        .collect();
    assert!(
        retains.is_empty(),
        "slot {slot} still has retains: {retains:#?}"
    );
}

fn assert_slot_has_no_rc(body: &str, slot: &str) {
    let calls = rc_calls_for_slot(body, slot);
    assert!(
        calls.is_empty(),
        "slot {slot} still has RC calls: {calls:#?}"
    );
}

fn assert_slot_is_owning(body: &str, slot: &str) {
    let retains: Vec<_> = rc_calls_for_slot(body, slot)
        .into_iter()
        .filter(|call| call.contains("@__mdh_retain") || call.contains("@__mdh_rc_retain"))
        .collect();
    assert!(
        !retains.is_empty(),
        "slot {slot} unexpectedly has no retain:\n{body}"
    );
}

fn slot_load_values(body: &str, slot: &str) -> HashSet<String> {
    let slot_ptrs = slot_pointers(body, slot);
    body.lines()
        .filter_map(|line| {
            let (lhs, rhs) = line.trim().split_once(" = ")?;
            if !rhs.starts_with("load ") {
                return None;
            }
            let (_, pointer_and_attrs) = rhs.rsplit_once(", ptr ")?;
            let pointer_tokens = llvm_value_tokens(pointer_and_attrs);
            let pointer = pointer_tokens.first()?;
            slot_ptrs.contains(pointer).then(|| lhs.to_string())
        })
        .collect()
}

fn slot_stored_values(body: &str, slot: &str) -> HashSet<String> {
    let slot_ptrs = slot_pointers(body, slot);
    body.lines()
        .filter_map(|line| {
            let (value, destination) = line.trim().strip_prefix("store ")?.rsplit_once(", ptr ")?;
            let pointer_tokens = llvm_value_tokens(destination);
            let pointer = pointer_tokens.first()?;
            if !slot_ptrs.contains(pointer) {
                return None;
            }
            llvm_value_tokens(value).last().cloned()
        })
        .collect()
}

fn is_materialized_retain(line: &str) -> bool {
    line.contains("@__mdh_retain")
        || line.contains("@__mdh_rc_retain(")
        || line.contains("@__mdh_rc_retain_obj")
}

fn retain_calls_for_values(body: &str, values: &HashSet<String>) -> Vec<String> {
    body.lines()
        .filter(|line| {
            let line = line.trim();
            is_materialized_retain(line)
                && llvm_value_tokens(line)
                    .iter()
                    .any(|token| values.contains(token))
        })
        .map(|line| line.trim().to_string())
        .collect()
}

/// Follow only aggregate/pointer representations of these exact values. Phi inputs
/// matter because an initializer can retain on its incoming edges. A call result owns
/// its own reference and must never become provenance of its callee or arguments.
fn representation_values(body: &str, mut values: HashSet<String>) -> HashSet<String> {
    loop {
        let mut changed = false;
        for line in body.lines() {
            let Some((lhs, rhs)) = line.trim().split_once(" = ") else {
                continue;
            };
            let tokens = llvm_value_tokens(rhs);
            if values.contains(lhs) && (rhs.starts_with("phi ") || rhs.starts_with("select ")) {
                for token in &tokens {
                    changed |= values.insert(token.clone());
                }
            }
            if [
                "extractvalue ",
                "insertvalue ",
                "phi ",
                "select ",
                "inttoptr ",
                "ptrtoint ",
                "bitcast ",
                "freeze ",
            ]
            .iter()
            .any(|prefix| rhs.starts_with(prefix))
                && tokens.iter().any(|token| values.contains(token))
            {
                changed |= values.insert(lhs.to_string());
            }
        }
        if !changed {
            return values;
        }
    }
}

fn initializer_retains(body: &str, slot: &str) -> Vec<String> {
    let stored = slot_stored_values(body, slot);
    assert!(
        !stored.is_empty(),
        "slot {slot} has no initialized values in:\n{body}"
    );
    retain_calls_for_values(body, &representation_values(body, stored))
}

fn assert_slot_initializer_is_borrowed(body: &str, slot: &str) {
    let retains = initializer_retains(body, slot);
    assert!(
        retains.is_empty(),
        "slot {slot} initializer is still retained: {retains:#?}"
    );
}

fn assert_slot_initializer_is_owning(body: &str, slot: &str) {
    let retains = initializer_retains(body, slot);
    assert!(
        !retains.is_empty(),
        "slot {slot} initializer unexpectedly has no retain:\n{body}"
    );
}

fn slot_value_stores(body: &str, slot: &str) -> Vec<String> {
    let slot_ptrs = slot_pointers(body, slot);
    let loads = slot_load_values(body, slot);
    body.lines()
        .filter_map(|line| {
            let line = line.trim();
            let (value, destination) = line.strip_prefix("store ")?.rsplit_once(", ptr ")?;
            let source_tokens = llvm_value_tokens(value);
            let target_tokens = llvm_value_tokens(destination);
            let source = source_tokens.last()?;
            let target = target_tokens.first()?;
            (loads.contains(source) && !slot_ptrs.contains(target)).then(|| line.to_string())
        })
        .collect()
}

fn assert_slot_has_no_park(body: &str, slot: &str) {
    let parks = slot_value_stores(body, slot);
    assert!(
        parks.is_empty(),
        "slot {slot} read is still parked outside its owner slot: {parks:#?}"
    );
}

fn assert_slot_has_park(body: &str, slot: &str) {
    let parks = slot_value_stores(body, slot);
    assert!(
        !parks.is_empty(),
        "slot {slot} read unexpectedly has no park:\n{body}"
    );
}

fn assert_slot_read_is_retained(body: &str, slot: &str) {
    let loads = slot_load_values(body, slot);
    let retains = retain_calls_for_values(body, &loads);
    assert!(
        !retains.is_empty(),
        "slot {slot} read unexpectedly has no materialized retain:\n{body}"
    );
}

fn late_dispatches(ir: &str) -> Vec<&str> {
    ir.split("\ndefine ")
        .filter(|part| {
            part.lines()
                .next()
                .is_some_and(|line| line.contains("@__mdh_late_call_"))
        })
        .collect()
}

#[test]
fn c1_replacing_a_root_element_keeps_the_borrowed_element_alive() {
    assert_exact_run(
        r#"
dae run() {
    ken xs = [["old"]]
    ken p = xs[0]
    xs[0] = ["new"]
    blether p[0]
}
run()
"#,
        "old\n",
    );
}

#[test]
fn c2_popping_a_root_element_keeps_the_borrowed_element_alive() {
    assert_exact_run(
        r#"
dae run() {
    ken xs = [["old"], ["next"]]
    ken p = xs[0]
    yank(xs)
    blether p[0]
}
run()
"#,
        "old\n",
    );
}

#[test]
fn c3_a_mutating_direct_call_keeps_the_borrowed_element_alive() {
    assert_exact_run(
        r#"
dae replace_first(xs) {
    xs[0] = ["new"]
}
dae run() {
    ken xs = [["old"]]
    ken p = xs[0]
    replace_first(xs)
    blether p[0]
}
run()
"#,
        "old\n",
    );
}

#[test]
fn c4_a_closure_that_mutates_the_root_keeps_the_borrowed_element_alive() {
    assert_exact_run(
        r#"
dae run() {
    ken xs = [["old"]]
    ken p = xs[0]
    ken replace = || { xs[0] = ["new"] }
    replace()
    blether p[0]
}
run()
"#,
        "old\n",
    );
}

#[test]
fn c5_replacing_the_root_keeps_the_borrowed_element_alive() {
    assert_exact_run(
        r#"
dae run() {
    ken xs = [["old"]]
    ken p = xs[0]
    xs = []
    blether p[0]
}
run()
"#,
        "old\n",
    );
}

#[test]
fn c6_a_temporary_root_does_not_make_the_local_slot_borrowable() {
    assert_exact_run(
        r#"
dae make() {
    gie [["fresh"]]
}
dae run() {
    ken p = make()[0]
    blether p[0]
}
run()
"#,
        "fresh\n",
    );
}

#[test]
fn c7_try_catch_keeps_the_slot_owning_for_handler_reads() {
    let source = r#"
dae run() {
    ken xs = [["old"]]
    ken p = xs[0]
    hae_a_bash {
        xs[0] = ["new"]
        hurl "jump"
    } gin_it_gangs_wrang e {
        blether p[0]
    }
}
run()
"#;
    let ir = compile_ir(source);
    assert_slot_is_owning(function_body(&ir, "__mdh_fn_run"), "p");
    assert_exact_run_with_one_caught_error(source, "old\n");
}

#[test]
fn c8_reassignment_releases_are_correct_on_return_break_and_continue() {
    assert_exact_run(
        r#"
dae via_return(xs) {
    ken p = xs[0]
    ken i = 0
    whiles i < len(xs) {
        p = xs[i]
        i = i + 1
        gin i == 2 { gie p }
    }
    gie p
}
dae via_break(xs) {
    ken p = xs[0]
    ken i = 0
    whiles i < len(xs) {
        p = xs[i]
        i = i + 1
        gin i == 2 { brak }
    }
    gie p
}
dae via_continue(xs) {
    ken p = xs[0]
    ken i = 0
    whiles i < len(xs) {
        p = xs[i]
        i = i + 1
        haud
    }
    gie p
}
ken xs = [["a"], ["b"], ["c"]]
ken a = via_return(xs)
ken b = via_break(xs)
ken c = via_continue(xs)
blether a[0]
blether b[0]
blether c[0]
"#,
        "b\nb\nc\n",
    );
}

#[test]
fn c9_a_borrowed_slot_cannot_be_a_root_for_another_borrowed_slot() {
    assert_exact_run(
        r#"
dae run() {
    ken xs = [[[1]]]
    ken a = xs[0]
    ken p = a[0]
    xs = []
    blether p[0]
}
run()
"#,
        "1\n",
    );
}

#[test]
fn c10_a_call_that_mutates_the_root_keeps_its_borrowed_argument_alive() {
    assert_exact_run(
        r#"
dae inspect_after_mutation(xs, p) {
    xs[0] = ["new"]
}
dae run() {
    ken xs = [["old"]]
    ken p = xs[0]
    inspect_after_mutation(xs, p)
    blether p[0]
}
run()
"#,
        "old\n",
    );
}

#[test]
fn c11_a_value_copied_or_captured_from_a_borrowed_slot_is_rooted() {
    assert_exact_run(
        r#"
dae through_slot() {
    ken xs = [["slot"]]
    ken p = xs[0]
    ken q = p
    xs = []
    gie q
}
dae through_capture() {
    ken xs = [["capture"]]
    ken p = xs[0]
    ken q = p
    ken read = || q[0]
    xs = []
    gie read()
}
ken a = through_slot()
blether a[0]
blether through_capture()
"#,
        "slot\ncapture\n",
    );
}

#[test]
fn c12_mutation_through_a_second_alias_keeps_the_borrowed_element_alive() {
    assert_exact_run(
        r#"
dae run() {
    ken xs = [["old"]]
    ken ys = xs
    ken p = xs[0]
    ys[0] = ["new"]
    blether p[0]
}
run()
"#,
        "old\n",
    );
}

#[test]
fn c13_mixed_roots_and_reads_live_across_slot_overwrites_stay_owning() {
    assert_exact_run(
        r#"
dae mixed(xs, choose_fresh) {
    ken p = xs[0]
    gin choose_fresh { p = ["fresh"] }
    xs[0] = ["changed"]
    gie p[0]
}
dae two_roots(left, right, choose_right) {
    ken p = left[0]
    gin choose_right { p = right[0] }
    left = []
    right = []
    gie p[0]
}
dae old_derived_value(xs) {
    ken p = xs[0]
    ken q = p[0]
    p = xs[1]
    xs[0] = ["changed"]
    gie q[0]
}
blether mixed([["old"]], nae)
blether mixed([["old"]], aye)
blether two_roots([["left"]], [["right"]], nae)
blether two_roots([["left"]], [["right"]], aye)
blether old_derived_value([[["old"]], [["new"]]])
"#,
        "old\nfresh\nleft\nright\nold\n",
    );
}

#[test]
fn c14_a_method_that_mutates_its_receiver_keeps_the_local_element_alive() {
    assert_exact_run(
        r#"
kin Basket {
    dae init(items) { masel.items = items }
    dae replace_first() { masel.items[0] = ["new"] }
    dae read_then_replace(i) {
        ken p = masel.items[i]
        masel.replace_first()
        gie p[0]
    }
}
ken basket = Basket([["old"]])
blether basket.read_then_replace(0)
"#,
        "old\n",
    );
}

#[test]
fn c15_the_protected_interval_starts_at_the_root_load_in_the_initializer() {
    assert_exact_run(
        r#"
dae run() {
    ken xs = [[1]]
    ken p = xs[(xs = [[0]])[0][0]]
    blether p[0]
}
run()
"#,
        "1\n",
    );
}

#[test]
fn closure_fn_borrows_the_closure_loaded_from_its_list() {
    let source = include_str!("../benchmarks/rc/closure_fn.braw").replace("r < 1000000", "r < 100");
    let ir = compile_ir(&source);
    let run_body = function_body(&ir, "__mdh_fn_run");
    assert_slot_initializer_is_borrowed(run_body, "f");
    assert_slot_has_no_retain(run_body, "f");
    assert_slot_has_no_park(run_body, "f");
    assert_called_alias_is_unparked(run_body, "f");
    assert_exact_run(&source, "5400\n");
}

fn assert_called_alias_is_unparked(body: &str, slot: &str) {
    let alias = body
        .lines()
        .filter_map(|line| line.trim().split_once(" = "))
        .find(|(lhs, rhs)| llvm_name_matches_slot(lhs, slot) && rhs.starts_with("load "))
        .expect("specific called alias load")
        .0;
    let parks: Vec<_> = body
        .lines()
        .filter(|line| {
            line.trim_start().starts_with("store ")
                && line
                    .split_once(", ptr ")
                    .is_some_and(|(value, _)| llvm_value_tokens(value).iter().any(|v| v == alias))
        })
        .collect();
    assert!(
        parks.is_empty(),
        "called alias {alias} still parked in its temp: {parks:#?}"
    );
}

#[test]
fn particle_sim_keeps_pj_owning_for_live_container_stores() {
    let source = include_str!("../benchmarks/rc/particle_sim.braw");
    let ir = compile_ir(source);
    let update = function_body(&ir, "__mdh_fn_update_velocities");
    // R5-8: C(d) rejects the generic concat stores. Tag-set dataflow or an audited
    // list-initialisation exception is follow-up; neither is part of this phase.
    assert_slot_is_owning(update, "pj");
    // Pin the benchmark's deterministic correctness section; scaling prints timings.
    assert_exact_run(source.split("# Scaling tests").next().unwrap(),
        "=== Particle Simulation Benchmark ===\nCorrectness check:\n  5 particles created\n  Particle 0: [0, 0, 0.1, 0.1, 1]\n  Initial energy: 0.15000000000000002\n  After 10 steps:\n    Final energy: 0.1500009927954386\n");
}

#[test]
fn a_terminal_error_callee_in_the_interval_still_allows_the_slot_borrow() {
    let source = r#"
dae maybe_hurl(stop) {
    gin stop { hurl "terminal" }
}
dae read(xs, stop) {
    ken p = xs[0]
    maybe_hurl(stop)
    blether len(p)
    gie 0
}
read([["safe"]], nae)
"#;
    let ir = compile_ir(source);
    let body = function_body(&ir, "__mdh_fn_read");
    assert_slot_initializer_is_borrowed(body, "p");
    // Dead epilogue loads are allowed; only ownership operations on p matter.
    assert_slot_has_no_rc(body, "p");
    assert_slot_has_no_park(body, "p");
    assert_exact_run(source, "1\n");
}

#[test]
fn c_slot_with_a_root_clearing_callee_stays_owning() {
    let source = r#"
dae clear_root(xs) {
    xs[0] = []
}
dae read(xs) {
    ken p = xs[0]
    clear_root(xs)
    gie p[0]
}
blether read([["safe"]])
"#;
    let ir = compile_ir(source);
    let read_body = function_body(&ir, "__mdh_fn_read");
    assert_slot_initializer_is_owning(read_body, "p");
    assert_exact_run(source, "safe\n");
}

#[test]
fn c_borrowed_slot_read_stays_retained_across_an_unrelated_local_release() {
    let source = r#"
dae read(xs) {
    ken spare = ["scratch"]
    ken p = xs[0]
    blether p[(spare = 0)]
}
read([["safe"]])
"#;
    let ir = compile_ir(source);
    let read_body = function_body(&ir, "__mdh_fn_read");
    assert_slot_initializer_is_borrowed(read_body, "p");
    assert_slot_read_is_retained(read_body, "p");
    assert_slot_has_park(read_body, "p");
    assert_exact_run(source, "safe\n");
}

#[test]
fn c_downgraded_reads_allow_normal_and_terminal_late_calls() {
    let source = r#"
dae adder(n) { gie |x| x + n }
ken functions = [adder(9), || 9]
dae run(functions, index) {
    ken s = 0
    ken r = 0
    whiles r < 1 {
        ken f = functions[index]
        s = s + f(1)
        r = r + 1
    }
    gie s
}
blether run(functions, 0)
hae_a_bash {
    run(functions, 1)
} gin_it_gangs_wrang e {
    blether "caught"
}
functions = []
"#;
    let ir = compile_ir(source);
    let run_body = function_body(&ir, "__mdh_fn_run");
    assert_slot_initializer_is_borrowed(run_body, "f");
    assert_slot_has_no_retain(run_body, "f");
    assert_slot_has_no_park(run_body, "f");
    assert_called_alias_is_unparked(run_body, "f");
    assert!(run_body.contains("@__mdh_late_call_1"));
    let arity_one = late_dispatches(&ir)
        .into_iter()
        .find(|body| {
            body.lines()
                .next()
                .is_some_and(|line| line.contains("@__mdh_late_call_1"))
        })
        .expect("one-argument late-call dispatcher");
    assert!(arity_one.contains("@__mdh_fn_adder"));
    assert!(arity_one.contains("@__mdh_hurl_text_at_line"));
    assert!(arity_one.contains("unreachable"));
    assert_exact_run_with_one_caught_error(source, "10\ncaught\n");
}

#[test]
fn c_returning_the_late_call_result_keeps_the_read_retained() {
    let source = r#"
dae identity(x) { gie x }
ken functions = [|| 9]
dae run(functions, unused) {
    ken f = functions[0]
    blether f()
    gie f(1)
}
"#;
    let ir = compile_ir(source);
    let run_body = function_body(&ir, "__mdh_fn_run");
    assert_slot_initializer_is_borrowed(run_body, "f");
    // R5 follows defined-call result provenance to the return, extending this
    // read's region through epilogue releases. C's terminal exemption does not
    // exempt these ordinary nonterminal drops. This is deliberately conservative.
    assert_slot_read_is_retained(run_body, "f");
    assert_slot_has_park(run_body, "f");
}

#[test]
fn an_ordinary_holder_none_indexed_callee_keeps_its_terminal_retain_and_park() {
    let source = r#"
dae identity(x) { gie x }
dae run(functions, unused) {
    blether functions[0](1)
}
"#;
    let ir = compile_ir(source);
    let body = function_body(&ir, "__mdh_fn_run");
    // Calling the element directly creates an ordinary holder-None record,
    // without a C candidate local and therefore without C downgrade authority.
    let call = body
        .lines()
        .find(|line| line.contains("call ") && line.contains("@__mdh_late_call_1("))
        .expect("indexed late callee");
    let args = call.split_once("@__mdh_late_call_1(").unwrap().1;
    let alias = llvm_value_tokens(args)
        .into_iter()
        .next()
        .expect("callee value");
    let aliases = representation_values(body, HashSet::from([alias.clone()]));
    assert!(
        !retain_calls_for_values(body, &aliases).is_empty(),
        "ordinary holder-None alias {alias} lost its retain:\n{body}"
    );
    assert!(
        body.lines().any(|line| {
            line.trim_start().starts_with("store ")
                && line
                    .split_once(", ptr ")
                    .is_some_and(|(value, _)| llvm_value_tokens(value).contains(&alias))
        }),
        "ordinary holder-None alias {alias} lost its park:\n{body}"
    );
}

#[test]
fn a_same_function_try_catch_keeps_the_slot_owning() {
    let source = r#"
dae guarded(xs) {
    ken p = xs[0]
    hae_a_bash {
        blether "inside"
    } gin_it_gangs_wrang e {
        blether e
    }
    gie p[0]
}
blether guarded([["kept"]])
"#;
    let ir = compile_ir(source);
    assert_slot_is_owning(function_body(&ir, "__mdh_fn_guarded"), "p");
    assert_exact_run(source, "inside\nkept\n");
}

#[test]
fn a_catch_before_the_root_still_keeps_the_slot_owning() {
    let source = r#"
dae guarded(xs) {
    hae_a_bash {
        blether "before"
    } gin_it_gangs_wrang e {
        blether e
    }
    ken p = xs[0]
    gie len(p)
}
blether guarded([["kept"]])
"#;
    let ir = compile_ir(source);
    let body = function_body(&ir, "__mdh_fn_guarded");
    // C(b) excludes the entire function, even when the catch lies before the
    // root load and cannot itself be an effect in p's protected interval.
    assert_slot_initializer_is_owning(body, "p");
    assert_exact_run(source, "before\n1\n");
}

#[test]
fn no_native_object_removes_the_dispatch_arm_but_log_span_keeps_it() {
    let no_native = r#"
ken f = |x| x + 1
blether f(2)
"#;
    let no_native_ir = compile_ir(no_native);
    assert!(
        !late_dispatches(&no_native_ir)
            .iter()
            .any(|body| body.contains("@__mdh_native_call1")),
        "native-call fallback remains without any native-object constructor"
    );
    assert_exact_run(no_native, "3\n");

    let with_span = r#"
fetch "tri" tae tri
ken f = |x| x + 1
ken span = log_span("seed")
blether f(2)
ken object = tri.Thing3D()
blether object.type
span = naething
object = naething
f = naething
"#;
    let span_ir = compile_ir(with_span);
    assert!(
        late_dispatches(&span_ir)
            .iter()
            .any(|body| body.contains("@__mdh_native_call1")),
        "log_span's possible native object must preserve the dynamic-call arm"
    );
    // The tri runtime's process cache also survives in pristine trunk (29 objects).
    assert_exact_run_with_total(with_span, "3\nThing3D\n", 29);
}

#[test]
fn owning_reads_of_ineligible_slots_do_not_add_error_path_leaks() {
    // C(b) rejects arr. Materialisation for a possible C slot must not add an
    // owning read reference to either terminal path when the slot stays owning.
    let source = include_str!("golden/edge_cases/index_set_non_int.braw");
    assert_exact_run_with_one_caught_error(source, "list_str_set\nlist_bool_set\ndone\n");
}

#[test]
fn owning_reads_in_outer_scopes_still_materialize_for_a_later_write() {
    let source = r#"
dae run() {
    ken xs = [[1]]
    ken p = xs[{
        ken index = 0
        xs = [[0]]
        index
    }]
    blether p[0]
}
run()
"#;
    assert_exact_run(source, "1\n");
}

#[test]
fn dead_dispatch_edges_do_not_add_caught_error_leaks() {
    let source = r#"
dae grumble(msg) { hurl "~" + msg }
dae command() { gie {"verb": "dan" + "ce"} }
kin Box {
    dae init() { masel.words = {"go": 1} }
    dae check() {
        ken cmd = command()
        hae_a_bash {
            gin nae contains(masel.words, cmd["verb"]) { grumble("expected") }
        } gin_it_gangs_wrang e { blether "caught" }
    }
}
Box().check()
"#;
    // Pristine trunk leaves two strings on this accepted error path.
    assert_exact_run_with_total(source, "caught\n", 2);
}
