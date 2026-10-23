//! mdhavers Language Server - Gie yer editor some Scots smarts!
//!
//! This provides LSP support fer mdhavers, includin':
//! - Diagnostics (error reportin')
//! - Hover documentation
//! - Completions fer keywords an' builtins
//! - Go tae definition

use std::collections::HashMap;
use std::error::Error;

use lsp_server::{
    Connection, ErrorCode, ExtractError, Message, Notification, Request, RequestId, Response,
};
use lsp_types::{
    notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
    request::{Completion, GotoDefinition, HoverRequest, Request as _},
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Diagnostic,
    DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, InitializeParams, Location, MarkupContent, MarkupKind, Position, Range, Uri,
};
#[cfg(not(test))]
use lsp_types::{
    CompletionOptions, HoverProviderCapability, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};
use serde_json::Value;

// Import the mdhavers parser and lexer
// We need to make these modules public in lib.rs
mod mdhavers_bindings;
use mdhavers_bindings::{get_diagnostics, get_keyword_info, get_keywords_and_builtins};

/// A wee document store tae keep track o' open files
struct DocumentStore {
    documents: HashMap<Uri, String>,
}

impl DocumentStore {
    fn new() -> Self {
        DocumentStore {
            documents: HashMap::new(),
        }
    }

    fn open(&mut self, uri: Uri, text: String) {
        self.documents.insert(uri, text);
    }

    fn update(&mut self, uri: &Uri, text: String) {
        self.documents.insert(uri.clone(), text);
    }

    fn close(&mut self, uri: &Uri) {
        self.documents.remove(uri);
    }

    fn get(&self, uri: &Uri) -> Option<&String> {
        self.documents.get(uri)
    }
}

#[cfg(not(test))]
fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Suppress the colored output fer LSP mode
    colored::control::set_override(false);

    eprintln!("🏴󠁧󠁢󠁳󠁣󠁴󠁿 mdhavers LSP Server startin' up! Haud on...");

    // Create the transport via stdio
    let (connection, io_threads) = Connection::stdio();

    // Run the server
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![".".to_string()]),
            ..Default::default()
        }),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();

    let initialization_params = match connection.initialize(server_capabilities) {
        Ok(it) => it,
        Err(e) => {
            return Err(e.into());
        }
    };

    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("🏴󠁧󠁢󠁳󠁣󠁴󠁿 mdhavers LSP Server shuttin' doon. Cheerio!");
    Ok(())
}

fn main_loop(connection: Connection, params: Value) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();

    let mut documents = DocumentStore::new();

    eprintln!("🏴󠁧󠁢󠁳󠁣󠁴󠁿 Ready tae help ye write guid mdhavers code!");

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }

                let response = handle_request(&documents, req);
                connection.sender.send(Message::Response(response))?;
            }
            Message::Notification(not) => {
                handle_notification(&connection, &mut documents, not)?;
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

/// Answer a request. Every request gets a response: unknown methods get
/// MethodNotFound (-32601) an' unreadable params get InvalidParams (-32602),
/// so a client is never left hangin'.
fn handle_request(documents: &DocumentStore, req: Request) -> Response {
    let id = req.id.clone();
    let method = req.method.clone();

    let invalid = |id: RequestId| {
        Response::new_err(
            id,
            ErrorCode::InvalidParams as i32,
            format!("Och, the params fer '{method}' are no' richt"),
        )
    };

    if method == HoverRequest::METHOD {
        return match cast_request::<HoverRequest>(req) {
            Ok((id, params)) => Response::new_ok(id, handle_hover(documents, params)),
            Err(_) => invalid(id),
        };
    }
    if method == Completion::METHOD {
        return match cast_request::<Completion>(req) {
            Ok((id, params)) => Response::new_ok(id, handle_completion(documents, params)),
            Err(_) => invalid(id),
        };
    }
    if method == GotoDefinition::METHOD {
        return match cast_request::<GotoDefinition>(req) {
            Ok((id, params)) => Response::new_ok(id, handle_goto_definition(documents, params)),
            Err(_) => invalid(id),
        };
    }

    Response::new_err(
        id,
        ErrorCode::MethodNotFound as i32,
        format!("Haud on, A dinnae ken the request '{method}'"),
    )
}

fn handle_notification(
    connection: &Connection,
    documents: &mut DocumentStore,
    not: Notification,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    // Handle document opened
    if let Ok(params) = cast_notification::<DidOpenTextDocument>(not.clone()) {
        let DidOpenTextDocumentParams { text_document } = params;
        documents.open(text_document.uri.clone(), text_document.text.clone());
        publish_diagnostics(connection, &text_document.uri, &text_document.text)?;
        return Ok(());
    }

    // Handle document changed
    if let Ok(params) = cast_notification::<DidChangeTextDocument>(not.clone()) {
        let DidChangeTextDocumentParams {
            text_document,
            content_changes,
        } = params;
        if let Some(change) = content_changes.into_iter().last() {
            documents.update(&text_document.uri, change.text.clone());
            publish_diagnostics(connection, &text_document.uri, &change.text)?;
        }
        return Ok(());
    }

    // Handle document closed
    if let Ok(params) = cast_notification::<DidCloseTextDocument>(not.clone()) {
        let DidCloseTextDocumentParams { text_document } = params;
        documents.close(&text_document.uri);
        return Ok(());
    }

    Ok(())
}

fn handle_hover(_documents: &DocumentStore, params: HoverParams) -> Option<Hover> {
    // Get the word at the cursor position
    // For now, we'll return documentation for keywords
    let position = params.text_document_position_params.position;

    // This is a simplified version - ideally we'd parse the document
    // and find the exact token at the position
    let (keyword, start16, end16) = get_word_span_at_position(&params, _documents)?;

    if let Some(info) = get_keyword_info(&keyword) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info,
            }),
            range: Some(Range {
                start: Position {
                    line: position.line,
                    character: start16,
                },
                end: Position {
                    line: position.line,
                    character: end16,
                },
            }),
        });
    }

    None
}

