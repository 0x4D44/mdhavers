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
        eprintln!("SKIPPED: `node` is no' on PATH, so this JS parity test checked naething");
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

/// MDH-BUG-FLU-00172: JS transcendentals (`Math.*`) may differ from the
/// interpreter's libm in the last place; docs/src/advanced/interop.md says to
/// compare them rounded. Rounded to nine decimals, both must print the same,
/// so a JS result that is wrong by more than the last digit still fails.
const FLOATS_ROUNDED_SOURCE: &str = r#"blether round(log(3) * 1000000000)
blether round(tan(2.5) * 1000000000)
blether round(sin(1.7) * 1000000000)
blether round(cos(2.2) * 1000000000)
blether round(exp(1.5) * 1000000000)
blether round(log10(7) * 1000000000)
blether round(log2(10) * 1000000000)
blether round(atan(0.5) * 1000000000)
blether round(asin(0.3) * 1000000000)
blether round(acos(0.3) * 1000000000)
blether round(pooer(1.7, 2.3) * 1000000000)"#;

const FLOATS_ROUNDED_EXPECTED: &str = "1098612289
-747022297
991664810
-588501117
4481689070
845098040
3321928095
463647609
304692654
1266103673
3388695291";

#[test]
fn parity_js_transcendentals_agree_when_rounded_mdh_bug_flu_00172() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[(
        "00172 rounded transcendentals",
        FLOATS_ROUNDED_SOURCE,
        FLOATS_ROUNDED_EXPECTED,
    )]);
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

#[test]
fn parity_js_creel_builtins_mdh_bug_flu_00104() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00104 creel construction, display, membership, add and remove",
            "ken s = creel([3, 1, 2, 2])\nblether s\nblether len(s)\nblether is_in_creel(s, 2)\nblether is_in_creel(s, \"2\")\ntoss_in(s, 5)\nheave_oot(s, 1)\nblether creel_tae_list(s)\nblether s",
            "creel{\"1\", \"2\", \"3\"}\n3\naye\nnae\n[2, 3, 5]\ncreel{\"2\", \"3\", \"5\"}",
        ),
        (
            "00104 creel set operations",
            "ken a = creel([1, 2])\nken b = creel([2, 3])\nblether creels_thegither(a, b)\nblether creels_baith(a, b)\nblether creels_differ(a, b)\nblether is_subset(creel([1]), a)\nblether is_superset(creel([1]), a)\nblether is_disjoint(creel([1]), creel([2]))",
            "creel{\"1\", \"2\", \"3\"}\ncreel{\"2\"}\ncreel{\"1\"}\naye\nnae\naye",
        ),
        (
            "00104 creel equality, truthiness, aliasing and identity members",
            "blether creel([1, 2]) == creel([2, 1])\nblether creel([1]) == creel([1, 2])\ngin empty_creel() { blether \"truthy\" } ither { blether \"falsy\" }\nken s = creel([1])\nken t = s\ntoss_in(t, 2)\nblether len(s)\nken l = [1]\nken c = creel([l])\nblether is_in_creel(c, l)\nblether is_in_creel(c, [1])",
            "aye\nnae\nfalsy\n2\naye\nnae",
        ),
    ]);
}

#[test]
fn parity_js_typed_dict_keys_mdh_bug_flu_00109() {
    if !node_available() {
        return;
    }
    let mixed = "ken m = {\"b\": 1, 2: \"two\", \"1\": \"str\", 1: \"int\", 1.5: \"flt\", 1.0: \"one\", aye: \"yes\", naething: \"nil\"}\n";
    let source = |body: &str| format!("{mixed}{body}");
    let display = source("blether m\nblether len(m)");
    let listing = source(
        "blether keys(m)\nblether gaun(keys(m), whit_kind)\nblether values(m)\nblether items(m)",
    );
    let lookup = source("blether m[1]\nblether m[\"1\"]\nblether m[1.0]\nblether m[aye]\nblether m[naething]\nblether contains(m, 1)\nblether contains(m, \"2\")");
    let update = source("m[2] = \"deux\"\nm[\"2\"] = \"string two\"\nblether m\nblether len(m)");
    let fer =
        source("hae_a_bash { fer k in m { blether k } } gin_it_gangs_wrang err { blether err }");
    assert_js_parity(&[
        (
            "00109 integer keys index lists and stay integers",
            "ken d = {0: \"a\"}\nken xs = [10]\nfer k in keys(d) { blether xs[k] }\nfer k in keys(d) { blether whit_kind(k) }\nfer k in keys(d) { blether k + 1 }",
            "10\ninteger\n1",
        ),
        (
            "00109 int, float, numeric-string, bool and naething keys display in insertion order",
            &display,
            "{\"b\": 1, \"2\": two, \"1\": str, \"1\": int, \"1.5\": flt, \"1\": one, \"aye\": yes, \"naething\": nil}\n8",
        ),
        (
            "00109 keys, values and items keep key types",
            &listing,
            "[b, 2, 1, 1, 1.5, 1, aye, naething]\n[string, integer, string, integer, float, float, bool, naething]\n[1, two, str, int, flt, one, yes, nil]\n[[b, 1], [2, two], [1, str], [1, int], [1.5, flt], [1, one], [aye, yes], [naething, nil]]",
        ),
        (
            "00109 lookup and contains are typed",
            &lookup,
            "int\nstr\none\nyes\nnil\naye\nnae",
        ),
        (
            "00109 index assignment is typed and appends new keys last",
            &update,
            "{\"b\": 1, \"2\": deux, \"1\": str, \"1\": int, \"1.5\": flt, \"1\": one, \"aye\": yes, \"naething\": nil, \"2\": string two}\n9",
        ),
        (
            "00109 fer ower a dict is an error",
            &fer,
            "That's pure mince! Type error at line 2: Cannae iterate ower a dict",
        ),
        (
            "00109 == compares typed keys",
            "blether {1: \"a\"} == {1: \"a\"}\nblether {1: \"a\"} == {\"1\": \"a\"}\nblether {1: \"a\"} == {1.0: \"a\"}",
            "aye\nnae\nnae",
        ),
        (
            "00109 dict_merge keeps typed keys and first positions",
            "blether dict_merge({1: \"a\", \"x\": 1}, {\"1\": \"b\", 1: \"c\"})",
            "{\"1\": c, \"x\": 1, \"1\": b}",
        ),
        (
            "00109 json_stringify writes non-string keys as their text",
            "blether json_stringify({1: \"a\", \"b\": [1, 2.5], \"c\": {2: naething}})\nblether json_stringify({aye: 1.0, \"q\\\"\": \"x\\ny\"})",
            "{\"1\": \"a\", \"b\": [1, 2.5], \"c\": {\"2\": null}}\n{\"aye\": 1, \"q\\\"\": \"x\\ny\"}",
        ),
        (
            "00109 special string keys are ordinary keys",
            "ken e = {\"__proto__\": 1, \"0\": 2, \"\": 3}\ne[\"__proto__\"] = 4\nblether e\nblether keys(e)\nblether gaun(keys(e), whit_kind)",
            "{\"__proto__\": 4, \"0\": 2, \"\": 3}\n[__proto__, 0, ]\n[string, string, string]",
        ),
        (
            "00109 list and dict keys are keyed by identity",
            "ken l = [1]\nken d = {}\nd[l] = \"list\"\nblether d[l]\nblether contains(d, [1])\nblether whit_kind(keys(d)[0])",
            "list\nnae\nlist",
        ),
    ]);
}

