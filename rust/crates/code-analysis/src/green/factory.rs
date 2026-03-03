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
        GreenTokenElement::create(kind)
    }

    pub(crate) fn token_with_trivia(leading_trivia: Option<GreenNode>, kind: SyntaxKind, trailing_trivia: Option<GreenNode>) -> GreenTokenElement {
        GreenTokenElement::create_with_trivia(kind, leading_trivia, trailing_trivia)
    }

    pub(crate) fn whitespace(text: &[u8]) -> GreenTrivia {
        GreenTrivia::new(SyntaxKind::WhitespaceTrivia, text)
    }

    pub(crate) fn end_of_line(text: &[u8]) -> GreenTrivia {
        GreenTrivia::new(SyntaxKind::EndOfLineTrivia, text)
    }
}
