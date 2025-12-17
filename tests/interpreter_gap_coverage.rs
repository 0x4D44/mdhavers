use std::fs;

use mdhavers::{parse, Interpreter};

#[test]
fn interpreter_can_fetch_modules_with_and_without_alias() {
    let dir = tempfile::tempdir().unwrap();
    let module_path = dir.path().join("mymod.braw");
    fs::write(
        &module_path,
        r#"
ken a = 10
dae f() { gie 32 }
"#,
    )
    .unwrap();

    // Import with alias (namespace dict).
    {
        let code = r#"
fetch "mymod" tae m
blether m["a"]
blether m["f"]()
"#;
        let program = parse(code).unwrap();
        let mut interp = Interpreter::new();
        interp.set_current_dir(dir.path());
        interp.interpret(&program).unwrap();
        let out = interp.get_output().join("\n");
        assert_eq!(out.trim(), "10\n32");
    }

    // Import without alias (bring all exports into current environment).
    {
        let code = r#"
fetch "mymod"
blether a
blether f()
"#;
        let program = parse(code).unwrap();
        let mut interp = Interpreter::new();
        interp.set_current_dir(dir.path());
        interp.interpret(&program).unwrap();
        let out = interp.get_output().join("\n");
        assert_eq!(out.trim(), "10\n32");
    }

    // Importing the same module twice should be a no-op (loaded_modules guard).
    {
        let code = r#"
fetch "mymod"
fetch "mymod"
blether a
"#;
        let program = parse(code).unwrap();
        let mut interp = Interpreter::new();
        interp.set_current_dir(dir.path());
        interp.interpret(&program).unwrap();
        let out = interp.get_output().join("\n");
        assert_eq!(out.trim(), "10");
    }
}

#[test]
fn interpreter_trace_match_assert_and_destructure_paths() {
    use mdhavers::interpreter::TraceMode;

    let code = r#"
ken sum = 0
fer i in 1..4 {
    sum = sum + i
}

dae add(a, b = 2) {
    gie a + b
}

blether add(3)

hae_a_bash {
    mak_siccar nae, "forced failure"
    blether "unreachable"
} gin_it_gangs_wrang e {
    blether "caught"
}

ken x = 2
keek x {
    whan 1 -> { blether "one" }
    whan 2 -> { blether "two" }
    whan _ -> { blether "other" }
}

hae_a_bash {
    keek 99 {
        whan 1 -> { blether "nope" }
    }
} gin_it_gangs_wrang e {
    blether "no match"
}

ken [a, b, ...rest] = [1, 2, 3, 4]
blether a
blether b
blether len(rest)
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    // Enable tracing to hit otherwise-cold trace/trace_verbose paths.
    interp.set_trace_mode(TraceMode::Verbose);
    interp.interpret(&program).unwrap();

    let out = interp.get_output().join("\n");
    // add(3) => 5, then caught, then two, then no match, then 1/2/len(rest)=2
    assert_eq!(out.trim(), "5\ncaught\ntwo\nno match\n1\n2\n2");
}

#[test]
fn interpreter_public_helpers_stacktrace_and_log_level() {
    use mdhavers::interpreter::{
        clear_stack_trace, get_global_log_level, get_stack_trace, print_stack_trace,
        push_stack_frame, set_crash_handling, set_global_log_level,
    };
    use mdhavers::ast::LogLevel;

    // Empty stack trace path.
    clear_stack_trace();
    assert!(get_stack_trace().is_empty());
    print_stack_trace();

    // Non-empty stack trace path.
    set_crash_handling(true);
    assert!(mdhavers::interpreter::is_crash_handling_enabled());
    push_stack_frame("test_frame", 123);
    assert!(!get_stack_trace().is_empty());
    print_stack_trace();
    set_crash_handling(false);
    assert!(!mdhavers::interpreter::is_crash_handling_enabled());

    // Cover log level encode/decode paths.
    for lvl in [
        LogLevel::Wheesht,
        LogLevel::Roar,
        LogLevel::Holler,
        LogLevel::Blether,
        LogLevel::Mutter,
        LogLevel::Whisper,
    ] {
        set_global_log_level(lvl);
        assert_eq!(get_global_log_level(), lvl);
    }

    // Cover Interpreter's per-instance log level helpers (distinct from global log level).
    let mut interp = Interpreter::new();
    interp.set_log_level(LogLevel::Mutter);
    assert_eq!(interp.get_log_level(), LogLevel::Mutter);
    interp.set_log_level(LogLevel::Whisper);
    assert_eq!(interp.get_log_level(), LogLevel::Whisper);
}

