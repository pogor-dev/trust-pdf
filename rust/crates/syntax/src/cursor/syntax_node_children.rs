use crate::{SyntaxKind, cursor::node::SyntaxNode};

#[derive(Clone, Debug)]
pub struct SyntaxNodeChildren {
    parent: SyntaxNode,
    next: Option<SyntaxNode>,
    next_initialized: bool,
}

impl SyntaxNodeChildren {
    pub(super) fn new(parent: SyntaxNode) -> SyntaxNodeChildren {
        SyntaxNodeChildren {
            parent,
            next: None,
            next_initialized: false,
        }
    }

    pub fn by_kind<F: Fn(SyntaxKind) -> bool>(self, matcher: F) -> SyntaxNodeChildrenByKind<F> {
        if !self.next_initialized {
            SyntaxNodeChildrenByKind {
                next: self.parent.first_child_by_kind(&matcher),
                matcher,
            }
        } else {
            SyntaxNodeChildrenByKind {
                next: self.next.and_then(|node| {
                    if matcher(node.kind()) {
                        Some(node)
                    } else {
                        node.next_sibling_by_kind(&matcher)
                    }
                }),
                matcher,
            }
        }
    }
}

impl Iterator for SyntaxNodeChildren {
    type Item = SyntaxNode;
    fn next(&mut self) -> Option<SyntaxNode> {
        if !self.next_initialized {
            self.next = self.parent.first_child();
            self.next_initialized = true;
        } else {
            self.next = self.next.take().and_then(|next| next.to_next_sibling());
        }

        self.next.clone()
    }
}

#[derive(Clone, Debug)]
pub struct SyntaxNodeChildrenByKind<F: Fn(SyntaxKind) -> bool> {
    next: Option<SyntaxNode>,
    matcher: F,
}

impl<F: Fn(SyntaxKind) -> bool> Iterator for SyntaxNodeChildrenByKind<F> {
    type Item = SyntaxNode;
    fn next(&mut self) -> Option<SyntaxNode> {
        self.next.take().inspect(|next| {
            self.next = next.next_sibling_by_kind(&self.matcher);
        })
    }
}
