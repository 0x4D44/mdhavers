#![cfg(feature = "llvm")]

/// Round 7 Change B: eligible entry-module globals become main locals with RC on or off.
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

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
    output_too_large: bool,
}

#[derive(Clone, Copy, Debug)]
enum RcMode {
    Off,
    On,
}

/// Creates independently selectable tests for each compiler/runtime RC mode.
macro_rules! rc_mode_test {
    ($off_test:ident, $on_test:ident, $case:ident) => {
        #[test]
        fn $off_test() {
            $case(RcMode::Off);
        }

        #[test]
        fn $on_test() {
            $case(RcMode::On);
        }
    };
}

/// Bounds the child at 30 seconds and 1 MiB combined output, and keeps stdin closed.
/// Reader threads continue draining after the cap so a noisy child cannot block on a
/// full pipe before the parent kills it. RC-related environment is set explicitly for
/// each mode so the caller's environment cannot select or alter the test mode.
fn run_bounded(executable: &Path, mode: RcMode) -> Run {
    let mut command = Command::new(executable);
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
    let mut child = command.spawn().expect("spawn native test program");

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
    Run {
        code: status.code(),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
        output_too_large: too_large.load(Ordering::Relaxed),
    }
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

fn run_native(source: &str, modules: &[(&str, &str)], mode: RcMode) -> Run {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join(".scratch");
    fs::create_dir_all(&scratch).expect("create worktree scratch directory");
    let dir = Builder::new()
        .prefix("rc-toplevel-")
        .tempdir_in(&scratch)
        .expect("create entry-relative tempdir");
    for (name, contents) in modules {
        fs::write(dir.path().join(name), contents).expect("write module source");
    }
    let main_path = dir.path().join("main.braw");
    fs::write(&main_path, source).expect("write entry source");
    let program = parse(source).expect("parse entry source");
    let executable = dir.path().join("rc_toplevel_exe");

    LLVMCompiler::new()
        .with_refcounting(matches!(mode, RcMode::On))
        .compile_to_native_with_source(&program, &executable, 2, Some(&main_path))
        .expect("compile native program at -O2");
    let run = run_bounded(&executable, mode);

    // TempDir removes the executable and generated object/source files beside it here.
    drop(dir);
    run
}

fn assert_exact_run(source: &str, modules: &[(&str, &str)], expected: &str, mode: RcMode) {
    let run = run_native(source, modules, mode);
    assert!(
        !run.timed_out,
        "native test program exceeded 30 seconds with {mode:?}"
    );
    assert!(
        !run.output_too_large,
        "native test program exceeded 1 MiB output with {mode:?}"
    );
    assert_eq!(run.code, Some(0), "mode: {mode:?}\n{}", run.stderr);
    assert_eq!(run.stdout, expected, "mode: {mode:?}\n{}", run.stderr);
    if matches!(mode, RcMode::On) {
        let report = run
            .stderr
            .lines()
            .find(|line| line.starts_with("mdhavers rc: live at exit:"))
            .unwrap_or_else(|| panic!("no RC report in stderr: {}", run.stderr));
        let total = report
            .split(" total=")
            .nth(1)
            .and_then(|tail| tail.split_whitespace().next())
            .unwrap_or_else(|| panic!("no total in RC report: {report}"));
        assert_eq!(total, "0", "{}", run.stderr);
    }
}

fn compile_ir(source: &str, mode: RcMode) -> String {
    let program = parse(source).expect("parse source for IR assertion");
    LLVMCompiler::new()
        .with_refcounting(matches!(mode, RcMode::On))
        .compile_to_ir(&program)
        .expect("compile source to IR")
}

fn assert_top_level_global(source: &str, name: &str, mode: RcMode) {
    let ir = compile_ir(source, mode);
    assert!(
        ir.contains(&format!("@__mdh_global_{name}")),
        "a top-level variable referenced outside main must remain global: {name}"
    );
}

fn function_body<'a>(ir: &'a str, name: &str) -> &'a str {
    let symbol = format!("@{name}(");
    let function = ir
        .split("\ndefine ")
        .find(|part| {
            part.lines()
                .next()
                .is_some_and(|line| line.contains(&symbol))
        })
        .unwrap_or_else(|| panic!("no function {name} in IR"));
    function
        .split_once('{')
        .expect("function opening brace")
        .1
        .split("\n}")
        .next()
        .expect("function body")
}

