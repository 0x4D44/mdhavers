//! File-aware JS compilation bundles source modules, preserving interpreter fetch semantics.
#![cfg(feature = "cli")]

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

const OUTPUT_LIMIT: usize = 1024 * 1024;
static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);
static DELTIC_AVAILABLE: OnceLock<bool> = OnceLock::new();

fn wait_bounded(child: &mut Child) -> ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().expect("poll fixture command") {
            return status;
        }
        if Instant::now() >= deadline {
            // A racing exit can make kill fail; always reap the child afterward.
            let _ = child.kill();
            return child.wait().expect("reap timed-out fixture command");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn deltic_available() -> bool {
    *DELTIC_AVAILABLE.get_or_init(|| {
        Command::new("deltic")
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|mut child| wait_bounded(&mut child).success())
            .unwrap_or(false)
    })
}

fn node_available() -> bool {
    let available = Command::new("node")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|mut child| wait_bounded(&mut child).success())
        .unwrap_or(false);
    if !available {
        eprintln!("SKIPPED: `node` is no' on PATH");
    }
    available
}

struct Captured {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

fn read_capped(mut pipe: impl Read, mut output: fs::File) -> (Vec<u8>, usize) {
    let mut retained = Vec::new();
    let mut total = 0;
    let mut buffer = [0; 8192];
    loop {
        let size = pipe.read(&mut buffer).expect("read child output");
        if size == 0 {
            break;
        }
        total += size;
        let keep = size.min(OUTPUT_LIMIT - retained.len());
        retained.extend_from_slice(&buffer[..keep]);
        output
            .write_all(&buffer[..keep])
            .expect("write capped child output");
    }
    (retained, total)
}

struct Fixture {
    root: PathBuf,
    files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}

impl Fixture {
    fn new() -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join(".scratch");
        fs::create_dir_all(&scratch).expect("create worktree scratch");
        let root = scratch.join(format!(
            "js-module-fetch-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("create isolated module fixture");
        Self {
            root,
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    fn write(&mut self, relative: &str, source: &str) {
        let path = self.root.join(relative);
        let mut parents: Vec<_> = path
            .parent()
            .unwrap()
            .ancestors()
            .take_while(|p| *p != self.root)
            .filter(|p| !p.exists())
            .map(Path::to_path_buf)
            .collect();
        parents.reverse();
        for parent in parents {
            fs::create_dir(&parent).expect("create fixture subdirectory");
            self.directories.push(parent);
        }
        fs::write(&path, source).expect("write module fixture");
        if !self.files.contains(&path) {
            self.files.push(path);
        }
    }

    fn command(&mut self, program: &str, args: &[&str]) -> Captured {
        let command_id = self.files.len();
        let stdout_path = self.root.join(format!("command-{command_id}.stdout"));
        let stderr_path = self.root.join(format!("command-{command_id}.stderr"));
        self.files.push(stdout_path.clone());
        self.files.push(stderr_path.clone());
        let stdout_file = fs::File::create(stdout_path).expect("create stdout capture");
        let stderr_file = fs::File::create(stderr_path).expect("create stderr capture");
        let mut command = if deltic_available() {
            let mut command = Command::new("deltic");
            command.args(["timeout", "30", program]);
            command
        } else {
            Command::new(program)
        };
        let mut child = command
            .args(args)
            .current_dir(&self.root)
            .env("MDHAVERS_HOME", self.root.join("home"))
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start bounded fixture command");
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let stdout_reader = thread::spawn(move || read_capped(stdout, stdout_file));
        let stderr_reader = thread::spawn(move || read_capped(stderr, stderr_file));
        let status = wait_bounded(&mut child);
        let (stdout, stdout_size) = stdout_reader.join().expect("stdout reader");
        let (stderr, stderr_size) = stderr_reader.join().expect("stderr reader");
        assert!(
            stdout_size <= OUTPUT_LIMIT && stderr_size <= OUTPUT_LIMIT,
            "{program} exceeded the 1 MiB output limit"
        );
        Captured {
            status,
            stdout: String::from_utf8(stdout).expect("UTF-8 stdout"),
            stderr: String::from_utf8(stderr).expect("UTF-8 stderr"),
        }
    }

    fn success(&mut self, program: &str, args: &[&str]) -> String {
        let output = self.command(program, args);
        assert!(
            output.status.success(),
            "{program} {args:?} failed ({}):\nstdout:\n{}\nstderr:\n{}",
            output.status,
            output.stdout,
            output.stderr
        );
        output.stdout.replace("\r\n", "\n")
    }

    fn compile(&mut self) {
        // Register the output before invoking the compiler, including error paths.
        self.files.push(self.root.join("out.js"));
        self.success(
            env!("CARGO_BIN_EXE_mdhavers"),
            &["compile", "main.braw", "-o", "out.js"],
        );
    }

    fn parity(&mut self, expected: &str) {
        let interpreted = self.success(env!("CARGO_BIN_EXE_mdhavers"), &["run", "main.braw"]);
        assert_eq!(interpreted, expected, "interpreter fixture contract");
        self.compile();
        let compiled = self.success("node", &["out.js"]);
        assert_eq!(compiled, expected, "bundled JS output");
        assert_eq!(compiled, interpreted, "JS/interpreter fetch parity");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Remove only paths created by this fixture; never recursively remove shared scratch.
        for file in self.files.iter().rev() {
            let _ = fs::remove_file(file);
        }
        for directory in self.directories.iter().rev() {
            let _ = fs::remove_dir(directory);
        }
        let _ = fs::remove_dir(&self.root);
    }
}

const MUTABLE_MODULE: &str = r#"
ken mg = "m" + "g"
dae setmg() { mg = "n" + "h" gie "xy" }
dae getmg() { gie mg }
"#;

#[test]
fn module_function_writes_are_shared_and_operands_keep_evaluation_order() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", MUTABLE_MODULE);
    f.write(
        "main.braw",
        "fetch \"m\"\nblether mg + setmg()\nblether mg\n",
    );
    let interpreted = f.success(env!("CARGO_BIN_EXE_mdhavers"), &["run", "main.braw"]);
    assert_eq!(interpreted, "mgxy\nnh\n");
    f.compile();
    fs::remove_file(f.root.join("main.braw")).expect("remove bundled entry fixture");
    fs::remove_file(f.root.join("m.braw")).expect("remove bundled module fixture");
    assert_eq!(f.success("node", &["out.js"]), interpreted);
}

#[test]
fn lambda_default_captures_the_same_parameter_cell_as_the_function_body() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write(
        "m.braw",
        "dae capture(x = 1, read = || x) {\nx = 2\ngie read()\n}\n",
    );
    f.write(
        "main.braw",
        "fetch \"m\"\nblether capture()\nblether capture(8)\n",
    );
    f.parity("2\n2\n");
}

#[test]
fn module_functions_preserve_loop_destructure_lambda_and_class_scopes() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write(
        "m.braw",
        r#"
ken offset = 10
kin Reader {
    dae read(extra = 1) { gie offset + extra }
}
dae compute(values) {
    ken [first, ...rest] = values
    ken total = first
    fer item in rest { total = total + item }
    ken add_offset = |n| n + offset
    gie add_offset(total)
}
"#,
    );
    f.write("main.braw", "fetch \"m\"\nblether compute([1, 2, 3])\nken reader = Reader()\nblether reader.read()\noffset = 20\nblether compute([1, 2, 3])\nblether reader.read(3)\n");
    f.parity("16\n11\n26\n23\n");
}

#[test]
fn importer_assignment_writes_the_module_cell() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", MUTABLE_MODULE);
    f.write(
        "main.braw",
        "fetch \"m\"\nmg = \"importer\"\nblether getmg()\nblether setmg()\nblether mg\n",
    );
    f.parity("importer\nxy\nnh\n");
}

