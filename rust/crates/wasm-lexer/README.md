# lexer-wasm

WebAssembly bindings for the trust-pdf lexer.

## Overview

This crate exposes a minimal, JS-friendly API over the core `lexer` crate via `wasm-bindgen`.
It provides a stateful `Lexer` wrapper that owns the input bytes and yields one `TokenResult`
per call to `next_token()`, advancing by each token's full width (including trivia).

- `TokenResult.kind`: human-readable token kind (e.g., `NumericLiteralToken`)
- `TokenResult.text`: token text as UTF-8 (lossy for invalid sequences)
- `TokenResult.width`: full token width in bytes, including trivia

Note: getters return owned `String`s and therefore clone on each call due to wasm-bindgen ABI.
Read each field once per token to avoid repeated clones.

## Requirements

- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-pack` (latest)
- Node.js (for `--node` testing) or a modern browser for headless testing

Install prerequisites:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

## Build

Build a web-targeted package (generates `pkg/`):

```bash
# From repository root
wasm-pack build crates/lexer-wasm --target web
```

The output (`pkg/`) contains `.wasm`, JS glue, and `.d.ts` types for TypeScript.

## Test

Run tests using Node.js:

```bash
# From repository root
wasm-pack test --node crates/lexer-wasm
```

Run tests in headless Chrome (example):

```bash
wasm-pack test --headless --chrome crates/lexer-wasm
```

If you prefer `cargo test`, you can configure a runner in `rust/.cargo/config.toml`:

```toml
[target.wasm32-unknown-unknown]
runner = "wasm-bindgen-test-runner"
```

Note: ensure the `wasm-bindgen-test-runner` binary is available in your PATH or use `wasm-pack test`.

## Usage from TypeScript

After `wasm-pack build --target web`, import from the generated JS:

```ts
import init, { Lexer } from "./pkg/lexer_wasm.js";

await init();

const bytes = new TextEncoder().encode("%PDF-2.0\n");
const lx = new Lexer(bytes);

for (let i = 0; i < 1024; i++) {
  const t = lx.next_token();
  console.log(t.kind, t.text, t.width);
  if (t.kind === "EndOfFileToken") break;
}
```

## Design Notes

- Allocation trade-off: the wrapper constructs a fresh internal Rust lexer per call.
  This avoids self-referential borrows while keeping the wrapper simple and safe.
  Future refactors may introduce a reusable, non-borrowing internal lexer to reduce per-call overhead.
- Core crate separation: `lexer` remains `rlib`-only; `lexer-wasm` is responsible for `cdylib` output.
