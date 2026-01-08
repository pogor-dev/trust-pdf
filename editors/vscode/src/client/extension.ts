import { type OutputChannel, Uri, window, workspace, type ExtensionContext } from "vscode"
import { Wasm } from "@vscode/wasm-wasi/v1"
import {
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions,
} from "vscode-languageclient/node"
import { createStdioOptions, createUriConverters, startServer } from "@vscode/wasm-wasi-lsp"

const extensionName = "TRust PDF"

let client: LanguageClient

export async function activate(context: ExtensionContext) {
  const wasm: Wasm = await Wasm.load()
  const channel = window.createOutputChannel(`${extensionName} Server`)
  const serverOptions: ServerOptions = createServerOptions(context, wasm, channel)
  await configureClientOptions(channel, serverOptions)
}

export function deactivate() {
  return client.stop()
}

function createServerOptions(
  context: ExtensionContext,
  wasm: Wasm,
  channel: OutputChannel,
): ServerOptions {
  return async () => {
    const filename = Uri.joinPath(
      context.extensionUri,
      "server",
      "target",
      "wasm32-wasip1-threads",
      "release",
      "lsp-server.wasm",
    )

    const bits = (await workspace.fs.readFile(filename)) as Uint8Array<ArrayBuffer>
    const module = await WebAssembly.compile(bits)

    const process = await wasm.createProcess(
      "trust-pdf-lsp-server",
      module,
      { initial: 160, maximum: 160, shared: true },
      {
        stdio: createStdioOptions(),
        mountPoints: [{ kind: "workspaceFolder" }],
      },
    )

    const decoder = new TextDecoder("utf-8")
    if (process.stderr) {
      process.stderr.onData(data => {
        channel.append(decoder.decode(data))
      })
    }

    return startServer(process)
  }
}

async function configureClientOptions(channel: OutputChannel, serverOptions: ServerOptions) {
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ language: "plaintext", pattern: "**/*.pdf" }],
    outputChannel: channel,
    uriConverters: createUriConverters(),
  }

  client = new LanguageClient(
    "trust-pdf-lsp-client",
    `${extensionName} Client`,
    serverOptions,
    clientOptions,
  )

  try {
    await client.start()
  } catch (error) {
    client.error(`${extensionName} start failed`, error, "force")
  }
}
