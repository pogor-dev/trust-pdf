## Run as WebAssembly Module

```sh
wasm-pack build crates/lexer --target web
```

Open the HTML playground:

```sh
npx http-server
```

Then navigate to `http://localhost:8081/wasm.html` in your web browser.