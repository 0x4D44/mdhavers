#![cfg(feature = "llvm")]
//! `mdhavers build --target wasm32-emscripten`: the browser build.

use std::fs;
use std::path::Path;
use std::process::Command;

use mdhavers::{parse, BuildTarget, LLVMCompiler};

const PROGRAM: &str = r#"
fetch "helper"
ken words = split("braw bricht nicht", " ")
shove(words, "the nicht")
blether join(words, "-")
blether upper(words[0]) + " " + tae_string(len(words))
ken d = {"name": "Hamish", "age": 42}
d["toon"] = "Embra"
blether d
kin Beastie {
    dae init(name) {
        masel.name = name
    }
    dae hullo() {
        gie "Hullo fae " + masel.name
    }
}
blether Beastie("Nessie").hullo()
ken add = |a, b| a + b
blether gaun([1, 2, 3], |x| add(x, 10))
hae_a_bash {
    hurl "Och"
} gin_it_gangs_wrang err {
    blether "Caught: " + err
}
blether 3000000000 * 3
blether 7.0 / 2
blether twice(21)
"#;

const HELPER: &str = "dae twice(n) {\n    gie n * 2\n}\n";

fn write_program(dir: &Path) -> std::path::PathBuf {
    fs::write(dir.join("helper.braw"), HELPER).unwrap();
    let main = dir.join("main.braw");
    fs::write(&main, PROGRAM).unwrap();
    main
}

