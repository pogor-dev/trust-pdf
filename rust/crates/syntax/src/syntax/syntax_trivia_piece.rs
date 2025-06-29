use std::{fmt, marker::PhantomData, ops::Range};

use crate::{
    cursor,
    syntax::{
        Language, SyntaxToken, SyntaxTriviaPieceComments, SyntaxTriviaPieceNewline,
        SyntaxTriviaPieceSkipped, SyntaxTriviaPieceWhitespace, TriviaPiece, TriviaPieceKind,
    },
};

#[derive(Clone)]
pub struct SyntaxTriviaPiece<L: Language> {
    pub(crate) raw: cursor::SyntaxTrivia,
    /// Absolute offset from the beginning of the file
    pub(crate) offset: u32,
    pub(crate) trivia: TriviaPiece,
    pub(crate) _p: PhantomData<L>,
}

impl<L: Language> SyntaxTriviaPiece<L> {
    pub(crate) fn into_raw_piece(self) -> TriviaPiece {
        self.trivia
    }

    /// Returns the internal kind of this trivia piece
    pub fn kind(&self) -> TriviaPieceKind {
        self.trivia.kind()
    }

    pub fn text(&self) -> &[u8] {
        let token = self.raw.token();
        let txt = token.text();

        // Compute the offset relative to the token
        let start = self.offset - token.text_range().start();
        let end = start + self.text_len();

        // Don't use self.raw.text(). It iterates over all pieces
        &txt[start.into()..end.into()]
    }

    pub fn text_len(&self) -> u32 {
        self.trivia.text_len()
    }

    pub fn text_range(&self) -> Range<u32> {
        Range {
            start: self.offset,
            end: self.offset + self.text_len(),
        }
    }

    pub fn is_newline(&self) -> bool {
        self.trivia.kind.is_newline()
    }

    pub fn is_whitespace(&self) -> bool {
        self.trivia.kind.is_whitespace()
    }

    pub const fn is_comment(&self) -> bool {
        self.trivia.kind.is_comment()
    }

    /// Returns true if this trivia piece is a [SyntaxTriviaPieceSkipped].
    pub fn is_skipped(&self) -> bool {
        self.trivia.kind.is_skipped()
    }

    pub fn as_newline(&self) -> Option<SyntaxTriviaPieceNewline<L>> {
        match &self.trivia.kind {
            TriviaPieceKind::Newline => Some(SyntaxTriviaPieceNewline(self.clone())),
            _ => None,
        }
    }

    pub fn as_whitespace(&self) -> Option<SyntaxTriviaPieceWhitespace<L>> {
        match &self.trivia.kind {
            TriviaPieceKind::Whitespace => Some(SyntaxTriviaPieceWhitespace(self.clone())),
            _ => None,
        }
    }

    pub fn as_comments(&self) -> Option<SyntaxTriviaPieceComments<L>> {
        match &self.trivia.kind {
            TriviaPieceKind::SingleLineComment | TriviaPieceKind::MultiLineComment => {
                Some(SyntaxTriviaPieceComments(self.clone()))
            }
            _ => None,
        }
    }

    /// Casts this piece to a skipped trivia piece.
    pub fn as_skipped(&self) -> Option<SyntaxTriviaPiece<L>> {
        match &self.trivia.kind {
            TriviaPieceKind::Skipped => Some(SyntaxTriviaPieceSkipped(self.clone())),
            _ => None,
        }
    }

    pub fn token(&self) -> SyntaxToken<L> {
        SyntaxToken::from(self.raw.token().clone())
    }
}

impl<L: Language> fmt::Debug for SyntaxTriviaPiece<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.trivia.kind {
            TriviaPieceKind::Newline => write!(f, "Newline(")?,
            TriviaPieceKind::Whitespace => write!(f, "Whitespace(")?,
            TriviaPieceKind::SingleLineComment | TriviaPieceKind::MultiLineComment => {
                write!(f, "Comment(")?
            }
            TriviaPieceKind::Skipped => write!(f, "Skipped(")?,
        }
        print_debug_str(self.text(), f)?;
        write!(f, ")")
    }
}

fn print_debug_str(text: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if text.len() < 25 {
        write!(f, "{:?}", text)
    } else {
        for idx in 21..25 {
            if text.is_char_boundary(idx) {
                let text = format!("{} ...", &text[..idx]);
                return write!(f, "{:?}", text);
            }
        }
        write!(f, "")
    }
}
