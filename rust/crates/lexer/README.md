## WebAssembly Support

For WebAssembly bindings, see the [`wasm-lexer`](../wasm-lexer) crate.

To build the WASM module:

```sh
wasm-pack build crates/wasm-lexer --target web
```

Open the HTML playground:

```sh
npx http-server
```

Then navigate to `http://localhost:8081/crates/wasm-lexer/wasm.html` in your web browser.