fn available(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn wasm_target_parses_its_names() {
    assert_eq!(
        BuildTarget::parse("wasm32-emscripten"),
        Ok(BuildTarget::Wasm32Emscripten)
    );
    assert_eq!(
        BuildTarget::parse("wasm32-unknown-emscripten"),
        Ok(BuildTarget::Wasm32Emscripten)
    );
    assert_eq!(BuildTarget::parse("native"), Ok(BuildTarget::Native));
    assert!(BuildTarget::parse("riscv")
        .unwrap_err()
        .contains("Dinnae ken"));
}

/// wasm32 IR: the wasm triple, and `size_t` narrowed to 32 bits where C has it.
#[test]
fn wasm_ir_has_the_wasm_triple_and_32_bit_size_t() {
    let dir = tempfile::tempdir().unwrap();
    let main = write_program(dir.path());
    let program = parse(PROGRAM).unwrap();
    let ir = LLVMCompiler::new()
        .with_target(BuildTarget::Wasm32Emscripten)
        .compile_to_ir_with_source(&program, Some(&main))
        .unwrap();
    assert!(
        ir.contains("target triple = \"wasm32-unknown-emscripten\""),
        "no wasm triple"
    );
    assert!(
        ir.contains("declare i32 @strlen(ptr"),
        "strlen not narrowed"
    );
    assert!(
        ir.contains("@__mdh_obj_alloc(i32, i32)") || ir.contains("@__mdh_obj_alloc(i32 "),
        "__mdh_obj_alloc not narrowed"
    );
    assert!(!ir.contains("size_t64("), "a wide declaration is left");

    // The native IR is untouched.
    let native = LLVMCompiler::new()
        .compile_to_ir_with_source(&program, Some(&main))
        .unwrap();
    assert!(!native.contains("wasm32"));
    assert!(native.contains("declare i64 @strlen(ptr"));
}

/// Built for wasm and run under node, a program prints what its native build prints.
/// Skips (passes) when emcc or node is missing.
#[test]
fn wasm_build_runs_under_node_like_native() {
    if !available("emcc") || !available("node") {
        eprintln!("skipping: emcc or node is not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let main = write_program(dir.path());
    let program = parse(PROGRAM).unwrap();

    let exe = dir.path().join("native_exe");
    LLVMCompiler::new()
        .compile_to_native_with_source(&program, &exe, 1, Some(&main))
        .unwrap();
    let native = Command::new(&exe).output().unwrap();
    assert!(native.status.success());

    let js = dir.path().join("main.js");
    LLVMCompiler::new()
        .compile_to_wasm_with_source(&program, &js, 1, Some(&main), &[])
        .unwrap();
    assert!(
        js.with_extension("wasm").exists(),
        "no .wasm beside the .js"
    );
    let wasm = Command::new("node").arg(&js).output().unwrap();
    assert!(
        wasm.status.success(),
        "node failed: {}",
        String::from_utf8_lossy(&wasm.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&wasm.stdout),
        String::from_utf8_lossy(&native.stdout)
    );
}

/// Unbounded recursion in the browser build raises the catchable stack error, as it does
/// natively, before the engine's own stack runs out (an uncatchable RangeError).
/// Skips (passes) when emcc or node is missing.
#[test]
fn wasm_deep_recursion_raises_the_stack_error() {
    if !available("emcc") || !available("node") {
        eprintln!("skipping: emcc or node is not on PATH");
        return;
    }
    const DEEP: &str = r#"
dae deep(n) {
    gie deep(n + 1) + 1
}
hae_a_bash {
    blether deep(0)
} gin_it_gangs_wrang err {
    blether "Caught: " + err
}
dae deepstr(n) {
    gie "x" + deepstr(n + 1)
}
hae_a_bash {
    blether deepstr(0)
} gin_it_gangs_wrang err {
    blether "Caught: " + err
}
blether "still standin"
"#;
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("deep.braw");
    fs::write(&main, DEEP).unwrap();
    let program = parse(DEEP).unwrap();
    let js = dir.path().join("deep.js");
    LLVMCompiler::new()
        .compile_to_wasm_with_source(&program, &js, 1, Some(&main), &[])
        .unwrap();
    let run = Command::new("node").arg(&js).output().unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run.status.success(),
        "node failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        stdout,
        "Caught: Stack's fair puggled! Too many nested calls at line 3 - yer recursion's gone radge!\n\
         Caught: Stack's fair puggled! Too many nested calls at line 11 - yer recursion's gone radge!\n\
         still standin\n"
    );
}

/// A count past wasm32's 32-bit size_t is "ower muckle", not wrapped to a small buffer
/// that the fill then overruns. Skips (passes) when emcc or node is missing.
#[test]
fn wasm_counts_past_size_t_are_ower_muckle() {
    if !available("emcc") || !available("node") {
        eprintln!("skipping: emcc or node is not on PATH");
        return;
    }
    // 2^32 + 4: wraps to 4 in a 32-bit size_t.
    let calls = [
        "tattie_scone(\"ab\", big)",
        "centre(\"ab\", big)",
        "leftpad(\"ab\", big, \"*\")",
        "rightpad(\"ab\", big, \"*\")",
        "sporran_fill(\"ab\", big, \"*\")",
    ];
    let mut source = String::from("ken big = 4294967300\n");
    for call in calls {
        source.push_str(&format!(
            "hae_a_bash {{\n    blether len({call})\n}} gin_it_gangs_wrang err {{\n    blether err\n}}\n"
        ));
    }
    source.push_str("blether \"still standin\"\n");
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("big.braw");
    fs::write(&main, &source).unwrap();
    let program = parse(&source).unwrap();
    let js = dir.path().join("big.js");
    LLVMCompiler::new()
        .compile_to_wasm_with_source(&program, &js, 1, Some(&main), &[])
        .unwrap();
    let run = Command::new("node").arg(&js).output().unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run.status.success(),
        "node failed: {}\n{stdout}",
        String::from_utf8_lossy(&run.stderr)
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), calls.len() + 1, "{stdout}");
    for (call, line) in calls.iter().zip(&lines) {
        assert!(line.contains("ower muckle"), "{call}: {line}");
    }
    assert_eq!(lines[calls.len()], "still standin");
}

