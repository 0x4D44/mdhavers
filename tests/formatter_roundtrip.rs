//! Corpus round-trip test fer `mdhavers fmt` (MDH-BUG-FLU-00025..00031).
//!
//! For every example, stdlib module an' golden fixture that parses:
//! - the checked formatter either succeeds or refuses (only allowlisted files may refuse);
//! - its output re-parses tae an AST equal tae the original (spans aside);
//! - formattin' is idempotent: format(format(x)) == format(x).

use std::path::{Path, PathBuf};

use mdhavers::formatter::{ast_fingerprint, format_source_checked, FormatError};
use mdhavers::parse;

/// Files the formatter legitimately refuses, wi' the reason.
/// Keep this list short: each entry is a file `fmt` will no' touch.
const REFUSE_ALLOWLIST: &[(&str, &str)] = &[];

fn collect_braw(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_braw(&path, true, out);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("braw") {
            out.push(path);
        }
    }
}

fn corpus() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_braw(&root.join("examples"), true, &mut files);
    collect_braw(&root.join("stdlib"), false, &mut files);
    collect_braw(&root.join("tests/golden"), true, &mut files);
    files.sort();
    files
}

fn relative(path: &Path) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn formatter_round_trips_the_corpus() {
    let files = corpus();
    assert!(files.len() > 100, "corpus looks too wee: {}", files.len());

    let mut failures = Vec::new();
    let mut refused = Vec::new();
    let mut formatted_count = 0;

    for path in &files {
        let name = relative(path);
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(original) = parse(&source) else {
            continue; // Only files that parse are in scope
        };

        let out = match format_source_checked(&source) {
            Ok(out) => out,
            Err(FormatError::Refused { line, reason }) => {
                refused.push(name.clone());
                if !REFUSE_ALLOWLIST.iter().any(|(f, _)| *f == name) {
                    failures.push(format!("{}: refused at {:?}: {}", name, line, reason));
                }
                continue;
            }
            Err(FormatError::Parse(e)) => {
                failures.push(format!("{}: parse error in checked path: {}", name, e));
                continue;
            }
        };
        formatted_count += 1;

        match parse(&out) {
            Ok(reparsed) => {
                if ast_fingerprint(&original) != ast_fingerprint(&reparsed) {
                    failures.push(format!("{}: formatted AST differs", name));
                    continue;
                }
            }
            Err(e) => {
                failures.push(format!("{}: formatted output disnae parse: {}", name, e));
                continue;
            }
        }

        match format_source_checked(&out) {
            Ok(again) if again == out => {}
            Ok(_) => failures.push(format!("{}: formattin' is no' idempotent", name)),
            Err(e) => failures.push(format!("{}: second format refused: {}", name, e)),
        }
    }

    for (file, _) in REFUSE_ALLOWLIST {
        if !refused.iter().any(|r| r == file) {
            failures.push(format!(
                "{}: allowlisted but no longer refused - drop it",
                file
            ));
        }
    }

    eprintln!(
        "formatter corpus: {} files, {} formatted, {} refused",
        files.len(),
        formatted_count,
        refused.len()
    );
    assert!(
        formatted_count > 100,
        "too few files formatted: {}",
        formatted_count
    );
    assert!(
        failures.is_empty(),
        "{} formatter round-trip failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
