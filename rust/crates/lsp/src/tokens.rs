use crate::line_map::{compute_line_starts, offset_to_line_col};
use lexer::Lexer;
use lsp_types::SemanticToken;
use syntax::{GreenTrait, Slot, SyntaxKind};

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum TokenType {
    Keyword = 0,
    String = 1,
    Number = 2,
    Property = 3,
    Comment = 4,
}

/// Builder for semantic tokens using relative encoding.
/// Tokens are encoded as deltas from the previous token position.
struct SemanticTokensBuilder {
    data: Vec<SemanticToken>,
    prev_line: u32,
    prev_char: u32,
}

impl SemanticTokensBuilder {
    fn new() -> Self {
        SemanticTokensBuilder {
            data: Vec::new(),
            prev_line: 0,
            prev_char: 0,
        }
    }

    /// Push a token with absolute line/character position and length
    fn push(&mut self, line: u32, char: u32, length: u32, token_type: TokenType) {
        let mut delta_line = line;
        let mut delta_start = char;

        if !self.data.is_empty() {
            delta_line -= self.prev_line;
            if delta_line == 0 {
                delta_start -= self.prev_char;
            }
        }

        self.data.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type: token_type as u32,
            token_modifiers_bitset: 0,
        });

        self.prev_line = line;
        self.prev_char = char;
    }

    fn build(self) -> Vec<SemanticToken> {
        self.data
    }
}

pub fn map_kind(kind: SyntaxKind) -> Option<TokenType> {
    use SyntaxKind as K;
    let t = match kind {
        K::TrueKeyword
        | K::FalseKeyword
        | K::NullKeyword
        | K::IndirectObjectKeyword
        | K::IndirectEndObjectKeyword
        | K::IndirectReferenceKeyword
        | K::StreamKeyword
        | K::EndStreamKeyword
        | K::XRefKeyword
        | K::XRefFreeEntryKeyword
        | K::XRefInUseEntryKeyword
        | K::FileTrailerKeyword
        | K::StartXRefKeyword
        | K::PdfVersionToken
        | K::EndOfFileMarkerToken => TokenType::Keyword,
        K::StringLiteralToken | K::HexStringLiteralToken => TokenType::String,
        K::NumericLiteralToken => TokenType::Number,
        K::NameLiteralToken => TokenType::Property,
        _ => return None,
    };
    Some(t)
}

pub fn compute_semantic_tokens(text: &str) -> Vec<SemanticToken> {
    let mut builder = SemanticTokensBuilder::new();
    let line_starts = compute_line_starts(text.as_bytes());

    let mut lexer = Lexer::new(text.as_bytes());
    let mut offset: usize = 0;

    loop {
        let token = lexer.next_token();
        let token_kind = token.kind();
        let token_width = token.width() as usize;
        let token_full_width = token.full_width() as usize;

        if token_kind == SyntaxKind::EndOfFileToken {
            break;
        }

        // Leading trivia comments
        let mut leading_consumed: usize = 0;
        if let Some(leading_node) = token.leading_trivia() {
            for slot in leading_node.slots() {
                if let Slot::Trivia { trivia, .. } = slot {
                    let trivia_width = trivia.width() as usize;
                    if trivia.kind() == SyntaxKind::CommentTrivia {
                        let abs_offset = offset + leading_consumed;
                        let (line, col) = offset_to_line_col(abs_offset, &line_starts);
                        builder.push(line, col as u32, trivia.width() as u32, TokenType::Comment);
                    }
                    leading_consumed += trivia_width;
                }
            }
        }

        // Main token
        if let Some(token_type) = map_kind(token_kind) {
            let abs_offset = offset + leading_consumed;
            let (line, col) = offset_to_line_col(abs_offset, &line_starts);
            builder.push(line, col as u32, token_width as u32, token_type);
        }

        // Trailing trivia comments
        let trailing_base = offset + leading_consumed + token_width;
        let mut trailing_consumed: usize = 0;
        if let Some(trailing_node) = token.trailing_trivia() {
            for slot in trailing_node.slots() {
                if let Slot::Trivia { trivia, .. } = slot {
                    let trivia_width = trivia.width() as usize;
                    if trivia.kind() == SyntaxKind::CommentTrivia {
                        let abs_offset = trailing_base + trailing_consumed;
                        let (line, col) = offset_to_line_col(abs_offset, &line_starts);
                        builder.push(line, col as u32, trivia.width() as u32, TokenType::Comment);
                    }
                    trailing_consumed += trivia_width;
                }
            }
        }

        offset += token_full_width;
    }

    builder.build()
}
