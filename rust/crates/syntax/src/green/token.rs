use std::borrow::Cow;

use crate::{GreenTriviaList, SyntaxKind};

#[derive(Debug)]
pub struct GreenToken<'token> {
    kind: SyntaxKind,
    text: Cow<'token, [u8]>,
    full_width: usize,
    leading_trivia: GreenTriviaList,
    trailing_trivia: GreenTriviaList,
}
