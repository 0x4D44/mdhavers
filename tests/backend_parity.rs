//! Backend parity smoke tests.
//!
//! Goal: catch basic semantic drift between the interpreter and other backends.

use mdhavers::Interpreter;
use std::process::Command;

fn run_interpreter(source: &str) -> Result<String, String> {
    let program = mdhavers::parse(source).map_err(|e| format!("{e}"))?;
    let mut interp = Interpreter::new();
    interp.interpret(&program).map_err(|e| format!("{e}"))?;
    Ok(interp.get_output().join("\n"))
}

fn run_js(source: &str) -> Result<String, String> {
    run_js_with_prelude("", source)
}

/// Run the compiled program under node wi' `prelude` (plain JS) run first.
fn run_js_with_prelude(prelude: &str, source: &str) -> Result<String, String> {
    let js = mdhavers::compile_to_js(source).map_err(|e| format!("{e}"))?;
    let js = format!("{prelude}\n{js}");
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let js_path = dir.path().join("parity.js");
    std::fs::write(&js_path, js).map_err(|e| e.to_string())?;

    let output = Command::new("node")
        .arg(&js_path)
        .output()
        .map_err(|e| format!("Failed to run node: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "node exited with {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string())
}

#[test]
fn parity_interpreter_vs_js_smoke() {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("Skipping JS parity tests: node not found");
        return;
    }

    let cases: &[(&str, &str)] = &[
        (r#"blether "hullo""#, "hullo"),
        ("blether 1 + 2 * 3", "7"),
        (
            r#"
ken x = 10
gin x > 5 { blether "big" } ither { blether "wee" }
"#,
            "big",
        ),
        (
            r#"
dae add(a, b = 1) { gie a + b }
blether add(41)
"#,
            "42",
        ),
        (
            r#"
ken i = 0
whiles i < 3 {
  blether i
  i = i + 1
}
"#,
            "0\n1\n2",
        ),
        (
            r#"
ken s = "abc"
blether s[0]
"#,
            "a",
        ),
    ];

    for (source, expected) in cases {
        let interp_out = run_interpreter(source).unwrap_or_else(|e| panic!("interpreter: {e}"));
        assert_eq!(interp_out.trim(), *expected);

        let js_out = run_js(source).unwrap_or_else(|e| panic!("js: {e}"));
        assert_eq!(js_out.trim(), *expected);
    }
}

fn node_available() -> bool {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("Skipping JS parity tests: node not found");
        return false;
    }
    true
}

/// Each case must print the same under the interpreter an' under node.
fn assert_js_parity(cases: &[(&str, &str, &str)]) {
    let mut failures = Vec::new();
    for (bug, source, expected) in cases {
        let interp_out =
            run_interpreter(source).unwrap_or_else(|e| panic!("{bug} interpreter: {e}"));
        assert_eq!(
            interp_out.trim(),
            *expected,
            "{bug}: interpreter output changed"
        );

        match run_js(source) {
            Ok(js_out) if js_out.trim() == *expected => {}
            Ok(js_out) => failures.push(format!(
                "{bug}: expected {expected:?}, JS printed {:?}",
                js_out.trim()
            )),
            Err(e) => failures.push(format!("{bug}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "JS parity failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn parity_js_string_escaping_mdh_bug_flu_00037_00038_00042() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00037 match string pattern",
            r#"ken v = 1
keek v {
    whan "\" || console.log(\"INJECTED\") || \"" -> { blether "matched" }
    whan _ -> { blether "other" }
}"#,
            "other",
        ),
        (
            "00042 f-string backslashes",
            r#"ken who = "nae"
blether f"C:\\users\\{who}""#,
            r"C:\users\nae",
        ),
        (
            "00042 f-string backtick and dollar",
            r#"ken n = 5
blether f"`cost` ${n}""#,
            "`cost` $5",
        ),
        (
            "00037 string literal quotes",
            r#"blether "it's \"quoted\" `tick` ${x}""#,
            r#"it's "quoted" `tick` ${x}"#,
        ),
    ]);
}

#[test]
fn parity_js_import_path_not_injected_mdh_bug_flu_00038() {
    if !node_available() {
        return;
    }
    // Unescaped, this path would become require(''+console.log('INJECTED')+''),
    // runnin' the payload while the argument is evaluated.
    let source = "fetch \"'+console.log('INJECTED')+'\" tae m\nblether \"done\"\n";
    let js = mdhavers::compile_to_js(source).expect("compile");
    let dir = tempfile::tempdir().expect("tempdir");
    let js_path = dir.path().join("inject.js");
    std::fs::write(&js_path, js).expect("write js");
    let output = Command::new("node")
        .arg(&js_path)
        .output()
        .expect("run node");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("INJECTED"), "injected code ran: {stdout}");
    // The crafted path is just a (missin') module name, no' a syntax error.
    assert!(
        !stderr.contains("SyntaxError"),
        "emitted JS is invalid: {stderr}"
    );
    assert!(
        stderr.contains("Cannot find module"),
        "unexpected node error: {stderr}"
    );
}

#[test]
fn parity_js_identifier_hygiene_mdh_bug_flu_00039() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00039 reserved identifiers",
            "ken new = 1\nken class = 2\nken this = 3\nblether new\nblether class\nblether this",
            "1\n2\n3",
        ),
        (
            "00039 reserved function and params",
            "dae function(var, typeof) { gie var - typeof }\nblether function(5, 3)",
            "2",
        ),
        (
            "00039 builtin name collision",
            "dae len(x) { gie 99 }\nblether len([1])\nken max = 5\nblether max",
            "99\n5",
        ),
        (
            "00039 builtin used before shadowing binding",
            "blether len(\"abc\")\nken len = |s| 999\nblether len(\"abc\")",
            "3\n999",
        ),
        (
            "00039 runtime namespace name",
            "ken __havers = 7\nblether __havers\nblether len(\"ab\")",
            "7\n2",
        ),
    ]);
}

