use crate::syntax::TriviaPieceKind;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TriviaPiece {
    pub(crate) kind: TriviaPieceKind,
    pub(crate) length: u32,
}

impl TriviaPiece {
    /// Creates a new whitespace trivia piece with the given length
    pub fn whitespace<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::Whitespace, len)
    }

    /// Creates a new newline trivia piece with the given text length
    pub fn newline<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::Newline, len)
    }

    pub fn comment<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::SingleLineComment, len)
    }

    pub fn new<L: Into<u32>>(kind: TriviaPieceKind, length: L) -> Self {
        Self {
            kind,
            length: length.into(),
        }
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
