use lsp_types::{
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions,
};

use crate::lsp::semantic_tokens::{SUPPORTED_MODIFIERS, SUPPORTED_TYPES};

pub(crate) fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
            legend: SemanticTokensLegend {
                token_types: SUPPORTED_TYPES.to_vec(),
                token_modifiers: SUPPORTED_MODIFIERS.to_vec(),
            },
            full: Some(SemanticTokensFullOptions::Bool(true)),
            range: Some(true),
            work_done_progress_options: Default::default(),
        })),
        text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::FULL),
            will_save: None,
            will_save_wait_until: None,
            save: None,
        })),
        ..Default::default()
    }
}