#[test]
fn nested_fetch_resolves_relative_to_its_own_source_file() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("dep.braw", "ken answer = 99\n");
    f.write("nested/dep.braw", "ken answer = 42\n");
    f.write(
        "nested/m.braw",
        "fetch \"./dep\"\ndae answer_from_module() { gie answer }\n",
    );
    f.write(
        "main.braw",
        "fetch \"nested/m.braw\"\nblether answer_from_module()\n",
    );
    f.parity("42\n");
}

#[test]
fn exported_function_fetch_uses_the_active_module_directory_at_call_time() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("dep.braw", "ken answer = 11\n");
    f.write("sub/dep.braw", "ken answer = 22\n");
    f.write(
        "sub/m.braw",
        "dae load_dep() {\nfetch \"./dep\" tae dep\ngie dep.answer\n}\nblether load_dep()\n",
    );
    f.write("main.braw", "fetch \"sub/m\"\nblether load_dep()\n");
    // Module evaluation temporarily changes the interpreter's active directory;
    // calling an exported function afterward uses the restored entry directory.
    f.parity("22\n11\n");
}

#[test]
fn two_importers_share_one_canonical_module_and_evaluate_it_once() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write(
        "m.braw",
        "blether \"loaded\"\nken counter = 0\ndae bump() { counter = counter + 1 gie counter }\n",
    );
    f.write("a.braw", "fetch \"m\"\ndae first() { gie bump() }\n");
    f.write(
        "b.braw",
        "fetch \"./m.braw\"\ndae second() { gie bump() }\n",
    );
    f.write(
        "main.braw",
        "fetch \"a\"\nfetch \"b\"\nblether first()\nblether second()\nblether counter\n",
    );
    f.parity("loaded\n1\n2\n2\n");
}

