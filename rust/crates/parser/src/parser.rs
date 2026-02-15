use lexer::Lexer;
use syntax::{GreenPdfDocumentSyntax, GreenToken};

pub struct Parser<'source> {
    pub(super) lexer: Lexer<'source>,
    pub(super) lexed_tokens: Vec<Option<GreenToken>>,
    /// Global token index for `lexed_tokens[0]` in the full stream.
    pub(super) window_start: usize,
    /// Slot offset of the current token within `lexed_tokens`.
    pub(super) window_offset: usize,
    /// Number of valid cached tokens in `lexed_tokens`.
    pub(super) window_size: usize,
}

// TODO: we should return red nodes instead, but as temporary measure we return green nodes
impl<'source> Parser<'source> {
    pub(super) const CACHED_TOKEN_ARRAY_SIZE: usize = 64;

    pub fn new(lexer: Lexer<'source>) -> Self {
        let mut parser = Self {
            lexer,
            lexed_tokens: vec![None; Self::CACHED_TOKEN_ARRAY_SIZE],
            window_offset: 0,
            window_size: 0,
            window_start: 0,
        };

        parser.pre_lex();
        parser
    }

    pub fn parse_pdf_document(&mut self) -> GreenPdfDocumentSyntax {
        // Parsing logic goes here
        unreachable!()
    }
}
