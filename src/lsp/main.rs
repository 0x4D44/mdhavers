//! mdhavers Language Server - Gie yer editor some Scots smarts!
//!
//! This provides LSP support fer mdhavers, includin':
//! - Diagnostics (error reportin')
//! - Hover documentation
//! - Completions fer keywords an' builtins
//! - Go tae definition

use std::collections::HashMap;
use std::error::Error;

use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
    request::{Completion, GotoDefinition, HoverRequest},
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, MarkupContent, MarkupKind, Position,
    Range, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
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

                let result = handle_request(&documents, req);
                if let Some((id, response)) = result {
                    connection.sender.send(Message::Response(Response {
                        id,
                        result: Some(response),
                        error: None,
                    }))?;
                }
            }
            Message::Notification(not) => {
                handle_notification(&connection, &mut documents, not)?;
            }
            Message::Response(_) => {}
        }
    }
    Ok(())
}

fn handle_request(documents: &DocumentStore, req: Request) -> Option<(RequestId, Value)> {
    // Handle hover request
    if let Ok((id, params)) = cast_request::<HoverRequest>(req.clone()) {
        let result = handle_hover(documents, params);
        return Some((id, serde_json::to_value(result).unwrap()));
    }

    // Handle completion request
    if let Ok((id, params)) = cast_request::<Completion>(req.clone()) {
        let result = handle_completion(documents, params);
        return Some((id, serde_json::to_value(result).unwrap()));
    }

    // Handle go-to-definition request
    if let Ok((id, params)) = cast_request::<GotoDefinition>(req.clone()) {
        let result = handle_goto_definition(documents, params);
        return Some((id, serde_json::to_value(result).unwrap()));
    }

    None
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
    let keyword = get_word_at_position(&params, _documents)?;

    if let Some(info) = get_keyword_info(&keyword) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info,
            }),
            range: Some(Range {
                start: position,
                end: Position {
                    line: position.line,
                    character: position.character + keyword.len() as u32,
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
    _documents: &DocumentStore,
    _params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    // For now, we don't support go-to-definition
    // This would require tracking function definitions
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

fn get_word_at_position(params: &HoverParams, documents: &DocumentStore) -> Option<String> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    let text = documents.get(uri)?;
    let lines: Vec<&str> = text.lines().collect();

    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let col = position.character as usize;

    if col >= line.len() {
        return None;
    }

    // Find word boundaries
    let chars: Vec<char> = line.chars().collect();
    let mut start = col;
    let mut end = col;

    // Find start of word
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }

    // Find end of word
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    if start < end {
        Some(chars[start..end].iter().collect())
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
    use lsp_server::{Connection, Message, Notification as LspNotification, Request as LspRequest};
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
            text_document: VersionedTextDocumentIdentifier { uri: uri.clone(), version: 2 },
            content_changes: Vec::new(),
        };
        let notification =
            LspNotification::new(DidChangeTextDocument::METHOD.to_string(), params);
        handle_notification(&server, &mut docs, notification).unwrap();
        assert_eq!(docs.get(&uri).unwrap(), "ken x = 1\n");
    }

    #[test]
    fn completion_item_kind_and_diagnostic_severity_cover_fallbacks() {
        assert_eq!(completion_item_kind("keyword"), CompletionItemKind::KEYWORD);
        assert_eq!(completion_item_kind("unknown"), CompletionItemKind::TEXT);

        assert_eq!(diagnostic_severity("error"), DiagnosticSeverity::ERROR);
        assert_eq!(diagnostic_severity("warning"), DiagnosticSeverity::WARNING);
        assert_eq!(
            diagnostic_severity("info"),
            DiagnosticSeverity::INFORMATION
        );
    }
}