#[test]
fn parity_js_bindings_mdh_bug_flu_00040() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00040 ken redeclare",
            "ken x = 1\nken x = 2\nblether x",
            "2",
        ),
        (
            "00040 fer variable assignment",
            "fer i in [1, 2] {\n    i = i * 10\n    blether i\n}",
            "10\n20",
        ),
        (
            "00040 destructure two ignores",
            "ken [_, _, c] = [1, 2, 3]\nblether c\nc = 4\nblether c",
            "3\n4",
        ),
        (
            "00040 inner ken reads outer name",
            "ken x = 10\ngin aye {\n    ken x = x + 1\n    blether x\n}\nblether x",
            "11\n10",
        ),
        (
            "00040 param redeclared",
            "dae f(a) {\n    ken a = a + 1\n    gie a\n}\nblether f(1)",
            "2",
        ),
    ]);
}

#[test]
fn parity_js_classes_mdh_bug_flu_00041() {
    if !node_available() {
        return;
    }
    let animals = r#"kin Animal {
    dae init(name) { masel.name = name }
    dae speak() { blether masel.name + " grunts" }
}
kin Dog fae Animal {
    dae init(name) { masel.name = name + "!" }
    dae speak() { blether masel.name + " woofs" }
}
kin Puppy fae Dog {
}
"#;
    let subclass_init = format!("{animals}ken d = Dog(\"Rex\")\nd.speak()");
    let inherited_init = format!("{animals}Puppy(\"Wee\").speak()");
    let class_value = format!("{animals}ken K = Animal\nK(\"Bess\").speak()");
    assert_js_parity(&[
        ("00041 subclass init", subclass_init.as_str(), "Rex! woofs"),
        ("00041 inherited init", inherited_init.as_str(), "Wee! woofs"),
        ("00041 class as value", class_value.as_str(), "Bess grunts"),
        (
            "00041 capitalised function and lowercase kin",
            "dae Square(x) { gie x * x }\nblether Square(3)\nkin wee {\n    dae init() { blether \"made\" }\n}\nken w = wee()",
            "9\nmade",
        ),
        (
            "00041 instance display",
            "kin Foo {\n}\nken f = Foo()\nblether f",
            "<Foo instance>",
        ),
    ]);
}