#[test]
fn parity_js_predicates_use_mdhavers_truthiness_mdh_bug_flu_00110() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00110 sieve keeps only truthy results",
            "blether len(sieve([{}, {\"a\": 1}, []], |d| d))\nblether sieve([0, 1, \"\", \"x\", [], [2], naething, {}, 0.0], |v| v)",
            "1\n[1, x, [2]]",
        ),
        (
            "00110 aw, ony and hunt judge empty containers falsy",
            "blether aw([1, [], 2], |v| v)\nblether aw([1, [1], {\"a\": 1}], |v| v)\nblether ony([{}, [], \"\", 0], |v| v)\nblether ony([{}, [5]], |v| v)\nblether hunt([[], {}, [3]], |v| v)\nblether hunt([[], {}], |v| v)",
            "nae\naye\nnae\naye\n[3]\nnaething",
        ),
        (
            "00110 wheesht drops falsy list items and trims ASCII whitespace",
            "blether wheesht([0, 1, \"\", \"x\", [], [2], naething, aye, nae, {}])\nblether \"[\" + wheesht(\"  hi \\t\") + \"]\"",
            "[1, x, [2], aye]\n[hi]",
        ),
        (
            "00110 callbacks get only the item, as in the interpreter",
            "dae f(x, extra = 5) { gie x + extra }\nblether gaun([1, 2], f)\nblether sieve([1, 2], |x| x > 1)\nblether tumble([1, 2, 3], 0, |a, x| a + x)",
            "[6, 7]\n[2]\n6",
        ),
    ]);
}

#[test]
fn parity_js_missing_builtins_mdh_bug_flu_00113() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00113 ilk, grup_up and pairt_by",
            r#"dae p(x) { blether x * 10 }
blether ilk([1, 2], p)
blether grup_up([1, 2, 3, 4], |x| x % 2)
blether grup_up(["a", "bb", "c"], len)
blether pairt_by([1, 2, 3, 4], |x| x > 2)
blether pairt_by([{}, [1]], |x| x)"#,
            "10\n20\nnaething\n{\"1\": [1, 3], \"0\": [2, 4]}\n{\"1\": [a, c], \"2\": [bb]}\n[[3, 4], [1, 2]]\n[[[1]], [{}]]",
        ),
        (
            "00113 dict_get, dict_has, dict_remove, dict_invert and fae_pairs use typed keys",
            r#"ken d = {"a": 1, 2: "two"}
blether dict_get(d, "a", 0)
blether dict_get(d, "zz", 0)
blether dict_get(d, 2, 0)
blether dict_get(d, "2", 0)
blether dict_has(d, 2)
blether dict_has(d, 2.0)
blether dict_remove(d, "a")
blether d
blether dict_invert({"a": 1, "b": 1, "c": 2})
blether dict_invert({1: "x"})
blether fae_pairs([["a", 1], [2, 3], [9], "no"])"#,
            "1\n0\ntwo\n0\naye\nnae\n{\"2\": two}\n{\"a\": 1, \"2\": two}\n{\"1\": b, \"2\": c}\n{\"x\": 1}\n{\"a\": 1, \"2\": 3}",
        ),
        (
            "00113 higher-order builtins raise the interpreter's errors",
            r#"hae_a_bash {
    sieve(1, |x| x)
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    gaun([1], 5)
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    tumble("s", 0, |a, x| a)
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    grup_up(1, len)
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    ilk([1], naething)
} gin_it_gangs_wrang e {
    blether e
}
blether ony([], 5)"#,
            "That's pure mince! Type error at line 2: sieve() expects a list as first argument\nWhit's aw this aboot? '5' isnae a function at line 7\nThat's pure mince! Type error at line 12: tumble() expects a list as first argument\nThat's pure mince! Type error at line 17: grup_up() expects a list as first argument\nWhit's aw this aboot? 'naething' isnae a function at line 22\nnae",
        ),
        (
            "00113 dict builtins raise the interpreter's errors",
            r#"hae_a_bash {
    dict_get(1, 2, 3)
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    dict_has([], 1)
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    dict_remove("x", 1)
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    dict_invert(1)
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    fae_pairs(1)
} gin_it_gangs_wrang e {
    blether e
}"#,
            "Jings! Something went awfy wrang: dict_get() needs a dictionary\nJings! Something went awfy wrang: dict_has() needs a dictionary\nJings! Something went awfy wrang: dict_remove() needs a dictionary\nJings! Something went awfy wrang: dict_invert() needs a dictionary\nJings! Something went awfy wrang: fae_pairs() needs a list o' pairs",
        ),
    ]);
}

