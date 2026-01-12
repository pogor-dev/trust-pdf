use crate::line_map::{compute_line_starts, offset_to_line_col};
use lexer::Lexer;
use lsp_types::SemanticToken;
use syntax::SyntaxKind;

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum TokenType {
    Keyword = 0,
    String = 1,
    Number = 2,
    Property = 3,
    Comment = 4,
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
        | K::StartXRefKeyword => TokenType::Keyword,
        K::StringLiteralToken | K::HexStringLiteralToken => TokenType::String,
        K::NumericLiteralToken => TokenType::Number,
        K::NameLiteralToken => TokenType::Property,
        _ => return None,
    };
    Some(t)
}

pub fn compute_semantic_tokens(text: &str) -> Vec<SemanticToken> {
    let mut data: Vec<SemanticToken> = Vec::new();
    let line_starts = compute_line_starts(text.as_bytes());

    let mut lexer = Lexer::new(text.as_bytes());
    let mut offset: usize = 0;
    let mut prev_line: u32 = 0;
    let mut prev_col: u32 = 0;

    // Helper to push a semantic token given an absolute byte offset
    let mut emit = |abs_offset: usize, length: u32, token_type: TokenType, prev_line: &mut u32, prev_col: &mut u32| {
        let (line, col) = offset_to_line_col(abs_offset, &line_starts);
        let (dl, dc) = if line == *prev_line { (0, col - *prev_col) } else { (line - *prev_line, col) };
        *prev_line = line;
        *prev_col = col;

        data.push(SemanticToken {
            delta_line: dl,
            delta_start: dc,
            length,
            token_type: token_type as u32,
            token_modifiers_bitset: 0,
        });
    };

    loop {
        let tok = lexer.next_token();
        let kind: SyntaxKind = tok.kind().into();
        let width = tok.full_width() as usize;

        if kind == SyntaxKind::EndOfFileToken {
            break;
        }

        // Leading trivia comments
        let mut leading_consumed: usize = 0;
        for piece in tok.leading_trivia().pieces() {
            let pkind: SyntaxKind = piece.kind().into();
            let plen = piece.full_width() as usize;
            if pkind == SyntaxKind::CommentTrivia {
                let abs = offset + leading_consumed;
                emit(abs, piece.full_bytes().len() as u32, TokenType::Comment, &mut prev_line, &mut prev_col);
            }
            leading_consumed += plen;
        }

        // Main token
        if let Some(token_type) = map_kind(kind) {
            let token_start = offset + tok.leading_trivia().full_width() as usize;
            let token_len = tok.bytes().len() as u32;
            emit(token_start, token_len, token_type, &mut prev_line, &mut prev_col);
        }

        // Trailing trivia comments
        let trailing_base = offset + tok.leading_trivia().full_width() as usize + tok.bytes().len();
        let mut trailing_consumed: usize = 0;
        for piece in tok.trailing_trivia().pieces() {
            let pkind: SyntaxKind = piece.kind().into();
            let plen = piece.full_width() as usize;
            if pkind == SyntaxKind::CommentTrivia {
                let abs = trailing_base + trailing_consumed;
                emit(abs, piece.full_bytes().len() as u32, TokenType::Comment, &mut prev_line, &mut prev_col);
            }
            trailing_consumed += plen;
        }

        offset += width;
    }

    data
}