#[test]
fn interpreter_clear_output_resets_buffer() {
    let program = parse("blether 123").unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    assert!(!interp.get_output().is_empty());
    interp.clear_output();
    assert!(interp.get_output().is_empty());
}

#[test]
fn interpreter_builtin_edges_and_errors_smoke() {
    // Table-driven sweep intended to exercise a wide variety of native builtin branches,
    // including type errors and corner cases that are otherwise rarely hit.
    let cases: &[(&str, bool)] = &[
        // arithmetic: mixed numeric types + error paths
        ("1 - 2.0", true),
        ("1.0 - 2", true),
        ("\"a\" - 1", false),
        ("\"ha\" * 3", true),
        ("1 / 0", false),
        ("1.0 / 0.0", false),
        // len: all supported types + error path
        ("len(\"abcd\")", true),
        ("len([1, 2, 3])", true),
        ("len({\"a\": 1, \"b\": 2})", true),
        ("len(empty_creel())", true),
        ("len(naething)", false),
        // log level helpers
        ("set_log_level(\"blether\")", true),
        ("get_log_level()", true),
        ("set_log_level(0)", true),
        ("set_log_level(5)", true),
        ("set_log_level(99)", false),
        // math numeric coercions + error paths
        ("abs(-5)", true),
        ("abs(-5.5)", true),
        ("abs(\"nae\")", false),
        ("sqrt(9)", true),
        ("sqrt(9.0)", true),
        ("sqrt(\"nae\")", false),
        ("floor(3.9)", true),
        ("ceil(3.1)", true),
        ("round(3.5)", true),
        ("floor(\"nae\")", false),
        // bitwise ops
        ("bit_an(6, 3)", true),
        ("bit_or(6, 3)", true),
        ("bit_xor(6, 3)", true),
        ("bit_an(6, \"3\")", false),
        // json
        (r#"json_parse("{\"a\": 1, \"b\": [2, 3]}")"#, true),
        ("json_stringify({\"a\": 1})", true),
        ("json_pretty({\"a\": 1})", true),
        ("json_parse(123)", false),
        // sets/creels
        ("creel([1, 1, 2, 3])", true),
        ("toss_in(empty_creel(), 1)", true),
        ("heave_oot(creel([1, 2, 3]), 2)", true),
        ("is_in_creel(creel([\"a\", \"b\"]), \"a\")", true),
        ("creels_thegither(creel([1]), creel([2]))", true),
        ("creels_baith(creel([1, 2]), creel([2, 3]))", true),
        ("creels_differ(creel([1, 2]), creel([2, 3]))", true),
        ("creel_tae_list(creel([3, 1, 2]))", true),
        ("toss_in([1, 2], 3)", false),
        // strings
        ("upper(\"hello\")", true),
        ("lower(\"HELLO\")", true),
        ("trim(\"  hi  \")", true),
        ("starts_with(\"hello\", \"he\")", true),
        ("ends_with(\"hello\", \"lo\")", true),
        ("split(\"a,b,c\", \",\")", true),
        ("join([\"a\", \"b\"], \",\")", true),
        ("replace(\"hello\", \"l\", \"L\")", true),
        ("index_of(\"hello\", \"lo\")", true),
        ("contains(\"hello\", \"z\")", true),
        ("upper(123)", false),
        ("is_empty(\"\")", true),
        ("is_empty([])", true),
        ("is_empty({})", true),
        ("is_empty(1)", false),
        ("is_blank(\"  \\n\\t\")", true),
        ("is_blank(1)", false),
        // lists
        ("slap([1, 2], [3])", true),
        ("shove([1, 2], 3)", true),
        ("yank([1, 2, 3])", true),
        ("yank([])", false),
        ("sumaw([1, 2, 3])", true),
        ("sumaw([1.0, 2.0])", true),
        ("sumaw([1, \"2\"])", false),
        ("minaw([3, 1, 2])", true),
        ("maxaw([3.0, 1.0, 2.0])", true),
        ("drap([1, 2, 3], 2)", true),
        ("tak([1, 2, 3], 2)", true),
        ("grup([1, 2, 3, 4, 5], 2)", true),
        ("grup([1, 2], 0)", false),
        ("pair_up([1, 2, 3, 4])", true),
        ("fankle([1, 2], [3, 4, 5])", true),
        ("birl([1, 2, 3, 4], 1)", true),
        ("birl([1, 2, 3, 4], -1)", true),
        ("skelp(\"abcdef\", 2)", true),
        ("skelp(\"a\", 0)", false),
        ("indices_o([1, 2, 1, 3], 1)", true),
        ("indices_o(\"banana\", \"na\")", true),
        ("indices_o(\"banana\", \"\")", false),
        ("indices_o(1, 2)", false),
        // dicts
        ("keys({\"a\": 1, \"b\": 2})", true),
        ("values({\"a\": 1, \"b\": 2})", true),
        ("items({\"a\": 1})", true),
        ("keys([1, 2])", false),
        // debug helpers
        ("clype([1, 2, 3])", true),
        ("clype({\"a\": 1})", true),
        ("clype(creel([1, 2, 3]))", true),
        ("clype(\"hi\")", true),
        ("clype(123)", true),
        ("clype(aye)", true),
        ("clype(naething)", true),
        ("stooshie(\"abcd\")", true),
        ("stooshie(123)", false),
        // randomness (non-deterministic values; just cover branches)
        ("random()", true),
        ("random_int(0, 0)", true),
        ("random_int(0, 10)", true),
        ("random_int(10, 0)", false),
        ("random_choice([1, 2, 3])", true),
        ("random_choice([])", true),
        ("random_choice(123)", false),
        // date formatting
        ("braw_date(0)", true),
        ("braw_date(naething)", true),
        ("braw_date(\"no\")", false),
        // date/time helpers (stdlib expansion)
        ("date_now()", true),
        ("date_format(0, \"%Y-%m-%d\")", true),
        ("date_parse(\"2020-01-02 03:04:05\", \"%Y-%m-%d %H:%M:%S\")", true),
        ("date_add(0, 1, \"seconds\")", true),
        ("date_add(0, 1, \"minutes\")", true),
        ("date_add(0, 1, \"hours\")", true),
        ("date_add(0, 1, \"days\")", true),
        ("date_add(0, 1, \"weeks\")", true),
        ("date_add(0, 1, \"fortnights\")", false),
        ("date_diff(0, 1000, \"milliseconds\")", true),
        ("date_diff(0, 1000, \"seconds\")", true),
        ("date_diff(0, 1000, \"minutes\")", true),
        ("date_diff(0, 1000, \"hours\")", true),
        ("date_diff(0, 1000, \"days\")", true),
        ("date_diff(0, 1000, \"weeks\")", true),
        ("date_diff(0, 1000, \"nae\")", false),
        // number properties
        ("is_even(2)", true),
        ("is_odd(3)", true),
        ("is_prime(1)", true),
        ("is_prime(2)", true),
        ("is_prime(9)", true),
        ("is_prime(\"nae\")", false),
        // regex helpers
        ("regex_test(\"hello\", \"h.*o\")", true),
        ("regex_test(\"hello\", \"*\")", false),
        ("regex_match(\"abc123\", \"[0-9]+\")", true),
        ("regex_match(\"abc\", \"[0-9]+\")", true),
        ("regex_match_all(\"abc123def456\", \"[0-9]+\")", true),
        ("regex_replace(\"a1b2\", \"[0-9]\", \"\")", true),
        ("regex_replace_first(\"a1b2\", \"[0-9]\", \"\")", true),
        // timing helpers
        ("noo()", true),
        ("tick()", true),
        ("bide(0)", true),
        (
            r#"
dae foo() { gie 1 }
stopwatch(foo)
"#,
            true,
        ),
        (
            r#"
log_blether "hi"
log_whisper "shh"
"#,
            true,
        ),
        // env/shell helpers (keep safe + portable)
        ("env_get(\"MDH_TEST_NO_SUCH_VAR\")", true),
        ("env_all()", true),
        ("shell(\"echo hello\")", true),
        ("shell_status(\"exit 0\")", true),
        ("args()", true),
        ("cwd()", true),
        // Scots-y string/utility helpers
        ("jings(\"wow\")", true),
        ("crivvens(\"wow\")", true),
        ("help_ma_boab(\"wow\")", true),
        ("braw(naething)", true),
        ("braw(aye)", true),
        ("braw(0)", true),
        ("braw(1)", true),
        ("braw(0.0)", true),
        ("braw(1.0)", true),
        ("braw(\"\")", true),
        ("braw(\"x\")", true),
        ("braw([])", true),
        ("braw([1])", true),
        ("braw({})", true),
        ("braw({\"a\": 1})", true),
        ("clarty([1, 2, 1])", true),
        ("clarty([1, 2, 3])", true),
        ("clarty(\"aba\")", true),
        ("clarty(\"abc\")", true),
        ("clarty(1)", false),
        ("dreich(\"\")", true),
        ("dreich(\"aaaa\")", true),
        ("dreich(\"ab\")", true),
        ("dreich(1)", false),
        ("scottify(\"hello\")", true),
        ("unique([1, 2, 1, 3])", true),
        ("unique(1)", false),
        ("haver()", true),
        ("slainte()", true),
        ("braw_time()", true),
        // JSON parsing: exercise escape handling
        (r#"json_parse("{\"s\":\"a\\nb\"}")"#, true),
        (r#"json_parse("{\"s\":\"a\\tb\"}")"#, true),
        (r#"json_parse("{\"s\":\"a\\rb\"}")"#, true),
        (r#"json_parse("{\"s\":\"a\\/b\"}")"#, true),
        (r#"json_parse("{\"s\":\"\\u0041\"}")"#, true),
        (r#"json_parse("{\"s\":\"\\u12\"}")"#, false),
        // Dict indexing: missing key error path
        (
            r#"
ken d = {"a": 1}
d["b"]
"#,
            false,
        ),
        // Higher-order builtins (gaun/sieve/tumble/ilk/hunt/ony/aw/grup_up/pairt_by)
        (
            r#"
ken xs = [1, 2, 3, 4]
ken ys = gaun(xs, |x| x + 1)
ken zs = sieve(ys, |x| x > 3)
ken sum = tumble(zs, 0, |a, b| a + b)
ilk(zs, |x| { blether x })
blether hunt(zs, |x| x == 4)
blether ony(zs, |x| x == 999)
blether aw(zs, |x| x > 0)
ken g = grup_up(xs, |x| x % 2)
blether len(keys(g))
ken p = pairt_by(xs, |x| x % 2 == 0)
blether len(p)
"#,
            true,
        ),
        // operator overloading paths (instance + binary ops)
        (
            r#"
kin Ops {
    dae __pit_thegither__(o) { gie "add" }
    dae __tak_awa__(o) { gie "sub" }
    dae __times__(o) { gie "mul" }
    dae __pairt__(o) { gie "div" }
    dae __lave__(o) { gie "mod" }
    dae __same_as__(o) { gie aye }
    dae __differs_fae__(o) { gie aye }
    dae __wee_er__(o) { gie aye }
    dae __wee_er_or_same__(o) { gie aye }
    dae __muckle_er__(o) { gie aye }
    dae __muckle_er_or_same__(o) { gie aye }
}
ken a = Ops()
a + 1
a - 1
a * 1
a / 1
a % 1
a == 1
a != 1
a < 1
a <= 1
a > 1
a >= 1
"#,
            true,
        ),
        (
            r#"
kin BadOps {
    dae __pit_thegither__(a, b) { gie 1 }
}
ken b = BadOps()
b + 1
"#,
            false,
        ),
        // match patterns: identifier binding and range
        (
            r#"
ken v = 5
keek v {
    whan x -> { x }
}
"#,
            true,
        ),
        (
            r#"
ken v = 5
keek v {
    whan 1..10 -> { 1 }
    whan _ -> { 0 }
}
"#,
            true,
        ),
        // control flow: break/continue error paths
        ("brak", false),
        ("haud", false),
        ("haud_yer_wheesht", false),
        // match failure path (uncaught)
        (
            r#"
keek 99 {
    whan 1 -> { 1 }
}
"#,
            false,
        ),
    ];

    for (src, should_succeed) in cases {
        let program = parse(src).unwrap_or_else(|e| panic!("parse failed for:\n{src}\nerr={e:?}"));
        let mut interp = Interpreter::new();
        let result = interp.interpret(&program);
        if *should_succeed {
            assert!(result.is_ok(), "expected success for:\n{src}\nerr={result:?}");
        } else {
            assert!(result.is_err(), "expected error for:\n{src}");
        }
    }
}

#[test]
fn interpreter_file_io_builtins_smoke() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("io_test.txt");

    let code = format!(
        r#"
scrieve("{p}", "hello\nworld")
append_file("{p}", "\n!")
blether file_exists("{p}")
blether read_file("{p}")
ken lines = read_lines("{p}")
blether len(lines)
"#,
        p = path.display()
    );

    let program = parse(&code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();

    let out = interp.get_output().join("\n");
    assert!(out.contains("aye"));
    assert!(out.contains("hello"));
    assert!(out.contains("world"));
    assert!(out.contains("!"));
    assert!(out.contains('\n'));
}
