//! Preorder tree traversal including both nodes and tokens.
//!
//! ```text
//!     📋 PreorderWithTokens
//!           🌳
//!          ╱ ╲    
//!        🌿   📝   Traverses: Everything in tree
//!       ╱ ╲   📝   • nodes AND tokens
//!      📝 📝       • complete tree coverage
//! ```

use crate::{
    NodeOrToken,
    cursor::{node::SyntaxNode, syntax_element::SyntaxElement},
    utility_types::WalkEvent,
};

#[derive(Debug, Clone)]
pub struct PreorderWithTokens {
    start: SyntaxElement,
    next: Option<WalkEvent<SyntaxElement>>,
    skip_subtree: bool,
}

impl PreorderWithTokens {
    /// Creates a new preorder iterator starting from the given node.
    pub(super) fn new(start: SyntaxNode) -> PreorderWithTokens {
        let next = Some(WalkEvent::Enter(start.clone().into()));
        PreorderWithTokens {
            start: start.into(),
            next,
            skip_subtree: false,
        }
    }

    /// Skips the subtree of the current element in the traversal.
    pub fn skip_subtree(&mut self) {
        self.skip_subtree = true;
    }

    #[cold]
    fn do_skip(&mut self) {
        self.next = self.next.take().map(|next| match next {
            WalkEvent::Enter(first_child) => WalkEvent::Leave(first_child.parent().unwrap().into()),
            WalkEvent::Leave(parent) => WalkEvent::Leave(parent),
        })
    }
}

impl Iterator for PreorderWithTokens {
    type Item = WalkEvent<SyntaxElement>;

    fn next(&mut self) -> Option<WalkEvent<SyntaxElement>> {
        if self.skip_subtree {
            self.do_skip();
            self.skip_subtree = false;
        }
        let next = self.next.take();
        self.next = next.as_ref().and_then(|next| {
            Some(match next {
                WalkEvent::Enter(el) => match el {
                    NodeOrToken::Node(node) => match node.first_child_or_token() {
                        Some(child) => WalkEvent::Enter(child),
                        None => WalkEvent::Leave(node.clone().into()),
                    },
                    NodeOrToken::Token(token) => WalkEvent::Leave(token.clone().into()),
                },
                WalkEvent::Leave(el) if el == &self.start => return None,
                WalkEvent::Leave(el) => match el.next_sibling_or_token() {
                    Some(sibling) => WalkEvent::Enter(sibling),
                    None => WalkEvent::Leave(el.parent()?.into()),
                },
            })
        });
        next
    }
}