#[test]
fn parity_js_pure_builtins_mdh_bug_flu_00114() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "00114 string builtins match the interpreter by code point and error text",
            r####"dae tryit(f) {
    hae_a_bash {
        blether f()
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether chars("a😀b")
blether char_at("a😀b", 1)
blether char_at("a😀b", -1)
blether char_at("abc", 1.9)
tryit(|| char_at("abc", 3))
tryit(|| char_at("abc", -4))
tryit(|| char_at(5, 0))
tryit(|| char_at("abc", "x"))
blether ord("😀")
blether chr(128512)
blether chr(65)
tryit(|| ord(""))
tryit(|| chr(55296))
tryit(|| chr(1114112))
tryit(|| chr(65.0))
tryit(|| chr("a"))
blether "[" + trim("  \t hi \n ") + "]"
blether "[" + trim_start("  hi  ") + "]"
blether "[" + trim_end("  hi  ") + "]"
blether "[" + strip_left("xxyhixy", "xy") + "]"
blether "[" + strip_right("xxyhixy", "xy") + "]"
blether words("  a  b\tc\nd  ")
blether lines("a\nb\r\nc\n")
blether len(lines("a\r")[0])
blether lines("\n\nx")
blether lines("")
blether title("hello wORLD  éa x")
blether capitalize("éa")
blether capitalize("abc")
blether swapcase("aBc é1")
blether "[" + center("ab", 7) + "]"
blether "[" + center("ab", 7, "*") + "]"
blether "[" + center("ab", 1) + "]"
blether "[" + pad_left("ab", 5, " ") + "]"
blether "[" + pad_left("ab", 5, "😀") + "]"
blether "[" + pad_right("ab", 5, "-") + "]"
blether "[" + pad_right("ab", 5, "") + "]"
tryit(|| pad_right("ab", 5, 1))
tryit(|| center("ab", 5.0))
tryit(|| pad_left(1, 2))
blether repeat("ab", 3)
blether repeat("😀", 2)
tryit(|| repeat("ab", -1))
tryit(|| repeat("ab", 1.5))
tryit(|| repeat("ab", 2000000000))
blether replace("a-b-c", "-", "+")
blether replace("a😀b", "", "_")
blether replace_first("a-b-c", "-", "+")
blether replace_first("abc", "", "_")
tryit(|| replace("a", "b", 1))
blether starts_with("hello", "he")
blether starts_wi("hello", "lo")
blether ends_with("hello", "lo")
blether ends_wi("hello", "he")
tryit(|| starts_with("a", 1))
blether index_of("a😀bc", "c")
blether index_of("abc", "z")
blether index_of([1, 2, [3]], [3])
blether index_of([1, 2], 2.0)
blether last_index_of("a😀bab", "b")
blether last_index_of("abc", "")
blether indices_o("a😀aa😀a", "a")
blether indices_o("aaaa", "aa")
blether indices_o([1, 2, 1, 1.0], 1)
tryit(|| indices_o("abc", ""))
tryit(|| indices_o("abc", 1))
tryit(|| indices_o(5, 1))
blether substring("a😀bcd", 1, 3)
blether substring("abcd", -3, -1)
blether substring("abcd", 3, 1)
blether substring("abcd", -10, 99)
blether substring("abcd", 1.7, 3.2)
tryit(|| substring("abc", "a", 2))
blether substr_between("a[bc]d", "[", "]")
blether substr_between("a[bcd", "[", "]")
blether substr_between("aé[x]", "[", "]")
blether is_alpha("abC")
blether is_alpha("ab1")
blether is_alpha("")
blether is_alpha("é")
blether is_digit("123")
blether is_digit("")
blether is_space(" \t\n")
blether is_space("")
blether is_upper("AB1")
blether is_upper("1")
blether is_upper("Ab")
blether is_lower("ab1")
blether is_lower("aB")
blether is_blank("")
blether is_blank("  \t")
blether is_blank("a")
blether is_empty("")
blether is_empty([])
blether is_empty({})
blether is_empty([1])
tryit(|| is_empty(5))
tryit(|| is_alpha(5))
tryit(|| words(5))
tryit(|| trim(5))
blether split_by([1, 2, 3, -3, 2.5, "a", naething, 0], "even")
blether split_by([1, 2, 3, -3, 2.5, "a", naething, 0], "odd")
blether split_by([1, 2, 3, -3, 2.5, "a", naething, 0], "positive")
blether split_by([1, 2, 3, -3, 2.5, "a", naething, 0], "negative")
blether split_by([1, 2, 3, -3, 2.5, "a", naething, 0, []], "truthy")
blether split_by([1, 2, 3, -3, 2.5, "a", naething, 0], "nil")
blether split_by([1, 2, 3, -3, 2.5, "a", naething, 0], "string")
blether split_by([1, 2, 3, -3, 2.5, "a", naething, 0], "number")
tryit(|| split_by([1], "wat"))
tryit(|| split_by([1], 5))"####,
            r####"[a, 😀, b]
😀
b
b
Jings! Something went awfy wrang: Index 3 oot o' bounds fer string o' length 3
Jings! Something went awfy wrang: Index -1 oot o' bounds fer string o' length 3
Jings! Something went awfy wrang: char_at() needs a string
Jings! Something went awfy wrang: char_at() needs an integer index
128512
😀
A
Jings! Something went awfy wrang: Cannae get ord o' empty string!
Jings! Something went awfy wrang: Invalid Unicode codepoint: 55296
Jings! Something went awfy wrang: chr() needs a valid Unicode codepoint (0 to 1114111), got 1114112
Jings! Something went awfy wrang: chr() needs an integer
Jings! Something went awfy wrang: chr() needs an integer
[hi]
[hi  ]
[  hi]
[hixy]
[xxyhi]
[a, b, c, d]
[a, b, c]
2
[, , x]
[]
Hello World  éa X
éa
Abc
AbC é1
[  ab   ]
[**ab***]
[ab]
[   ab]
[😀😀😀ab]
[ab---]
[ab   ]
Jings! Something went awfy wrang: pad_right() needs (string, width, pad_char)
Jings! Something went awfy wrang: center() needs a width
Jings! Something went awfy wrang: pad_left() needs (string, width, pad_char)
ababab
😀😀
Jings! Something went awfy wrang: Cannae repeat a negative number o' times!
Jings! Something went awfy wrang: repeat() needs a string and an integer
Jings! Something went awfy wrang: repeat() wid be ower muckle (4000000000 is mair than the 1073741824 limit), ya greedy gannet!
a+b+c
_a_😀_b_
a+b-c
_abc
Jings! Something went awfy wrang: replace() needs three strings
aye
nae
aye
nae
Jings! Something went awfy wrang: starts_with() needs two strings
3
-1
2
1
4
3
[0, 2, 3, 5]
[0, 2]
[0, 2, 3]
Jings! Something went awfy wrang: Cannae search fer an empty string, ya numpty!
Jings! Something went awfy wrang: indices_o() on string needs a string needle
Jings! Something went awfy wrang: indices_o() needs a list or string
😀b
bc

abcd
bc
Jings! Something went awfy wrang: substring() needs integer indices
bc
naething
x
aye
nae
nae
nae
aye
nae
aye
nae
aye
nae
nae
aye
nae
aye
aye
nae
aye
aye
aye
nae
Jings! Something went awfy wrang: is_empty() needs a string, list, or dict
Jings! Something went awfy wrang: is_alpha() needs a string
Jings! Something went awfy wrang: words() needs a string
Jings! Something went awfy wrang: trim() needs a string
[[2, 0], [1, 3, -3, 2.5, a, naething]]
[[1, 3, -3], [2, 2.5, a, naething, 0]]
[[1, 2, 3, 2.5], [-3, a, naething, 0]]
[[-3], [1, 2, 3, 2.5, a, naething, 0]]
[[1, 2, 3, -3, 2.5, a], [naething, 0, []]]
[[naething], [1, 2, 3, -3, 2.5, a, 0]]
[[a], [1, 2, 3, -3, 2.5, naething, 0]]
[[1, 2, 3, -3, 2.5, 0], [a, naething]]
Jings! Something went awfy wrang: Unknown predicate 'wat'. Try: even, odd, positive, negative, truthy, nil, string, number
Jings! Something went awfy wrang: split_by() needs a list and a predicate string"####,
        ),
        (
            "00114 list builtins and tae_bool match the interpreter",
            r####"dae tryit(f) {
    hae_a_bash {
        blether f()
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether zip([1, 2, 3], ["a", "b"])
blether zip([], [1])
tryit(|| zip([1], 2))
blether enumerate(["x", "y"])
tryit(|| enumerate("x"))
blether flatten([1, [2, 3], [[4]], []])
tryit(|| flatten(1))
blether uniq([1, 1.0, "1", 1, [1], [1], {"a": 1}, {"a": 1}, naething, naething, aye, 1.0, 0.0])
blether uniq([[1, 2], [1, 2.0], [1, 2]])
tryit(|| uniq(1))
blether unique([1, 1.0, "1", 1, [1], [1.0], {"a": 1}, {"a": 1.0}, naething, naething])
tryit(|| unique("a"))
blether chunks([1, 2, 3, 4, 5], 2)
blether chunks([1, 2, 3], 10)
blether chunks([], 3)
blether chunks([1, 2, 3], 2.9)
tryit(|| chunks([1], 0))
tryit(|| chunks([1], "a"))
tryit(|| chunks(1, 2))
blether grup([1, 2, 3, 4, 5], 2)
blether grup([1, 2, 3], 9223372036854775807)
tryit(|| grup([1], 0))
tryit(|| grup([1], 1.0))
tryit(|| grup(1, 1))
blether pair_up([1, 2, 3, 4, 5])
blether pair_up([])
tryit(|| pair_up(1))
blether interleave([1, 2, 3], ["a"])
blether fankle([1], ["a", "b", "c"])
blether ceilidh([1, 2], ["a", "b"])
blether ceilidh([], [])
tryit(|| interleave(1, [2]))
tryit(|| fankle([1], 2))
tryit(|| ceilidh(1, 2))
blether tak([1, 2, 3], 2)
blether tak([1, 2, 3], -1)
blether tak([1, 2, 3], 99)
blether tak([1, 2, 3], 1.9)
blether tak([1, 2, 3], 9223372036854775807)
blether drap([1, 2, 3], 2)
blether drap([1, 2, 3], -5)
blether drap([1, 2, 3], 99)
blether drap([1, 2, 3], 1.5)
tryit(|| tak(1, 2))
tryit(|| tak([1], "a"))
tryit(|| drap(1, 1))
blether grup_runs([1, 1, 2, 2.0, 1, 3])
blether grup_runs([])
blether grup_runs([[1], [1], [2]])
tryit(|| grup_runs(1))
blether indices_o([1, 2, 1], 1)
blether tae_bool(0)
blether tae_bool([])
blether tae_bool("a")
blether tae_bool(naething)
blether tae_bool(0.0)"####,
            r####"[[1, a], [2, b]]
[]
Jings! Something went awfy wrang: zip() needs two lists
[[0, x], [1, y]]
Jings! Something went awfy wrang: enumerate() needs a list
[1, 2, 3, [4]]
Jings! Something went awfy wrang: flatten() needs a list
[1, 1, 1, [1], {"a": 1}, naething, aye, 0]
[[1, 2], [1, 2]]
Jings! Something went awfy wrang: uniq() needs a list
[1, 1, [1], {"a": 1}, naething]
Jings! Something went awfy wrang: unique() needs a list
[[1, 2], [3, 4], [5]]
[[1, 2, 3]]
[]
[[1, 2], [3]]
Jings! Something went awfy wrang: chunks() size must be positive
Jings! Something went awfy wrang: chunks() needs an integer size
Jings! Something went awfy wrang: chunks() needs a list
[[1, 2], [3, 4], [5]]
[[1, 2, 3]]
Jings! Something went awfy wrang: grup() needs a positive chunk size
Jings! Something went awfy wrang: grup() needs a list and an integer
Jings! Something went awfy wrang: grup() needs a list and an integer
[[1, 2], [3, 4], [5]]
[]
Jings! Something went awfy wrang: pair_up() needs a list
[1, a, 2, 3]
[1, a, b, c]
[1, a, 2, b]
[]
Jings! Something went awfy wrang: interleave() needs two lists
Jings! Something went awfy wrang: fankle() needs two lists
Jings! Something went awfy wrang: ceilidh needs two lists
[1, 2]
[]
[1, 2, 3]
[1]
[1, 2, 3]
[3]
[1, 2, 3]
[]
[2, 3]
Jings! Something went awfy wrang: tak() needs a list and an integer
Jings! Something went awfy wrang: tak() needs a list and an integer
Jings! Something went awfy wrang: drap() needs a list and an integer
[[1, 1], [2, 2], [1], [3]]
[]
[[[1], [1]], [[2]]]
Jings! Something went awfy wrang: grup_runs() needs a list
[0, 2]
nae
nae
aye
nae
nae"####,
        ),
        (
            "00114 conversion and bit builtins match the interpreter on i64 edges",
            r####"dae tryit(f) {
    hae_a_bash {
        blether f()
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether tae_binary(10)
blether tae_binary(0)
blether tae_binary(-1)
blether tae_hex(255)
blether tae_hex(-255)
blether tae_hex(-9223372036854775807 - 1)
blether tae_octal(64)
blether tae_octal(-8)
tryit(|| tae_binary(1.5))
tryit(|| tae_hex("a"))
tryit(|| tae_octal(naething))
blether fae_binary("1010")
blether fae_binary("0b11")
blether fae_binary("0b0b11")
blether fae_binary("-101")
blether fae_binary("+11")
tryit(|| fae_binary("102"))
tryit(|| fae_binary(""))
tryit(|| fae_binary("0b"))
tryit(|| fae_binary("-0b1"))
tryit(|| fae_binary(repeat("1", 64)))
tryit(|| fae_binary(5))
blether fae_hex("ff")
blether fae_hex("0xFF")
blether fae_hex("-ff")
blether fae_hex("7fffffffffffffff")
blether fae_hex("0b1")
tryit(|| fae_hex("8000000000000000"))
tryit(|| fae_hex("xyz"))
tryit(|| fae_hex("1_0"))
tryit(|| fae_hex(1))
blether bit_an(12, 10)
blether bit_or(12, 10)
blether bit_xor(12, 10)
blether bit_an(-1, 255)
blether bit_or(-8, 3)
blether bit_xor(-1, 5)
blether bit_nae(0)
blether bit_nae(-1)
blether bit_nae(9223372036854775807)
blether bit_shove_left(1, 4)
blether bit_shove_left(1, 62)
blether bit_shove_left(1, 63)
blether bit_shove_left(3, 63)
blether bit_shove_left(-1, 60)
blether bit_shove_right(256, 4)
blether bit_shove_right(-256, 4)
blether bit_shove_right(-1, 63)
tryit(|| bit_shove_left(1, 64))
tryit(|| bit_shove_right(1, -1))
tryit(|| bit_an(1.0, 1))
tryit(|| bit_or("a", 1))
tryit(|| bit_xor(1, naething))
tryit(|| bit_nae(1.5))
tryit(|| bit_shove_left(1.0, 1))
blether bit_coont(255)
blether bit_coont(0)
blether bit_coont(-1)
blether bit_coont(-9223372036854775807 - 1)
tryit(|| bit_coont("a"))"####,
            r####"1010
0
1111111111111111111111111111111111111111111111111111111111111111
ff
ffffffffffffff01
8000000000000000
100
1777777777777777777770
Jings! Something went awfy wrang: tae_binary() needs an integer
Jings! Something went awfy wrang: tae_hex() needs an integer
Jings! Something went awfy wrang: tae_octal() needs an integer
10
3
3
-5
3
Jings! Something went awfy wrang: Cannae parse '102' as binary
Jings! Something went awfy wrang: Cannae parse '' as binary
Jings! Something went awfy wrang: Cannae parse '0b' as binary
Jings! Something went awfy wrang: Cannae parse '-0b1' as binary
Jings! Something went awfy wrang: Cannae parse '1111111111111111111111111111111111111111111111111111111111111111' as binary
Jings! Something went awfy wrang: fae_binary() needs a string
255
255
-255
9223372036854775807
177
Jings! Something went awfy wrang: Cannae parse '8000000000000000' as hex
Jings! Something went awfy wrang: Cannae parse 'xyz' as hex
Jings! Something went awfy wrang: Cannae parse '1_0' as hex
Jings! Something went awfy wrang: fae_hex() needs a string
8
14
6
255
-5
-6
-1
0
-9223372036854775808
16
4611686018427387904
-9223372036854775808
-9223372036854775808
-1152921504606846976
16
-16
-1
Jings! Something went awfy wrang: Shift amount must be 0-63, ya numpty!
Jings! Something went awfy wrang: Shift amount must be 0-63, ya numpty!
Jings! Something went awfy wrang: bit_an() needs two integers
Jings! Something went awfy wrang: bit_or() needs two integers
Jings! Something went awfy wrang: bit_xor() needs two integers
Jings! Something went awfy wrang: bit_nae() needs an integer
Jings! Something went awfy wrang: bit_shove_left() needs two integers
8
0
64
1
Jings! Something went awfy wrang: bit_coont() needs an integer"####,
        ),
        (
            "00114 json_parse, json_pretty and assertions match the interpreter",
            r####"dae tryit(f) {
    hae_a_bash {
        blether f()
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether json_parse("  [1, 2.5, -3, 1e2, 1E-2, true, false, null, \"hi\"]  ")
blether json_parse("{\"a\": 1, \"b\": [1, {\"c\": null}], \"a\": 2, \"1\": \"one\"}")
blether json_parse("{}")
blether json_parse("[]")
blether json_parse("12345678901234567890")
blether json_parse("9223372036854775807")
blether json_parse("-9223372036854775808")
blether json_parse("-0")
blether json_parse("1 2")
blether json_parse("1.")
blether json_parse("\"a\\n\\t\\\"\\\\\\/b\\u0041\\u00e9\\ud83d\\q\"")
blether json_parse("\"😀 é\"")
blether json_parse("1e999")
blether json_parse("1.5e+3")
blether whit_kind(json_parse("1"))
blether whit_kind(json_parse("1.0"))
blether whit_kind(json_parse("1e2"))
tryit(|| json_parse(""))
tryit(|| json_parse("   "))
tryit(|| json_parse("["))
tryit(|| json_parse("[1"))
tryit(|| json_parse("[1 2]"))
tryit(|| json_parse("{\"a\" 1}"))
tryit(|| json_parse("{\"a\": 1"))
tryit(|| json_parse("{a: 1}"))
tryit(|| json_parse("{\"a\": 1 \"b\": 2}"))
tryit(|| json_parse("tru"))
tryit(|| json_parse("fals"))
tryit(|| json_parse("nul"))
tryit(|| json_parse("?"))
tryit(|| json_parse("-"))
tryit(|| json_parse("-e5"))
tryit(|| json_parse("1e"))
tryit(|| json_parse("1.5e+"))
tryit(|| json_parse("\"abc"))
tryit(|| json_parse("\"abc\\"))
tryit(|| json_parse("\"\\u00\""))
tryit(|| json_parse(5))
tryit(|| json_parse(repeat("[", 600) + repeat("]", 600)))
blether len(json_parse(repeat("[", 512) + repeat("]", 512)))
blether json_pretty([1, [2, {"a": [], "b": {}}], "x"])
blether json_pretty({"a": 1, 2: 3.5, "c": [naething, aye, nae]})
blether json_pretty([])
blether json_pretty({})
blether json_pretty("q\"uote\n")
blether json_pretty(1.0)
blether json_pretty(naething)
blether json_stringify_pretty({"k": [1, 2]})
ken cyc = [1]
shove(cyc, cyc)
tryit(|| json_pretty(cyc))
tryit(|| json_stringify_pretty(cyc))
tryit(|| assert(aye, "fine"))
blether assert(1, "fine")
tryit(|| assert(nae, "it broke"))
tryit(|| assert(0, 42))
tryit(|| assert([], [1, "a"]))
blether assert_equal(1, 1.0)
blether assert_equal([1, {"a": 2}], [1, {"a": 2}])
tryit(|| assert_equal(1, 2))
tryit(|| assert_equal("a", "b"))
tryit(|| assert_equal([1], [2]))
blether assert_nae_equal(1, 2)
tryit(|| assert_nae_equal(1, 1.0))
tryit(|| assert_nae_equal("x", "x"))"####,
            r####"[1, 2.5, -3, 100, 0.01, aye, nae, naething, hi]
{"a": 2, "b": [1, {"c": naething}], "1": one}
{}
[]
12345678901234567000
9223372036854775807
-9223372036854775808
0
1
1
a
	"\/bAéq
😀 é
inf
1500
integer
float
float
Jings! Something went awfy wrang: Empty JSON string
Jings! Something went awfy wrang: Empty JSON string
Jings! Something went awfy wrang: Unexpected end of JSON
Jings! Something went awfy wrang: Unterminated JSON array
Jings! Something went awfy wrang: Expected ']' or ',' in JSON array, got '2'
Jings! Something went awfy wrang: Expected ':' in JSON object
Jings! Something went awfy wrang: Unterminated JSON object
Jings! Something went awfy wrang: Expected string key in JSON object
Jings! Something went awfy wrang: Expected '}' or ',' in JSON object, got '"'
Jings! Something went awfy wrang: Invalid JSON value 'true'
Jings! Something went awfy wrang: Invalid JSON value 'false'
Jings! Something went awfy wrang: Invalid JSON value 'null'
Jings! Something went awfy wrang: Unexpected character '?' in JSON
Jings! Something went awfy wrang: Invalid integer: -
Jings! Something went awfy wrang: Invalid number: -e5
Jings! Something went awfy wrang: Invalid number: 1e
Jings! Something went awfy wrang: Invalid number: 1.5e+
Jings! Something went awfy wrang: Unterminated JSON string
Jings! Something went awfy wrang: Unterminated string escape
Jings! Something went awfy wrang: Invalid unicode escape
Jings! Something went awfy wrang: json_parse() expects a string, ya numpty!
Jings! Something went awfy wrang: JSON's nested ower deep (mair than 512 levels) - ah'm no' diggin' that far!
1
[
  1,
  [
    2,
    {
      "a": [],
      "b": {}
    }
  ],
  "x"
]
{
  "a": 1,
  "2": 3.5,
  "c": [
    null,
    true,
    false
  ]
}
[]
{}
"q\"uote\n"
1
null
{
  "k": [
    1,
    2
  ]
}
Jings! Something went awfy wrang: Cannae write JSON fer a list or dict that contains itsel - it wid gang roond forever!
Jings! Something went awfy wrang: Cannae write JSON fer a list or dict that contains itsel - it wid gang roond forever!
aye
aye
Jings! Something went awfy wrang: Assertion failed: it broke
Jings! Something went awfy wrang: Assertion failed: 42
Jings! Something went awfy wrang: Assertion failed: [1, a]
aye
aye
Jings! Something went awfy wrang: Assertion failed: expected 1 but got 2
Jings! Something went awfy wrang: Assertion failed: expected a but got b
Jings! Something went awfy wrang: Assertion failed: expected [1] but got [2]
aye
Jings! Something went awfy wrang: Assertion failed: 1 should not equal 1
Jings! Something went awfy wrang: Assertion failed: x should not equal x"####,
        ),
    ]);
}