#[test]
fn parity_js_values_mdh_bug_flu_00043() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00043 print format",
            "blether aye\nblether naething\nblether [1, \"a\"]\nblether {\"a\": 1}\nblether [nae, [2.5]]",
            "aye\nnaething\n[1, a]\n{\"a\": 1}\n[nae, [2.5]]",
        ),
        (
            "00043 f-string and tae_string display",
            "ken xs = [1, aye]\nblether f\"xs={xs}\"\nblether tae_string(naething)\nblether \"x\" + aye",
            "xs=[1, aye]\nnaething\nxaye",
        ),
        (
            "00043 truthiness",
            "gin [] { blether \"truthy\" } ither { blether \"falsy\" }\ngin \"\" { blether \"truthy\" } ither { blether \"falsy\" }\nblether 0 or \"fallback\"\nblether [] an 5\nken empty = []\nblether !empty",
            "falsy\nfalsy\nfallback\n[]\naye",
        ),
        (
            "00043 list equality and negative index",
            "ken xs = [1, 2, 3]\nblether xs[-1]\nblether [1] == [1]\nblether [1, [2]] != [1, [2]]",
            "3\naye\nnae",
        ),
        (
            "00043 catch binding",
            "hae_a_bash {\n    hurl \"boom\"\n} gin_it_gangs_wrang err {\n    blether \"Caught: \" + err\n}",
            "Caught: Hurled at line 2: boom",
        ),
        (
            "00043 division by zero raises",
            "ken z = 0\nhae_a_bash {\n    blether 10 / z\n} gin_it_gangs_wrang e {\n    blether e\n}\nhae_a_bash {\n    blether 10 % z\n} gin_it_gangs_wrang e {\n    blether e\n}",
            "Ye numpty! Tryin' tae divide by zero at line 3\nYe numpty! Tryin' tae divide by zero at line 8",
        ),
        (
            "00043 dict literal key is an expression",
            "ken k = \"colour\"\nken d = {k: 1}\nblether d",
            "{\"colour\": 1}",
        ),
        (
            "00043 integer literal division",
            "blether 7 / 2\nblether -7 / 2\nblether [1] + [2]",
            "3\n-3\n[1, 2]",
        ),
    ]);
}

#[test]
fn parity_js_parser_followups_mdh_bug_flu_00066_00075_00078() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00066 lambda returning a dict",
            "ken f = |x| {\"k\": x * 2}\nblether f(4)[\"k\"]\nken g = |k| {\"a\": 1, \"b\": 2}[k]\nblether g(\"b\")\nken h = |k| {\"a\": 1, \"b\": 3}.b\nblether h(0)",
            "8\n2\n3",
        ),
        (
            "00075 multi-line call arguments and leading-dot chain",
            "dae add(a, b) {\n    gie a + b\n}\nblether add(\n    1,\n    2\n)\nken d = {\"a\": {\"b\": 5}}\nblether d\n    .a\n    .b",
            "3\n5",
        ),
        (
            "00078 inclusive range pattern",
            "fer i in 0..5 {\n    keek i {\n        whan 1..=2 -> blether \"in\"\n        whan 3..4 -> blether \"mid\"\n        whan _ -> blether \"oot\"\n    }\n}",
            "oot\nin\nin\nmid\noot",
        ),
    ]);
}

#[test]
fn parity_js_sort_total_order_mdh_bug_flu_00077() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00077 numbers then strings then others, others keep order",
            "blether sort([10, \"b\", 2, \"B\", \"a\", naething, aye, 7])",
            "[2, 7, 10, B, a, b, naething, aye]",
        ),
        (
            "00077 numbers compare by value not text",
            "blether sort([10, 9, 100, 1])",
            "[1, 9, 10, 100]",
        ),
        (
            "00077 strings compare by code point",
            "blether sort([\"b\", \"Z\", \"a\", \"10\", \"9\"])",
            "[10, 9, Z, a, b]",
        ),
    ]);
}