/// The browser build's screen_loop hands frames to the host loop (under node a timer,
/// in a page requestAnimationFrame): frames run until one gives nae, then the program
/// exits 0 without running what follows the call; an uncaught error in a frame exits 1
/// with its message. Offscreen (MDH_SCREEN=none), so it runs in node. Skips (passes)
/// when emcc or node is missing.
#[test]
fn wasm_screen_loop_runs_frames_from_the_host_loop() {
    if !available("emcc") || !available("node") {
        eprintln!("skipping: emcc or node is not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let build = |name: &str, source: &str| {
        let main = dir.path().join(format!("{name}.braw"));
        fs::write(&main, source).unwrap();
        let js = dir.path().join(format!("{name}.js"));
        LLVMCompiler::new()
            .compile_to_wasm_with_source(&parse(source).unwrap(), &js, 1, Some(&main), &[])
            .unwrap();
        Command::new("node")
            .arg(&js)
            .env("MDH_SCREEN", "none")
            .output()
            .unwrap()
    };
    let run = build(
        "frames",
        r#"
screen_open(2, 2, "Loop")
sound_open(22050, 1)
ken frames = 0
dae tick(box, limit) {
    box["n"] = box["n"] + 1
    blether "frame " + tae_string(box["n"])
    screen_shaw(bytes(4), bytes(768))
    gie box["n"] < limit
}
ken box = {"n": 0}
hae_a_bash {
    screen_loop(|dt| tick(box, 3))
} gin_it_gangs_wrang err {
    blether "never here"
}
blether "nor here"
"#,
    );
    assert!(run.status.success(), "{run:?}");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "frame 1\nframe 2\nframe 3\n"
    );
    let shown = String::from_utf8_lossy(&run.stderr)
        .lines()
        .filter(|line| line.starts_with("mdh_screen frame "))
        .count();
    assert_eq!(shown, 3, "{run:?}");

    let run = build(
        "fails",
        r#"
screen_open(2, 2, "Loop")
dae frame(dt) {
    hurl "frame fell ower"
}
hae_a_bash {
    screen_loop(frame)
} gin_it_gangs_wrang err {
    blether "never here"
}
"#,
    );
    assert_eq!(run.status.code(), Some(1), "{run:?}");
    assert!(run.stdout.is_empty(), "{run:?}");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("frame fell ower"),
        "{run:?}"
    );
}

// ---------------------------------------------------------------------------------------
// The `build --target` command line, and the runtime build's failures. These need no
// Emscripten: a failing build stops before the link, and stand-in emcc/emar scripts
// (EMCC, EMAR) with a throwaway runtime (MDH_RUNTIME_DIR) and cache
// (MDH_WASM_RUNTIME_DIR) drive each failure.

fn mdhavers_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mdhavers"))
}

fn stderr_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn cli_build_rejects_an_unknown_target() {
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("main.braw");
    fs::write(&main, "blether 1\n").unwrap();
    let output = mdhavers_cli()
        .args(["build", "--target", "riscv"])
        .arg(&main)
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(
        stderr_of(&output).contains("Dinnae ken the target 'riscv'"),
        "{output:?}"
    );
}

#[test]
fn cli_emits_wasm_ir_for_the_wasm_target() {
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("main.braw");
    fs::write(&main, "blether len(\"braw\")\n").unwrap();
    let ir = dir.path().join("main.ll");
    let output = mdhavers_cli()
        .args([
            "build",
            "--target",
            "wasm32-unknown-emscripten",
            "--emit-llvm",
        ])
        .arg(&main)
        .arg("-o")
        .arg(&ir)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let ir = fs::read_to_string(&ir).unwrap();
    assert!(ir.contains("wasm32-unknown-emscripten"), "no wasm triple");
}

/// A compile error stops a browser build before anything is linked.
#[test]
fn cli_wasm_build_reports_compile_errors() {
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("main.braw");
    fs::write(&main, "screen_loop()\n").unwrap();
    let output = mdhavers_cli()
        .args(["build", "--target", "wasm32-emscripten"])
        .arg(&main)
        .arg("-o")
        .arg(dir.path().join("main.js"))
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(
        stderr_of(&output).contains("screen_loop expects 1 argument"),
        "{output:?}"
    );
}

#[cfg(unix)]
mod runtime_build_failures {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    /// emcc: `--version` answers; otherwise creates the `-o` file, prints
    /// $FAKE_EMCC_STDERR and exits $FAKE_EMCC_EXIT (0).
    const FAKE_EMCC: &str = r#"#!/bin/sh
if [ "$1" = "--version" ]; then echo "fake emcc 1.0"; exit 0; fi
out=""; prev=""
for a in "$@"; do [ "$prev" = "-o" ] && out="$a"; prev="$a"; done
[ -n "$out" ] && : > "$out"
[ -n "$FAKE_EMCC_STDERR" ] && echo "$FAKE_EMCC_STDERR" >&2
exit "${FAKE_EMCC_EXIT:-0}"
"#;
    /// emar `rcs <archive> <objects>`: creates the archive, exits $FAKE_EMAR_EXIT (0).
    const FAKE_EMAR: &str = r#"#!/bin/sh
: > "$2"
exit "${FAKE_EMAR_EXIT:-0}"
"#;

    struct Setup {
        dir: tempfile::TempDir,
        main: PathBuf,
    }