#[test]
fn parity_js_unicode_string_builtins_mdh_bug_flu_00116() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[(
        "00116 upper/lower/heid/tail/bum/reverse/split work by code point, ASCII-only case",
        r####"dae tryit(f) {
    hae_a_bash {
        blether f()
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether upper("straße café ÿ abc")
blether lower("STRASSE ÉCOLE ABC")
blether heid("😀ab")
blether heid("éx")
blether tail("😀ab")
blether tail("éx")
blether bum("ab😀")
blether bum("xé")
blether reverse("a😀b")
blether reverse("ßé")
blether split("a😀b", "")
blether split("", "")
blether split("a😀b😀c", "😀")
blether coont("a😀b", "")
blether coont("a😀b😀", "😀")
tryit(|| heid(""))
tryit(|| bum(""))
tryit(|| upper(5))
tryit(|| split("a", 1))
blether len(tail("😀"))"####,
        "STRAßE CAFé ÿ ABC\nstrasse École abc\n😀\ne\nab\ńx\n😀\ń\nb😀a\ńeß\n[a😀b]\n[]\n[a, b, c]\n4\n2\nJings! Something went awfy wrang: Cannae get heid o' empty string!\nJings! Something went awfy wrang: Cannae get bum o' empty string!\nJings! Something went awfy wrang: upper() expects a string\nJings! Something went awfy wrang: split() expects two strings\n0",
    )]);
}

