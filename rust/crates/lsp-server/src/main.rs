use lsp_server::{Connection, ExtractError, Message, RequestId};
use lsp_types::{
    InitializeParams,
    notification::{DidChangeTextDocument, DidOpenTextDocument},
    request::SemanticTokensFullRequest,
};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

mod capabilities;
mod handlers;
mod line_map;
mod tokens;

// NOTE: token legend and mapping moved to `tokens` module

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Note that  we must have our logging only write out to stderr since the communication with the client
    // is done via stdin/stdout. If we write to stdout, we will corrupt the communication.
    eprintln!("Starting TRust PDF LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Advertise capabilities including semantic tokens for highlighting.
    let semantic_tokens_options = capabilities::build_semantic_tokens_options();
    let server_capabilities = capabilities::build_server_capabilities(semantic_tokens_options).map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

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