#[test]
fn parity_js_semantics_decisions_f1_f2_f3_block_value() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "F1 dicts compare structurally",
            "blether {\"a\": 1, \"b\": [1]} == {\"b\": [1], \"a\": 1}\nblether {\"a\": 1} == {\"a\": 2}\nblether {\"a\": {\"x\": 1}} != {\"a\": {\"x\": 1}}",
            "aye\nnae\nnae",
        ),
        (
            "F1 cyclic dicts compare without hanging",
            "ken a = {\"n\": 1}\na[\"me\"] = a\nken b = {\"n\": 1}\nb[\"me\"] = b\nblether a == b",
            "aye",
        ),
        (
            "F2 empty dict is falsy",
            "gin {} { blether \"truthy\" } ither { blether \"falsy\" }\nblether !{}\nblether {} or \"fallback\"",
            "falsy\naye\nfallback",
        ),
        (
            "F3 fer closures capture each iteration",
            "ken fs = []\nfer i in 0..3 { shove(fs, || i) }\nblether gaun(fs, |f| f())",
            "[0, 1, 2]",
        ),
        (
            "64 block value is its last expression",
            "blether {0 + {0 + 1}}\nken f = |x| {\n    ken y = x + 1\n    y * 2\n}\nblether f(3)\nblether { ken z = 1 }",
            "1\n8\nnaething",
        ),
    ]);
}

#[test]
fn parity_js_integer_model_f5() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "F5 integer overflow raises the interpreter's error",
            "ken big = 9223372036854775807\nhae_a_bash {\n    blether big + 1\n} gin_it_gangs_wrang e {\n    blether e\n}\nhae_a_bash {\n    blether -big - 2\n} gin_it_gangs_wrang e {\n    blether e\n}\nhae_a_bash {\n    blether big * big\n} gin_it_gangs_wrang e {\n    blether e\n}\nken wee = -big - 1\nhae_a_bash {\n    blether -wee\n} gin_it_gangs_wrang e {\n    blether e\n}\nhae_a_bash {\n    blether wee % -1\n} gin_it_gangs_wrang e {\n    blether e\n}",
            "Wheesht! Yer number's too muckle at line 3 - it's overflowed!\nWheesht! Yer number's too muckle at line 8 - it's overflowed!\nWheesht! Yer number's too muckle at line 13 - it's overflowed!\nWheesht! Yer number's too muckle at line 19 - it's overflowed!\nWheesht! Yer number's too muckle at line 24 - it's overflowed!",
        ),
        (
            "F5 integer division and remainder truncate toward zero",
            "ken a = 7\nken b = -2\nblether a / b\nblether a % b\nblether -a / 2\nblether -a % 2\nblether a / 2.0\nken z = 0.0\nhae_a_bash {\n    blether a / z\n} gin_it_gangs_wrang e {\n    blether e\n}",
            "-3\n1\n-3\n-1\n3.5\nYe numpty! Tryin' tae divide by zero at line 10",
        ),
        (
            "F5 products beyond 2^53 stay exact",
            "ken r = 3037000499\nblether r * r\nblether 9007199254740992 + 1\nblether 9007199254740993 - 2\nken f = 1\nfer i in 1..=20 { f = f * i }\nblether f",
            "9223372030926249001\n9007199254740993\n9007199254740991\n2432902008176640000",
        ),
        (
            "F5 int/float mixing",
            "blether 1 + 2.5\nblether 3 - 0.5\nblether 2 * 1.5\nblether whit_kind(1 + 1.0)\nblether whit_kind(6 / 3)\nblether 1 == 1.0\nblether [1, 2] == [1.0, 2.0]\nblether 1 < 1.5\nblether 2 >= 2.0\nhae_a_bash {\n    blether 7 % 2.0\n} gin_it_gangs_wrang e {\n    blether e\n}\nhae_a_bash {\n    blether \"a\" < 1\n} gin_it_gangs_wrang e {\n    blether e\n}",
            "3.5\n2.5\n3\nfloat\ninteger\naye\naye\naye\naye\nThat's pure mince! Type error at line 11: Cannae get remainder o' integer by float\nThat's pure mince! Type error at line 16: Cannae compare string wi' integer",
        ),
        (
            "F5 display of integers and floats",
            "blether 2.0\nblether -0.5\nblether [1, 2.5, 3.0]\nblether f\"{10 / 4} {10.0 / 4}\"\nblether \"n=\" + 5\nblether tae_string(12) + tae_string(1.5)",
            "2\n-0.5\n[1, 2.5, 3]\n2 2.5\nn=5\n121.5",
        ),
        (
            "F5 builtins take and gie integers",
            "ken xs = [10, 20, 30]\nblether len(xs) - 1\nblether xs[len(xs) - 1]\nblether xs[-1]\nblether xs[1:]\nblether xs[::-1]\nblether range(1, 4)\nblether whit_kind(len(\"abc\"))\nblether tae_int(\"41\") + 1\nblether whit_kind(tae_float(3))\nblether floor(2.7) + ceil(2.1) + round(2.5)\nblether abs(-5) + abs(-2.5)\nblether min(3, 2) + max(1, 4)\nblether sumaw([1, 2, 3]) + sumaw([0.5])\nblether coont([1, 1.0, 2], 1)\nblether scran(\"hello\", 1, 3)\nblether sqrt(16)\nblether sort([3, 1.5, 2])\nxs[0] = 5\nblether xs",
            "2\n30\n30\n[20, 30]\n[30, 20, 10]\n[1, 2, 3]\ninteger\n42\nfloat\n8\n7.5\n6\n6.5\n2\nel\n4\n[1.5, 2, 3]\n[5, 20, 30]",
        ),
        (
            "F5 match arms and ranges compare by value",
            "keek 2.0 {\n    whan 2 -> blether \"two\"\n    whan _ -> blether \"other\"\n}\nkeek 5 {\n    whan 1..=5 -> blether \"in\"\n    whan _ -> blether \"oot\"\n}\nfer i in 0..3 { blether i * 10 }",
            "two\nin\n0\n10\n20",
        ),
    ]);
}

