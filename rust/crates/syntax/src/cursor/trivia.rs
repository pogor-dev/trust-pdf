use std::ops::Range;

use crate::{GreenTrivia, cursor::SyntaxToken};

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
enum SyntaxTriviaType {
    Leading,
    Trailing,
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub(crate) struct SyntaxTrivia {
    token: SyntaxToken,
    kind: SyntaxTriviaType,
}

impl SyntaxTrivia {
    pub(super) fn leading(token: SyntaxToken) -> Self {
        Self {
            token,
            kind: SyntaxTriviaType::Leading,
        }
    }

    pub(super) fn trailing(token: SyntaxToken) -> Self {
        Self {
            token,
            kind: SyntaxTriviaType::Trailing,
        }
    }

    pub(crate) fn text(&self) -> Vec<u8> {
        match self.kind {
            SyntaxTriviaType::Leading => self.token.green().leading_trivia().text(),
            SyntaxTriviaType::Trailing => self.token.green().trailing_trivia().text(),
        }
    }

    pub(crate) fn token(&self) -> &SyntaxToken {
        &self.token
    }

    pub(crate) fn span(&self) -> Range<usize> {
        let length = self.green_trivia().width();
        let token_range = self.token.full_span();

        match self.kind {
            SyntaxTriviaType::Leading => token_range.start..token_range.start + length,
            SyntaxTriviaType::Trailing => token_range.end - length..length,
        }
    }

    /// Get the number of TriviaPiece inside this trivia
    pub(crate) fn width(&self) -> usize {
        self.green_trivia().width()
    }

    /// Gets index-th trivia piece when the token associated with this trivia was created.
    /// See [SyntaxTriviaPiece].
    pub(crate) fn get_piece(&self, index: usize) -> Option<&TriviaPiece> {
        self.green_trivia().get_piece(index)
    }

    fn green_trivia(&self) -> &GreenTrivia {
        match self.kind {
            SyntaxTriviaType::Leading => self.token.green().leading_trivia(),
            SyntaxTriviaType::Trailing => self.token.green().trailing_trivia(),
        }
    }

    /// Returns the last trivia piece element
    pub(crate) fn last(&self) -> Option<&TriviaPiece> {
        self.green_trivia().pieces().last()
    }

    /// Returns the first trivia piece element
    pub(crate) fn first(&self) -> Option<&TriviaPiece> {
        self.green_trivia().pieces().first()
    }
}
