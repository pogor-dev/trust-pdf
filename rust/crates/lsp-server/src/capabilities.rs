use lsp_types::{
    SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
};

use crate::tokens;

pub fn build_semantic_tokens_options() -> SemanticTokensOptions {
    tokens::build_semantic_tokens_options()
}

pub fn build_server_capabilities(semantic_tokens_options: SemanticTokensOptions) -> Result<serde_json::Value, serde_json::Error> {
    let text_sync = TextDocumentSyncOptions {
        open_close: Some(true),
        change: Some(TextDocumentSyncKind::FULL),
        will_save: None,
        will_save_wait_until: None,
        save: None,
    };

    serde_json::to_value(&ServerCapabilities {
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(semantic_tokens_options)),
        text_document_sync: Some(TextDocumentSyncCapability::Options(text_sync)),
        ..Default::default()
    })
}