#[test]
fn parity_js_gie_in_block_expression_returns_from_function_mdh_bug_flu_00094() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00094 gie in a block returns frae the dae",
            "dae f() {\n    ken b = { gie 4 }\n    gie 9\n}\nblether f()",
            "4",
        ),
        (
            "00094 gie in a block in a lambda returns frae the lambda",
            "dae outer() {\n    ken inner = |x| {\n        ken y = { gie x * 10 }\n        gie 0\n    }\n    gie inner(5) + 1\n}\nblether outer()",
            "51",
        ),
        (
            "00094 expression-bodied lambda",
            "ken g = |x| x + { gie x * 3 }\nblether g(2)",
            "6",
        ),
        (
            "00094 a lambda's ain block body still returns frae the lambda",
            "ken add_one = |x| { gie x + 1 }\nblether add_one(2)",
            "3",
        ),
        (
            "00094 nested blocks in a loop",
            "dae first_big(xs) {\n    fer x in xs {\n        ken seen = {\n            ken inner = {\n                gin x > 2 {\n                    gie x\n                }\n                x\n            }\n            inner\n        }\n    }\n    gie -1\n}\nblether first_big([1, 2, 5, 7])\nblether first_big([1, 2])",
            "5\n-1",
        ),
        (
            "00094 hae_a_bash does no' catch the gie",
            "dae guarded() {\n    hae_a_bash {\n        ken b = { gie \"oot\" }\n    } gin_it_gangs_wrang e {\n        gie \"caught\"\n    }\n    gie \"fell through\"\n}\nblether guarded()",
            "oot",
        ),
        (
            "00094 method and recursion",
            "kin Coonter {\n    dae pick(n) {\n        ken b = { gie n * 2 }\n        gie 0\n    }\n}\nblether Coonter().pick(21)\ndae down(n) {\n    ken r = {\n        gin n == 0 {\n            gie \"bottom\"\n        }\n        down(n - 1)\n    }\n    gie r\n}\nblether down(3)",
            "42\nbottom",
        ),
    ]);
}

#[test]
fn parity_js_gie_in_default_parameter_mdh_bug_flu_00098() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00098 gie in a default value's block returns frae the dae",
            "dae f(a, b = {\n    gin a > 1 {\n        gie \"early\"\n    }\n    5\n}) {\n    gie b\n}\nblether f(0)\nblether f(2)\nblether f(2, 9)",
            "5\nearly\n9",
        ),
        (
            "00098 method default sees masel",
            "kin K {\n    dae init(v) {\n        masel.v = v\n    }\n    dae pick(a = {\n        gin masel.v > 1 {\n            gie \"big\"\n        }\n        masel.v\n    }) {\n        gie a\n    }\n}\nblether K(5).pick()\nblether K(1).pick()",
            "big\n1",
        ),
        (
            "00098 recursion through the default",
            "dae down(n, r = {\n    gin n == 0 {\n        gie \"bottom\"\n    }\n    down(n - 1)\n}) {\n    gie r\n}\nblether down(3)",
            "bottom",
        ),
    ]);
}