#[test]
fn parity_js_scots_builtins_and_class_semantics_residual_gaps() {
    if !node_available() {
        return;
    }
    assert_js_parity(&[
        (
            "scots string/list builtins: roar..chynge, clype, errors",
            r####"dae tryit(f) {
    hae_a_bash {
        blether f()
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether roar("abcß")
blether mutter("ABCÜ")
tryit(|| roar(1))
tryit(|| mutter(1))
blether wheesht_aw(" a  b\t\nc ")
tryit(|| wheesht_aw(1))
blether clarty([1, 2, 1])
blether clarty([1, 2, 3])
blether clarty("abca")
blether clarty("")
tryit(|| clarty(5))
blether dreich("aaa")
blether dreich("")
blether dreich("ab")
tryit(|| dreich(1))
blether geggie("")
blether geggie("x")
blether geggie("😀ab")
tryit(|| geggie(1))
blether banter("abc", "x")
blether banter("", "😀😀")
tryit(|| banter(1, "a"))
blether skelp("abcde", 2)
blether skelp("", 3)
tryit(|| skelp("a", 0))
tryit(|| skelp("a", "b"))
blether haverin("")
blether haverin(" a ")
blether haverin("ab")
blether haverin("😀")
blether haverin(naething)
blether haverin([])
blether haverin([1])
blether haverin(5)
blether glaikit(naething)
blether glaikit(0)
blether glaikit(0.0)
blether glaikit(1)
blether glaikit("")
blether glaikit(" \t")
blether glaikit("a")
blether glaikit([])
blether glaikit({})
blether glaikit({"a": 1})
blether haggis_hunt("aaaa", "aa")
blether haggis_hunt("a😀b😀", "😀")
tryit(|| haggis_hunt(1, "a"))
tryit(|| haggis_hunt("a", 1))
blether sporran_fill("ab", 5, "*")
blether sporran_fill("ab", 6, "😀")
tryit(|| sporran_fill(1, 5, "*"))
tryit(|| sporran_fill("a", "x", "*"))
tryit(|| sporran_fill("a", 5, 1))
tryit(|| sporran_fill("a", 9999999999999, "*"))
blether tattie_scone("ab", 1)
blether tattie_scone("ab", 2)
tryit(|| tattie_scone(1, 2))
tryit(|| tattie_scone("a", "b"))
tryit(|| tattie_scone("a", 9999999999999))
blether birl([1, 2, 3], 4)
blether birl([1, 2, 3], -4)
blether birl([], 3)
tryit(|| birl([1], 1.5))
blether chynge([1, 2, 3], 3, 9)
blether chynge([1, 2, 3], -3, 9)
blether chynge([1, 2, 3], 1.9, 9)
tryit(|| chynge([1, 2, 3], 4, 9))
tryit(|| chynge([1, 2, 3], -4, 9))
tryit(|| chynge(5, 0, 9))
blether clype([1, 2])
blether clype({"a": 1})
blether clype(5)
blether clype(2.5)
blether clype(aye)
blether clype(naething)
blether clype("héllo")
blether clype(creel([1]))
blether len(stooshie("a😀bc"))
tryit(|| blooter(1))
tryit(|| stooshie(1))"####,
            "ABCß!\n...abcÜ...\nJings! Something went awfy wrang: roar() expects a string\nJings! Something went awfy wrang: mutter() expects a string\na b c\nJings! Something went awfy wrang: wheesht_aw() needs a string\naye\nnae\naye\nnae\nJings! Something went awfy wrang: clarty() needs a list or string\naye\naye\nnae\nJings! Something went awfy wrang: dreich() needs a string\n\nxx\n😀b\nJings! Something went awfy wrang: geggie() needs a string\naxbc\n😀😀\nJings! Something went awfy wrang: banter() needs two strings\n[ab, cd, e]\n[]\nJings! Something went awfy wrang: skelp() size must be positive\nJings! Something went awfy wrang: skelp() needs integer size\naye\naye\nnae\nnae\naye\naye\nnae\nnae\naye\naye\naye\nnae\naye\naye\nnae\naye\naye\nnae\n[0, 2]\n[1, 3]\nJings! Something went awfy wrang: haggis_hunt needs a string tae search\nJings! Something went awfy wrang: haggis_hunt needs a string tae find\n*ab**\n😀😀ab😀😀\nJings! Something went awfy wrang: sporran_fill needs a string\nJings! Something went awfy wrang: sporran_fill needs a width\nJings! Something went awfy wrang: sporran_fill needs a fill character\nJings! Something went awfy wrang: sporran_fill wid be ower muckle (9999999999998 is mair than the 1073741824 limit), ya greedy gannet!\nab\nab | ab\nJings! Something went awfy wrang: tattie_scone needs a string\nJings! Something went awfy wrang: tattie_scone needs a number\nJings! Something went awfy wrang: tattie_scone wid be ower muckle (39999999999996 is mair than the 1073741824 limit), ya greedy gannet!\n[2, 3, 1]\n[3, 1, 2]\n[]\nJings! Something went awfy wrang: birl needs a rotation count\n[1, 2, 3, 9]\n[9, 1, 2, 3]\n[1, 9, 2, 3]\nJings! Something went awfy wrang: Index 4 oot o' bounds fer list o' length 3\nJings! Something went awfy wrang: Index 18446744073709551615 oot o' bounds fer list o' length 3\nJings! Something went awfy wrang: chynge() needs a list\n[list] list wi' 2 items\n[dict] dict wi' 1 entries\n[integer] integer: 5\n[float] float: 2.5\n[bool] boolean: aye\n[naething] naething\n[string] string o' 5 characters\n[creel] creel wi' 1 items\n4\nJings! Something went awfy wrang: blooter() expects a string\nJings! Something went awfy wrang: stooshie needs a string",
        ),
        (
            "class instances: operator overloads, bound method values, own field vs method, masel in nested dae, missing field",
            r####"dae tryit(f) {
    hae_a_bash {
        blether f()
    } gin_it_gangs_wrang e {
        blether e
    }
}
kin V {
    dae init(n) { masel.n = n }
    dae __pit_thegither__(o) { gie V(masel.n + o.n) }
    dae __tak_awa__(o) { gie masel.n - o.n }
    dae __times__(o) { gie masel.n * o.n }
    dae __pairt__(o) { gie masel.n / o.n }
    dae __lave__(o) { gie masel.n % o.n }
    dae __same_as__(o) { gie masel.n == o.n }
    dae __differs_fae__(o) { gie "differs" }
    dae __wee_er__(o) { gie masel.n < o.n }
    dae __wee_er_or_same__(o) { gie masel.n <= o.n }
    dae __muckle_er__(o) { gie masel.n > o.n }
    dae __muckle_er_or_same__(o) { gie masel.n >= o.n }
    dae show() { gie masel.n }
    dae later(k = masel.n) {
        dae inner(a = masel.n) { gie a + k }
        gie inner
    }
}
ken a = V(6)
ken b = V(3)
blether (a + b).show()
blether a - b
blether a * b
blether a / b
blether a % b
blether a == b
blether a != b
blether a < b
blether a <= b
blether a > b
blether a >= b
ken f = a.later()
blether f()
a.n = 10
blether f()
blether f(1)
ken g = a.show
a.n = 11
blether g()
tryit(|| a.nope)
tryit(|| a.nope())
kin P {
    dae init() { masel.go = |x| x + 100 }
    dae go(x) { gie x + 1 }
}
blether P().go(1)
ken h = P().go
blether h(1)"####,
            "9\n3\n18\n2\n0\nnae\ndiffers\nnae\nnae\naye\naye\n12\n16\n7\n11\nAwa' an bile yer heid! 'nope' hasnae been defined yet at line 48\nAwa' an bile yer heid! 'nope' hasnae been defined yet at line 49\n2\n101",
        ),
    ]);
}