fn handle_completion(
    _documents: &DocumentStore,
    _params: CompletionParams,
) -> Option<CompletionResponse> {
    let items = get_keywords_and_builtins();

    let completion_items: Vec<CompletionItem> = items
        .into_iter()
        .map(|(name, kind, doc)| CompletionItem {
            label: name.clone(),
            kind: Some(completion_item_kind(kind.as_str())),
            detail: Some(doc.clone()),
            documentation: Some(lsp_types::Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc,
            })),
            ..Default::default()
        })
        .collect();

    Some(CompletionResponse::Array(completion_items))
}

fn handle_goto_definition(
    documents: &DocumentStore,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let pos_params = params.text_document_position_params;
    let uri = pos_params.text_document.uri;
    let text = documents.get(&uri)?;
    let word = word_at(
        text,
        pos_params.position.line,
        pos_params.position.character,
    )?;
    let range = find_top_level_definition(text, &word)?;
    Some(GotoDefinitionResponse::Scalar(Location { uri, range }))
}

/// Find a top-level `dae NAME` or `kin NAME` (startin' in column 0) an' give
/// back the range o' the name in UTF-16 columns.
fn find_top_level_definition(text: &str, name: &str) -> Option<Range> {
    for (line_no, line) in text.lines().enumerate() {
        for keyword in ["dae ", "kin "] {
            let Some(rest) = line.strip_prefix(keyword) else {
                continue;
            };
            let rest_trimmed = rest.trim_start();
            let ident: String = rest_trimmed
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if ident == name {
                let prefix = &line[..line.len() - rest_trimmed.len()];
                let start = prefix.encode_utf16().count() as u32;
                let end = start + ident.encode_utf16().count() as u32;
                return Some(Range {
                    start: Position {
                        line: line_no as u32,
                        character: start,
                    },
                    end: Position {
                        line: line_no as u32,
                        character: end,
                    },
                });
            }
        }
    }
    None
}

fn publish_diagnostics(
    connection: &Connection,
    uri: &Uri,
    text: &str,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let diagnostics = get_diagnostics(text);

    let lsp_diagnostics: Vec<Diagnostic> = diagnostics
        .into_iter()
        .map(|(line, col, message, severity)| Diagnostic {
            range: Range {
                start: Position {
                    line: line.saturating_sub(1) as u32,
                    character: col.saturating_sub(1) as u32,
                },
                end: Position {
                    line: line.saturating_sub(1) as u32,
                    character: (col + 10) as u32, // Approximate end
                },
            },
            severity: Some(diagnostic_severity(severity.as_str())),
            source: Some("mdhavers".to_string()),
            message,
            ..Default::default()
        })
        .collect();

    let notification = lsp_server::Notification::new(
        "textDocument/publishDiagnostics".to_string(),
        lsp_types::PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics: lsp_diagnostics,
            version: None,
        },
    );

    connection
        .sender
        .send(Message::Notification(notification))?;
    Ok(())
}