fn entry_alloca_pointers(body: &str) -> Vec<String> {
    let mut saw_block = false;
    let mut pointers = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.ends_with(':') {
            if saw_block {
                break;
            }
            saw_block = true;
            continue;
        }
        if saw_block {
            if let Some((pointer, _)) = line.split_once(" = alloca ") {
                pointers.push(pointer.to_string());
            }
        }
    }
    pointers
}

fn named_load_entry_alloca(body: &str, name_prefix: &str) -> Option<(String, String)> {
    let allocas = entry_alloca_pointers(body);
    body.lines().find_map(|line| {
        let Some((result, loaded)) = line.trim().split_once(" = load ") else {
            return None;
        };
        let Some(result_name) = result.strip_prefix('%') else {
            return None;
        };
        let Some(suffix) = result_name.strip_prefix(name_prefix) else {
            return None;
        };
        if !suffix.is_empty()
            && !suffix
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_digit() || ch == '_' || ch == '.')
        {
            return None;
        }
        let Some(pointer) = loaded.split("ptr %").nth(1).and_then(|tail| {
            tail.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .next()
        }) else {
            return None;
        };
        let pointer = format!("%{pointer}");
        allocas
            .iter()
            .any(|alloca| alloca == &pointer)
            .then(|| (result.to_string(), pointer))
    })
}

fn named_load_uses_entry_alloca(body: &str, name_prefix: &str) -> bool {
    named_load_entry_alloca(body, name_prefix).is_some()
}

fn entry_alloca_type<'a>(body: &'a str, pointer: &str) -> Option<&'a str> {
    body.lines().find_map(|line| {
        let (alloca_pointer, alloca_type) = line.trim().split_once(" = alloca ")?;
        (alloca_pointer == pointer)
            .then(|| alloca_type.split(", align").next().unwrap_or(alloca_type))
    })
}

fn list_value_entry_store_count(body: &str) -> usize {
    let allocas = entry_alloca_pointers(body);
    let Some(list_value) = body.lines().find_map(|line| {
        let (result, instruction) = line.trim().split_once(" = insertvalue { i8, i64 } ")?;
        instruction
            .starts_with("{ i8 5,")
            .then(|| result.to_string())
    }) else {
        return 0;
    };
    let mut stored_slots = Vec::new();
    for line in body.lines() {
        let Some((value, destination)) = line
            .trim()
            .strip_prefix("store { i8, i64 } ")
            .and_then(|tail| tail.split_once(", ptr "))
        else {
            continue;
        };
        let destination = destination.split(',').next().unwrap_or_default();
        if value == list_value
            && allocas.iter().any(|alloca| alloca == destination)
            && !stored_slots.iter().any(|slot| *slot == destination)
        {
            stored_slots.push(destination);
        }
    }
    stored_slots.len()
}

fn int_shadow_sync_reaches_entry_value_slot(body: &str, load_prefix: &str) -> bool {
    let Some((load_name, shadow_pointer)) = named_load_entry_alloca(body, load_prefix) else {
        return false;
    };
    let Some(inserted_value) = body.lines().find_map(|line| {
        let (result, instruction) = line.trim().split_once(" = insertvalue { i8, i64 } ")?;
        instruction
            .contains(&format!("i64 {load_name}, 1"))
            .then(|| result.to_string())
    }) else {
        return false;
    };
    let allocas = entry_alloca_pointers(body);
    body.lines().any(|line| {
        let Some((value, destination)) = line
            .trim()
            .strip_prefix("store { i8, i64 } ")
            .and_then(|tail| tail.split_once(", ptr "))
        else {
            return false;
        };
        let destination = destination.split(',').next().unwrap_or_default();
        value == inserted_value
            && destination != shadow_pointer
            && allocas.iter().any(|alloca| alloca == destination)
            && entry_alloca_type(body, destination) == Some("{ i8, i64 }")
    })
}