#[test]
fn aliases_snapshot_values_but_exported_functions_keep_module_cells() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", MUTABLE_MODULE);
    f.write("main.braw", "fetch \"m\" tae old\nfetch \"m\"\nblether old.mg\nblether old.setmg()\nblether mg\nblether old.mg\nfetch \"m\" tae fresh\nblether fresh.mg\nold.mg = \"private\"\nblether getmg()\n");
    f.parity("mg\nxy\nnh\nmg\nnh\nnh\n");
}

#[test]
fn importer_ken_shadows_instead_of_overwriting_the_module_cell() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", MUTABLE_MODULE);
    f.write("main.braw", "fetch \"m\"\nken mg = \"new\"\nblether mg\nblether getmg()\nblether setmg()\nblether mg\nblether getmg()\n");
    f.parity("new\nmg\nxy\nnew\nnh\n");
}

#[test]
fn fetch_inside_function_has_local_imports_and_reuses_module_state() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write(
        "m.braw",
        "blether \"loaded\"\nken counter = 0\ndae bump() { counter = counter + 1 gie counter }\n",
    );
    f.write("main.braw", "dae call_module() { fetch \"m\" gie bump() }\nblether call_module()\nblether call_module()\n");
    f.parity("loaded\n1\n2\n");
}

#[test]
fn unexecuted_fetch_does_not_evaluate_its_module() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", "blether \"must not run\"\n");
    f.write("live.braw", "ken answer = 42\n");
    f.write(
        "main.braw",
        "gin nae { fetch \"m\" }\nblether \"done\"\nfetch \"live\"\nblether answer\n",
    );
    f.parity("done\n42\n");
}

#[test]
fn compiler_private_helpers_are_not_language_builtins() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", "ken value = 7\n");
    f.write("main.braw", "fetch \"m\"\nhae_a_bash { blether show(value) } gin_it_gangs_wrang err { blether \"private\" }\n");
    f.parity("private\n");
}

#[test]
fn imported_functions_do_not_capture_importer_locals() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write(
        "m.braw",
        "ken answer = 42\ndae get_answer() { gie answer }\n",
    );
    f.write(
        "main.braw",
        "dae local() { ken answer = 99 fetch \"m\" gie get_answer() }\nblether local()\n",
    );
    f.parity("42\n");
}

