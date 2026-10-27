#![cfg(feature = "llvm")]
//! Native backend: runtime errors raised from the Rust runtime leave it in a usable state
//! (MDH-BUG-FLU-00183), and running out of memory while building a string raises the
//! "ower muckle" error instead of crashing (MDH-BUG-FLU-00184, -00188, -00189).

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_native(source: &str, refcounting: bool) -> Run {
    run_native_with_env(source, refcounting, &[])
}

fn run_native_with_env(source: &str, refcounting: bool, env: &[(&str, &str)]) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("raise_safety_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// An audio error used to be raised while the audio state was borrowed, which the
/// longjmp then left borrowed: the next audio call panicked and aborted the program.
#[test]
fn a_caught_audio_error_leaves_audio_usable() {
    let src = r#"fer k in 0..3 {
    hae_a_bash {
        soond_luid("lood")
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether "efter"
"#;
    for refcounting in [false, true] {
        let run = run_native(src, refcounting);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        assert_eq!(lines.len(), 4, "rc={refcounting}: {lines:?}");
        assert_eq!(lines[3], "efter");
        if cfg!(feature = "audio") {
            assert_eq!(lines[0], "soond_luid needs a nummer");
        }
        assert!(
            lines[..3].iter().all(|l| *l == lines[0]),
            "rc={refcounting}: {lines:?}"
        );
    }
}

const TOO_BIG: &str = "Och! That string would be ower muckle tae fit in memory";

/// With the test-only knob of debug builds (runtime/gc_stub.c), every string allocation
/// of 1 MiB or more fails. Formatting a big list (C formatter, raising) and the JSON of a
/// big creel (formatted for the Rust runtime in quiet mode, reported as a status) both
/// used to write through the NULL and crash.
#[cfg(debug_assertions)]
#[test]
fn running_out_of_memory_formatting_raises_the_too_big_error() {
    let src = r#"ken big = []
fer i in 0..200000 {
    shove(big, i)
}
fer k in 0..2 {
    hae_a_bash {
        blether len(tae_string(big))
    } gin_it_gangs_wrang e {
        blether e
    }
    hae_a_bash {
        blether len(json_stringify([creel(big)]))
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether tae_string([1, 2, 3])
"#;
    for refcounting in [false, true] {
        let run = run_native_with_env(src, refcounting, &[("MDH_TEST_ALLOC_FAIL_OVER", "1048576")]);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        assert_eq!(
            lines,
            vec![TOO_BIG, TOO_BIG, TOO_BIG, TOO_BIG, "[1, 2, 3]"],
            "rc={refcounting}"
        );
    }
}

/// Making a string value that cannot get memory used to give a string with a NULL
/// pointer, which the next use dereferenced (MDH-BUG-FLU-00188). Compiled code's runtime
/// calls now raise the "ower muckle" error; the Rust runtime gets a status instead and
/// reports the same error once its own frames are gone. With the test-only knob of debug
/// builds, every string of 1 MiB or more fails: a 2 MiB file read by `slurp` (C runtime)
/// and a 2 MiB `regex_replace` result (Rust runtime).
#[cfg(debug_assertions)]
#[test]
fn running_out_of_memory_making_a_string_raises_the_too_big_error() {
    let dir = tempdir().expect("temp dir");
    let big_path = dir.path().join("big.txt");
    std::fs::write(&big_path, "x".repeat(2 * 1024 * 1024)).expect("write big file");
    let src = format!(
        r#"ken half = "a" * 524288
fer k in 0..2 {{
    hae_a_bash {{
        ken s = slurp("{}")
        blether len(s)
    }} gin_it_gangs_wrang e {{
        blether e
    }}
    hae_a_bash {{
        ken r = regex_replace(half, "a", "aaaa")
        blether len(r)
    }} gin_it_gangs_wrang e {{
        blether e
    }}
}}
blether len(regex_replace("abc", "b", "xyz"))
"#,
        big_path.display()
    );
    for refcounting in [false, true] {
        let run = run_native_with_env(
            &src,
            refcounting,
            &[("MDH_TEST_ALLOC_FAIL_OVER", "1048576")],
        );
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        assert_eq!(
            lines,
            vec![TOO_BIG, TOO_BIG, TOO_BIG, TOO_BIG, "5"],
            "rc={refcounting}"
        );
    }
}

/// `env_get` copies the variable's value with `__mdh_make_string` (C runtime), the path
/// the tests above miss: `slurp` builds its string another way and compiled code copies
/// with `__mdh_obj_strdup_checked`. With every string of 1000 bytes or more failing, the
/// copy of a 1500-byte value raises the "ower muckle" error instead of handing back a
/// string with no text (MDH-BUG-FLU-00188), which printed "(null)".
#[cfg(debug_assertions)]
#[test]
fn env_get_out_of_memory_raises_the_too_big_error() {
    let src = r#"fer k in 0..2 {
    hae_a_bash {
        blether env_get("MDH_BIG")
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether env_get("MDH_WEE")
"#;
    let big = "z".repeat(1500);
    for refcounting in [false, true] {
        let run = run_native_with_env(
            src,
            refcounting,
            &[
                ("MDH_TEST_ALLOC_FAIL_OVER", "1000"),
                ("MDH_BIG", &big),
                ("MDH_WEE", "wee"),
            ],
        );
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        assert_eq!(lines, vec![TOO_BIG, TOO_BIG, "wee"], "rc={refcounting}");
    }
}

const LIST_TOO_BIG: &str = "Och! This list's grown ower muckle";
const DICT_TOO_BIG: &str = "Och! That dictionary would be ower muckle tae fit in memory";

/// Compiled code copied strings with `__mdh_obj_strdup` and no NULL check: getenv's
/// value, split's last token (multi-character delimiter) and the key of a new instance
/// field (MDH-BUG-FLU-00189). With every string of 1000 bytes or more failing, each copy
/// gave a string with no text, which crashed the program. They now raise the "ower
/// muckle" error. The split source is a literal (static, so not allocated).
#[cfg(debug_assertions)]
#[test]
fn compiled_string_copies_raise_the_too_big_error() {
    let field = format!("f{}", "a".repeat(1500));
    let src = format!(
        r#"ken lang = "a::{tail}"
kin Thing {{
    dae init() {{
        masel.{field} = 1
    }}
}}
fer k in 0..2 {{
    hae_a_bash {{
        blether len(getenv("MDH_BIG"))
    }} gin_it_gangs_wrang e {{
        blether e
    }}
    hae_a_bash {{
        ken parts = split(lang, "::")
        blether len(parts[1])
    }} gin_it_gangs_wrang e {{
        blether e
    }}
    hae_a_bash {{
        ken t = Thing()
        blether t.{field}
    }} gin_it_gangs_wrang e {{
        blether e
    }}
}}
blether len(split("a::bc", "::")[1])
"#,
        tail = "y".repeat(1500),
    );
    let big = "z".repeat(1500);
    for refcounting in [false, true] {
        let run = run_native_with_env(
            &src,
            refcounting,
            &[("MDH_TEST_ALLOC_FAIL_OVER", "1000"), ("MDH_BIG", &big)],
        );
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        assert_eq!(
            lines,
            vec![TOO_BIG, TOO_BIG, TOO_BIG, TOO_BIG, TOO_BIG, TOO_BIG, "2"],
            "rc={refcounting}"
        );
    }
}

/// With the knob's buffer form, every list or dict backing array of 1 MiB or more fails
/// (MDH-BUG-FLU-00189): growing a list and a dict raise their "ower muckle" errors, and so
/// does a list made with room for 200000 items, by the C runtime (`skelp`) and by the
/// Rust runtime (`json_parse`). `__mdh_make_list` used to hand back a list with no items
/// array, which the first push wrote through.
#[cfg(debug_assertions)]
#[test]
fn container_buffers_out_of_memory_raise_the_too_big_errors() {
    let src = r#"ken xs = []
ken d = {}
fer k in 0..2 {
    hae_a_bash {
        fer i in 0..200000 {
            shove(xs, i)
        }
        blether len(xs)
    } gin_it_gangs_wrang e {
        blether e
    }
    hae_a_bash {
        fer i in 0..200000 {
            d[i] = i
        }
        blether len(d)
    } gin_it_gangs_wrang e {
        blether e
    }
    hae_a_bash {
        blether len(skelp("x" * 200000, 1))
    } gin_it_gangs_wrang e {
        blether e
    }
    hae_a_bash {
        blether len(json_parse("[" + ("0," * 199999) + "0]"))
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether len(skelp("abcd", 2))
"#;
    for refcounting in [false, true] {
        let run = run_native_with_env(src, refcounting, &[("MDH_TEST_BUF_FAIL_OVER", "1048576")]);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        let once = [LIST_TOO_BIG, DICT_TOO_BIG, TOO_BIG, TOO_BIG];
        let mut expected = [once, once].concat();
        expected.push("2");
        assert_eq!(lines, expected, "rc={refcounting}");
    }
}

/// An empty dict whose 16-byte payload cannot get memory: `__mdh_empty_dict` used to
/// write through the NULL, from the C runtime (`date_now`) and from the Rust runtime
/// (`regex_match`). Both now raise the "ower muckle" error (MDH-BUG-FLU-00189).
#[cfg(debug_assertions)]
#[test]
fn empty_dict_out_of_memory_raises_the_too_big_error() {
    let src = r#"fer k in 0..2 {
    hae_a_bash {
        ken now = date_now()
        blether "date"
    } gin_it_gangs_wrang e {
        blether e
    }
    hae_a_bash {
        ken m = regex_match("abc", "b")
        blether "regex"
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether "efter"
"#;
    for refcounting in [false, true] {
        let run = run_native_with_env(src, refcounting, &[("MDH_TEST_BUF_FAIL_OVER", "16")]);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        assert_eq!(
            lines,
            vec![TOO_BIG, TOO_BIG, TOO_BIG, TOO_BIG, "efter"],
            "rc={refcounting}"
        );
    }
}

/// `slurp`, `lines` and `words` copied their text through a temporary buffer that they
/// never checked (or freed): with every buffer of 1 MiB or more failing, a 2 MiB file or
/// line wrote through NULL. They now build the string directly, so they need no buffer
/// at all and succeed (MDH-BUG-FLU-00189).
#[cfg(debug_assertions)]
#[test]
fn slurp_lines_and_words_need_no_temporary_buffer() {
    let dir = tempdir().expect("temp dir");
    let big_path = dir.path().join("big.txt");
    std::fs::write(&big_path, "x".repeat(2 * 1024 * 1024)).expect("write big file");
    let src = format!(
        r#"ken big = "x" * 2097152
fer k in 0..2 {{
    blether len(slurp("{}"))
    blether len(lines(big)[0])
    blether len(words(big)[0])
}}
"#,
        big_path.display()
    );
    for refcounting in [false, true] {
        let run = run_native_with_env(&src, refcounting, &[("MDH_TEST_BUF_FAIL_OVER", "1048576")]);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        let once = ["2097152", "2097152", "2097152"];
        assert_eq!(lines, [once, once].concat(), "rc={refcounting}");
    }
}

/// Compiled code's own allocations had no NULL check (MDH-BUG-FLU-00193): with every
/// object of 1000 bytes or more failing, split's single-character path, a string
/// concatenation and a string grown in place by `s = s + "x"` wrote through NULL and
/// crashed (139). They now call the checked allocators, which raise the "ower muckle"
/// error. The split source is a literal (static, so not allocated).
#[cfg(debug_assertions)]
#[test]
fn compiled_object_allocations_raise_the_too_big_error() {
    let src = format!(
        r#"ken lang = "a,{tail}"
ken half = "{half}"
dae grow(n) {{
    ken s = ""
    fer i in 0..n {{
        s = s + "x"
    }}
    gie s
}}
fer k in 0..2 {{
    hae_a_bash {{
        ken parts = split(lang, ",")
        blether len(parts[1])
    }} gin_it_gangs_wrang e {{
        blether e
    }}
    hae_a_bash {{
        ken s = half + half
        blether len(s)
    }} gin_it_gangs_wrang e {{
        blether e
    }}
    hae_a_bash {{
        blether len(grow(1500))
    }} gin_it_gangs_wrang e {{
        blether e
    }}
}}
blether len(split("a,bc", ",")[1]) + len(half + "y") + len(grow(3))
"#,
        tail = "y".repeat(1500),
        half = "h".repeat(600),
    );
    for refcounting in [false, true] {
        let run = run_native_with_env(&src, refcounting, &[("MDH_TEST_ALLOC_FAIL_OVER", "1000")]);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        let once = [TOO_BIG, TOO_BIG, TOO_BIG];
        let mut want = [once, once].concat();
        want.push("606");
        assert_eq!(lines, want, "rc={refcounting}");
    }
}

/// The same for compiled code's buffers (MDH-BUG-FLU-00193): with every buffer of 1000
/// bytes or more failing, a 100-item list literal, a 100-entry dict literal and an
/// instance growing to 40 fields wrote through NULL and crashed; they now raise.
#[cfg(debug_assertions)]
#[test]
fn compiled_buffer_allocations_raise_the_too_big_error() {
    let items: Vec<String> = (0..100).map(|i| i.to_string()).collect();
    let entries: Vec<String> = (0..100).map(|i| format!("\"k{i}\": {i}")).collect();
    let fields: Vec<String> = (0..40)
        .map(|i| format!("        masel.f{i} = {i}"))
        .collect();
    let src = format!(
        r#"kin Big {{
    dae init() {{
{fields}
    }}
}}
fer k in 0..2 {{
    hae_a_bash {{
        ken xs = [{items}]
        blether len(xs)
    }} gin_it_gangs_wrang e {{
        blether e
    }}
    hae_a_bash {{
        ken d = {{{entries}}}
        blether len(d)
    }} gin_it_gangs_wrang e {{
        blether e
    }}
    hae_a_bash {{
        ken b = Big()
        blether b.f39
    }} gin_it_gangs_wrang e {{
        blether e
    }}
}}
blether len([1, 2, 3]) + len({{"a": 1}})
"#,
        fields = fields.join("\n"),
        items = items.join(", "),
        entries = entries.join(", "),
    );
    for refcounting in [false, true] {
        let run = run_native_with_env(&src, refcounting, &[("MDH_TEST_BUF_FAIL_OVER", "1000")]);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        let once = [TOO_BIG, TOO_BIG, TOO_BIG];
        let mut want = [once, once].concat();
        want.push("4");
        assert_eq!(lines, want, "rc={refcounting}");
    }
}