fn b1_functions_that_read_and_write_a_top_level_binding_keep_it_shared(mode: RcMode) {
    let source = r#"
ken message = "old" + ""
dae read_message() { gie message }
dae write_message(value) { message = value }
blether read_message()
write_message("new" + "")
blether read_message()
"#;
    assert_top_level_global(source, "message", mode);
    assert_exact_run(source, &[], "old\nnew\n", mode);
}
rc_mode_test!(
    b1_functions_that_read_and_write_a_top_level_binding_keep_it_shared_rc_off,
    b1_functions_that_read_and_write_a_top_level_binding_keep_it_shared_rc_on,
    b1_functions_that_read_and_write_a_top_level_binding_keep_it_shared
);

fn b2_a_top_level_binding_captured_by_a_lambda_stays_shared(mode: RcMode) {
    let source = r#"
ken message = "old" + ""
ken read_message = || { gie message }
blether read_message()
message = "new" + ""
blether read_message()
"#;
    assert_top_level_global(source, "message", mode);
    assert_exact_run(source, &[], "old\nnew\n", mode);
}
rc_mode_test!(
    b2_a_top_level_binding_captured_by_a_lambda_stays_shared_rc_off,
    b2_a_top_level_binding_captured_by_a_lambda_stays_shared_rc_on,
    b2_a_top_level_binding_captured_by_a_lambda_stays_shared
);

fn b3_a_thread_function_can_read_a_top_level_binding(mode: RcMode) {
    let source = r#"
ken message = "from-main" + ""
dae worker() {
    blether message
    gie 7
}
blether thread_join(thread_spawn(worker, []))
"#;
    assert_top_level_global(source, "message", mode);
    assert_exact_run(source, &[], "from-main\n7\n", mode);
}
rc_mode_test!(
    b3_a_thread_function_can_read_a_top_level_binding_rc_off,
    b3_a_thread_function_can_read_a_top_level_binding_rc_on,
    b3_a_thread_function_can_read_a_top_level_binding
);

fn b4_a_method_can_read_a_top_level_binding(mode: RcMode) {
    let source = r#"
ken material = "oak" + ""
kin Reader {
    dae read() { gie material }
}
ken reader = Reader()
blether reader.read()
material = "pine" + ""
blether reader.read()
"#;
    assert_top_level_global(source, "material", mode);
    assert_exact_run(source, &[], "oak\npine\n", mode);
}
rc_mode_test!(
    b4_a_method_can_read_a_top_level_binding_rc_off,
    b4_a_method_can_read_a_top_level_binding_rc_on,
    b4_a_method_can_read_a_top_level_binding
);

fn b5_function_parameter_defaults_resolve_in_the_callee_globals(mode: RcMode) {
    assert_exact_run(
        r#"
ken default_name = "before" + ""
dae choose(name = default_name) { gie name }
blether choose()
default_name = "after" + ""
blether choose()
"#,
        &[],
        "before\nafter\n",
        mode,
    );
}
rc_mode_test!(
    b5_function_parameter_defaults_resolve_in_the_callee_globals_rc_off,
    b5_function_parameter_defaults_resolve_in_the_callee_globals_rc_on,
    b5_function_parameter_defaults_resolve_in_the_callee_globals
);

fn b5_method_parameter_defaults_resolve_in_the_callee_globals(mode: RcMode) {
    assert_exact_run(
        r#"
ken default_name = "before" + ""
kin Reader {
    dae read(name = default_name) { gie name }
}
ken reader = Reader()
blether reader.read()
default_name = "after" + ""
blether reader.read()
"#,
        &[],
        "before\nafter\n",
        mode,
    );
}
rc_mode_test!(
    b5_method_parameter_defaults_resolve_in_the_callee_globals_rc_off,
    b5_method_parameter_defaults_resolve_in_the_callee_globals_rc_on,
    b5_method_parameter_defaults_resolve_in_the_callee_globals
);