#[test]
fn js_instance_equality_overload_runs_against_literals_mdh_bug_flu_00126() {
    if !node_available() {
        return;
    }
    let source = r#"kin W {
    dae __same_as__(o) { gie "same-called" }
    dae __differs_fae__(o) { gie "differs-called" }
}
ken w = W()
blether w == "a"
blether w != "a"
blether w == aye
blether w != nae
blether "a" == "a"
blether aye != nae
"#;
    let interp = run_interpreter(source).unwrap();
    assert_eq!(interp.lines().next(), Some("same-called"));
    assert_eq!(run_js(source).unwrap().trim_end(), interp.trim_end());
}

// MDH-BUG-FLU-00205: unaliased fetch exposes live module bindings. Keep the
// backend tests separate so an interpreter failure cannot hide native results.
const FETCH_SHARED_MODULE_00205: &str = r#"
ken mg = "m" + "g"
dae setmg() {
    mg = "n" + "h"
    gie "xy"
}
dae getmg() { gie mg }
ken reader = || mg
ken writer = |value| { mg = value }
"#;

fn fetch_shared_case_00205(case: &str) -> (&'static str, &'static str) {
    match case {
        "original" => (
            "fetch \"m\"\nblether mg + setmg()\nblether mg\n",
            "mgxy\nnh",
        ),
        "importer_writes" => (
            "fetch \"m\"\nmg = \"ab\"\nblether getmg()\nsetmg()\nblether mg\n",
            "ab\nnh",
        ),
        "nested" => (
            "fetch \"wrapper\"\nmg = \"ab\"\nblether getmg()\nsetmg()\nblether mg\n",
            "ab\nnh",
        ),
        "nested_shadow" => (
            "fetch \"shadow\"\nblether getmg()\nmg = \"ab\"\nblether shadow_get()\nshadow_set()\nblether mg\nsetmg()\nblether getmg()\nblether mg\n",
            "mg\nab\nchanged\nnh\nchanged",
        ),
        "reimport" => (
            "fetch \"m\"\nsetmg()\nfetch \"m\"\nblether mg\nmg = \"ab\"\nfetch \"m\"\nblether getmg()\n",
            "nh\nab",
        ),
        "closures" => (
            "fetch \"m\"\nmg = \"ab\"\nblether reader()\nwriter(\"cd\")\nblether mg\n",
            "ab\ncd",
        ),
        _ => panic!("unknown shared-fetch case: {case}"),
    }
}

