//! Regression tests for interpreter robustness: each of these used to crash
//! the process (stack overflow, panic, capacity overflow) or return a wrong
//! value. They must now give a catchable Scots error or the right answer.

use mdhavers::value::RangeValue;
use mdhavers::{parse, HaversError, Interpreter};

/// Run a program and return everything it blethered.
fn run_ok(code: &str) -> String {
    let program = parse(code).expect("parse");
    let mut interp = Interpreter::new();
    if let Err(e) = interp.interpret(&program) {
        panic!("program failed: {e}");
    }
    interp.get_output().join("\n")
}

/// Run a program that must fail, and return its error.
fn run_err(code: &str) -> HaversError {
    let program = parse(code).expect("parse");
    let mut interp = Interpreter::new();
    match interp.interpret(&program) {
        Ok(_) => panic!("expected an error, got output: {:?}", interp.get_output()),
        Err(e) => e,
    }
}

const MIN: &str = "ken m = -9223372036854775807 - 1\n";

// ---------- MDH-BUG-FLU-00001: deep recursion ----------

#[test]
fn infinite_recursion_is_a_catchable_stack_overflow() {
    let out = run_ok(
        r#"
dae f(n) {
    gie f(n + 1)
}
hae_a_bash {
    f(0)
} gin_it_gangs_wrang e {
    blether "caught: " + e
}
blether "still alive"
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert!(
        lines[0].starts_with("caught: Stack's fair puggled!"),
        "{out}"
    );
    assert_eq!(lines[1], "still alive");
}

#[test]
fn uncaught_infinite_recursion_returns_stack_overflow_error() {
    let err = run_err("dae f(n) {\n    gie f(n + 1)\n}\nf(0)\n");
    assert!(matches!(err, HaversError::StackOverflow { .. }), "{err:?}");
}

#[test]
fn infinite_method_recursion_is_a_stack_overflow() {
    let err = run_err(
        r#"
kin Loop {
    dae go(n) {
        gie masel.go(n + 1)
    }
}
Loop().go(0)
"#,
    );
    assert!(matches!(err, HaversError::StackOverflow { .. }), "{err:?}");
}

#[test]
fn modest_recursion_still_works() {
    // Debug-build evaluator frames are tens of KB per mdhavers call, so a
    // 2 MB test thread only fits a few dozen levels. Give this one room, as
    // the CLI does, and check the guard does not trip early.
    let out = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(|| {
            run_ok(
                r#"
dae sum_to(n) {
    gin n == 0 {
        gie 0
    }
    gie n + sum_to(n - 1)
}
blether sum_to(1000)
"#,
            )
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(out, "500500");
}

#[test]
fn shallow_recursion_works_on_a_default_test_thread() {
    let out = run_ok(
        r#"
dae sum_to(n) {
    gin n == 0 {
        gie 0
    }
    gie n + sum_to(n - 1)
}
blether sum_to(20)
"#,
    );
    assert_eq!(out, "210");
}

// ---------- MDH-BUG-FLU-00002: self-referential values ----------

#[test]
fn self_referential_dict_survives_assignment_and_prints() {
    let out = run_ok(
        r#"
ken d = {}
d["me"] = d
blether "still alive"
blether d
"#,
    );
    assert_eq!(out, "still alive\n{\"me\": {...}}");
}

#[test]
fn self_referential_list_survives_decl_print_and_equality() {
    let out = run_ok(
        r#"
ken a = [1]
shove(a, a)
ken b = a
blether a
blether a == b
ken x = []
shove(x, x)
ken y = []
shove(y, y)
blether x == y
blether [1, [2]] == [1, [3]]
blether [1, [2]] == [1, [2]]
"#,
    );
    assert_eq!(out, "[1, [...]]\naye\naye\nnae\naye");
}

#[test]
fn self_referential_list_in_sclaff_is_an_error() {
    let out = run_ok(
        r#"
ken a = [1]
shove(a, a)
hae_a_bash {
    sclaff(a)
} gin_it_gangs_wrang e {
    blether "caught"
}
blether sclaff([1, [2, [3]]])
"#,
    );
    assert_eq!(out, "caught\n[1, 2, 3]");
}

// ---------- MDH-BUG-FLU-00003: JSON depth and cycles ----------

#[test]
fn json_parse_of_deep_nesting_is_a_catchable_error() {
    let out = run_ok(
        r#"
ken s = "[" * 200000
hae_a_bash {
    blether json_parse(s)
} gin_it_gangs_wrang e {
    blether "caught"
}
blether len(json_parse("[" * 100 + "]" * 100))
"#,
    );
    assert_eq!(out, "caught\n1");
}

#[test]
fn json_stringify_of_a_cycle_is_a_catchable_error() {
    let out = run_ok(
        r#"
ken a = []
shove(a, a)
hae_a_bash {
    blether json_stringify(a)
} gin_it_gangs_wrang e {
    blether "caught"
}
ken d = {}
d["me"] = d
hae_a_bash {
    blether json_pretty(d)
} gin_it_gangs_wrang e {
    blether "caught"
}
blether json_stringify([1, [2]])
"#,
    );
    assert_eq!(out, "caught\ncaught\n[1, [2]]");
}

// ---------- MDH-BUG-FLU-00004: checked integer maths in natives ----------

fn assert_overflow_caught(expr: &str) {
    let code = format!(
        "{MIN}hae_a_bash {{\n    blether {expr}\n}} gin_it_gangs_wrang e {{\n    blether \"caught: \" + e\n}}\n"
    );
    let out = run_ok(&code);
    assert!(
        out.starts_with("caught: ") && out.contains("too muckle"),
        "{expr} gave {out}"
    );
}

#[test]
fn native_integer_overflow_is_an_error_not_a_panic_or_wrap() {
    assert_overflow_caught("abs(m)");
    assert_overflow_caught("pooer(2, 64)");
    assert_overflow_caught("pooer(2, 4294967296)");
    assert_overflow_caught("gcd(m, 0)");
    assert_overflow_caught("lcm(9223372036854775807, 2)");
    assert_overflow_caught("lcm(m, 3)");
    assert_overflow_caught("sumaw([9223372036854775807, 1])");
    assert_overflow_caught("product([9223372036854775807, 2])");
}

#[test]
fn native_integer_edge_cases_give_exact_answers() {
    let out = run_ok(&format!(
        "{MIN}{}",
        r#"
blether gcd(m, -1)
blether pooer(1, 4294967296)
blether pooer(-1, 4294967297)
blether pooer(0, 4294967296)
blether pooer(2, 62)
blether lcm(4, 6)
blether lcm(-4, 6)
blether sumaw([9223372036854775807, 1, -1])
blether product([9223372036854775807, 2, 0])
blether is_wee(m)
blether is_muckle(m)
"#
    ));
    assert_eq!(
        out,
        "1\n1\n-1\n0\n4611686018427387904\n12\n12\n9223372036854775807\n0\nnae\naye"
    );
}

#[test]
fn random_ranges_at_the_i64_extremes_do_not_overflow() {
    let out = run_ok(&format!(
        "{MIN}{}",
        r#"
ken a = jammy(m, 9223372036854775807)
ken b = random_int(m, 9223372036854775807)
blether whit_kind(a)
blether whit_kind(b)
blether jammy(5, 6)
blether random_int(7, 7)
"#
    ));
    assert_eq!(out, "integer\ninteger\n5\n7");
}

// ---------- MDH-BUG-FLU-00005: huge repeat / pad / bytes sizes ----------

fn assert_size_caught(expr: &str) {
    let code = format!(
        "hae_a_bash {{\n    ken s = {expr}\n}} gin_it_gangs_wrang e {{\n    blether \"caught: \" + e\n}}\n"
    );
    let out = run_ok(&code);
    assert!(
        out.starts_with("caught: ") && out.contains("ower muckle"),
        "{expr} gave {out}"
    );
}

#[test]
fn huge_string_and_buffer_sizes_are_errors_not_aborts() {
    assert_size_caught(r#""ab" * 9223372036854775807"#);
    assert_size_caught(r#"9223372036854775807 * "ab""#);
    assert_size_caught(r#""ab" * 10000000000"#);
    assert_size_caught(r#"repeat("ab", 9223372036854775807)"#);
    assert_size_caught(r#"pad_left("x", 9223372036854775807)"#);
    assert_size_caught(r#"pad_right("x", 9223372036854775807, "*")"#);
    assert_size_caught(r#"center("x", 9223372036854775807)"#);
    assert_size_caught(r#"sporran_fill("x", 9223372036854775807, "*")"#);
    assert_size_caught(r#"tattie_scone("ab", 9223372036854775807)"#);
    assert_size_caught("bytes(9223372036854775807)");
    assert_size_caught("0..10000000000");
    assert_size_caught("range(0, 10000000000)");
}

#[test]
fn ordinary_repeat_and_pad_still_work() {
    let out = run_ok(
        r#"
blether "ab" * 3
blether repeat("xy", 2)
blether pad_left("7", 3, "0")
blether tattie_scone("a", 3)
blether bytes_len(bytes(4))
"#,
    );
    assert_eq!(out, "ababab\nxyxy\n007\na | a | a\n4");
}

// ---------- MDH-BUG-FLU-00006: ranges and slices at i64::MAX ----------

#[test]
fn inclusive_range_to_i64_max_does_not_overflow() {
    let out = run_ok(
        r#"
blether 9223372036854775806..=9223372036854775807
ken n = 0
fer i in 9223372036854775806..=9223372036854775807 {
    n = n + 1
}
blether n
"#,
    );
    assert_eq!(out, "[9223372036854775806, 9223372036854775807]\n2");
}

#[test]
fn range_iterator_stops_at_i64_max() {
    let items: Vec<i64> = RangeValue::new(i64::MAX - 1, i64::MAX, true)
        .iter()
        .collect();
    assert_eq!(items, vec![i64::MAX - 1, i64::MAX]);
    let empty: Vec<i64> = RangeValue::new(i64::MAX, i64::MAX, false).iter().collect();
    assert!(empty.is_empty());
    assert_eq!(
        RangeValue::new(0, 10, false).iter().size_hint(),
        (10, Some(10))
    );
}

#[test]
fn for_loop_over_a_huge_range_literal_is_lazy() {
    let out = run_ok(
        r#"
fer i in 0..10000000000 {
    blether i
    brak
}
"#,
    );
    assert_eq!(out, "0");
}

#[test]
fn slice_with_a_huge_step_does_not_overflow() {
    let out = run_ok(
        r#"
ken xs = [1, 2, 3]
blether xs[1:3:9223372036854775807]
blether "abc"[1:3:9223372036854775807]
blether xs[0:3:2]
"#,
    );
    assert_eq!(out, "[2]\nb\n[1, 3]");
}

// ---------- MDH-BUG-FLU-00007 / 00008: RefCell borrows across calls ----------

#[test]
fn operator_overload_that_mutates_masel_works() {
    let out = run_ok(
        r#"
kin Counter {
    dae init() {
        masel.n = 0
    }
    dae __pit_thegither__(that) {
        masel.n = masel.n + that
        gie masel
    }
}
ken c = Counter()
ken d = c + 1
blether d.n
"#,
    );
    assert_eq!(out, "1");
}

#[test]
fn bytes_append_to_itself_doubles_the_buffer() {
    let out = run_ok(
        r#"
ken b = bytes(2)
bytes_set(b, 0, 7)
ken c = bytes_append(b, b)
blether bytes_len(c)
blether bytes_get(c, 2)
"#,
    );
    assert_eq!(out, "4\n7");
}

// ---------- MDH-BUG-FLU-00009: sort order ----------

#[test]
fn sort_orders_mixed_integers_and_floats_numerically() {
    let out = run_ok(
        r#"
blether sort([3, 1.5, 2])
ken big = []
fer i in 0..40 {
    gin i % 2 == 0 {
        shove(big, 40 - i)
    } ither {
        shove(big, (40 - i) * 1.0 + 0.5)
    }
}
ken s = sort(big)
ken ok = aye
fer i in 1..len(s) {
    gin s[i - 1] > s[i] {
        ok = nae
    }
}
blether ok
blether len(s)
"#,
    );
    assert_eq!(out, "[1.5, 2, 3]\naye\n40");
}

#[test]
fn sort_puts_numbers_then_strings_then_other_kinds() {
    let out = run_ok(r#"blether sort(["b", 2, nae, 1.5, "a", aye])"#);
    assert_eq!(out, "[1.5, 2, a, b, nae, aye]");
}

// ---------- MDH-BUG-FLU-00010: constructor and method arity ----------

const PAIR: &str = r#"
kin Pair {
    dae init(a, b) {
        masel.a = a
        masel.b = b
    }
    dae sum() {
        gie masel.a + masel.b
    }
    dae scaled(k = 2) {
        gie masel.a * k
    }
}
"#;

#[test]
fn constructor_with_too_few_args_is_an_arity_error() {
    let err = run_err(&format!("{PAIR}Pair(1)\n"));
    assert!(matches!(err, HaversError::WrongArity { .. }), "{err:?}");
}

#[test]
fn constructor_with_too_many_args_is_an_arity_error() {
    let err = run_err(&format!("{PAIR}Pair(1, 2, 3)\n"));
    assert!(matches!(err, HaversError::WrongArity { .. }), "{err:?}");
}

#[test]
fn method_with_too_many_args_is_an_arity_error() {
    let err = run_err(&format!("{PAIR}ken q = Pair(1, 2)\nq.sum(9, 9)\n"));
    assert!(matches!(err, HaversError::WrongArity { .. }), "{err:?}");
}

#[test]
fn method_default_params_still_work() {
    let out = run_ok(&format!(
        "{PAIR}ken q = Pair(3, 4)\nblether q.sum()\nblether q.scaled()\nblether q.scaled(10)\n"
    ));
    assert_eq!(out, "7\n6\n30");
}

// ---------- MDH-BUG-FLU-00011: random numbers ----------

#[test]
fn jammy_gives_varied_values() {
    let out = run_ok(
        r#"
ken seen = []
fer i in 0..200 {
    shove(seen, jammy(0, 8))
}
blether len(unique(seen))
ken ok = aye
fer v in seen {
    gin v < 0 or v >= 8 {
        ok = nae
    }
}
blether ok
"#,
    );
    let mut lines = out.lines();
    let distinct: usize = lines.next().unwrap().parse().unwrap();
    assert!(
        distinct >= 4,
        "only {distinct} distinct values from jammy(0, 8)"
    );
    assert_eq!(lines.next(), Some("aye"));
}

#[test]
fn shuffle_keeps_every_element() {
    let out = run_ok("blether sort(shuffle([5, 3, 1, 4, 2]))");
    assert_eq!(out, "[1, 2, 3, 4, 5]");
}

// ---------- MDH-BUG-FLU-00012: big JSON integers ----------

#[test]
fn json_parse_big_integer_falls_back_to_float() {
    let out = run_ok(
        r#"
ken x = json_parse("12345678901234567890")
blether whit_kind(x)
blether x
blether json_parse("-9223372036854775808")
"#,
    );
    assert_eq!(out, "float\n12345678901234567000\n-9223372036854775808");
}