fn b5_constructor_parameter_defaults_resolve_in_the_callee_globals(mode: RcMode) {
    assert_exact_run(
        r#"
ken default_name = "before" + ""
kin Label {
    dae init(name = default_name) { masel.name = name }
    dae read() { gie masel.name }
}
ken first = Label()
blether first.read()
default_name = "after" + ""
ken second = Label()
blether second.read()
"#,
        &[],
        "before\nafter\n",
        mode,
    );
}
rc_mode_test!(
    b5_constructor_parameter_defaults_resolve_in_the_callee_globals_rc_off,
    b5_constructor_parameter_defaults_resolve_in_the_callee_globals_rc_on,
    b5_constructor_parameter_defaults_resolve_in_the_callee_globals
);

fn b5_function_value_defaults_resolve_in_the_callee_globals(mode: RcMode) {
    assert_exact_run(
        r#"
ken default_name = "before" + ""
dae choose(name = default_name) { gie name }
ken indirect = choose
blether indirect()
default_name = "after" + ""
blether indirect()
"#,
        &[],
        "before\nafter\n",
        mode,
    );
}
rc_mode_test!(
    b5_function_value_defaults_resolve_in_the_callee_globals_rc_off,
    b5_function_value_defaults_resolve_in_the_callee_globals_rc_on,
    b5_function_value_defaults_resolve_in_the_callee_globals
);

fn b6_a_top_level_local_written_before_an_error_is_current_in_handler_and_after(mode: RcMode) {
    assert_exact_run(
        r#"
ken message = "before" + ""
hae_a_bash {
    message = "written" + ""
    # This built-in uses an immortal static error message, so the RC total isolates slot pinning.
    blether "x" * -1
} gin_it_gangs_wrang error {
    blether message
}
blether message
"#,
        &[],
        "written\nwritten\n",
        mode,
    );
}
rc_mode_test!(
    b6_a_top_level_local_written_before_an_error_is_current_in_handler_and_after_rc_off,
    b6_a_top_level_local_written_before_an_error_is_current_in_handler_and_after_rc_on,
    b6_a_top_level_local_written_before_an_error_is_current_in_handler_and_after
);

fn b7_fetch_keeps_importer_bindings_shared_with_module_initialization_and_functions(mode: RcMode) {
    let modules = [(
        "reader.braw",
        r#"
blether "init:" + entry_message
dae read_entry_message() { gie entry_message }
"#,
    )];
    assert_exact_run(
        r#"
ken entry_message = "from-entry" + ""
dae load_reader() {
    fetch "reader" tae reader
    gie reader
}
ken reader = load_reader()
blether reader.read_entry_message()
entry_message = "updated-entry" + ""
blether reader.read_entry_message()
"#,
        &modules,
        "init:from-entry\nfrom-entry\nupdated-entry\n",
        mode,
    );
}
rc_mode_test!(
    b7_fetch_keeps_importer_bindings_shared_with_module_initialization_and_functions_rc_off,
    b7_fetch_keeps_importer_bindings_shared_with_module_initialization_and_functions_rc_on,
    b7_fetch_keeps_importer_bindings_shared_with_module_initialization_and_functions
);

fn b8_imported_bindings_share_writes_both_ways_and_preserve_cache_and_name_collisions(
    mode: RcMode,
) {
    let modules = [
        (
            "alpha.braw",
            r#"
blether "alpha-init:" + import_origin
ken token = "alpha-zero" + ""
dae get() { gie token }
dae set(value) { token = value }
"#,
        ),
        (
            "beta.braw",
            r#"
ken token = "beta-zero" + ""
dae get() { gie token }
dae set(value) { token = value }
"#,
        ),
    ];
    assert_exact_run(
        r#"
ken import_origin = "from-entry" + ""
fetch "alpha" tae alpha
fetch "beta" tae beta
blether alpha.get()
blether beta.get()
fetch "alpha"
blether token
token = "from-entry" + ""
blether alpha.get()
fetch "alpha" tae alpha_again
alpha_again.set("from-module" + "")
blether get()
blether token
blether beta.get()
ken token = "shadow" + ""
blether token
blether alpha.get()
fetch "beta" tae beta_again
beta_again.set("beta-updated" + "")
blether beta.get()
blether alpha.get()
"#,
        &modules,
        "alpha-init:from-entry\nalpha-zero\nbeta-zero\nalpha-zero\nfrom-entry\nfrom-module\nfrom-module\n\
         beta-zero\nshadow\nfrom-module\nbeta-updated\nfrom-module\n",
        mode,
    );
}
rc_mode_test!(
    b8_imported_bindings_share_writes_both_ways_and_preserve_cache_and_name_collisions_rc_off,
    b8_imported_bindings_share_writes_both_ways_and_preserve_cache_and_name_collisions_rc_on,
    b8_imported_bindings_share_writes_both_ways_and_preserve_cache_and_name_collisions
);

