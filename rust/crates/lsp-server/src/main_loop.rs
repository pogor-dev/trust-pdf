use std::{collections::HashMap, sync::Arc};

use lsp_server::{Connection, ExtractError, Message, RequestId};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument},
    request::SemanticTokensFullRequest,
};

use crate::handlers;

pub(crate) fn main_loop(connection: Connection) -> anyhow::Result<()> {
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
                        if let Err(e) = handlers::handle_semantic_tokens_full(&connection, id, params, &docs) {
                            eprintln!("Failed to handle semantic tokens request: {e}");
                        }
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
                        eprintln!("Received DidOpen notification");
                        handlers::on_did_open(&mut docs, params);
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(not)) => not,
                };

                let _not = match cast_notification::<DidChangeTextDocument>(not) {
                    Ok(params) => {
                        eprintln!("Received DidChange notification");
                        handlers::on_did_change(&mut docs, params);
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