fn get_word_span_at_position(
    params: &HoverParams,
    documents: &DocumentStore,
) -> Option<(String, u32, u32)> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    let text = documents.get(uri)?;
    word_span_at(text, position.line, position.character)
}

#[cfg(test)]
fn get_word_at_position(params: &HoverParams, documents: &DocumentStore) -> Option<String> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    let text = documents.get(uri)?;
    word_at(text, position.line, position.character)
}

/// Map an LSP column (UTF-16 code units) tae an index intae `chars`.
/// A column in the middle o' a surrogate pair maps tae that character.
/// Gives `None` when the column is at or past the end o' the line.
fn utf16_col_to_char_index(chars: &[char], utf16_col: u32) -> Option<usize> {
    let target = utf16_col as usize;
    let mut offset = 0usize;
    for (index, c) in chars.iter().enumerate() {
        let width = c.len_utf16();
        if target < offset + width {
            return Some(index);
        }
        offset += width;
    }
    None
}

/// The identifier word at an LSP position (line, UTF-16 column), if any.
fn word_at(text: &str, line: u32, character: u32) -> Option<String> {
    word_span_at(text, line, character).map(|(word, _, _)| word)
}

/// The identifier word at an LSP position, plus its start and end columns
/// (UTF-16 code units) on that line.
fn word_span_at(text: &str, line: u32, character: u32) -> Option<(String, u32, u32)> {
    let line = text.lines().nth(line as usize)?;
    let chars: Vec<char> = line.chars().collect();
    let col = utf16_col_to_char_index(&chars, character)?;

    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let mut start = col;
    let mut end = col;

    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    while end < chars.len() && is_word(chars[end]) {
        end += 1;
    }

    if start < end {
        let utf16_len = |cs: &[char]| cs.iter().map(|c| c.len_utf16() as u32).sum::<u32>();
        let start16 = utf16_len(&chars[..start]);
        let end16 = start16 + utf16_len(&chars[start..end]);
        Some((chars[start..end].iter().collect(), start16, end16))
    } else {
        None
    }
}

fn cast_request<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_notification<N>(not: Notification) -> Result<N::Params, ExtractError<Notification>>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    not.extract(N::METHOD)
}

fn completion_item_kind(kind: &str) -> CompletionItemKind {
    match kind {
        "keyword" => CompletionItemKind::KEYWORD,
        "function" => CompletionItemKind::FUNCTION,
        "constant" => CompletionItemKind::CONSTANT,
        _ => CompletionItemKind::TEXT,
    }
}

