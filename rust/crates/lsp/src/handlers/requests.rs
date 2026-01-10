use std::{collections::HashMap, error::Error, sync::Arc};

use lsp_server::{Connection, Message, RequestId, Response};
use lsp_types::{SemanticTokens, SemanticTokensParams};

use crate::tokens::compute_semantic_tokens;

pub(crate) fn handle_semantic_tokens_full(
    connection: &Connection,
    id: RequestId,
    params: SemanticTokensParams,
    docs: &HashMap<String, Arc<String>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let uri = params.text_document.uri.to_string();
    eprintln!("semanticTokens request for: {}", uri);
    eprintln!("Available documents: {:?}", docs.keys().collect::<Vec<_>>());
    let text = docs.get(&uri).cloned().unwrap_or_else(|| Arc::new(String::new()));
    eprintln!("Document length: {} bytes", text.len());

    let data = compute_semantic_tokens(&text);
    eprintln!("Computed {} semantic tokens", data.len());
    let result = SemanticTokens { result_id: None, data };
    let result = serde_json::to_value(result)?;

    let resp = Response {
        id,
        result: Some(result),
        error: None,
    };
    connection.sender.send(Message::Response(resp))?;
    Ok(())
}