const MATHS_SOURCE: &str = r#"blether sin(0)
blether cos(0)
blether whit_kind(cos(0))
blether tan(0)
blether round(degrees(radians(180)))
blether atan2(1, 1) * 4 == PI
blether asin(1) * 2 == PI
blether acos(1)
blether atan(0)
blether pooer(2, 10)
blether whit_kind(pooer(2, 10))
blether pooer(2, -2)
blether pooer(1.5, 2)
blether pooer(1.1, 3)
blether pow(2, 0.5)
blether hypot(3, 4)
blether log10(1000)
blether log2(8)
blether log(1)
blether exp(0)
blether sign(-7)
blether sign(2.5)
blether trunc(-2.7)
blether clamp(15, 0, 10)
blether clamp(0.5, 1.0, 3.0)
blether lerp(0, 10, 0.25)
blether gcd(12, -18)
blether lcm(4, 6)
blether factorial(20)
blether is_even(4)
blether is_odd(4)
blether is_prime(97)
blether is_prime(91)
blether average([1, 2, 3, 4])
blether median([3, 1, 2])
blether median([1.0, 2.0, 3.0, 100.0])
blether product([2, 3, 4])
blether product([2, 3.0])
blether minaw([3, 1, 2])
blether maxaw([1.5, 0.5])
blether range_o([1, 5, 3])
blether TAU == 2 * PI
blether pi() == PI
blether e() == E
blether tau() == TAU
ken r = random_int(1, 6)
blether r >= 1 an r <= 6
blether whit_kind(random())
blether random_choice([7])
hae_a_bash {
    blether sin(aye)
} gin_it_gangs_wrang err {
    blether err
}
hae_a_bash {
    blether pooer(2, 63)
} gin_it_gangs_wrang err {
    blether err
}
hae_a_bash {
    blether factorial(21)
} gin_it_gangs_wrang err {
    blether err
}
hae_a_bash {
    blether clamp(1, 2.0, 3)
} gin_it_gangs_wrang err {
    blether err
}
hae_a_bash {
    blether median([])
} gin_it_gangs_wrang err {
    blether err
}
hae_a_bash {
    blether product([9223372036854775807, 2])
} gin_it_gangs_wrang err {
    blether err
}"#;

const MATHS_EXPECTED: &str = "0\n1\nfloat\n0\n180\naye\naye\n0\n0\n1024\ninteger\n0.25\n2.25\n\
1.3310000000000004\n1.4142135623730951\n5\n3\n3\n0\n1\n-1\n1\n-2\n10\n1\n2.5\n6\n12\n\
2432902008176640000\naye\nnae\naye\nnae\n2.5\n2\n2.5\n24\n6\n1\n1.5\n4\naye\naye\naye\naye\n\
aye\nfloat\n7\n\
Jings! Something went awfy wrang: sin() needs a number\n\
Jings! Something went awfy wrang: Wheesht! Yer number's too muckle in pooer() - it's overflowed!\n\
Jings! Something went awfy wrang: Factorial too big! Max is 20\n\
Jings! Something went awfy wrang: clamp() needs three numbers o' the same type\n\
Jings! Something went awfy wrang: Cannae calculate median o' empty list!\n\
Jings! Something went awfy wrang: Wheesht! Yer number's too muckle in product() - it's overflowed!";

#[test]
fn parity_js_maths_builtins_mdh_bug_flu_00100() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[("00100 maths builtins", MATHS_SOURCE, MATHS_EXPECTED)]);
}

