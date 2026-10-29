#![cfg(feature = "llvm")]
//! MDH-BUG-FLU-00218: scratch slots must have one allocation per function call.

use mdhavers::{parse, LLVMCompiler};
use std::fs::{self, File};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const ITERATIONS: usize = 2_000_000;
const OUTPUT_LIMIT: u64 = 1024 * 1024;

fn check(setup: &str, body: &str, output: &str, expected: &str, iterations: usize) {
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".scratch");
    fs::create_dir_all(&scratch).expect("scratch directory");
    let source = format!(
        "{setup}\nken i = 0\nwhiles i < {iterations} {{\n{body}\ni = i + 1\n}}\n{output}\n"
    );
    let program = parse(&source).unwrap_or_else(|e| panic!("parse: {e:?}\n{source}"));
    let mut failures = Vec::new();
    for rc in [false, true] {
        let dir = tempfile::tempdir_in(&scratch).expect("scratch test directory");
        let exe = dir.path().join(if cfg!(windows) {
            "scratch.exe"
        } else {
            "scratch"
        });
        if let Err(error) = LLVMCompiler::new()
            .with_refcounting(rc)
            .compile_to_native(&program, &exe, 0)
        {
            failures.push(format!("RC={rc}: compilation failed: {error:?}"));
            continue;
        }
        let stdout = dir.path().join("stdout");
        let stderr = dir.path().join("stderr");
        let mut child = Command::new(&exe)
            .env("MDH_RC", if rc { "1" } else { "0" })
            .stdin(Stdio::null())
            .stdout(File::create(&stdout).expect("stdout file"))
            .stderr(File::create(&stderr).expect("stderr file"))
            .spawn()
            .expect("run scratch executable");
        let start = Instant::now();
        let (status, stopped) = loop {
            if let Some(status) = child.try_wait().expect("poll scratch executable") {
                break (Some(status), false);
            }
            let too_much_output = [&stdout, &stderr].iter().any(|path| {
                fs::metadata(path)
                    .map(|m| m.len() > OUTPUT_LIMIT)
                    .unwrap_or(false)
            });
            if start.elapsed() > Duration::from_secs(30) || too_much_output {
                child.kill().unwrap_or_else(|error| {
                    panic!(
                        "cannot kill bounded scratch executable PID {}: {error}",
                        child.id()
                    )
                });
                child.wait().expect("reap scratch executable");
                break (None, true);
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        // Reading at most the limit also bounds failure diagnostics.
        let read = |path: &std::path::Path| {
            use std::io::Read;
            let mut bytes = Vec::new();
            File::open(path)
                .expect("output file")
                .take(OUTPUT_LIMIT)
                .read_to_end(&mut bytes)
                .expect("read output");
            String::from_utf8_lossy(&bytes).into_owned()
        };
        let out = read(&stdout);
        let err = read(&stderr);
        fs::remove_file(&exe).expect("remove built executable");
        if stopped || !status.is_some_and(|s| s.success()) || out != expected || !err.is_empty() {
            failures.push(format!("RC={rc}: status={status:?}, bounded_stop={stopped}, stdout={out:?}, stderr={err:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{}\nsource:\n{source}",
        failures.join("\n")
    );
}

macro_rules! scratch_case {
    ($name:ident, $setup:expr, $body:expr, $output:expr, $expected:expr) => {
        #[test]
        fn $name() {
            check($setup, $body, $output, $expected, ITERATIONS);
        }
    };
}

scratch_case!(
    tail,
    "ken xs = [1]\nken result = naething",
    "result = tail(xs)",
    "blether result",
    "[]\n"
);
scratch_case!(
    sample,
    "ken xs = [1]\nken result = naething",
    "result = sample(xs, 0)",
    "blether result",
    "[]\n"
);
scratch_case!(
    slap,
    "ken xs = []\nken result = naething",
    "result = slap(xs, xs)",
    "blether result",
    "[]\n"
);
scratch_case!(
    zipwith,
    "ken xs = []\nken f = |a, b| a + b\nken result = naething",
    "result = zipwith(f, xs, xs)",
    "blether result",
    "[]\n"
);
scratch_case!(
    reverse,
    "ken xs = []\nken result = naething",
    "result = reverse(xs)",
    "blether result",
    "[]\n"
);
scratch_case!(
    sumaw,
    "ken xs = [1, 2]\nken result = 0",
    "result = sumaw(xs)",
    "blether result",
    "3\n"
);
scratch_case!(
    wheesht_list,
    "ken xs = []\nken result = naething",
    "result = wheesht(xs)",
    "blether result",
    "[]\n"
);
scratch_case!(
    product,
    "ken xs = [2, 3]\nken result = 0",
    "result = product(xs)",
    "blether result",
    "6\n"
);
scratch_case!(
    upper,
    "ken result = naething",
    "result = upper(\"a\")",
    "blether result",
    "A\n"
);
scratch_case!(
    lower,
    "ken result = naething",
    "result = lower(\"A\")",
    "blether result",
    "a\n"
);
scratch_case!(
    char_class,
    "ken result = nae",
    "result = is_upper(\"ABC\")",
    "blether result",
    "aye\n"
);
scratch_case!(
    wheesht_string,
    "ken result = naething",
    "result = wheesht(\" x \")",
    "blether result",
    "x\n"
);
scratch_case!(
    coont,
    "ken result = 0",
    "result = coont(\"aba\", \"a\")",
    "blether result",
    "2\n"
);
scratch_case!(
    string_self_append,
    "ken result = \"x\"",
    "result = result + \"\"",
    "blether result",
    "x\n"
);
scratch_case!(
    popcount,
    "ken result = 0",
    "result = popcount(7)",
    "blether result",
    "3\n"
);
scratch_case!(
    is_number,
    "ken result = nae",
    "result = is_nummer(\"3.5\")",
    "blether result",
    "aye\n"
);
scratch_case!(
    is_prime,
    "ken result = nae",
    "result = is_prime(7)",
    "blether result",
    "aye\n"
);
scratch_case!(
    gcd,
    "ken result = 0",
    "result = gcd(12, 18)",
    "blether result",
    "6\n"
);
scratch_case!(
    lcm,
    "ken result = 0",
    "result = lcm(4, 6)",
    "blether result",
    "12\n"
);
scratch_case!(
    zip,
    "ken xs = []\nken result = naething",
    "result = zip(xs, xs)",
    "blether result",
    "[]\n"
);
scratch_case!(
    replicate,
    "ken result = naething",
    "result = replicate(1, 0)",
    "blether result",
    "[]\n"
);
scratch_case!(
    mean,
    "ken xs = [2, 4]\nken result = 0",
    "result = mean(xs)",
    "blether result",
    "3\n"
);
scratch_case!(
    factorial,
    "ken result = 0",
    "result = factorial(4)",
    "blether result",
    "24\n"
);
scratch_case!(
    split_single,
    "ken result = naething",
    "result = split(\"a\", \",\")",
    "blether len(result)\nblether result[0]",
    "1\na\n"
);
scratch_case!(
    split_multiple,
    "ken result = naething",
    "result = split(\"a\", \",,\")",
    "blether len(result)\nblether result[0]",
    "1\na\n"
);
scratch_case!(
    join,
    "ken xs = [\"a\"]\nken result = naething",
    "result = join(xs, \",\")",
    "blether result",
    "a\n"
);
scratch_case!(
    gaun,
    "ken xs = []\nken f = |x| x\nken result = naething",
    "result = gaun(xs, f)",
    "blether result",
    "[]\n"
);
scratch_case!(
    each,
    "ken xs = [1]\nken calls = 0\ndae f(x) { calls = calls + x }",
    "each(xs, f)",
    "blether calls",
    "2000000\n"
);
scratch_case!(
    sieve,
    "ken xs = [1]\nken f = |x| nae\nken result = naething",
    "result = sieve(xs, f)",
    "blether result",
    "[]\n"
);
scratch_case!(
    pairt_by,
    "ken xs = []\nken f = |x| aye\nken result = naething",
    "result = pairt_by(xs, f)",
    "blether result",
    "[[], []]\n"
);
scratch_case!(
    grup_up,
    "ken xs = []\nken f = |x| x\nken result = naething",
    "result = grup_up(xs, f)",
    "blether len(result)",
    "0\n"
);
scratch_case!(
    tumble,
    "ken xs = [1]\nken f = |acc, x| acc + x\nken result = 0",
    "result = tumble(xs, 2, f)",
    "blether result",
    "3\n"
);
scratch_case!(
    aw_truthy,
    "ken xs = [1]\nken result = nae",
    "result = aw(xs)",
    "blether result",
    "aye\n"
);
scratch_case!(
    ony_truthy,
    "ken xs = [0]\nken result = aye",
    "result = ony(xs)",
    "blether result",
    "nae\n"
);
scratch_case!(
    aw_predicate,
    "ken xs = [1]\nken f = |x| x == 1\nken result = nae",
    "result = aw(xs, f)",
    "blether result",
    "aye\n"
);
scratch_case!(
    ony_predicate,
    "ken xs = [1]\nken f = |x| x == 2\nken result = aye",
    "result = ony(xs, f)",
    "blether result",
    "nae\n"
);
scratch_case!(
    hunt,
    "ken xs = [1]\nken f = |x| nae\nken result = 0",
    "result = hunt(xs, f)",
    "blether result",
    "naething\n"
);
scratch_case!(
    find_index,
    "ken xs = [1]\nken f = |x| nae\nken result = 0",
    "result = find_index(xs, f)",
    "blether result",
    "-1\n"
);
scratch_case!(
    ilk,
    "ken xs = []\nken f = |x| x\nken result = naething",
    "result = ilk(xs, f)",
    "blether result",
    "naething\n"
);
scratch_case!(
    ilkane_list,
    "ken xs = [1]\nken calls = 0\ndae f(x, index) { calls = calls + x }",
    "ilkane(xs, f)",
    "blether calls",
    "2000000\n"
);
scratch_case!(
    ilkane_dict,
    "ken xs = {}\nken f = |k, v| v",
    "ilkane(xs, f)",
    "blether len(xs)",
    "0\n"
);
scratch_case!(
    instance_fields,
    "kin Box { dae init() { masel.x = 1 } }\nken b = Box()\nken result = 0",
    "b.x = 2\nresult = b.x",
    "blether result",
    "2\n"
);
scratch_case!(
    find_predicate,
    "ken xs = [1]\nken f = |x| nae\nken result = 0",
    "result = find(xs, f)",
    "blether result",
    "naething\n"
);

// Spread is covered by the IR property in llvm_stack_allocas.rs: its fixed 2048-slot
// buffer would allocate over 65 GB at this stress count with RC disabled.