fn stage_fetch_shared_modules_00205(source: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("module tempdir");
    for (name, text) in [
        ("m.braw", FETCH_SHARED_MODULE_00205),
        ("wrapper.braw", "fetch \"m\"\n"),
        (
            "shadow.braw",
            "fetch \"m\"\nken mg = \"wrapper\"\ndae shadow_get() { gie mg }\ndae shadow_set() { mg = \"changed\" }\n",
        ),
        ("main.braw", source),
    ] {
        std::fs::write(dir.path().join(name), text).expect("write module fixture");
    }
    dir
}

fn assert_fetch_shared_interpreter_00205(case: &str) {
    let (source, expected) = fetch_shared_case_00205(case);
    let dir = stage_fetch_shared_modules_00205(source);
    let program = mdhavers::parse(source).expect("parse main");
    let mut interp = Interpreter::with_dir(dir.path());
    interp.interpret(&program).expect("interpret main");
    assert_eq!(interp.get_output().join("\n"), expected, "{case}");
    let variables = interp.get_user_variables();
    let mg = variables.iter().find(|(name, _, _)| name == "mg");
    assert!(mg.is_some(), "imported global missing from inspection");
    let expected_mg = expected.rsplit('\n').next().unwrap();
    assert_eq!(
        mg.unwrap().2,
        expected_mg,
        "inspection must show the live value {expected_mg:?}: {mg:?}"
    );
}

