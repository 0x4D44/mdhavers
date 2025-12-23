#![cfg(feature = "cli")]

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn mdhavers_lsp_bin() -> PathBuf {
    // `cargo llvm-cov` builds into `target/llvm-cov-target` and sets LLVM_PROFILE_FILE.
    // Use the instrumented binary in that mode so spawned subprocesses contribute to coverage.
    if std::env::var_os("LLVM_PROFILE_FILE").is_some() {
        let p = PathBuf::from("target/llvm-cov-target/debug/mdhavers-lsp");
        if p.exists() {
            return p;
        }
    }

    if let Some(p) = std::env::var_os("CARGO_BIN_EXE_mdhavers-lsp") {
        return PathBuf::from(p);
    }

    PathBuf::from("target/debug/mdhavers-lsp")
}

fn lsp_frame(json: &str) -> String {
    let bytes = json.as_bytes();
    format!("Content-Length: {}\r\n\r\n{}", bytes.len(), json)
}

#[test]
fn lsp_binary_handles_initialize_requests_and_shutdown() {
    use std::io::Write;

    let mut child = Command::new(mdhavers_lsp_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mdhavers-lsp");

    let stdin = child.stdin.as_mut().expect("stdin");

    // 1) initialize
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}"#;
    stdin.write_all(lsp_frame(init).as_bytes()).unwrap();

    // lsp-server expects an "initialized" notification after initialize.
    let initialized = r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;
    stdin.write_all(lsp_frame(initialized).as_bytes()).unwrap();

    // 2) open a document (valid)
    let uri = "file:///tmp/lsp_coverage_test.braw";
    let did_open = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"{uri}","languageId":"mdhavers","version":1,"text":"ken x = 1\nblether x\n"}}}}}}"#
    );
    stdin.write_all(lsp_frame(&did_open).as_bytes()).unwrap();

    // 3) change the document (invalid) to force diagnostics paths
    let did_change = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"{uri}","version":2}},"contentChanges":[{{"text":"ken =\n"}}]}}}}"#
    );
    stdin.write_all(lsp_frame(&did_change).as_bytes()).unwrap();

    // 3b) trigger lexer error paths
    let did_change_lex_err = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"{uri}","version":3}},"contentChanges":[{{"text":"ken x = 1\n£\n"}}]}}}}"#
    );
    stdin
        .write_all(lsp_frame(&did_change_lex_err).as_bytes())
        .unwrap();

    // 3c) trigger bracket-matching diagnostics branches
    let bracket_cases = [
        // Mismatched closing bracket with a different opener on stack
        ("(}", 4),
        ("{)", 5),
        ("{]", 6),
        // Closing bracket with empty stack
        ("}", 7),
        (")", 8),
        ("]", 9),
        // Unclosed opener at EOF
        ("{", 10),
    ];
    for (text, version) in bracket_cases {
        let did_change = format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"{uri}","version":{version}}},"contentChanges":[{{"text":"{text}\n"}}]}}}}"#
        );
        stdin.write_all(lsp_frame(&did_change).as_bytes()).unwrap();
    }

    // Restore to a normal document so hover/completion can find a word at the cursor.
    let did_change_restore = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"{uri}","version":11}},"contentChanges":[{{"text":"ken x = 1\nblether x\n"}}]}}}}"#
    );
    stdin
        .write_all(lsp_frame(&did_change_restore).as_bytes())
        .unwrap();

    // 4) completion request
    let completion = format!(
        r#"{{"jsonrpc":"2.0","id":3,"method":"textDocument/completion","params":{{"textDocument":{{"uri":"{uri}"}},"position":{{"line":0,"character":1}}}}}}"#
    );
    stdin.write_all(lsp_frame(&completion).as_bytes()).unwrap();

    // 5) hover request
    let hover = format!(
        r#"{{"jsonrpc":"2.0","id":4,"method":"textDocument/hover","params":{{"textDocument":{{"uri":"{uri}"}},"position":{{"line":0,"character":1}}}}}}"#
    );
    stdin.write_all(lsp_frame(&hover).as_bytes()).unwrap();

    // 5b) hover at whitespace (forces start==end None path)
    let hover_whitespace = format!(
        r#"{{"jsonrpc":"2.0","id":8,"method":"textDocument/hover","params":{{"textDocument":{{"uri":"{uri}"}},"position":{{"line":0,"character":3}}}}}}"#
    );
    stdin
        .write_all(lsp_frame(&hover_whitespace).as_bytes())
        .unwrap();

    // 5c) hover out of range (forces None path in word lookup)
    let hover_oob = format!(
        r#"{{"jsonrpc":"2.0","id":6,"method":"textDocument/hover","params":{{"textDocument":{{"uri":"{uri}"}},"position":{{"line":999,"character":999}}}}}}"#
    );
    stdin.write_all(lsp_frame(&hover_oob).as_bytes()).unwrap();

    // 5d) hover beyond line length (col >= line.len() path)
    let hover_col_oob = format!(
        r#"{{"jsonrpc":"2.0","id":9,"method":"textDocument/hover","params":{{"textDocument":{{"uri":"{uri}"}},"position":{{"line":0,"character":999}}}}}}"#
    );
    stdin
        .write_all(lsp_frame(&hover_col_oob).as_bytes())
        .unwrap();

    // 5e) go-to-definition request (currently unsupported but should respond)
    let def = format!(
        r#"{{"jsonrpc":"2.0","id":5,"method":"textDocument/definition","params":{{"textDocument":{{"uri":"{uri}"}},"position":{{"line":0,"character":1}}}}}}"#
    );
    stdin.write_all(lsp_frame(&def).as_bytes()).unwrap();

    // 5f) unknown request type should be ignored (handle_request fallthrough).
    let unknown_req = format!(
        r#"{{"jsonrpc":"2.0","id":7,"method":"textDocument/formatting","params":{{"textDocument":{{"uri":"{uri}"}},"options":{{}}}}}}"#
    );
    stdin.write_all(lsp_frame(&unknown_req).as_bytes()).unwrap();

    // 5g) unknown notification type should be ignored (handle_notification fallthrough).
    let unknown_not =
        r#"{"jsonrpc":"2.0","method":"workspace/didChangeConfiguration","params":{}}"#;
    stdin.write_all(lsp_frame(unknown_not).as_bytes()).unwrap();

    // 5h) send a response frame (server ignores Message::Response)
    let client_response = r#"{"jsonrpc":"2.0","id":999,"result":null}"#;
    stdin
        .write_all(lsp_frame(client_response).as_bytes())
        .unwrap();

    // 5i) close document
    let did_close = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didClose","params":{{"textDocument":{{"uri":"{uri}"}}}}}}"#
    );
    stdin.write_all(lsp_frame(&did_close).as_bytes()).unwrap();

    // 6) shutdown + exit (required by lsp-server's handle_shutdown)
    let shutdown = r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#;
    stdin.write_all(lsp_frame(shutdown).as_bytes()).unwrap();
    let exit = r#"{"jsonrpc":"2.0","method":"exit","params":null}"#;
    stdin.write_all(lsp_frame(exit).as_bytes()).unwrap();

    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "mdhavers-lsp failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // We should at least see initialize response and our request ids.
    assert!(
        stdout.contains(r#""id":1"#),
        "missing initialize response: {stdout}"
    );
    assert!(
        stdout.contains(r#""id":2"#),
        "missing shutdown response: {stdout}"
    );

    // Diagnostics publication is a notification; ensure it showed up.
    assert!(
        stdout.contains("publishDiagnostics"),
        "expected diagnostics publication in stdout, got: {stdout}"
    );
}