/// A wee stand-in fer three.js: every numeric input must be a JS number, an'
/// `add` only takes objects it made itsel' (no' a Proxy roond one).
const FAKE_THREE: &str = r#"
globalThis.THREE = (() => {
  const made = new WeakSet();
  const num = (v, what) => {
    if (typeof v !== "number") throw new TypeError(what + " got a " + typeof v);
    return v;
  };
  class Vector3 {
    constructor() { this._x = 0; this._y = 0; this._z = 0; }
    get x() { return this._x; } set x(v) { this._x = num(v, "x"); }
    get y() { return this._y; } set y(v) { this._y = num(v, "y"); }
    get z() { return this._z; } set z(v) { this._z = num(v, "z"); }
    set(x, y, z) { this.x = x; this.y = y; this.z = z; return this; }
  }
  class Object3D {
    constructor() { made.add(this); this.position = new Vector3(); this.children = []; }
    add(o) {
      if (!made.has(o)) throw new TypeError("add got a stranger");
      this.children.push(o);
      return this;
    }
    lookAt(x, y, z) { num(x, "lookAt"); num(y, "lookAt"); num(z, "lookAt"); }
  }
  class Scene extends Object3D {}
  class PerspectiveCamera extends Object3D {
    constructor(fov, aspect, near, far) {
      super();
      num(fov, "fov"); num(aspect, "aspect"); num(near, "near"); num(far, "far");
    }
  }
  class DirectionalLight extends Object3D {
    constructor(colour, intensity) { super(); num(intensity, "intensity"); }
  }
  class WebGLRenderer {
    constructor(opts) { num(opts.width, "width"); }
    setAnimationLoop(f) { f(500); }
  }
  return { Scene, Object3D, PerspectiveCamera, DirectionalLight, WebGLRenderer };
})();
"#;

#[test]
fn js_tri_interop_passes_numbers_to_three_mdh_bug_flu_00097() {
    if !node_available() {
        return;
    }
    let source = r##"fetch "tri" tae tri
ken sicht = tri.Sicht()
ken kamera = tri.PerspectivKamera(75, 16 / 9, 1, 1000)
ken sun = tri.DireksionalLicht("#ffffff", 1)
sun.position.x = 2
sun.position.y = 4
sicht.adde(sun)
kamera.luik_at(0, 0, 0)
kamera.position.set(1, 2, 3)
blether sun.position.x + sun.position.y
blether whit_kind(sun.position.x)
blether kamera.position.z
blether len(sicht.children)
ken renderar = tri.Renderar({"width": 800})
renderar.loop(|dt| {
    blether dt
})
"##;
    let out = run_js_with_prelude(FAKE_THREE, source).unwrap_or_else(|e| panic!("js: {e}"));
    // Numbers read back fae three.js are JS numbers, so they read as floats.
    assert_eq!(out.trim(), "6\nfloat\n3\n1\n0.5");
}

#[test]
fn js_fetched_module_receives_numbers_mdh_bug_flu_00097() {
    if !node_available() {
        return;
    }
    // Node's Buffer.alloc rejects a BigInt size.
    let source = "fetch \"buffer\" tae b\nblether b.Buffer.alloc(3).length\n";
    let out = run_js(source).unwrap_or_else(|e| panic!("js: {e}"));
    assert_eq!(out.trim(), "3");
}

const SNOOZE_SOURCE: &str = r#"
ken start = noo()
snooze(60)
ken gone = noo() - start
blether gone >= 55
blether snooze(0)
hae_a_bash {
    snooze(-1)
} gin_it_gangs_wrang err {
    blether err
}
hae_a_bash {
    snooze("soon")
} gin_it_gangs_wrang err {
    blether err
}
"#;

#[test]
fn parity_js_snooze_mdh_bug_flu_00101() {
    if !node_available() {
        return;
    }
    let interp = run_interpreter(SNOOZE_SOURCE).expect("interpreter");
    let js = run_js(SNOOZE_SOURCE).expect("node");
    assert_eq!(interp, js, "snooze: JS differs from the interpreter");
    assert!(interp.starts_with("aye\nnaething\n"), "got {interp:?}");
}

/// Wi' nae TTY (stdin is null under `Command::output`) the JS `get_key` has nae
/// key tae give, so it returns the empty string rather than blockin' or throwin'.
#[test]
fn js_get_key_without_tty_returns_empty_string_mdh_bug_flu_00101() {
    if !node_available() {
        return;
    }
    let out =
        run_js("ken k = get_key()\nblether \"[\" + k + \"]\"\nblether k == \"\"").expect("node");
    assert_eq!(out, "[]\naye");
}
