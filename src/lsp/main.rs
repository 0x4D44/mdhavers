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
    HoverParams, HoverProviderCapability, InitializeParams, MarkupContent, MarkupKind,
    Position, Range, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
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
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };

    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    eprintln!("🏴󠁧󠁢󠁳󠁣󠁴󠁿 mdhavers LSP Server shuttin' doon. Cheerio!");
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
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
            kind: Some(match kind.as_str() {
                "keyword" => CompletionItemKind::KEYWORD,
                "function" => CompletionItemKind::FUNCTION,
                "constant" => CompletionItemKind::CONSTANT,
                _ => CompletionItemKind::TEXT,
            }),
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
            severity: Some(match severity.as_str() {
                "error" => DiagnosticSeverity::ERROR,
                "warning" => DiagnosticSeverity::WARNING,
                _ => DiagnosticSeverity::INFORMATION,
            }),
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

    connection.sender.send(Message::Notification(notification))?;
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
