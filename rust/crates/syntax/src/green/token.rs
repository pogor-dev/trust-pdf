use std::{borrow::Cow, fmt};

use crate::{GreenNode, SyntaxKind, green::Trivia, syntax_kind_facts};

pub struct GreenToken<'a> {
    kind: SyntaxKind,
    full_width: usize,
    text: Cow<'a, [u8]>,
    leading_trivia: Option<Trivia<'a>>,
    trailing_trivia: Option<Trivia<'a>>,
}

impl<'a> GreenToken<'a> {
    #[inline]
    pub fn new_with_kind(kind: SyntaxKind) -> Self {
        let text = syntax_kind_facts::get_text(kind);

        let full_width = text.len();
        Self {
            kind,
            full_width,
            text: text.into(),
            leading_trivia: None,
            trailing_trivia: None,
        }
    }

    #[inline]
    pub fn new_with_text(kind: SyntaxKind, text: Cow<'a, [u8]>) -> Self {
        let full_width = text.len();
        Self {
            kind,
            full_width,
            text,
            leading_trivia: None,
            trailing_trivia: None,
        }
    }
}

impl<'a> GreenNode<'a> for GreenToken<'a> {
    fn kind(&self) -> SyntaxKind {
        todo!()
    }

    fn to_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    fn to_full_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    fn full_width(&self) -> u64 {
        todo!()
    }

    fn slot(&'_ self, _index: u8) -> Option<super::NodeOrToken<'_>> {
        todo!()
    }

    fn slot_count(&self) -> u8 {
        todo!()
    }

    fn leading_trivia(&'_ self) -> Option<Trivia<'_>> {
        todo!()
    }

    fn trailing_trivia(&'_ self) -> Option<Trivia<'_>> {
        todo!()
    }

    fn leading_trivia_width(&self) -> u64 {
        todo!()
    }

    fn trailing_trivia_width(&self) -> u64 {
        todo!()
    }
}

impl Clone for GreenToken<'_> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            full_width: self.full_width,
            text: self.text.clone(),
            leading_trivia: self.leading_trivia.clone(),
            trailing_trivia: self.trailing_trivia.clone(),
        }
    }
}

impl PartialEq for GreenToken<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.full_width == other.full_width
            && self.text == other.text
            && self.leading_trivia == other.leading_trivia
            && self.trailing_trivia == other.trailing_trivia
    }
}

impl Eq for GreenToken<'_> {}

unsafe impl Send for GreenToken<'_> {}
unsafe impl Sync for GreenToken<'_> {}

impl fmt::Debug for GreenToken<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GreenToken")
            .field("kind", &self.kind())
            .field("full_text", &String::from_utf8_lossy(&self.to_full_string()))
            .field("full_width", &self.full_width())
            .finish()
    }
}
