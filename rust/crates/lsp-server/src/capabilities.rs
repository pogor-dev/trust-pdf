use lsp_types::{SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities};

use crate::tokens;

pub fn build_semantic_tokens_options() -> SemanticTokensOptions {
    tokens::build_semantic_tokens_options()
}

pub fn build_server_capabilities(semantic_tokens_options: SemanticTokensOptions) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(&ServerCapabilities {
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(semantic_tokens_options)),
        ..Default::default()
    })
}
