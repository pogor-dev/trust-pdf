use std::{fmt, ops::Range};

use crate::{
    cursor::{token::SyntaxToken, trivia_pieces_iterator::SyntaxTriviaPiecesIterator},
    green::trivia::GreenTrivia,
};

#[derive(PartialEq, Eq, Clone, Hash)]
pub(crate) struct SyntaxTrivia {
    token: SyntaxToken,
    is_leading: bool,
}

impl SyntaxTrivia {
    pub(crate) fn text(&self) -> &[u8] {
        let trivia_range = self.text_range();
        let token_offset = self.token.data().offset;

        let relative_range = Range {
            start: (trivia_range.start - token_offset) as usize,
            end: (trivia_range.end - trivia_range.start) as usize,
        };

        &self.token.text()[relative_range]
    }

    /// Get the number of TriviaPiece inside this trivia
    pub(crate) fn len(&self) -> usize {
        self.green_trivia().len()
    }

    pub(crate) fn text_range(&self) -> Range<u64> {
        let length = self.green_trivia().text_len();
        let token_range = self.token.text_range();

        match self.is_leading {
            true => Range {
                start: token_range.start,
                end: length,
            },
            false => Range {
                start: token_range.end - length,
                end: length,
            },
        }
    }

    fn green_trivia(&self) -> &GreenTrivia {
        match self.is_leading {
            true => self.token.green().leading_trivia(),
            false => self.token.green().trailing_trivia(),
        }
    }

    /// Iterate over all pieces of the trivia. The iterator returns the offset
    /// of the trivia as [TextSize] and its data as [Trivia], which contains its length.
    /// See [SyntaxTriviaPiece].
    pub(crate) fn pieces(&self) -> SyntaxTriviaPiecesIterator {
        let range = self.text_range();
        SyntaxTriviaPiecesIterator {
            raw: self.clone(),
            next_index: 0,
            next_offset: range.start,
            end_index: self.len(),
            end_offset: range.end,
        }
    }
}

impl fmt::Debug for SyntaxTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("SyntaxTrivia");
        f.field("text_range", &self.text_range());
        f.finish()
    }
}

impl fmt::Display for SyntaxTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.text()))
    }
}
