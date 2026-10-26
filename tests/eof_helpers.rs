//! MDH-BUG-FLU-00191: the stdlib and example input helpers return `naething` at end of
//! input (as `speir` does) instead of calling `exit(0)`, and their callers wind the program
//! up on their own. Run through the CLI with stdin closed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mdhavers"))
}

/// Run a command with stdin closed, kill it after `secs`, give back (exit code, stdout, stderr).
fn run_closed_stdin(cmd: &mut Command, secs: u64) -> (Option<i32>, String, String) {
    // Output goes to files, not pipes: a native build's linker warnings alone overfill a pipe.
    let dir = tempfile::tempdir().expect("temp dir");
    let out_path = dir.path().join("stdout");
    let err_path = dir.path().join("stderr");
    let mut child = cmd
        .current_dir(root())
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .stdout(fs::File::create(&out_path).expect("stdout file"))
        .stderr(fs::File::create(&err_path).expect("stderr file"))
        .spawn()
        .expect("spawn");
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait") {
            break status;
        }
        if start.elapsed() > Duration::from_secs(secs) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("timed out after {secs}s");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let read = |p: &Path| String::from_utf8_lossy(&fs::read(p).unwrap_or_default()).to_string();
    (status.code(), read(&out_path), read(&err_path))
}

fn run_interpreter(file: &Path) -> (Option<i32>, String, String) {
    run_closed_stdin(Command::new(bin()).arg("run").arg(file), 60)
}

#[cfg(feature = "llvm")]
fn run_native(file: &Path) -> (Option<i32>, String, String) {
    let dir = tempfile::tempdir().expect("temp dir");
    let exe = dir.path().join("eof_exe");
    let (code, out, err) = run_closed_stdin(
        Command::new(bin())
            .arg("build")
            .arg(file)
            .arg("-o")
            .arg(&exe)
            .arg("-O")
            .arg("0"),
        1500,
    );
    assert_eq!(code, Some(0), "build failed: {out}\n{err}");
    run_closed_stdin(&mut Command::new(&exe), 60)
}

const ENDED: &str = "Input's ended - cheerio, and haste ye back!";

const EXAMPLES: &[&str] = &[
    "examples/scottish/scots_calculator.braw",
    "examples/apps/todo_app.braw",
    "examples/games/adventure.braw",
    "examples/games/dungeon_crawler.braw",
    "examples/games/guessing_game.braw",
    "examples/games/haggis_adventure.braw",
    "examples/games/hangman.braw",
    "examples/games/highland_adventure.braw",
    "examples/games/highland_heist.braw",
    "examples/games/pub_quiz.braw",
    "examples/games/scottish_blackjack.braw",
    "examples/games/word_scramble.braw",
];

const SHOWCASES: &[&str] = &[
    "examples/showcases/fractals/fractals.braw",
    "examples/showcases/roguelike/roguelike.braw",
    "examples/showcases/sort_viz/sort_viz.braw",
    "examples/showcases/game_2048/game_2048.braw",
    "examples/showcases/game_of_life/game_of_life.braw",
];

fn assert_ends_cleanly(label: &str, run: (Option<i32>, String, String)) {
    let (code, out, err) = run;
    assert_eq!(code, Some(0), "{label}: exit code; stderr: {err}\n{out}");
    assert_eq!(
        out.matches(ENDED).count(),
        1,
        "{label}: should say cheerio exactly once\n{out}"
    );
    // The program goes on past the helper (its own closing lines), rather than exiting at it.
    // examples/games/adventure.braw has nothing to say once its loop ends.
    if !label.ends_with("games/adventure.braw") {
        let after = out.split(ENDED).nth(1).unwrap_or("");
        assert!(
            !after.trim().is_empty(),
            "{label}: nothing printed after the input ended\n{out}"
        );
    }
}

#[test]
fn examples_end_cleanly_at_end_of_input_in_the_interpreter() {
    for rel in EXAMPLES.iter().chain(SHOWCASES) {
        assert_ends_cleanly(rel, run_interpreter(&root().join(rel)));
    }
}

// One native test per program: each is a separate LLVM build.
#[cfg(feature = "llvm")]
macro_rules! native_ends_cleanly {
    ($($name:ident => $rel:expr,)*) => {
        $(
            #[test]
            fn $name() {
                assert_ends_cleanly($rel, run_native(&root().join($rel)));
            }
        )*
    };
}

