#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum TriviaPieceKind {
    /// A line break (`\n`, `\r`, `\r\n`, ...)
    Newline,
    /// Any whitespace character
    Whitespace,
    /// Comment that does not contain any line breaks
    Comment,
    /// Token that the parser skipped for some reason.
    Skipped,
}

impl TriviaPieceKind {
    pub const fn is_newline(&self) -> bool {
        matches!(self, TriviaPieceKind::Newline)
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
