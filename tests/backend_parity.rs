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
    let js = mdhavers::compile_to_js(source).map_err(|e| format!("{e}"))?;
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
