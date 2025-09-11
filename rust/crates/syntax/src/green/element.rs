use std::borrow::Cow;

use crate::{
    GreenToken, NodeOrToken, SyntaxKind,
    green::{
        GreenNode,
        node::{GreenNodeData, Slot},
        token::GreenTokenData,
    },
};

pub(crate) type GreenElement = NodeOrToken<GreenNode, GreenToken>;
pub(crate) type GreenElementRef<'a> = NodeOrToken<&'a GreenNodeData, &'a GreenTokenData>;

impl GreenElement {
    /// Returns kind of this element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.as_deref().kind()
    }

    /// Returns the length of the text covered by this element.
    #[inline]
    pub fn text_len(&self) -> u32 {
        self.as_deref().text_len()
    }

    #[inline]
    pub fn full_text_len(&self) -> u32 {
        self.as_deref().full_text_len()
    }
}

impl From<GreenNode> for GreenElement {
    #[inline]
    fn from(node: GreenNode) -> GreenElement {
        NodeOrToken::Node(node)
    }
}

impl From<GreenToken> for GreenElement {
    #[inline]
    fn from(token: GreenToken) -> GreenElement {
        NodeOrToken::Token(token)
    }
}

impl From<&Slot> for GreenElement {
    #[inline]
    fn from(slot: &Slot) -> Self {
        match slot {
            Slot::Node { node, .. } => NodeOrToken::Node(node.to_owned()),
            Slot::Token { token, .. } => NodeOrToken::Token(token.to_owned()),
        }
    }
}

impl From<Cow<'_, GreenNodeData>> for GreenElement {
    #[inline]
    fn from(cow: Cow<'_, GreenNodeData>) -> Self {
        NodeOrToken::Node(cow.into_owned())
    }
}

impl GreenElementRef<'_> {
    /// Returns kind of this element.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    /// Returns the length of the text covered by this element.
    #[inline]
    pub fn text_len(self) -> u32 {
        match self {
            NodeOrToken::Node(it) => it.text_len(),
            NodeOrToken::Token(it) => it.text_len(),
        }
    }

    #[inline]
    pub fn full_text_len(self) -> u32 {
        match self {
            NodeOrToken::Node(it) => it.full_text_len(),
            NodeOrToken::Token(it) => it.full_text_len(),
        }
    }
}

impl<'a> From<&'a GreenNode> for GreenElementRef<'a> {
    #[inline]
    fn from(node: &'a GreenNode) -> GreenElementRef<'a> {
        NodeOrToken::Node(node)
    }
}

impl<'a> From<&'a GreenToken> for GreenElementRef<'a> {
    #[inline]
    fn from(token: &'a GreenToken) -> GreenElementRef<'a> {
        NodeOrToken::Token(token)
    }
}

impl GreenElementRef<'_> {
    pub fn to_owned(self) -> GreenElement {
        match self {
            NodeOrToken::Node(it) => NodeOrToken::Node(it.to_owned()),
            NodeOrToken::Token(it) => NodeOrToken::Token(it.to_owned()),
        }
    }
}

#[cfg(test)]
mod element_tests {
    use rstest::rstest;
    use std::borrow::Borrow;

    use super::*;
    use crate::GreenTrivia;

    fn create_whitespace_trivia() -> GreenTrivia {
        GreenTrivia::new_single(SyntaxKind(0), b" ")
    }

    fn create_eol_trivia() -> GreenTrivia {
        GreenTrivia::new_single(SyntaxKind(1), b"\n")
    }

    #[rstest]
    fn test_kind() {
        let token = GreenToken::new(SyntaxKind(2), b"foo");
        let node = GreenNode::new_list(SyntaxKind(3), vec![token.clone().into()]);
        let element_token: GreenElement = token.into();
        let element_node: GreenElement = node.into();

        assert_eq!(element_token.kind(), SyntaxKind(2));
        assert_eq!(element_node.kind(), SyntaxKind(3));
    }

