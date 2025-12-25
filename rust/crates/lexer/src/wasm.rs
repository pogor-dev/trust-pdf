use crate::Lexer as RustLexer;
use syntax::SyntaxKind;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TokenResult {
    kind: String,
    text: String,
    width: usize,
}

#[wasm_bindgen]
impl TokenResult {
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        self.kind.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.text.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> usize {
        self.width
    }
}

#[wasm_bindgen]
pub struct Lexer {
    source: Vec<u8>,
    position: usize,
}

#[wasm_bindgen]
impl Lexer {
    #[wasm_bindgen(constructor)]
    pub fn new(source: &[u8]) -> Lexer {
        Lexer {
            source: source.to_vec(),
            position: 0,
        }
    }

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
