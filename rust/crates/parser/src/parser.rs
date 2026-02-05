use std::cmp::min;

use lexer::Lexer;
use syntax::{GreenPdfDocumentSyntax, GreenToken};

pub struct Parser<'source> {
    pub(super) lexer: Lexer<'source>,
    pub(super) current_token: Option<GreenToken>,
    pub(super) lexer_tokens: Vec<GreenToken>,
    /// The index of the first token in the lexer_tokens vector
    pub(super) first_token: usize,
    /// The offset of the current token in the lexer_tokens vector
    pub(super) token_offset: usize,
    /// The total number of tokens cached in the lexer_tokens vector
    pub(super) token_count: usize,
}

// TODO: we should return red nodes instead, but as temporary measure we return green nodes
impl<'source> Parser<'source> {
    const MAX_CACHED_TOKENS: usize = 64;

    pub fn new(lexer: Lexer<'source>) -> Self {
        let source_len = lexer.source_length();
        Self {
            lexer,
            current_token: None,
            lexer_tokens: Vec::with_capacity(min(Self::MAX_CACHED_TOKENS, source_len / 4)),
            token_offset: 0,
            token_count: 0,
            first_token: 0,
        }
    }

    pub fn parse_pdf_document(&mut self) -> GreenPdfDocumentSyntax {
        // Parsing logic goes here
        unreachable!()
    }
}