    impl Setup {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            for (name, script) in [("emcc", FAKE_EMCC), ("emar", FAKE_EMAR)] {
                let path = dir.path().join(name);
                fs::write(&path, script).unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
            }
            // A runtime with nothing the stand-in emcc needs to read.
            fs::create_dir_all(dir.path().join("runtime/platform")).unwrap();
            let main = dir.path().join("main.braw");
            fs::write(&main, "blether \"hullo\"\n").unwrap();
            Setup { dir, main }
        }

        fn path(&self, name: &str) -> PathBuf {
            self.dir.path().join(name)
        }

        /// `mdhavers build --target wasm32-emscripten` with the stand-ins; `envs`
        /// override them. Returns stderr, after checking the build failed.
        fn build_fails(&self, envs: &[(&str, PathBuf)], extra: &[(&str, &str)]) -> String {
            self.build_with_args_fails(&[], envs, extra)
        }

        /// The same, with `args` after the source file.
        fn build_with_args_fails(
            &self,
            args: &[&str],
            envs: &[(&str, PathBuf)],
            extra: &[(&str, &str)],
        ) -> String {
            let mut command = mdhavers_cli();
            command
                .args(["build", "--target", "wasm32-emscripten"])
                .arg(&self.main)
                .args(args)
                .arg("-o")
                .arg(self.path("main.js"))
                .env("EMCC", self.path("emcc"))
                .env("EMAR", self.path("emar"))
                .env("MDH_RUNTIME_DIR", self.path("runtime"))
                .env("MDH_WASM_RUNTIME_DIR", self.path("cache"));
            for (name, value) in envs {
                command.env(name, value);
            }
            for (name, value) in extra {
                command.env(name, value);
            }
            let output = command.output().unwrap();
            assert!(!output.status.success(), "{output:?}");
            stderr_of(&output)
        }
    }

    #[test]
    fn a_missing_emcc_is_reported() {
        let setup = Setup::new();
        let err = setup.build_fails(&[("EMCC", setup.path("no-such-emcc"))], &[]);
        assert!(err.contains("Cannae run emcc"), "{err}");
    }

    #[test]
    fn a_missing_runtime_source_is_reported() {
        let setup = Setup::new();
        let err = setup.build_fails(&[("MDH_RUNTIME_DIR", setup.path("nowhere"))], &[]);
        assert!(err.contains("Cannae read the runtime"), "{err}");
    }

    #[test]
    fn an_unmakeable_cache_is_reported() {
        let setup = Setup::new();
        fs::write(setup.path("a-file"), "").unwrap();
        let err = setup.build_fails(
            &[("MDH_WASM_RUNTIME_DIR", setup.path("a-file").join("cache"))],
            &[],
        );
        assert!(err.contains("Cannae make"), "{err}");
    }

    #[test]
    fn an_emcc_failure_is_reported_with_its_output() {
        let setup = Setup::new();
        let err = setup.build_fails(
            &[],
            &[
                ("FAKE_EMCC_EXIT", "1"),
                ("FAKE_EMCC_STDERR", "och, a compile error"),
            ],
        );
        assert!(err.contains("emcc mdh_runtime.c failed"), "{err}");
        assert!(err.contains("och, a compile error"), "{err}");
    }

    /// wasm-ld only warns of a signature mismatch (and makes the call a trap): that
    /// warning fails the step even though emcc succeeded.
    #[test]
    fn a_signature_mismatch_warning_fails_the_step() {
        let setup = Setup::new();
        let err = setup.build_fails(
            &[],
            &[(
                "FAKE_EMCC_STDERR",
                "wasm-ld: warning: function signature mismatch: f",
            )],
        );
        assert!(err.contains("emcc mdh_runtime.c failed"), "{err}");
        assert!(err.contains("signature mismatch"), "{err}");
    }

    #[test]
    fn an_emar_failure_is_reported() {
        let setup = Setup::new();
        let err = setup.build_fails(&[], &[("FAKE_EMAR_EXIT", "1")]);
        assert!(err.contains("emar libmdh_runtime.a failed"), "{err}");
    }

    #[test]
    fn a_missing_emar_is_reported() {
        let setup = Setup::new();
        let err = setup.build_fails(&[("EMAR", setup.path("no-such-emar"))], &[]);
        assert!(err.contains("Cannae run emar"), "{err}");
    }

    #[test]
    fn an_unreadable_runtime_source_is_reported() {
        let setup = Setup::new();
        let source = setup.path("runtime/platform/locked.c");
        fs::write(&source, "int x;\n").unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o000)).unwrap();
        if fs::read(&source).is_ok() {
            eprintln!("skipping: running as a user who can read a mode-000 file");
            return;
        }
        let err = setup.build_fails(&[], &[]);
        assert!(err.contains("locked.c"), "{err}");
    }

    /// The Rust runtime's cargo build leaves out the build settings of a cargo it runs
    /// under (cargo-llvm-cov's instrumenting RUSTFLAGS, say): they are for the host.
    #[test]
    fn the_rust_runtime_build_drops_the_parents_build_settings() {
        let setup = Setup::new();
        let cargo = setup.path("cargo");
        fs::write(
            &cargo,
            "#!/bin/sh\necho \"inherited [$RUSTFLAGS][$CARGO_ENCODED_RUSTFLAGS][$RUSTC_WRAPPER]\
             [$LLVM_PROFILE_FILE][$CARGO_LLVM_COV]\" >&2\nexit 1\n",
        )
        .unwrap();
        fs::set_permissions(&cargo, fs::Permissions::from_mode(0o755)).unwrap();
        let err = setup.build_fails(
            &[("CARGO", cargo)],
            &[
                ("RUSTFLAGS", "-Cinstrument-coverage"),
                ("CARGO_ENCODED_RUSTFLAGS", "-Cinstrument-coverage"),
                ("RUSTC_WRAPPER", "/no/such/wrapper"),
                ("LLVM_PROFILE_FILE", "/tmp/x.profraw"),
                ("CARGO_LLVM_COV", "1"),
            ],
        );
        assert!(err.contains("inherited [][][][][]"), "{err}");
    }

    /// --emcc-arg cannot lift the memory limit past 2GB (an address must fit a signed
    /// 32-bit intptr_t); the build stops before any of it runs.
    #[test]
    fn a_memory_limit_past_2gb_is_refused() {
        let setup = Setup::new();
        let err = setup.build_with_args_fails(&["--emcc-arg=-sMAXIMUM_MEMORY=4GB"], &[], &[]);
        assert!(err.contains("ower muckle for the browser"), "{err}");
        assert!(!setup.path("cache").exists(), "the runtime was built");
    }

    /// The C archives build (stand-ins), then the Rust runtime's cargo build fails: this
    /// runtime has no mdh_runtime_rs.
    #[test]
    fn a_rust_runtime_failure_is_reported() {
        let setup = Setup::new();
        let err = setup.build_fails(&[], &[]);
        assert!(
            err.contains("cargo build of the Rust runtime for wasm32 failed"),
            "{err}"
        );
        // Every archive was put in place, and the stamp written.
        for lib in ["libmdh_runtime.a", "libmdh_sleep_yield.a", "c.stamp"] {
            assert!(setup.path("cache").join(lib).exists(), "no {lib}");
        }
    }
}