fn eligible_closure_benchmark_bindings_are_main_allocas_and_function_references_are_globals(
    mode: RcMode,
) {
    let ir = compile_ir(include_str!("../benchmarks/rc/closure.braw"), mode);
    let main = function_body(&ir, "main");
    for name in ["fs", "s", "r", "f"] {
        assert!(
            !ir.contains(&format!("@__mdh_global_{name}")),
            "eligible {name} unexpectedly has a global binding"
        );
    }
    // The line-note pass assigns numeric names to entry allocas. Named MdhValue loads for
    // s/f and the int shadow loads for r let the test trace each local back to that entry
    // block. R5-3 gives RC-on indexing a real owning fs load for C's root proof;
    // RC-off continues to read its cached list-pointer shadow.
    let fs_load = match mode {
        RcMode::On => "fs",
        RcMode::Off => "list_ptr_shadow_rd",
    };
    for name in ["s", "r", "f", fs_load] {
        assert!(
            named_load_uses_entry_alloca(main, name),
            "{name} does not load from an entry-block alloca:\n{main}"
        );
    }
    assert!(
        int_shadow_sync_reaches_entry_value_slot(main, "r_sync"),
        "r's int-shadow sync does not reach its MdhValue entry alloca:\n{main}"
    );
    let fs_list_value_stores = list_value_entry_store_count(main);
    match mode {
        RcMode::On => assert!(
            fs_list_value_stores >= 2,
            "RC-on fs's list value is not stored in both its temporary and owned main slot:\n{main}"
        ),
        RcMode::Off => assert!(
            fs_list_value_stores >= 1,
            "RC-off fs's list value is not stored in its main binding alloca:\n{main}"
        ),
    }

    let shared_ir = compile_ir(
        r#"
ken shared_message = "global"
dae read_message() { gie shared_message }
blether read_message()
"#,
        mode,
    );
    assert!(
        shared_ir.contains("@__mdh_global_shared_message"),
        "a top-level variable referenced by a function must remain global"
    );
}
rc_mode_test!(
    eligible_closure_benchmark_bindings_are_main_allocas_and_function_references_are_globals_rc_off,
    eligible_closure_benchmark_bindings_are_main_allocas_and_function_references_are_globals_rc_on,
    eligible_closure_benchmark_bindings_are_main_allocas_and_function_references_are_globals
);

fn deeply_nested_defaults_and_assignment_targets_keep_their_top_level_name_global(mode: RcMode) {
    let ir = compile_ir(
        r#"
ken guarded_name = "global" + ""
dae outer() {
    dae middle() {
        dae inner(value = guarded_name) {
            guarded_name = value
            gie guarded_name
        }
        gie inner()
    }
    gie middle()
}
blether guarded_name
"#,
        mode,
    );
    assert!(
        ir.contains("@__mdh_global_guarded_name"),
        "a deep nested default and assignment target must retain the global binding"
    );
}
rc_mode_test!(
    deeply_nested_defaults_and_assignment_targets_keep_their_top_level_name_global_rc_off,
    deeply_nested_defaults_and_assignment_targets_keep_their_top_level_name_global_rc_on,
    deeply_nested_defaults_and_assignment_targets_keep_their_top_level_name_global
);
