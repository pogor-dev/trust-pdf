//! WebAssembly bindings for the PDF lexer.
//!
//! Exposes a JS-friendly API over the core `lexer` crate so it can be used
//! from TypeScript/JavaScript via `wasm-bindgen`.

use lexer::Lexer as RustLexer;
use syntax::SyntaxKind;
use wasm_bindgen::prelude::*;

/// Token returned by the WASM lexer.
///
/// Contains a user-friendly `kind` string, the token `text`, and its `width`
/// in bytes including trivia. Getters return owned `String`s due to
/// `wasm-bindgen` ABI requirements; calling them clones the underlying data.
/// Consider reading each field once per token to avoid repeated clones.
#[wasm_bindgen]
pub struct TokenResult {
    kind: String,
    text: String,
    width: usize,
}

#[wasm_bindgen]
impl TokenResult {
    /// Kind of token as a string (e.g., "NumericLiteralToken").
    ///
    /// Note: returns an owned `String` and clones on each call.
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        self.kind.clone()
    }

    /// Original token text as UTF-8 (lossy for invalid sequences).
    ///
    /// Note: returns an owned `String` and clones on each call.
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// Full token width in bytes, including leading/trailing trivia.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> usize {
        self.width
    }
}

/// Stateful lexer for tokenizing a PDF byte stream in WebAssembly.
///
/// The lexer owns the provided `source` and yields one token per
/// `next_token()` call, advancing by the token's full width (including trivia).
///
/// Performance note: The current design re-instantiates the internal Rust
/// lexer per call to avoid self-referential borrowing between the owned source
/// and the underlying lexer. This keeps the wrapper simple and safe; future
/// refactors may introduce an internal, non-borrowing lexer or reusable cache
/// to reduce per-call setup overhead.
#[wasm_bindgen]
pub struct Lexer {
    source: Vec<u8>,
    position: usize,
}

#[wasm_bindgen]
impl Lexer {
    /// Creates a new WASM lexer over the provided bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(source: &[u8]) -> Lexer {
        Lexer {
            source: source.to_vec(),
            position: 0,
        }
    }

    /// Produces the next token and advances the internal position.
    ///
    /// Returns an `EndOfFileToken` with zero width once all input is consumed.
    #[wasm_bindgen]
    pub fn next_token(&mut self) -> TokenResult {
        if self.position >= self.source.len() {
            return TokenResult {
                kind: "EndOfFileToken".to_string(),
                text: String::new(),
                width: 0,
            };
        }

        let mut lexer = RustLexer::new(&self.source[self.position..]);
        let token = lexer.next_token();

        // Get token properties
        let syntax_kind: SyntaxKind = token.kind().into();
        let kind = format!("{:?}", syntax_kind);
        let text = String::from_utf8_lossy(&token.bytes()).to_string();
        let full_width = token.full_width() as usize;

        // Advance position by token's full width (including trivia)
        self.position += full_width;

        TokenResult { kind, text, width: full_width }
    }
}
