#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum TriviaPieceKind {
    /// A line break character, such as '\r'
    CarriageReturn,
    /// A line break character, such as '\n'
    LineFeed,
    /// A line break character, such as '\r\n'
    CarriageReturnLineFeed,
    /// Any whitespace character
    Whitespace,
    /// Comment that does not contain any line breaks
    Comment,
    /// Token that the parser skipped for some reason.
    Skipped,
}

impl TriviaPieceKind {
    pub const fn is_carriage_return(&self) -> bool {
        matches!(self, TriviaPieceKind::CarriageReturn)
    }

    pub const fn is_line_feed(&self) -> bool {
        matches!(self, TriviaPieceKind::LineFeed)
    }

    pub const fn is_carriage_return_line_feed(&self) -> bool {
        matches!(self, TriviaPieceKind::CarriageReturnLineFeed)
    }

    pub const fn is_newline(&self) -> bool {
        matches!(
            self,
            TriviaPieceKind::CarriageReturn
                | TriviaPieceKind::LineFeed
                | TriviaPieceKind::CarriageReturnLineFeed
        )
    }

    pub const fn is_whitespace(&self) -> bool {
        matches!(self, TriviaPieceKind::Whitespace)
    }

    pub const fn is_comment(&self) -> bool {
        matches!(self, TriviaPieceKind::Comment)
    }

    pub const fn is_skipped(&self) -> bool {
        matches!(self, TriviaPieceKind::Skipped)
    }
}
