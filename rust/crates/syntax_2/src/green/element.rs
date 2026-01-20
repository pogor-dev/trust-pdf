use crate::{GreenNode, GreenNodeData, GreenToken, GreenTokenData, GreenTrivia, GreenTriviaData, NodeOrTokenOrTrivia, SyntaxKind};

pub(super) type GreenElement = NodeOrTokenOrTrivia<GreenNode, GreenToken, GreenTrivia>;
pub(crate) type GreenElementRef<'a> = NodeOrTokenOrTrivia<&'a GreenNodeData, &'a GreenTokenData, &'a GreenTriviaData>;

impl GreenElement {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenElement::Node(n) => n.kind(),
            GreenElement::Token(t) => t.kind(),
            GreenElement::Trivia(tr) => tr.kind(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenElement::Node(n) => n.width(),
            GreenElement::Token(t) => t.width(),
            GreenElement::Trivia(tr) => tr.width(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            GreenElement::Node(n) => n.full_width(),
            GreenElement::Token(t) => t.full_width(),
            GreenElement::Trivia(tr) => tr.width(),
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_element_memory_layout() {
        // GreenElement is an enum with 3 variants, each containing an Arc-based type
        // Size should be discriminant + largest variant (pointer size)
        assert!(std::mem::size_of::<GreenElement>() >= std::mem::size_of::<usize>());
    }
}

#[cfg(test)]
mod green_element_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn empty_trivia_list() -> Option<GreenNode> {
        Some(GreenNode::new(SyntaxKind::List, vec![], None))
    }

    #[test]
    fn test_kind_when_node_expect_node_kind() {
        let node = GreenNode::new(SyntaxKind::ArrayExpression, vec![], None);
        let element = GreenElement::Node(node);
        assert_eq!(element.kind(), SyntaxKind::ArrayExpression);
    }

    #[test]
    fn test_kind_when_token_expect_token_kind() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list(), None);
        let element = GreenElement::Token(token);
        assert_eq!(element.kind(), SyntaxKind::NumericLiteralToken);
    }

    #[test]
    fn test_kind_when_trivia_expect_trivia_kind() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let element = GreenElement::Trivia(trivia);
        assert_eq!(element.kind(), SyntaxKind::WhitespaceTrivia);
    }

    #[test]
    fn test_width_when_node_expect_node_width() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"123", empty_trivia_list(), empty_trivia_list(), None);
        let node = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token)], None);
        let element = GreenElement::Node(node);
        assert_eq!(element.width(), 3);
    }

    #[test]
    fn test_width_when_token_expect_token_width() {
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"456", empty_trivia_list(), empty_trivia_list(), None);
        let element = GreenElement::Token(token);
        assert_eq!(element.width(), 3);
    }

    #[test]
    fn test_width_when_trivia_expect_trivia_width() {
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"    ");
        let element = GreenElement::Trivia(trivia);
        assert_eq!(element.width(), 4);
    }

    #[test]
    fn test_full_width_when_node_expect_node_full_width() {
        let leading_trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let trailing_trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let leading = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(leading_trivia)], None);
        let trailing = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(trailing_trivia)], None);
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", Some(leading), Some(trailing), None);
        let node = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token)], None);
        let element = GreenElement::Node(node);
        assert_eq!(element.full_width(), 5); // 2 (leading) + 2 (token) + 1 (trailing)
    }

    #[test]
    fn test_full_width_when_token_expect_token_full_width() {
        let leading_trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trailing_trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"  ");
        let leading = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(leading_trivia)], None);
        let trailing = GreenNode::new(SyntaxKind::List, vec![GreenElement::Trivia(trailing_trivia)], None);
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"99", Some(leading), Some(trailing), None);
        let element = GreenElement::Token(token);
        assert_eq!(element.full_width(), 5); // 1 (leading) + 2 (token) + 2 (trailing)
    }

    #[test]
    fn test_full_width_when_trivia_expect_trivia_width() {
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, b"% comment");
        let element = GreenElement::Trivia(trivia);
        assert_eq!(element.full_width(), 9);
    }

    #[test]
    fn test_eq_when_same_node_expect_equal() {
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![], None);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![], None);
        let element1 = GreenElement::Node(node1);
        let element2 = GreenElement::Node(node2);
        assert_eq!(element1, element2);
    }

    #[test]
    fn test_eq_when_same_token_expect_equal() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list(), None);
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list(), None);
        let element1 = GreenElement::Token(token1);
        let element2 = GreenElement::Token(token2);
        assert_eq!(element1, element2);
    }

    #[test]
    fn test_eq_when_same_trivia_expect_equal() {
        let trivia1 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let trivia2 = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");
        let element1 = GreenElement::Trivia(trivia1);
        let element2 = GreenElement::Trivia(trivia2);
        assert_eq!(element1, element2);
    }

    #[test]
    fn test_eq_when_different_variants_expect_not_equal() {
        let node = GreenNode::new(SyntaxKind::ArrayExpression, vec![], None);
        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list(), None);
        let element1 = GreenElement::Node(node);
        let element2 = GreenElement::Token(token);
        assert_ne!(element1, element2);
    }

    #[test]
    fn test_eq_when_different_node_content_expect_not_equal() {
        let token1 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"1", empty_trivia_list(), empty_trivia_list(), None);
        let token2 = GreenToken::new(SyntaxKind::NumericLiteralToken, b"2", empty_trivia_list(), empty_trivia_list(), None);
        let node1 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token1)], None);
        let node2 = GreenNode::new(SyntaxKind::ArrayExpression, vec![GreenElement::Token(token2)], None);
        let element1 = GreenElement::Node(node1);
        let element2 = GreenElement::Node(node2);
        assert_ne!(element1, element2);
    }

    #[test]
    fn test_clone_when_node_expect_cloned() {
        let node = GreenNode::new(SyntaxKind::DictionaryExpression, vec![], None);
        let element1 = GreenElement::Node(node);
        let element2 = element1.clone();
        assert_eq!(element1, element2);
        assert_eq!(element2.kind(), SyntaxKind::DictionaryExpression);
    }

    #[test]
    fn test_clone_when_token_expect_cloned() {
        let token = GreenToken::new(SyntaxKind::StringLiteralToken, b"test", empty_trivia_list(), empty_trivia_list(), None);
        let element1 = GreenElement::Token(token);
        let element2 = element1.clone();
        assert_eq!(element1, element2);
        assert_eq!(element2.kind(), SyntaxKind::StringLiteralToken);
        assert_eq!(element2.width(), 4);
    }

    #[test]
    fn test_clone_when_trivia_expect_cloned() {
        let trivia = GreenTrivia::new(SyntaxKind::CommentTrivia, b"% test");
        let element1 = GreenElement::Trivia(trivia);
        let element2 = element1.clone();
        assert_eq!(element1, element2);
        assert_eq!(element2.kind(), SyntaxKind::CommentTrivia);
        assert_eq!(element2.width(), 6);
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashSet;

        let token = GreenToken::new(SyntaxKind::NumericLiteralToken, b"42", empty_trivia_list(), empty_trivia_list(), None);
        let element1 = GreenElement::Token(token.clone());
        let element2 = GreenElement::Token(token);

        let mut set = HashSet::new();
        set.insert(element1.clone());

        // Same element should be found in set
        assert!(set.contains(&element2));
    }
}
