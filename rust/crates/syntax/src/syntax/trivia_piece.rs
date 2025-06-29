use crate::syntax::trivia_piece_kind::TriviaPieceKind;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TriviaPiece {
    pub(crate) kind: TriviaPieceKind,
    pub(crate) length: u32,
}

impl TriviaPiece {
    pub fn new<L: Into<u32>>(kind: TriviaPieceKind, length: L) -> Self {
        Self {
            kind,
            length: length.into(),
        }
    }

    /// Creates a new whitespace trivia piece with the given length
    pub fn whitespace<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::Whitespace, len)
    }

    pub fn carriage_return<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::CarriageReturn, len)
    }

    pub fn line_feed<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::LineFeed, len)
    }

    pub fn carriage_return_line_feed<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::CarriageReturnLineFeed, len)
    }

    /// Creates a new comment trivia piece that does not contain any line breaks.
    /// For example, JavaScript's `//` comments are guaranteed to not spawn multiple lines. However,
    /// this can also be a `/* ... */` comment if it doesn't contain any line break characters.
    pub fn comment<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::Comment, len)
    }

    /// Returns the trivia's length
    pub fn text_len(&self) -> u32 {
        self.length
    }

    /// Returns the trivia's kind
    pub fn kind(&self) -> TriviaPieceKind {
        self.kind
    }
}