#[cfg(feature = "llvm")]
native_ends_cleanly! {
    native_scots_calculator => "examples/scottish/scots_calculator.braw",
    native_adventure => "examples/games/adventure.braw",
    native_guessing_game => "examples/games/guessing_game.braw",
    native_hangman => "examples/games/hangman.braw",
    native_highland_heist => "examples/games/highland_heist.braw",
    native_scottish_blackjack => "examples/games/scottish_blackjack.braw",
    native_fractals => "examples/showcases/fractals/fractals.braw",
    native_game_of_life => "examples/showcases/game_of_life/game_of_life.braw",
}

// The stdlib helpers give back naething (or the prompt's default) instead of exiting, so the
// program carries on past them.
const CLI_PROGRAM: &str = r#"
fetch "stdlib/cli"
blether prompt_text("name") == naething
blether prompt_text("name", "wee") == "wee"
blether prompt_confirm("sure?")
blether prompt_confirm("sure?", nae)
blether prompt_choice("pick", ["a", "b"]) == naething
blether prompt_password("pw") == naething
blether "still here"
"#;

const CLI_ANSWERS: &str = "aye\naye\naye\nnae\naye\naye\nstill here\n";

const MENUS_PROGRAM: &str = r#"
fetch "stdlib/menus"
blether ask("name") == naething
blether confirm("sure?", aye)
ken m = Menu("Pick")
m.add("One", 1)
blether m.show() == naething
blether m.show_required() == naething
blether TextInput("Name").ask() == naething
blether NumberInput("Age").ask() == naething
ken c = ChoiceInput("Which")
c.add("x")
blether c.ask() == naething
blether "still here"
"#;

const MENUS_ANSWERS: &str = "aye\naye\naye\naye\naye\naye\naye\nstill here\n";

const ADVENTURE_PROGRAM: &str = r#"
fetch "stdlib/adventure"
ken g = make_adventure("EOF")
g.run()
blether "still here"
"#;

fn program_file(dir: &Path, name: &str, src: &str) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, src).expect("write");
    p
}

/// Just the yes/no answers and the final marker: prompts and room text are not the point.
fn only_answers(out: &str) -> String {
    out.lines()
        .filter(|l| matches!(*l, "aye" | "nae" | "still here"))
        .map(|l| format!("{l}\n"))
        .collect()
}

#[test]
fn cli_helpers_return_naething_at_end_of_input_in_the_interpreter() {
    let dir = tempfile::tempdir().expect("temp dir");
    let (code, out, err) = run_interpreter(&program_file(dir.path(), "cli.braw", CLI_PROGRAM));
    assert_eq!(code, Some(0), "stderr: {err}\n{out}");
    assert_eq!(only_answers(&out), CLI_ANSWERS, "{out}");
}

#[test]
fn menus_helpers_return_naething_at_end_of_input_in_the_interpreter() {
    let dir = tempfile::tempdir().expect("temp dir");
    let (code, out, err) = run_interpreter(&program_file(dir.path(), "menus.braw", MENUS_PROGRAM));
    assert_eq!(code, Some(0), "stderr: {err}\n{out}");
    assert_eq!(only_answers(&out), MENUS_ANSWERS, "{out}");
}

#[test]
fn adventure_run_ends_at_end_of_input_in_the_interpreter() {
    let dir = tempfile::tempdir().expect("temp dir");
    let (code, out, err) =
        run_interpreter(&program_file(dir.path(), "adv.braw", ADVENTURE_PROGRAM));
    assert_eq!(code, Some(0), "stderr: {err}\n{out}");
    assert!(out.ends_with("still here\n"), "{out}");
    assert_eq!(out.matches(ENDED).count(), 1, "{out}");
}

#[cfg(feature = "llvm")]
#[test]
fn cli_helpers_return_naething_at_end_of_input_natively() {
    let dir = tempfile::tempdir().expect("temp dir");
    let (code, out, err) = run_native(&program_file(dir.path(), "cli.braw", CLI_PROGRAM));
    assert_eq!(code, Some(0), "stderr: {err}\n{out}");
    assert_eq!(only_answers(&out), CLI_ANSWERS, "{out}");
}
