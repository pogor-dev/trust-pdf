use std::{collections::HashMap, sync::Arc};

use lsp_types::{DidChangeTextDocumentParams, DidOpenTextDocumentParams};

pub(crate) fn on_did_open(docs: &mut HashMap<String, Arc<String>>, params: DidOpenTextDocumentParams) {
    docs.insert(params.text_document.uri.to_string(), Arc::new(params.text_document.text));
}

pub(crate) fn on_did_change(docs: &mut HashMap<String, Arc<String>>, params: DidChangeTextDocumentParams) {
    if let Some(change) = params.content_changes.into_iter().last() {
        docs.insert(params.text_document.uri.to_string(), Arc::new(change.text));
    }
}