/// The command line end to end: --emcc-arg reaches the link, and a JSPI build's snooze
/// suspends (under node, on a timer) while a default build's sleeps. Skips (passes)
/// when emcc or node is missing.
#[test]
fn cli_wasm_build_passes_emcc_args_and_snoozes_with_or_without_jspi() {
    if !available("emcc") || !available("node") {
        eprintln!("skipping: emcc or node is not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("main.braw");
    fs::write(
        &main,
        "ken t = mono_ms()\nsnooze(20)\nblether mono_ms() - t >= 15\nblether \"waukened\"\n",
    )
    .unwrap();
    for (name, extra) in [("plain", None), ("jspi", Some("--emcc-arg=-sJSPI"))] {
        let js = dir.path().join(format!("{name}.js"));
        let mut command = mdhavers_cli();
        command
            .args(["build", "--target", "wasm32-emscripten"])
            .arg(&main)
            .arg("-o")
            .arg(&js);
        if let Some(arg) = extra {
            command.arg(arg);
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "{name}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("for the browser"),
            "{name}: {output:?}"
        );
        let js_text = fs::read_to_string(&js).unwrap();
        assert_eq!(
            js_text.contains("Suspending"),
            extra.is_some(),
            "{name}: JSPI glue present only when asked for"
        );
        let run = Command::new("node").arg(&js).output().unwrap();
        assert!(run.status.success(), "{name}: {run:?}");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "aye\nwaukened\n",
            "{name}"
        );
    }
}
