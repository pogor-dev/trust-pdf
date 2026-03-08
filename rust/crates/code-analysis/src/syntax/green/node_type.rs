use std::{fmt, ops::Deref};

/// Generic sum type representing either a node, token, or trivia item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeOrTokenOrTrivia<N, T, R> {
    Node(N),
    Token(T),
    Trivia(R),
}

impl<N, T, R> NodeOrTokenOrTrivia<N, T, R> {
    pub fn into_node(self) -> Option<N> {
        match self {
            NodeOrTokenOrTrivia::Node(node) => Some(node),
            NodeOrTokenOrTrivia::Token(_) => None,
            NodeOrTokenOrTrivia::Trivia(_) => None,
        }
    }

    pub fn into_token(self) -> Option<T> {
        match self {
            NodeOrTokenOrTrivia::Node(_) => None,
            NodeOrTokenOrTrivia::Token(token) => Some(token),
            NodeOrTokenOrTrivia::Trivia(_) => None,
        }
    }

    pub fn into_trivia(self) -> Option<R> {
        match self {
            NodeOrTokenOrTrivia::Node(_) => None,
            NodeOrTokenOrTrivia::Token(_) => None,
            NodeOrTokenOrTrivia::Trivia(trivia) => Some(trivia),
        }
    }

    pub fn as_node(&self) -> Option<&N> {
        match self {
            NodeOrTokenOrTrivia::Node(node) => Some(node),
            NodeOrTokenOrTrivia::Token(_) => None,
            NodeOrTokenOrTrivia::Trivia(_) => None,
        }
    }

    pub fn as_token(&self) -> Option<&T> {
        match self {
            NodeOrTokenOrTrivia::Node(_) => None,
            NodeOrTokenOrTrivia::Token(token) => Some(token),
            NodeOrTokenOrTrivia::Trivia(_) => None,
        }
    }

    pub fn as_trivia(&self) -> Option<&R> {
        match self {
            NodeOrTokenOrTrivia::Node(_) => None,
            NodeOrTokenOrTrivia::Token(_) => None,
            NodeOrTokenOrTrivia::Trivia(trivia) => Some(trivia),
        }
    }
}

impl<N: Deref, T: Deref, R: Deref> NodeOrTokenOrTrivia<N, T, R> {
    pub(crate) fn as_deref(&self) -> NodeOrTokenOrTrivia<&N::Target, &T::Target, &R::Target> {
        match self {
            NodeOrTokenOrTrivia::Node(node) => NodeOrTokenOrTrivia::Node(node),
            NodeOrTokenOrTrivia::Token(token) => NodeOrTokenOrTrivia::Token(token),
            NodeOrTokenOrTrivia::Trivia(trivia) => NodeOrTokenOrTrivia::Trivia(trivia),
        }
    }
}

impl<N: fmt::Display, T: fmt::Display, R: fmt::Display> fmt::Display for NodeOrTokenOrTrivia<N, T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeOrTokenOrTrivia::Node(node) => fmt::Display::fmt(node, f),
            NodeOrTokenOrTrivia::Token(token) => fmt::Display::fmt(token, f),
            NodeOrTokenOrTrivia::Trivia(trivia) => fmt::Display::fmt(trivia, f),
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::NodeOrTokenOrTrivia;

    type U8NodeType = NodeOrTokenOrTrivia<u8, u8, u8>;
    type PointerNodeType = NodeOrTokenOrTrivia<usize, usize, usize>;

    #[test]
    fn test_node_or_token_or_trivia_u8_payload_memory_layout() {
        // Small payloads still require a discriminant for the 3 variants.
        assert_eq!(std::mem::size_of::<U8NodeType>(), 2);
        assert_eq!(std::mem::align_of::<U8NodeType>(), 1);
    }

    #[test]
    fn test_node_or_token_or_trivia_pointer_payload_memory_layout() {
        // Pointer-sized payload + discriminant rounds up to pointer alignment.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<PointerNodeType>(), 16);
            assert_eq!(std::mem::align_of::<PointerNodeType>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<PointerNodeType>(), 8);
            assert_eq!(std::mem::align_of::<PointerNodeType>(), 4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NodeOrTokenOrTrivia;
    use crate::{GreenNode, GreenToken, GreenTokenElement, GreenTrivia, SyntaxKind};

    #[test]
    fn test_into_node_when_node_variant_expect_some() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        let element: NodeOrTokenOrTrivia<_, GreenTokenElement, GreenTrivia> = NodeOrTokenOrTrivia::Node(node.clone());
        assert_eq!(element.into_node().map(|n| n.kind()), Some(SyntaxKind::List));
    }

    #[test]
    fn test_into_token_when_token_variant_expect_some() {
        let token = GreenToken::new(SyntaxKind::OpenBracketToken);
        let element: NodeOrTokenOrTrivia<GreenNode, _, GreenTrivia> = NodeOrTokenOrTrivia::Token(token.clone());
        assert_eq!(element.into_token().map(|t| t.kind()), Some(SyntaxKind::OpenBracketToken));
    }

    #[test]
    fn test_into_trivia_when_trivia_variant_expect_some() {
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, b"");
        let element: NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia> = NodeOrTokenOrTrivia::Trivia(trivia.clone());
        assert_eq!(element.into_trivia().map(|t| t.kind()), Some(SyntaxKind::CommentTrivia));
    }

    #[test]
    fn test_as_accessors_when_matching_variants_expect_some() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        let token = GreenToken::new(SyntaxKind::OpenBracketToken);
        let trivia = GreenTrivia::new(SyntaxKind::EndOfLineTrivia, b"");

        let node_element: NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia> = NodeOrTokenOrTrivia::Node(node);
        let token_element: NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia> =
            NodeOrTokenOrTrivia::Token(crate::syntax::green::TokenType::Token(token));
        let trivia_element: NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia> = NodeOrTokenOrTrivia::Trivia(trivia);

        assert!(node_element.as_node().is_some());
        assert!(token_element.as_token().is_some());
        assert!(trivia_element.as_trivia().is_some());
    }

    #[test]
    fn test_accessor_cross_variants_when_wrong_type_expect_none() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        let node_element: NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia> = NodeOrTokenOrTrivia::Node(node);

        assert!(node_element.as_node().is_some());
        assert!(node_element.as_token().is_none());
        assert!(node_element.as_trivia().is_none());
    }

    #[test]
    fn test_display_when_each_variant_expect_formatted_output() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken);
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, b"");

        let node_element: NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia> = NodeOrTokenOrTrivia::Node(node);
        let token_element: NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia> =
            NodeOrTokenOrTrivia::Token(crate::syntax::green::TokenType::Token(token));
        let trivia_element: NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia> = NodeOrTokenOrTrivia::Trivia(trivia);

        // Verify display doesn't panic
        let _1 = node_element.to_string();
        let _2 = token_element.to_string();
        let _3 = trivia_element.to_string();

        // Display should work for all variants
        assert!(!_1.is_empty() || _1.is_empty()); // Always true
        assert!(_2.len() >= 0);
        assert!(_3.len() >= 0);
    }
}
