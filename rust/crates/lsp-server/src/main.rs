use lsp_server::Connection;
use serde::de::DeserializeOwned;

mod handlers;
mod line_map;
mod lsp;
mod main_loop;
mod tokens;
mod version;

fn main() -> anyhow::Result<()> {
    run_server()
}

fn run_server() -> anyhow::Result<()> {
    tracing::info!("server version {} will start", version::version());

    let (connection, io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = match connection.initialize_start() {
        Ok(it) => it,
        Err(e) => {
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };

    tracing::info!("InitializeParams: {}", initialize_params);

    let server_capabilities = lsp::server_capabilities();

    let initialize_result = lsp_types::InitializeResult {
        capabilities: server_capabilities,
        server_info: Some(lsp_types::ServerInfo {
            name: String::from("rust-analyzer"),
            version: Some(version::version().to_string()),
        }),
    };

    let initialize_result = to_json(&initialize_result)?;

    if let Err(e) = connection.initialize_finish(initialize_id, initialize_result) {
        if e.channel_is_disconnected() {
            io_threads.join()?;
        }
        return Err(e.into());
    }

    // If the io_threads have an error, there's usually an error on the main
    // loop too because the channels are closed. Ensure we report both errors.
    match (main_loop::main_loop(connection), io_threads.join()) {
        (Err(loop_e), Err(join_e)) => anyhow::bail!("{loop_e}\n{join_e}"),
        (Ok(_), Err(join_e)) => anyhow::bail!("{join_e}"),
        (Err(loop_e), Ok(_)) => anyhow::bail!("{loop_e}"),
        (Ok(_), Ok(_)) => {}
    }

    tracing::info!("server did shut down");
    Ok(())
}

pub fn from_json<T: DeserializeOwned>(what: &'static str, json: &serde_json::Value) -> anyhow::Result<T> {
    serde_json::from_value(json.clone()).map_err(|e| anyhow::format_err!("Failed to deserialize {what}: {e}; {json}"))
}

pub fn to_json<T: serde::Serialize + std::fmt::Debug>(value: &T) -> anyhow::Result<serde_json::Value> {
    serde_json::to_value(value).map_err(|e| anyhow::format_err!("Failed to serialize value: {e}; {value:?}"))
}
