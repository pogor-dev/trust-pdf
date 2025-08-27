use crate::{GreenNode, SyntaxKind, syntax_kind_facts};

pub struct SyntaxTrivia<'a> {
    kind: SyntaxKind,
    full_width: usize,
    text: &'a [u8],
}

impl<'a> SyntaxTrivia<'a> {
    #[inline]
    pub fn new_with_kind(kind: SyntaxKind) -> Self {
        let text = syntax_kind_facts::get_text(kind);
        let full_width = text.len();
        Self { kind, full_width, text }
    }

    #[inline]
    pub fn new_with_text(kind: SyntaxKind, text: &'a [u8]) -> Self {
        let full_width = text.len();
        Self { kind, full_width, text }
    }
}

impl GreenNode for SyntaxTrivia<'_> {
    #[inline]
    fn kind(&self) -> SyntaxKind {
        self.kind
    }

    #[inline]
    fn text(&self) -> &[u8] {
        &self.text
    }

    #[inline]
    fn full_text(&self) -> &[u8] {
        &self.text
    }

    #[inline]
    fn full_width(&self) -> usize {
        self.full_width
    }

    #[inline]
    fn is_trivia(&self) -> bool {
        true
    }

    #[inline]
    fn get_leading_trivia(&self) -> Option<&dyn GreenNode> {
        None
    }

    #[inline]
    fn get_trailing_trivia(&self) -> Option<&dyn GreenNode> {
        None
    }

    #[inline]
    fn get_leading_trivia_width(&self) -> usize {
        0
    }

    #[inline]
    fn get_trailing_trivia_width(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
}
