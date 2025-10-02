use std::borrow::Cow;

use crate::{GreenTriviaList, SyntaxKind};

#[derive(Debug)]
pub struct GreenToken<'token> {
    kind: SyntaxKind,
    text: Cow<'token, [u8]>,
    full_text: Cow<'token, [u8]>,
    width: usize,
    full_width: usize,
    leading_trivia: GreenTriviaList<'token>,
    trailing_trivia: GreenTriviaList<'token>,
    leading_trivia_width: usize,
    trailing_trivia_width: usize,
}
