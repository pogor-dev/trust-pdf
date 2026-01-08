use lsp_server::{Connection, ExtractError, Message, RequestId, Response};
use lsp_types::{
    InitializeParams, OneOf, SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities,
    notification::{DidChangeTextDocument, DidOpenTextDocument},
    request::SemanticTokensFullRequest,
};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use lexer::Lexer;
use syntax::SyntaxKind;

/// Semantic token types for PDF syntax highlighting
#[repr(u32)]
#[derive(Clone, Copy)]
enum TokenType {
    Keyword = 0,
    String = 1,
    Number = 2,
    Property = 3,
}

// Semantic token types - single source of truth
const TOKEN_TYPES_DEFS: &[SemanticTokenType] = &[
    lsp_types::SemanticTokenType::KEYWORD,
    lsp_types::SemanticTokenType::STRING,
    lsp_types::SemanticTokenType::NUMBER,
    lsp_types::SemanticTokenType::PROPERTY,
];

// TODO: combine TokenType and TOKEN_TYPES_DEFS into a single definition

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Note that  we must have our logging only write out to stderr since the communication with the client
    // is done via stdin/stdout. If we write to stdout, we will corrupt the communication.
    eprintln!("Starting TRust PDF LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Advertise capabilities including semantic tokens for highlighting.
    let legend = SemanticTokensLegend {
        token_types: TOKEN_TYPES_DEFS.to_vec(),
        token_modifiers: vec![],
    };

    let semantic_tokens_options = SemanticTokensOptions {
        legend,
        full: Some(SemanticTokensFullOptions::Bool(true)),
        range: Some(true),
        work_done_progress_options: Default::default(),
    };

    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(semantic_tokens_options)),
        ..Default::default()
    })
    .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("Shutting down server");
    Ok(())
}

fn main_loop(connection: Connection, params: serde_json::Value) -> Result<(), Box<dyn Error + Sync + Send>> {
    // We don't currently use initialize params; avoid unwrap by ignoring or logging.
    if let Err(e) = serde_json::from_value::<InitializeParams>(params) {
        eprintln!("Failed to parse InitializeParams: {e}");
    }
    let mut docs: HashMap<String, Arc<String>> = HashMap::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                // Semantic tokens full via typed cast
                match cast::<SemanticTokensFullRequest>(req.clone()) {
                    Ok((id, params)) => {
                        let uri = params.text_document.uri.to_string();
                        let text = docs.get(&uri).cloned().unwrap_or_else(|| Arc::new(String::new()));
                        let data = compute_semantic_tokens(&text);
                        let result = SemanticTokens { result_id: None, data };
                        let result = match serde_json::to_value(result) {
                            Ok(v) => v,
                            Err(e) => {
                                eprintln!("Failed to serialize SemanticTokens: {e}");
                                continue;
                            }
                        };
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
            }
            Message::Response(_resp) => {}
            Message::Notification(not) => {
                // Track opened/changed documents using typed notification casting.
                let not = match cast_notification::<DidOpenTextDocument>(not.clone()) {
                    Ok(params) => {
                        docs.insert(params.text_document.uri.to_string(), Arc::new(params.text_document.text));
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(not)) => not,
                };

                let _not = match cast_notification::<DidChangeTextDocument>(not) {
                    Ok(params) => {
                        if let Some(change) = params.content_changes.into_iter().last() {
                            let text = change.text;
                            docs.insert(params.text_document.uri.to_string(), Arc::new(text));
                        }
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(not)) => not,
                };
                // Ignore unrelated notifications thereafter
            }
        }
    }
    Ok(())
}

fn cast<R>(req: lsp_server::Request) -> Result<(RequestId, R::Params), ExtractError<lsp_server::Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_notification<N>(not: lsp_server::Notification) -> Result<N::Params, ExtractError<lsp_server::Notification>>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    not.extract(N::METHOD)
}

fn compute_semantic_tokens(text: &str) -> Vec<SemanticToken> {
    let mut data: Vec<SemanticToken> = Vec::new();

    // Precompute line starts for offset->(line,col) mapping
    let line_starts = compute_line_starts(text.as_bytes());

    let mut lexer = Lexer::new(text.as_bytes());
    let mut offset: usize = 0;
    let mut prev_line: u32 = 0;
    let mut prev_col: u32 = 0;

    loop {
        let tok = lexer.next_token();
        let kind: SyntaxKind = tok.kind().into();
        let width = tok.full_width() as usize;

        if kind == SyntaxKind::EndOfFileToken {
            break;
        }

        let (line, col) = offset_to_line_col(offset, &line_starts);
        let length = tok.bytes().len() as u32;
        if let Some(token_type) = map_kind(kind) {
            // delta encode
            let (dl, dc) = if line == prev_line { (0, col - prev_col) } else { (line - prev_line, col) };
            prev_line = line;
            prev_col = col;

            data.push(SemanticToken {
                delta_line: dl,
                delta_start: dc,
                length,
                token_type: token_type as u32,
                token_modifiers_bitset: 0,
            });
        }

        offset += width;
    }

    data
}

fn compute_line_starts(bytes: &[u8]) -> Vec<usize> {
    let mut starts = vec![0usize];
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                starts.push(i + 1);
            }
            b'\r' => {
                // handle \r\n as single newline
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    starts.push(i + 2);
                    i += 1;
                } else {
                    starts.push(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    starts
}

fn offset_to_line_col(offset: usize, line_starts: &[usize]) -> (u32, u32) {
    // binary search for the last line_start <= offset
    let mut lo = 0usize;
    let mut hi = line_starts.len();
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if line_starts[mid] <= offset {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let line = lo as u32;
    let col = (offset - line_starts[lo]) as u32;
    (line, col)
}

fn map_kind(kind: SyntaxKind) -> Option<TokenType> {
    use SyntaxKind as K;
    let t = match kind {
        // keywords & PDF grammar keywords
        K::TrueKeyword
        | K::FalseKeyword
        | K::NullKeyword
        | K::IndirectObjectKeyword
        | K::IndirectEndObjectKeyword
        | K::IndirectReferenceKeyword
        | K::StreamKeyword
        | K::EndStreamKeyword
        | K::XRefKeyword
        | K::XRefFreeEntryKeyword
        | K::XRefInUseEntryKeyword
        | K::FileTrailerKeyword
        | K::StartXRefKeyword => TokenType::Keyword,
        // literals
        K::StringLiteralToken | K::HexStringLiteralToken => TokenType::String,
        K::NumericLiteralToken => TokenType::Number,
        // names often act like dictionary keys; classify as property
        K::NameLiteralToken => TokenType::Property,
        // punctuation and structural nodes are ignored for highlighting
        _ => return None,
    };
    Some(t)
}