#[test]
fn module_functions_can_read_entry_globals_declared_after_fetch() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", "dae read_entry() { gie entry_answer }\n");
    f.write("main.braw", "fetch \"m\"\nken entry_answer = 61\nblether read_entry()\nentry_answer = 62\nblether read_entry()\n");
    f.parity("61\n62\n");
}

#[test]
fn block_local_fetch_shares_state_with_a_later_top_level_fetch() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", MUTABLE_MODULE);
    f.write(
        "main.braw",
        "gin aye {\nfetch \"m\"\nblether setmg()\nblether mg\n}\nfetch \"m\"\nblether mg\n",
    );
    f.parity("xy\nnh\nnh\n");
}

#[test]
fn pure_standard_library_module_is_bundled() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write(
        "main.braw",
        "fetch \"lib/maths\" tae maths\nblether maths.square(7)\nblether maths.gcd(18, 12)\n",
    );
    f.parity("Maths module loaded! Numbers are braw!\n49\n6\n");
}

#[test]
fn modules_resolve_from_mdhavers_home_lib() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write(
        "home/lib/fetch_home_fixture_00206.braw",
        "ken home_answer = 73\n",
    );
    f.write(
        "main.braw",
        "fetch \"lib/fetch_home_fixture_00206\"\nblether home_answer\n",
    );
    f.parity("73\n");
}

#[cfg(unix)]
#[test]
fn braw_symlinks_keep_source_module_semantics_after_canonicalization() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("module.source", "ken answer = 47\n");
    let link = f.root.join("m.braw");
    std::os::unix::fs::symlink("module.source", &link).expect("create module symlink");
    f.files.push(link);
    f.write("main.braw", "fetch \"m\"\nblether answer\n");
    f.parity("47\n");
}

#[test]
fn missing_source_module_error_is_catchable_and_matches_interpreter() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("main.braw", "hae_a_bash { fetch \"./missing_00206.braw\" } gin_it_gangs_wrang err { blether err }\nblether \"after\"\n");
    f.parity(
        "Cannae find module './missing_00206.braw' - hae ye checked the path is richt?\nafter\n",
    );
}

#[test]
fn circular_fetch_error_has_the_same_canonical_chain_as_interpreter() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("a.braw", "fetch \"b\"\n");
    f.write("b.braw", "fetch \"a\"\n");
    f.write(
        "main.braw",
        "hae_a_bash { fetch \"a\" } gin_it_gangs_wrang err { blether err }\n",
    );
    let a = f.root.join("a.braw").canonicalize().unwrap();
    let b = f.root.join("b.braw").canonicalize().unwrap();
    f.parity(&format!(
        "The import's gone in a fankle! Circular import detected: {} -> {} -> {}\n",
        a.display(),
        b.display(),
        a.display()
    ));
}

#[test]
fn node_builtin_fetch_keeps_require_interop() {
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", "ken file_path = \"a/b.txt\"\n");
    f.write(
        "main.braw",
        "fetch \"m\"\nfetch \"node:path\" tae paths\nblether paths.basename(file_path)\n",
    );
    f.compile();
    assert_eq!(f.success("node", &["out.js"]), "b.txt\n");
}

#[test]
fn source_only_playground_api_keeps_require_without_file_resolution() {
    let source = "fetch \"./m\" tae m\nblether m.value\n";
    let js = mdhavers::compile_to_js(source).expect("source-only compilation");
    assert!(
        js.contains("require('./m')"),
        "source-only fetch must retain require"
    );
    assert!(
        js.contains("__havers.foreign("),
        "require retains the JS interop boundary"
    );
    if !node_available() {
        return;
    }
    let mut f = Fixture::new();
    f.write("m.braw", "ken value = 9\n");
    f.write("main.braw", source);
    f.parity("9\n");
}
