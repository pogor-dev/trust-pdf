//! Iterator for child elements (nodes and tokens) of a syntax node.
//!
//! ```text
//!     👨‍👧‍👦 SyntaxElementChildren  
//!       🌳
//!       ├── 🌿  Iterator over:
//!       ├── 📝  • nodes AND tokens
//!       └── 🌿  • direct children only
//! ```

use crate::{
    SyntaxKind,
    cursor::{element::SyntaxElement, node::SyntaxNode},
};

#[derive(Clone, Debug)]
pub struct SyntaxElementChildren {
    parent: SyntaxNode,
    next: Option<SyntaxElement>,
    next_initialized: bool,
}

impl SyntaxElementChildren {
    /// Creates a new iterator over child elements of the given parent.
    pub(super) fn new(parent: SyntaxNode) -> SyntaxElementChildren {
        SyntaxElementChildren {
            parent,
            next: None,
            next_initialized: false,
        }
    }

    /// Filters the iteration to only yield children matching the provided predicate
    pub fn by_kind<F: Fn(SyntaxKind) -> bool>(self, matcher: F) -> SyntaxElementChildrenByKind<F> {
        if !self.next_initialized {
            SyntaxElementChildrenByKind {
                next: self.parent.first_child_or_token_by_kind(&matcher),
                matcher,
            }
        } else {
            SyntaxElementChildrenByKind {
                next: self.next.and_then(|node| {
                    if matcher(node.kind()) {
                        Some(node)
                    } else {
                        node.next_sibling_or_token_by_kind(&matcher)
                    }
                }),
                matcher,
            }
        }
    }
}

impl Iterator for SyntaxElementChildren {
    type Item = SyntaxElement;
    fn next(&mut self) -> Option<SyntaxElement> {
        if !self.next_initialized {
            self.next = self.parent.first_child_or_token();
            self.next_initialized = true;
        } else {
            self.next = self
                .next
                .take()
                .and_then(|next| next.to_next_sibling_or_token());
        }

        self.next.clone()
    }
}

#[derive(Clone, Debug)]
pub struct SyntaxElementChildrenByKind<F: Fn(SyntaxKind) -> bool> {
    next: Option<SyntaxElement>,
    matcher: F,
}

impl<F: Fn(SyntaxKind) -> bool> Iterator for SyntaxElementChildrenByKind<F> {
    type Item = SyntaxElement;
    fn next(&mut self) -> Option<SyntaxElement> {
        self.next.take().inspect(|next| {
            self.next = next.next_sibling_or_token_by_kind(&self.matcher);
        })
    }
}
