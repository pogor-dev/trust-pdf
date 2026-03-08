use std::sync::LazyLock;

use crate::{GreenNode, GreenTokenElement, GreenTrivia, SyntaxKind};

pub(crate) struct GreenSyntaxFactory;

impl GreenSyntaxFactory {
    pub(crate) fn space() -> GreenTrivia {
        static SPACE: LazyLock<GreenTrivia> = LazyLock::new(|| GreenSyntaxFactory::whitespace(b" "));
        SPACE.clone()
    }

    pub(crate) fn line_feed() -> GreenTrivia {
        static LINE_FEED: LazyLock<GreenTrivia> = LazyLock::new(|| GreenSyntaxFactory::end_of_line(b"\n"));
        LINE_FEED.clone()
    }

    pub(crate) fn carriage_return_line_feed() -> GreenTrivia {
        static CARRIAGE_RETURN_LINE_FEED: LazyLock<GreenTrivia> = LazyLock::new(|| GreenSyntaxFactory::end_of_line(b"\r\n"));
        CARRIAGE_RETURN_LINE_FEED.clone()
    }

    pub(crate) fn token(kind: SyntaxKind) -> GreenTokenElement {
        GreenTokenElement::create_with_trivia(kind, None, None)
    }

    pub(crate) fn token_with_leading_trivia(leading_trivia: Option<GreenNode>, kind: SyntaxKind) -> GreenTokenElement {
        GreenTokenElement::create_with_trivia(kind, leading_trivia, None)
    }

    pub(crate) fn token_with_trailing_trivia(kind: SyntaxKind, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_trivia(kind, None, trailing_trivia)
    }

    pub(crate) fn token_with_trivia(leading_trivia: Option<GreenNode>, kind: SyntaxKind, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_trivia(kind, leading_trivia, trailing_trivia)
    }

    pub(crate) fn token_with_int_value(kind: SyntaxKind, text: &[u8], value: i32) -> GreenTokenElement {
        GreenTokenElement::create_with_int_value_and_trivia(kind, text, value, None, None)
    }

    pub(crate) fn literal_int(leading_trivia: Option<GreenNode>, text: &[u8], value: i32, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_int_value_and_trivia(SyntaxKind::NumericLiteralToken, text, value, leading_trivia, trailing_trivia)
    }

    pub(crate) fn literal_float(leading_trivia: Option<GreenNode>, text: &[u8], value: f32, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_float_value_and_trivia(SyntaxKind::NumericLiteralToken, text, value, leading_trivia, trailing_trivia)
    }

    pub(crate) fn literal_string(leading_trivia: Option<GreenNode>, text: &[u8], value: String, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_string_value_and_trivia(SyntaxKind::StringLiteralToken, text, value, leading_trivia, trailing_trivia)
    }

    pub(crate) fn literal_hex_string(leading_trivia: Option<GreenNode>, text: &[u8], value: String, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_string_value_and_trivia(SyntaxKind::HexStringLiteralToken, text, value, leading_trivia, trailing_trivia)
    }

    pub(crate) fn literal_name(leading_trivia: Option<GreenNode>, text: &[u8], value: String, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_string_value_and_trivia(SyntaxKind::NameLiteralToken, text, value, leading_trivia, trailing_trivia)
    }

    pub(crate) fn end_of_file_marker(leading_trivia: Option<GreenNode>, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_trivia(SyntaxKind::EndOfFileMarkerToken, leading_trivia, trailing_trivia)
    }

    pub(crate) fn bad_token(leading_trivia: Option<GreenNode>, text: &[u8], trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_string_value_and_trivia(
            SyntaxKind::BadToken,
            text,
            String::from_utf8_lossy(text).to_string(),
            leading_trivia,
            trailing_trivia,
        )
    }

    pub(crate) fn comment(text: &[u8]) -> GreenTrivia {
        GreenTrivia::new(SyntaxKind::CommentTrivia, text)
    }

    pub(crate) fn whitespace(text: &[u8]) -> GreenTrivia {
        GreenTrivia::new(SyntaxKind::WhitespaceTrivia, text)
    }

    pub(crate) fn end_of_line(text: &[u8]) -> GreenTrivia {
        GreenTrivia::new(SyntaxKind::EndOfLineTrivia, text)
    }
}