fn diagnostic_severity(severity: &str) -> DiagnosticSeverity {
    match severity {
        "error" => DiagnosticSeverity::ERROR,
        "warning" => DiagnosticSeverity::WARNING,
        _ => DiagnosticSeverity::INFORMATION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_server::{
        Connection, Message, Notification as LspNotification, Request as LspRequest, Response,
    };
    use lsp_types::notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Notification as LspNotificationTrait,
    };
    use lsp_types::request::{
        Completion, GotoDefinition, HoverRequest, Request as LspRequestTrait,
    };
    use lsp_types::{
        CompletionParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, Position, TextDocumentContentChangeEvent,
        TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, Uri,
        VersionedTextDocumentIdentifier,
    };
    use serde_json::Value as JsonValue;
    use std::str::FromStr;

    fn hover_params(uri: &Uri, line: u32, character: u32) -> HoverParams {
        HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
        }
    }

    #[test]
    fn get_word_at_position_handles_oob_and_whitespace() {
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp.braw").unwrap();
        docs.open(uri.clone(), "ken x = 1\n".to_string());

        let params = hover_params(&uri, 0, 1);
        let word = get_word_at_position(&params, &docs);
        assert_eq!(word.as_deref(), Some("ken"));

        let whitespace = hover_params(&uri, 0, 3);
        let word = get_word_at_position(&whitespace, &docs);
        assert_eq!(word.as_deref(), Some("ken"));

        let non_word = hover_params(&uri, 0, 6);
        assert!(get_word_at_position(&non_word, &docs).is_none());

        let oob = hover_params(&uri, 99, 99);
        assert!(get_word_at_position(&oob, &docs).is_none());

        let col_oob = hover_params(&uri, 0, 99);
        assert!(get_word_at_position(&col_oob, &docs).is_none());

        let missing_uri = Uri::from_str("file:///tmp/coverage_lsp_missing.braw").unwrap();
        let missing = hover_params(&missing_uri, 0, 0);
        assert!(get_word_at_position(&missing, &docs).is_none());
    }

    #[test]
    fn get_word_at_position_handles_underscore_identifiers_for_coverage() {
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_underscore.braw").unwrap();
        docs.open(uri.clone(), "ken snake_case = 1\n".to_string());

        let after_underscore = hover_params(&uri, 0, 10);
        let word = get_word_at_position(&after_underscore, &docs);
        assert_eq!(word.as_deref(), Some("snake_case"));

        let before_underscore = hover_params(&uri, 0, 8);
        let word = get_word_at_position(&before_underscore, &docs);
        assert_eq!(word.as_deref(), Some("snake_case"));

        let at_start = hover_params(&uri, 0, 0);
        let word = get_word_at_position(&at_start, &docs);
        assert_eq!(word.as_deref(), Some("ken"));

        let at_equals = hover_params(&uri, 0, 15);
        assert!(get_word_at_position(&at_equals, &docs).is_none());
    }

    #[test]
    fn handle_hover_returns_none_when_word_lookup_returns_none_for_coverage() {
        let docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_missing_hover.braw").unwrap();
        let params = hover_params(&uri, 0, 0);
        assert!(handle_hover(&docs, params).is_none());
    }

    #[test]
    fn main_loop_processes_requests_and_notifications() {
        let (server, client) = Connection::memory();
        let params = serde_json::to_value(InitializeParams::default()).unwrap();
        let handle = std::thread::spawn(move || main_loop(server, params));

        let uri = Uri::from_str("file:///tmp/coverage_lsp.braw").unwrap();
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "mdhavers".to_string(),
                version: 1,
                text: "ken x = 1\nblether x\n".to_string(),
            },
        };
        client
            .sender
            .send(Message::Notification(LspNotification::new(
                DidOpenTextDocument::METHOD.to_string(),
                open_params,
            )))
            .unwrap();

        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 2,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "ken =\n".to_string(),
            }],
        };
        client
            .sender
            .send(Message::Notification(LspNotification::new(
                lsp_types::notification::DidChangeTextDocument::METHOD.to_string(),
                change_params,
            )))
            .unwrap();

        let hover = hover_params(&uri, 0, 3);
        client
            .sender
            .send(Message::Request(LspRequest::new(
                lsp_server::RequestId::from(1),
                HoverRequest::METHOD.to_string(),
                hover,
            )))
            .unwrap();

        let completion = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 1,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };
        client
            .sender
            .send(Message::Request(LspRequest::new(
                lsp_server::RequestId::from(2),
                Completion::METHOD.to_string(),
                completion,
            )))
            .unwrap();

        let goto_params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 1,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        client
            .sender
            .send(Message::Request(LspRequest::new(
                lsp_server::RequestId::from(3),
                GotoDefinition::METHOD.to_string(),
                goto_params,
            )))
            .unwrap();

        client
            .sender
            .send(Message::Request(LspRequest::new(
                lsp_server::RequestId::from(4),
                "textDocument/formatting".to_string(),
                JsonValue::Null,
            )))
            .unwrap();

        client
            .sender
            .send(Message::Response(Response {
                id: lsp_server::RequestId::from(6),
                result: None,
                error: None,
            }))
            .unwrap();

        client
            .sender
            .send(Message::Request(LspRequest::new(
                lsp_server::RequestId::from(5),
                "shutdown".to_string(),
                JsonValue::Null,
            )))
            .unwrap();
        client
            .sender
            .send(Message::Notification(LspNotification::new(
                "exit".to_string(),
                JsonValue::Null,
            )))
            .unwrap();
        drop(client.sender);

        handle.join().expect("join main_loop").unwrap();
    }

    #[test]
    fn main_loop_exits_cleanly_when_client_disconnects() {
        let (server, client) = Connection::memory();
        let params = serde_json::to_value(InitializeParams::default()).unwrap();
        let handle = std::thread::spawn(move || main_loop(server, params));
        drop(client.sender);
        handle.join().expect("join main_loop").unwrap();
    }

    #[test]
    fn handle_hover_returns_none_for_unknown_word() {
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_unknown.braw").unwrap();
        docs.open(uri.clone(), "foobarbaz".to_string());

        let params = hover_params(&uri, 0, 2);
        assert!(handle_hover(&docs, params).is_none());
    }

    #[test]
    fn hover_range_covers_the_whole_word_in_utf16_columns() {
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_hover_range.braw").unwrap();
        // The emoji is two UTF-16 units, so `gin` spans columns 7..10.
        docs.open(uri.clone(), "ken \u{1F600} gin x\n".to_string());

        // Cursor in the middle of `gin`.
        let hover = handle_hover(&docs, hover_params(&uri, 0, 8)).expect("gin hover");
        let range = hover.range.expect("range");
        assert_eq!(
            range.start,
            Position {
                line: 0,
                character: 7
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 0,
                character: 10
            }
        );

        // Cursor at the start of `ken`.
        let hover = handle_hover(&docs, hover_params(&uri, 0, 2)).expect("ken hover");
        let range = hover.range.expect("range");
        assert_eq!(
            range.start,
            Position {
                line: 0,
                character: 0
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 0,
                character: 3
            }
        );
    }

    #[test]
    fn handle_hover_returns_documentation_for_keywords_for_coverage() {
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_keyword.braw").unwrap();
        docs.open(uri.clone(), "ken x = 1\n".to_string());

        let params = hover_params(&uri, 0, 1);
        let hover = handle_hover(&docs, params).expect("expected hover");
        match hover.contents {
            HoverContents::Markup(markup) => {
                assert!(markup.value.contains("ken"));
            }
            other => panic!("unexpected hover contents: {other:?}"),
        }
    }

    #[test]
    fn handle_notification_closes_document() {
        let (server, _client) = Connection::memory();
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_close.braw").unwrap();
        docs.open(uri.clone(), "ken x = 1\n".to_string());

        let close_params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        };
        let notification =
            LspNotification::new(DidCloseTextDocument::METHOD.to_string(), close_params);
        handle_notification(&server, &mut docs, notification).unwrap();

        assert!(docs.get(&uri).is_none());
    }

    #[test]
    fn handle_notification_ignores_empty_did_change_text_document() {
        let (server, _client) = Connection::memory();
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_change.braw").unwrap();
        docs.open(uri.clone(), "ken x = 1\n".to_string());

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 2,
            },
            content_changes: Vec::new(),
        };
        let notification = LspNotification::new(DidChangeTextDocument::METHOD.to_string(), params);
        handle_notification(&server, &mut docs, notification).unwrap();
        assert_eq!(docs.get(&uri).unwrap(), "ken x = 1\n");
    }

    #[test]
    fn handle_notification_updates_document_on_last_change_for_coverage() {
        let (server, _client) = Connection::memory();
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_change_last.braw").unwrap();
        docs.open(uri.clone(), "ken x = 1\n".to_string());

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 2,
            },
            content_changes: vec![
                TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: "ken x = 2\n".to_string(),
                },
                TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: "ken x = 3\n".to_string(),
                },
            ],
        };
        let notification = LspNotification::new(DidChangeTextDocument::METHOD.to_string(), params);
        handle_notification(&server, &mut docs, notification).unwrap();
        assert_eq!(docs.get(&uri).unwrap(), "ken x = 3\n");
    }

    #[test]
    fn handle_notification_updates_document_on_single_change_for_coverage() {
        let (server, _client) = Connection::memory();
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_change_single.braw").unwrap();
        docs.open(uri.clone(), "ken x = 1\n".to_string());

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 2,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "ken x = 2\n".to_string(),
            }],
        };
        let notification = LspNotification::new(DidChangeTextDocument::METHOD.to_string(), params);
        handle_notification(&server, &mut docs, notification).unwrap();
        assert_eq!(docs.get(&uri).unwrap(), "ken x = 2\n");
    }

    #[test]
    fn handle_notification_ignores_unknown_notification_type_for_coverage() {
        let (server, _client) = Connection::memory();
        let mut docs = DocumentStore::new();
        let notification = LspNotification::new(
            "workspace/didChangeConfiguration".to_string(),
            JsonValue::Null,
        );
        handle_notification(&server, &mut docs, notification).unwrap();
    }

    #[test]
    fn completion_item_kind_and_diagnostic_severity_cover_fallbacks() {
        assert_eq!(completion_item_kind("keyword"), CompletionItemKind::KEYWORD);
        assert_eq!(
            completion_item_kind("function"),
            CompletionItemKind::FUNCTION
        );
        assert_eq!(
            completion_item_kind("constant"),
            CompletionItemKind::CONSTANT
        );
        assert_eq!(completion_item_kind("unknown"), CompletionItemKind::TEXT);

        assert_eq!(diagnostic_severity("error"), DiagnosticSeverity::ERROR);
        assert_eq!(diagnostic_severity("warning"), DiagnosticSeverity::WARNING);
        assert_eq!(diagnostic_severity("info"), DiagnosticSeverity::INFORMATION);
    }

    #[test]
    fn main_loop_propagates_notification_send_errors_for_coverage() {
        let (server, client) = Connection::memory();
        let params = serde_json::to_value(InitializeParams::default()).unwrap();
        let handle = std::thread::spawn(move || main_loop(server, params));

        // Force publish_diagnostics() send failure by dropping the receiver side.
        drop(client.receiver);

        let uri = Uri::from_str("file:///tmp/coverage_lsp_send_fail.braw").unwrap();
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "mdhavers".to_string(),
                version: 1,
                text: "ken =\n".to_string(),
            },
        };

        client
            .sender
            .send(Message::Notification(LspNotification::new(
                DidOpenTextDocument::METHOD.to_string(),
                open_params,
            )))
            .unwrap();
        drop(client.sender);

        let result = handle.join().expect("join main_loop");
        assert!(result.is_err());
    }

    #[test]
    fn handle_notification_propagates_did_change_send_errors_for_coverage() {
        let (server, client) = Connection::memory();
        let mut docs = DocumentStore::new();

        drop(client.receiver);

        let uri = Uri::from_str("file:///tmp/coverage_lsp_send_fail_change.braw").unwrap();
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: 1,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "ken =\n".to_string(),
            }],
        };

        let notification = LspNotification::new(DidChangeTextDocument::METHOD.to_string(), params);
        let err = handle_notification(&server, &mut docs, notification).unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn main_loop_propagates_shutdown_send_error_for_coverage() {
        let (server, client) = Connection::memory();
        let params = serde_json::to_value(InitializeParams::default()).unwrap();
        let handle = std::thread::spawn(move || main_loop(server, params));

        drop(client.receiver);

        client
            .sender
            .send(Message::Request(LspRequest::new(
                lsp_server::RequestId::from(1),
                "shutdown".to_string(),
                JsonValue::Null,
            )))
            .unwrap();
        drop(client.sender);

        let result = handle.join().expect("join main_loop");
        assert!(result.is_err());
    }

    #[test]
    fn main_loop_propagates_response_send_error_for_coverage() {
        let (server, client) = Connection::memory();
        let params = serde_json::to_value(InitializeParams::default()).unwrap();
        let handle = std::thread::spawn(move || main_loop(server, params));

        let uri = Uri::from_str("file:///tmp/coverage_lsp_send_fail_response.braw").unwrap();
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "mdhavers".to_string(),
                version: 1,
                text: "ken x = 1\n".to_string(),
            },
        };

        client
            .sender
            .send(Message::Notification(LspNotification::new(
                DidOpenTextDocument::METHOD.to_string(),
                open_params,
            )))
            .unwrap();

        // Wait for diagnostics, then drop the receiver to force response send failure.
        let _ = client.receiver.recv().expect("receive diagnostics");
        drop(client.receiver);

        let hover = hover_params(&uri, 0, 1);
        client
            .sender
            .send(Message::Request(LspRequest::new(
                lsp_server::RequestId::from(2),
                HoverRequest::METHOD.to_string(),
                hover,
            )))
            .unwrap();
        drop(client.sender);

        let result = handle.join().expect("join main_loop");
        assert!(result.is_err());
    }

    #[test]
    fn word_at_handles_multibyte_lines_without_panicking() {
        // 'é' is 1 UTF-16 unit (2 bytes); the emoji is 2 UTF-16 units (4 bytes).
        let text = "ken é = \"😀\" ken\n";
        // UTF-16 columns: k0 e1 n2 ' '3 é4 ' '5 =6 ' '7 "8 😀9,10 "11 ' '12 k13 e14 n15
        assert_eq!(word_at(text, 0, 1).as_deref(), Some("ken"));
        assert_eq!(word_at(text, 0, 4).as_deref(), Some("é"));
        assert_eq!(word_at(text, 0, 6), None);
        assert_eq!(word_at(text, 0, 8), None);
        // Both halves of the surrogate pair map tae the emoji (no' a word).
        assert_eq!(word_at(text, 0, 9), None);
        assert_eq!(word_at(text, 0, 10), None);
        // Past the emoji the columns are shifted by UTF-16 width, no' bytes.
        assert_eq!(word_at(text, 0, 13).as_deref(), Some("ken"));
        assert_eq!(word_at(text, 0, 15).as_deref(), Some("ken"));
        // At the end o' the line an' well past it: None, never a panic.
        assert_eq!(word_at(text, 0, 16), None);
        assert_eq!(word_at(text, 0, 17), None);
        assert_eq!(word_at(text, 0, u32::MAX), None);
        assert_eq!(word_at(text, 5, 0), None);
    }

    #[test]
    fn utf16_col_to_char_index_counts_surrogate_pairs() {
        let chars: Vec<char> = "a😀b".chars().collect();
        assert_eq!(utf16_col_to_char_index(&chars, 0), Some(0));
        assert_eq!(utf16_col_to_char_index(&chars, 1), Some(1));
        assert_eq!(utf16_col_to_char_index(&chars, 2), Some(1));
        assert_eq!(utf16_col_to_char_index(&chars, 3), Some(2));
        assert_eq!(utf16_col_to_char_index(&chars, 4), None);
    }

    #[test]
    fn handle_hover_on_multibyte_line_does_not_panic() {
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_multibyte.braw").unwrap();
        docs.open(uri.clone(), "blether \"é😀é😀é😀\" gin\n".to_string());
        for col in 0..40 {
            let _ = handle_hover(&docs, hover_params(&uri, 0, col));
        }
        // `gin` sits after the emoji; its UTF-16 column is 8 + 1 + 9 + 1 + 1 = 20.
        let hover = handle_hover(&docs, hover_params(&uri, 0, 21)).expect("gin hover");
        match hover.contents {
            HoverContents::Markup(markup) => assert!(markup.value.contains("gin")),
            other => panic!("unexpected hover contents: {other:?}"),
        }
    }

    #[test]
    fn unknown_request_gets_method_not_found() {
        let docs = DocumentStore::new();
        let req = LspRequest::new(
            lsp_server::RequestId::from(7),
            "textDocument/formatting".to_string(),
            JsonValue::Null,
        );
        let response = handle_request(&docs, req);
        assert_eq!(response.id, lsp_server::RequestId::from(7));
        assert!(response.result.is_none());
        let error = response.error.expect("error response");
        assert_eq!(error.code, -32601);
    }

    #[test]
    fn bad_params_get_invalid_params() {
        let docs = DocumentStore::new();
        let req = LspRequest::new(
            lsp_server::RequestId::from(8),
            HoverRequest::METHOD.to_string(),
            JsonValue::Null,
        );
        let response = handle_request(&docs, req);
        assert_eq!(response.error.expect("error").code, -32602);
    }

    #[test]
    fn goto_definition_finds_top_level_dae_and_kin() {
        let mut docs = DocumentStore::new();
        let uri = Uri::from_str("file:///tmp/coverage_lsp_goto.braw").unwrap();
        docs.open(
            uri.clone(),
            "dae greet(naam) {\n    blether naam\n}\nkin Animal {\n}\ngreet(\"x\")\nken a = Animal()\n"
                .to_string(),
        );
        let goto = |line: u32, character: u32| GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        match handle_goto_definition(&docs, goto(5, 1)) {
            Some(GotoDefinitionResponse::Scalar(loc)) => {
                assert_eq!(
                    loc.range.start,
                    Position {
                        line: 0,
                        character: 4
                    }
                );
                assert_eq!(
                    loc.range.end,
                    Position {
                        line: 0,
                        character: 9
                    }
                );
            }
            other => panic!("unexpected: {other:?}"),
        }
        match handle_goto_definition(&docs, goto(6, 11)) {
            Some(GotoDefinitionResponse::Scalar(loc)) => {
                assert_eq!(
                    loc.range.start,
                    Position {
                        line: 3,
                        character: 4
                    }
                );
            }
            other => panic!("unexpected: {other:?}"),
        }
        // Not a definition: a local variable.
        assert!(handle_goto_definition(&docs, goto(6, 5)).is_none());
    }
}
