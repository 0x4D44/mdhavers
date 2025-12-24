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
    use mdhavers::ast::LogLevel;
    use mdhavers::interpreter::{
        clear_stack_trace, get_global_log_level, get_stack_trace, print_stack_trace,
        push_stack_frame, set_crash_handling, set_global_log_level,
    };

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
        ("1.0 + 2", true),
        ("\"a\" + 1", true),
        ("1 + \"a\"", true),
        ("[] + 1", false),
        ("1 - 2.0", true),
        ("1.0 - 2", true),
        ("\"a\" - 1", false),
        ("\"ha\" * 3", true),
        ("1 * 2.0", true),
        ("2.0 * 1", true),
        ("1 / 0", false),
        ("1.0 / 0.0", false),
        ("1 / 2.0", true),
        ("2.0 / 1", true),
        ("1.0 < 2.0", true),
        ("1 < 2.0", true),
        ("1.0 < 2", true),
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
        ("set_log_level(1)", true),
        ("set_log_level(2)", true),
        ("set_log_level(3)", true),
        ("set_log_level(4)", true),
        ("set_log_level(5)", true),
        ("set_log_level(99)", false),
        // structured logging natives
        ("log_enabled(\"blether\")", true),
        ("log_enabled(\"blether\", \"\")", true),
        ("log_enabled()", false),
        ("log_enabled(\"blether\", \"\", \"extra\")", false),
        ("log_enabled(\"blether\", 1)", false),
        ("log_event(\"blether\")", false),
        ("log_event(\"blether\", \"hi\")", true),
        ("log_event(\"blether\", \"hi\", 1)", false),
        ("log_event(\"blether\", \"hi\", {\"a\": 1}, 1)", false),
        ("log_event(\"blether\", \"hi\", {1: 2})", false),
        ("log_set_filter(1)", false),
        ("log_set_filter(\"mutter\")", true),
        ("log_init(1)", false),
        ("log_init({\"level\": 1})", true),
        ("log_init({\"level\": 4})", true),
        ("log_init({\"filter\": 1})", false),
        ("log_init({\"filter\": \"mutter\"})", true),
        ("log_init({\"format\": 1})", false),
        ("log_init({\"format\": \"nae\"})", false),
        ("log_init({\"color\": 1})", false),
        ("log_init({\"timestamps\": 1})", false),
        ("log_init({\"sinks\": 1})", false),
        ("log_init({\"sinks\": [1]})", false),
        ("log_init({\"sinks\": [{\"kind\": 1}]})", false),
        ("log_init({\"sinks\": [{\"kind\": \"callback\"}]})", false),
        // defaults: file.append and memory.max
        ("log_init({\"sinks\": [{\"kind\": \"file\", \"path\": \"mdh_test.log\"}]})", true),
        ("log_init({\"sinks\": [{\"kind\": \"memory\"}]})", true),
        // reset logger back to defaults (config omitted)
        ("log_init()", true),
        // stacktrace builtin: empty + non-empty path
        ("stacktrace()", true),
        (
            r#"
dae f() { gie stacktrace() }
f()
"#,
            true,
        ),
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
        // exponent helpers
        ("pooer(2.0, 3)", true),
        ("pooer(2, 3.0)", true),
        ("pow(2, \"x\")", false),
        ("atan2(1, \"x\")", false),
        ("hypot(1, \"x\")", false),
        // bitwise ops
        ("bit_an(6, 3)", true),
        ("bit_or(6, 3)", true),
        ("bit_xor(6, 3)", true),
        ("bit_an(6, \"3\")", false),
        ("bit_shove_left(1, 64)", false),
        ("bit_shove_right(1, 64)", false),
        // json
        (r#"json_parse("{\"a\": 1, \"b\": [2, 3]}")"#, true),
        ("json_stringify({\"a\": 1})", true),
        ("json_pretty({\"a\": 1})", true),
        ("json_stringify_pretty({\"a\": 1})", true),
        ("json_parse(123)", false),
        // atomics/channels: type errors and edge cases
        ("atomic_store(atomic_new(1), \"x\")", false),
        ("atomic_add(atomic_new(1), \"x\")", false),
        ("atomic_cas(atomic_new(1), \"x\", 2)", false),
        ("atomic_cas(atomic_new(1), 1, \"x\")", false),
        ("chan_new(-1)", false),
        (
            r#"
ken ch = chan_new(0)
chan_close(ch)
chan_send(ch, 1)
"#,
            true,
        ),
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
        ("contains(\"abc\", 1)", false),
        ("contains({\"a\": 1}, 1)", true),
        ("upper(123)", false),
        ("capitalize(\"\")", true),
        ("title(\"hello world\")", true),
        ("substring(\"abc\", 0, 99)", true),
        ("substring(\"abc\", 99, 99)", true),
        ("is_empty(\"\")", true),
        ("is_empty([])", true),
        ("is_empty({})", true),
        ("is_empty(1)", false),
        ("is_blank(\"  \\n\\t\")", true),
        ("is_blank(1)", false),
        // lists
        ("sort([3, 1, 2])", true),
        ("sort([1, \"a\", 2])", true),
        ("slap([1, 2], [3])", true),
        ("tail([])", true),
        ("shove([1, 2], 3)", true),
        ("yank([1, 2, 3])", true),
        ("yank([])", false),
        ("sumaw([1, 2, 3])", true),
        ("sumaw([1.0, 2.0])", true),
        ("sumaw([1, \"2\"])", false),
        ("product([])", true),
        ("product([1, \"2\"])", false),
        ("average([])", false),
        ("average([1, \"2\"])", false),
        ("median([])", false),
        ("median([1, \"2\"])", false),
        ("minaw([3, 1, 2])", true),
        ("minaw([])", false),
        ("minaw([1, 2.0])", false),
        ("minaw([3.0, 1.0, 2.0])", true),
        ("maxaw([3, 1, 2])", true),
        ("maxaw([])", false),
        ("maxaw([1, 2.0])", false),
        ("maxaw([3.0, 1.0, 2.0])", true),
        ("range_o([1, 2, 3])", true),
        ("range_o([])", false),
        ("range_o([1, \"2\"])", false),
        ("drap([1, 2, 3], 2)", true),
        ("tak([1, 2, 3], 2)", true),
        ("grup([1, 2, 3, 4, 5], 2)", true),
        ("grup([1, 2], 0)", false),
        ("pair_up([1, 2, 3, 4])", true),
        ("fankle([1, 2], [3, 4, 5])", true),
        ("fankle([1, 2, 3], [4])", true),
        ("birl([1, 2, 3, 4], 1)", true),
        ("birl([1, 2, 3, 4], -1)", true),
        ("birl([], 1)", true),
        ("birl([1], \"x\")", false),
        ("skelp(\"abcdef\", 2)", true),
        ("skelp(\"a\", 0)", false),
        ("indices_o([1, 2, 1, 3], 1)", true),
        ("indices_o(\"banana\", \"na\")", true),
        ("indices_o(\"banana\", \"\")", false),
        ("indices_o(\"banana\", 1)", false),
        ("indices_o(1, 2)", false),
        // dicts
        ("keys({\"a\": 1, \"b\": 2})", true),
        ("values({\"a\": 1, \"b\": 2})", true),
        ("items({\"a\": 1})", true),
        ("dict_get({\"a\": 1}, 1, 0)", true),
        ("dict_has({\"a\": 1}, 1)", true),
        ("dict_remove({\"a\": 1}, 1)", true),
        (
            r#"
ken d = {1: 2}
d["1"]
"#,
            false,
        ),
        ("keys([1, 2])", false),
        // debug helpers
        ("clype([1, 2, 3])", true),
        ("clype({\"a\": 1})", true),
        ("clype(creel([1, 2, 3]))", true),
        ("clype(\"hi\")", true),
        ("clype(123)", true),
        ("clype(aye)", true),
        ("clype(naething)", true),
        ("clype(1..3)", true),
        ("clype(bytes_from_string(\"hi\"))", true),
        ("stooshie(\"abcd\")", true),
        ("stooshie(123)", false),
        ("blether_format(\"hi {1}\", {1: \"x\"})", true),
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
        ("braw_date(86400)", true),
        ("braw_date(172800)", true),
        ("braw_date(259200)", true),
        ("braw_date(63072000)", true),
        ("braw_date(naething)", true),
        ("braw_date(\"no\")", false),
        // interpreter module resolution: lib/ prefix and absolute path error branch
        (
            r#"
fetch "lib/colors" tae c
c["FG_RED"]
"#,
            true,
        ),
        (r#"fetch "/definitely/not/a/real/module/path""#, false),
        // date/time helpers (stdlib expansion)
        ("date_now()", true),
        ("date_format(0, \"%Y-%m-%d\")", true),
        ("date_format(0, 1)", false),
        (
            "date_parse(\"2020-01-02 03:04:05\", \"%Y-%m-%d %H:%M:%S\")",
            true,
        ),
        ("date_parse(\"2020-01-02\", 1)", false),
        ("date_add(0, 1, \"seconds\")", true),
        ("date_add(0, 1, \"minutes\")", true),
        ("date_add(0, 1, \"hours\")", true),
        ("date_add(0, 1, \"days\")", true),
        ("date_add(0, 1, \"weeks\")", true),
        ("date_add(0, 1, 1)", false),
        ("date_add(0, 1, \"fortnights\")", false),
        ("date_diff(0, 1000, \"milliseconds\")", true),
        ("date_diff(0, 1000, \"seconds\")", true),
        ("date_diff(0, 1000, \"minutes\")", true),
        ("date_diff(0, 1000, \"hours\")", true),
        ("date_diff(0, 1000, \"days\")", true),
        ("date_diff(0, 1000, \"weeks\")", true),
        ("date_diff(0, 1000, 1)", false),
        ("date_diff(0, 1000, \"nae\")", false),
        // number properties
        ("is_even(2)", true),
        ("is_odd(3)", true),
        ("is_prime(1)", true),
        ("is_prime(2)", true),
        ("is_prime(4)", true),
        ("is_prime(9)", true),
        ("is_prime(11)", true),
        ("is_prime(\"nae\")", false),
        // regex helpers
        ("regex_test(\"hello\", \"h.*o\")", true),
        ("regex_test(\"hello\", \"*\")", false),
        ("regex_test(\"hello\", 1)", false),
        ("regex_match(\"abc123\", \"[0-9]+\")", true),
        ("regex_match(\"abc\", \"[0-9]+\")", true),
        ("regex_match(\"abc\", 1)", false),
        ("regex_match_all(\"abc123def456\", \"[0-9]+\")", true),
        ("regex_match_all(\"abc\", 1)", false),
        ("regex_replace(\"a1b2\", \"[0-9]\", \"\")", true),
        ("regex_replace(\"a1b2\", \"[0-9]\", 1)", false),
        ("regex_replace_first(\"a1b2\", \"[0-9]\", \"\")", true),
        ("regex_replace_first(\"a1b2\", \"[0-9]\", 1)", false),
        ("regex_split(\"a1b2\", 1)", false),
        // timing helpers
        ("noo()", true),
        ("tick()", true),
        ("snooze(-1)", false),
        ("bide(0)", true),
        ("bide(0.0)", true),
        ("bide(\"x\")", false),
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
        ("env_get(1)", false),
        ("env_set(\"MDH_TEST_ENV_SET\", 123)", true),
        ("env_all()", true),
        ("shell(\"echo hello\")", true),
        ("shell(\"echo hello 1>&2\")", true),
        ("shell(1)", false),
        ("shell_status(\"exit 0\")", true),
        ("shell_status(1)", false),
        ("args()", true),
        ("cwd()", true),
        ("path_join(\"a\", 1)", false),
        ("scrieve(1, \"hi\")", false),
        ("read_file(1)", false),
        ("read_lines(1)", false),
        ("file_exists(1)", false),
        ("append_file(1, \"hi\")", false),
        // assertions
        ("assert(nae, \"nope\")", false),
        ("assert(nae, 123)", false),
        ("assert_equal(1, 1)", true),
        ("assert_nae_equal(1, 1)", false),
        // Scots-y string/utility helpers
        ("jings(\"wow\")", true),
        ("crivvens(\"wow\")", true),
        ("help_ma_boab(\"wow\")", true),
        ("numpty_check([])", true),
        ("stoater([1, \"abc\", 2])", true),
        ("banter(\"ab\", \"cdef\")", true),
        ("banter(\"abcd\", \"e\")", true),
        ("banter(\"ab\", 1)", false),
        ("haggis_hunt(\"banana\", \"na\")", true),
        ("haggis_hunt(\"banana\", 1)", false),
        ("sporran_fill(\"x\", 3, 1)", false),
        ("center(\"x\", 3, 1)", false),
        ("swapcase(\"a1\")", true),
        ("substr_between(\"a[bc\", \"[\", \"]\")", true),
        ("sign(0.0)", true),
        ("-\"x\"", false),
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
            assert!(
                result.is_ok(),
                "expected success for:\n{src}\nerr={result:?}"
            );
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
scrieve_append("{p}", 123)
blether file_exists("{p}")
blether read_file("{p}")
ken lines = read_lines("{p}")
blether len(lines)
blether file_size("{p}")
ken entries = list_dir("{d}")
blether len(entries)
make_dir("{d}/subdir")
blether is_dir("{d}/subdir")
blether path_join("{d}", "io_test.txt")
file_delete("{p}")
blether file_exists("{p}")
"#,
        p = path.display(),
        d = dir.path().display(),
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

#[test]
fn interpreter_prelude_load_paths_for_coverage() {
    // 1) No prelude found -> Ok path
    let mut interp = Interpreter::new();
    assert!(!interp.has_prelude());
    interp.load_prelude().unwrap();
    assert!(interp.has_prelude());

    // 2) Already-loaded guard -> early return path
    interp.load_prelude().unwrap();

    // 3) Prelude found but has syntax error -> parse error path
    let dir = tempfile::tempdir().unwrap();
    let stdlib = dir.path().join("stdlib");
    std::fs::create_dir_all(&stdlib).unwrap();
    std::fs::write(stdlib.join("prelude.braw"), "ken =\n").unwrap();

    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .load_prelude()
        .expect_err("expected prelude parse error");
    std::env::set_current_dir(&old_cwd).unwrap();

    let err_str = format!("{err:?}");
    assert!(
        err_str.contains("Prelude"),
        "expected prelude error, got: {err_str}"
    );

    // 4) No prelude file found -> Ok path (fallback)
    let empty_dir = tempfile::tempdir().unwrap();
    std::env::set_current_dir(empty_dir.path()).unwrap();
    let mut interp = Interpreter::new();
    assert!(!interp.has_prelude());
    interp.load_prelude().unwrap();
    assert!(interp.has_prelude());
    std::env::set_current_dir(&old_cwd).unwrap();
}

#[test]
fn interpreter_fetch_module_error_paths_for_coverage() {
    let dir = tempfile::tempdir().unwrap();

    // 1) Path resolves but read_to_string fails (directory with .braw extension).
    std::fs::create_dir_all(dir.path().join("badmod.braw")).unwrap();
    {
        let program = parse("fetch \"badmod\"").unwrap();
        let mut interp = Interpreter::new();
        interp.set_current_dir(dir.path());
        assert!(interp.interpret(&program).is_err());
    }

    // 2) Module parses with error -> ParseError mapping.
    std::fs::write(dir.path().join("syntaxmod.braw"), "ken =\n").unwrap();
    {
        let program = parse("fetch \"syntaxmod\"").unwrap();
        let mut interp = Interpreter::new();
        interp.set_current_dir(dir.path());
        let err = interp
            .interpret(&program)
            .expect_err("expected module parse error");
        let err_str = format!("{err:?}");
        assert!(
            err_str.contains("Error in module"),
            "expected module error, got: {err_str}"
        );
    }
}

#[test]
fn interpreter_additional_error_paths_for_coverage() {
    let cases: &[(&str, bool)] = &[
        // Optional-arity function error path (min!=max)
        (
            r#"
dae add(a, b = 1) { gie a + b }
add()
"#,
            false,
        ),
        (
            r#"
dae add(a, b = 1) { gie a + b }
add(1, 2, 3)
"#,
            false,
        ),
        // NativeFunction wrong arity path
        ("len()", false),
        ("len([1], 2)", false),
        // Spread operator in call args: non-list spread -> type error
        (
            r#"
dae f(x) { gie x }
f(...1)
"#,
            false,
        ),
        // Instance call: missing method/field -> UndefinedVariable path
        (
            r#"
kin C { dae init() { } }
ken c = C()
c.nope()
"#,
            false,
        ),
        // Index errors
        ("1[0]", false),
        ("ken x = 1\nx[0] = 2\n", false),
        // Binary-op type errors (hit Multiply/Divide/Modulo/Compare error branches)
        ("1 * {\"a\": 1}", false),
        ("1 / \"x\"", false),
        ("1 % \"x\"", false),
        ("1 < \"x\"", false),
        // Higher-order builtins wrong-arity/type-error paths
        (
            r#"
ken xs = [1, 2, 3]
tumble(xs, 0)
"#,
            false,
        ),
        (
            r#"
ilk([1])
"#,
            false,
        ),
        (
            r#"
tumble(1, 0, |a, b| a + b)
"#,
            false,
        ),
        (
            r#"
ony([1, 2], |x| x > 0, 3)
"#,
            false,
        ),
        // More HOF wrong-arity branches
        ("gaun([1])", false),
        ("sieve([1])", false),
        ("hunt([1])", false),
        ("aw([1])", false),
        ("grup_up([1])", false),
        ("pairt_by([1])", false),
        // Destructure error + trailing binding after rest
        ("ken [a] = 1\n", false),
        (
            r#"
ken [a, ...rest, last] = [1, 2, 3, 4]
blether a
blether last
blether len(rest)
"#,
            true,
        ),
        // Hurl statement path (caught)
        (
            r#"
hae_a_bash {
    hurl "boom"
} gin_it_gangs_wrang e {
    blether e
}
"#,
            true,
        ),
        // Struct constructor wrong arity path
        (
            r#"
thing Pair { a, b }
Pair(1)
"#,
            false,
        ),
        // range_o float path
        ("range_o([1.0, 2.0])", true),
        // IndexSet out-of-bounds path
        ("ken xs = [1]\nxs[99] = 2\n", false),
        // Import error paths (module not found)
        ("fetch \"definitely_no_such_module\"", false),
        // Return at top-level (value + no-value)
        ("gie 123", true),
        ("gie\n", true),
        // VarDecl without initializer path
        ("ken x\nx\n", true),
        // For-loop non-iterable type error
        ("fer i in 1 { }\n", false),
        // Return propagation out of loop bodies
        (
            r#"
dae f() {
    whiles aye {
        gie 1
    }
    gie 0
}
f()
"#,
            true,
        ),
        (
            r#"
dae f() {
    fer i in 1..3 {
        gie i
    }
    gie 0
}
f()
"#,
            true,
        ),
        (
            r#"
dae f() {
    gie
}
f()
"#,
            true,
        ),
        // Hurl non-string message formatting path
        (
            r#"
hae_a_bash {
    hurl 123
} gin_it_gangs_wrang e {
    blether e
}
"#,
            true,
        ),
        // Pattern literal types (float/bool/nil)
        (
            r#"
keek 1.5 { whan 1.5 -> { blether "float" } }
keek aye { whan aye -> { blether "bool" } }
keek naething { whan naething -> { blether "nil" } }
"#,
            true,
        ),
        // Range pattern: non-integer bounds + non-integer value
        (
            r#"
ken x = 5
keek x {
    whan 1.."a" -> { blether "nope" }
    whan _ -> { blether "ok" }
}
"#,
            true,
        ),
        (
            r#"
ken x = "hi"
keek x {
    whan 1..10 -> { blether "nope" }
    whan _ -> { blether "ok" }
}
"#,
            true,
        ),
        // Assign undefined variable error path
        ("x = 1\n", false),
        // Instance field callable path (method-call syntax on callable field)
        (
            r#"
kin C {
    dae init() {
        masel.f = |x| x + 1
    }
}
ken c = C()
c.f(1)
"#,
            true,
        ),
        // Property access errors (dict miss + wrong receiver type)
        ("ken d = {\"a\": 1}\nd.nope\n", false),
        ("kin C { dae init() { } }\nken c = C()\nc.nope\n", false),
        ("1.nope\n", false),
        // Break/continue inside methods -> ControlFlow mapping paths
        (
            r#"
kin C {
    dae breaky() { brak }
    dae conty() { haud }
}
ken c = C()
c.breaky()
c.conty()
"#,
            true,
        ),
        // Slice type errors
        ("[1, 2, 3][\"a\":]\n", false),
        ("[1, 2, 3][:\"b\"]\n", false),
        ("[1, 2, 3][::0]\n", false),
        ("[1, 2, 3][::\"x\"]\n", false),
        ("1[:1]\n", false),
        // Block expression return path
        ("ken x = { gie 42 }\nx\n", true),
    ];

    for (src, should_succeed) in cases {
        let program = parse(src).unwrap_or_else(|e| panic!("parse failed for:\n{src}\nerr={e:?}"));
        let mut interp = Interpreter::new();
        let result = interp.interpret(&program);
        if *should_succeed {
            assert!(
                result.is_ok(),
                "expected success for:\n{src}\nerr={result:?}"
            );
        } else {
            assert!(result.is_err(), "expected error for:\n{src}");
        }
    }
}
