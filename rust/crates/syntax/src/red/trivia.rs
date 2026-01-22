use std::{fmt, ops};

use crate::{GreenTrivia, SyntaxKind};

use super::SyntaxNode;

/// Positioned trivia in the red tree.
#[derive(Clone)]
pub struct SyntaxTrivia {
    green: GreenTrivia,
    parent: Option<Box<SyntaxNode>>,
    position: u32,
    index: u32,
}

impl SyntaxTrivia {
    /// Creates a new root trivia (rarely used).
    #[inline]
    pub fn new_root(green: crate::GreenTrivia) -> Self {
        Self {
            green,
            parent: None,
            position: 0,
            index: 0,
        }
    }

    /// Creates a new child trivia with parent link.
    #[inline]
    pub(crate) fn new_child(green: GreenTrivia, parent: SyntaxNode, position: u32, index: u32) -> Self {
        Self {
            green,
            parent: Some(Box::new(parent)),
            position,
            index,
        }
    }

    /// Returns the kind of this trivia.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.green.kind()
    }

    /// Returns a reference to the underlying green trivia.
    #[inline]
    pub fn green(&self) -> &GreenTrivia {
        &self.green
    }

    /// Returns a reference to the parent node if it exists.
    #[inline]
    pub fn parent(&self) -> Option<&SyntaxNode> {
        self.parent.as_deref()
    }

    /// Returns the absolute byte position of this trivia in the source.
    #[inline]
    pub fn position(&self) -> u32 {
        self.position
    }

    /// Returns the index of this trivia within its parent's children.
    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Returns the trivia text.
    #[inline]
    pub fn text(&self) -> &[u8] {
        self.green.text()
    }

    /// Returns the byte range span of this trivia.
    #[inline]
    pub fn full_span(&self) -> ops::Range<u32> {
        let start = self.position;
        let end = start + self.text().len() as u32;
        start..end
    }
}

impl PartialEq for SyntaxTrivia {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.green, &*other.green) && self.position == other.position
    }
}

impl Eq for SyntaxTrivia {}

impl fmt::Debug for SyntaxTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxTrivia")
            .field("kind", &self.kind())
            .field("text", &String::from_utf8_lossy(self.text()))
            .field("position", &self.position)
            .finish()
    }
}
