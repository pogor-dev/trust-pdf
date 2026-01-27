use std::{fmt, ops};

use crate::{GreenToken, GreenTrait, SyntaxKind};

use super::SyntaxNode;

/// Positioned token in the red tree.
#[derive(Clone)]
pub struct SyntaxToken {
    green: GreenToken,
    parent: Option<Box<SyntaxNode>>,
    position: u32,
    index: u32,
}

impl SyntaxToken {
    /// Creates a new root token (rarely used - tokens usually have parents).
    #[inline]
    pub fn new_root(green: crate::GreenToken) -> Self {
        Self {
            green,
            parent: None,
            position: 0,
            index: 0,
        }
    }

    /// Creates a new child token with parent link.
    #[inline]
    pub(crate) fn new_child(green: GreenToken, parent: SyntaxNode, position: u32, index: u32) -> Self {
        Self {
            green,
            parent: Some(Box::new(parent)),
            position,
            index,
        }
    }

    /// Returns the kind of this token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.green.kind()
    }

    /// Returns a reference to the underlying green token.
    #[inline]
    pub fn green(&self) -> &GreenToken {
        &self.green
    }

    /// Returns a reference to the parent node if it exists.
    #[inline]
    pub fn parent(&self) -> Option<&SyntaxNode> {
        self.parent.as_deref()
    }

    /// Returns the absolute byte position of this token in the source.
    #[inline]
    pub fn position(&self) -> u32 {
        self.position
    }

    /// Returns the index of this token within its parent's children.
    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Returns the full width of this token in bytes (including trivia).
    #[inline]
    pub fn full_width(&self) -> u32 {
        self.green.full_width()
    }

    /// Returns the token text without trivia.
    #[inline]
    pub fn text(&self) -> &[u8] {
        self.green.text()
    }

    /// Returns the byte range span of this token (including trivia).
    #[inline]
    pub fn full_span(&self) -> ops::Range<u32> {
        let start = self.position;
        let end = start + self.full_width();
        start..end
    }

    /// Returns the byte range of just the token text (excluding trivia).
    #[inline]
    pub fn span(&self) -> ops::Range<u32> {
        let leading_width = self.green.leading_trivia().map_or(0, |n| n.full_width());
        let start = self.position + leading_width;
        let end = start + self.green.text().len() as u32;
        start..end
    }

    /// Returns diagnostics attached to this token, if any.
    #[inline]
    pub fn diagnostics(&self) -> Option<&crate::GreenDiagnostics> {
        self.green.diagnostics()
    }
}

impl PartialEq for SyntaxToken {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.green, &*other.green) && self.position == other.position
    }
}

impl Eq for SyntaxToken {}

impl fmt::Debug for SyntaxToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxToken")
            .field("kind", &self.kind())
            .field("text", &String::from_utf8_lossy(self.text()))
            .field("position", &self.position)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{SyntaxElement, SyntaxKind, SyntaxNode, tree};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_token_span() {
        // Create token with leading/trailing trivia: "  42  "
        let green = tree! {
            SyntaxKind::List => {
                (SyntaxKind::NumericLiteralToken) => {
                    trivia(SyntaxKind::WhitespaceTrivia, b"  "),
                    text(b"42"),
                    trivia(SyntaxKind::WhitespaceTrivia, b"  ")
                }
            }
        };

        let root = SyntaxNode::new_root(green);
        let children: Vec<_> = root.children().collect();

        match &children[0] {
            SyntaxElement::Token(t) => {
                assert_eq!(t.full_width(), 6); // 2 + 2 + 2
                let trimmed = t.span();
                assert_eq!(trimmed.start, 2); // After leading trivia
                assert_eq!(trimmed.end, 4); // Before trailing trivia
            }
            _ => panic!("Expected token"),
        }
    }
}