#[cfg(feature = "llvm")]
fn assert_fetch_shared_native_00205(case: &str) {
    let (source, expected) = fetch_shared_case_00205(case);
    let dir = stage_fetch_shared_modules_00205(source);
    let program = mdhavers::parse(source).expect("parse main");
    let executable = dir.path().join("shared_fetch");
    mdhavers::LLVMCompiler::new()
        .compile_to_native_with_source(
            &program,
            &executable,
            2,
            Some(&dir.path().join("main.braw")),
        )
        .expect("compile native main");
    let output = Command::new(&executable)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run native main");
    assert!(
        output.status.success(),
        "{case}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim_end_matches('\n'),
        expected,
        "{case}"
    );
}

macro_rules! fetch_shared_tests_00205 {
    ($interpreter:ident, $native:ident, $case:literal) => {
        #[test]
        fn $interpreter() {
            assert_fetch_shared_interpreter_00205($case);
        }
        #[test]
        #[cfg(feature = "llvm")]
        fn $native() {
            assert_fetch_shared_native_00205($case);
        }
    };
}

fetch_shared_tests_00205!(
    fetch_shared_globals_00205_interpreter_original,
    fetch_shared_globals_00205_native_original,
    "original"
);
fetch_shared_tests_00205!(
    fetch_shared_globals_00205_interpreter_importer_writes,
    fetch_shared_globals_00205_native_importer_writes,
    "importer_writes"
);
fetch_shared_tests_00205!(
    fetch_shared_globals_00205_interpreter_nested,
    fetch_shared_globals_00205_native_nested,
    "nested"
);
fetch_shared_tests_00205!(
    fetch_shared_globals_00205_interpreter_nested_shadow,
    fetch_shared_globals_00205_native_nested_shadow,
    "nested_shadow"
);
fetch_shared_tests_00205!(
    fetch_shared_globals_00205_interpreter_reimport,
    fetch_shared_globals_00205_native_reimport,
    "reimport"
);
fetch_shared_tests_00205!(
    fetch_shared_globals_00205_interpreter_closures,
    fetch_shared_globals_00205_native_closures,
    "closures"
);