    #[rstest]
    fn test_text_len() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![token1.clone().into(), token2.clone().into()]);
        let element_token: GreenElement = token1.into();
        let element_node: GreenElement = node.into();

        assert_eq!(element_token.text_len(), 5);
        assert_eq!(element_node.text_len(), 12); // both tokens without first token's leading trivia and last token's trailing trivia
    }

    #[rstest]
    fn test_full_text_len() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![token1.clone().into(), token2.clone().into()]);
        let element_token: GreenElement = token1.into();
        let element_node: GreenElement = node.into();

        assert_eq!(element_token.full_text_len(), 7);
        assert_eq!(element_node.full_text_len(), 14);
    }

    #[rstest]
    fn test_from_slot() {
        let token = GreenToken::new(SyntaxKind(2), b"foo");
        let node = GreenNode::new_list(SyntaxKind(3), vec![token.clone().into()]);
        let slot_token = Slot::Token { rel_offset: 0, token: token };
        let slot_node = Slot::Node { rel_offset: 0, node: node };
        let element_from_slot_token: GreenElement = (&slot_token).into();
        let element_from_slot_node: GreenElement = (&slot_node).into();

        assert_eq!(element_from_slot_token.kind(), SyntaxKind(2));
        assert_eq!(element_from_slot_node.kind(), SyntaxKind(3));
    }

    #[rstest]
    fn test_from_cow() {
        let token = GreenToken::new(SyntaxKind(2), b"foo");
        let node = GreenNode::new_list(SyntaxKind(3), vec![token.clone().into()]);
        let borrowed: &GreenNodeData = node.borrow();
        let cow = Cow::Borrowed(borrowed);
        let element_from_cow: GreenElement = cow.into();

        assert_eq!(element_from_cow.kind(), SyntaxKind(3));
    }
}

#[cfg(test)]
mod element_ref_tests {
    use crate::GreenTrivia;
    use rstest::rstest;

    use super::*;

    fn create_whitespace_trivia() -> GreenTrivia {
        GreenTrivia::new_single(SyntaxKind(0), b" ")
    }

    fn create_eol_trivia() -> GreenTrivia {
        GreenTrivia::new_single(SyntaxKind(1), b"\n")
    }

    #[rstest]
    fn test_kind() {
        let token = GreenToken::new(SyntaxKind(2), b"foo");
        let node = GreenNode::new_list(SyntaxKind(3), vec![token.clone().into()]);
        let element_ref_token: GreenElementRef = (&token).into();
        let element_ref_node: GreenElementRef = (&node).into();

        assert_eq!(element_ref_token.kind(), SyntaxKind(2));
        assert_eq!(element_ref_node.kind(), SyntaxKind(3));
    }

    #[rstest]
    fn test_text_len() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![token1.clone().into(), token2.clone().into()]);
        let element_ref_token: GreenElementRef = (&token1).into();
        let element_ref_node: GreenElementRef = (&node).into();

        assert_eq!(element_ref_token.text_len(), 5);
        assert_eq!(element_ref_node.text_len(), 12); // both tokens without first token's leading trivia and last token's trailing trivia
    }

    #[rstest]
    fn test_full_text_len() {
        let token1 = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let token2 = GreenToken::new_with_trivia(SyntaxKind(2), b"token", create_whitespace_trivia(), create_eol_trivia());
        let node = GreenNode::new_list(SyntaxKind(3), vec![token1.clone().into(), token2.clone().into()]);
        let element_ref_token: GreenElementRef = (&token1).into();
        let element_ref_node: GreenElementRef = (&node).into();

        assert_eq!(element_ref_token.full_text_len(), 7);
        assert_eq!(element_ref_node.full_text_len(), 14);
    }

    #[rstest]
    fn test_to_owned() {
        let token = GreenToken::new(SyntaxKind(2), b"foo");
        let node = GreenNode::new_list(SyntaxKind(3), vec![token.clone().into()]);
        let element_ref_token: GreenElementRef = (&token).into();
        let element_ref_node: GreenElementRef = (&node).into();

        let owned_token = element_ref_token.to_owned();
        let owned_node = element_ref_node.to_owned();

        assert_eq!(owned_token.kind(), SyntaxKind(2));
        assert_eq!(owned_node.kind(), SyntaxKind(3));
    }

    #[rstest]
    fn test_from_green_node_ref() {
        let token = GreenToken::new(SyntaxKind(2), b"foo");
        let node = GreenNode::new_list(SyntaxKind(3), vec![token.clone().into()]);
        let element_ref: GreenElementRef = (&node).into();

        assert_eq!(element_ref.kind(), SyntaxKind(3));
    }

    #[rstest]
    fn test_from_green_token_ref() {
        let token = GreenToken::new(SyntaxKind(2), b"foo");
        let element_ref: GreenElementRef = (&token).into();

        assert_eq!(element_ref.kind(), SyntaxKind(2));
    }
}
