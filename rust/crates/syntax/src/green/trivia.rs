use std::borrow::Cow;

use crate::SyntaxKind;

#[derive(Debug, Default)]
pub struct GreenTriviaList<'trivia> {
    pieces: Vec<GreenTrivia<'trivia>>,
    full_width: usize,
}

#[derive(Debug)]
pub struct GreenTrivia<'trivia> {
    kind: SyntaxKind,
    full_text: Cow<'trivia, [u8]>,
    full_width: usize,
}
