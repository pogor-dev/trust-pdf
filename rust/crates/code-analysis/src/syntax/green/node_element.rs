use crate::{
    GreenDiagnostic, GreenNode, GreenNodeData, GreenToken, GreenTokenElement, GreenTokenElementRef, GreenTokenWithFloatValue,
    GreenTokenWithFloatValueAndTrivia, GreenTokenWithIntValue, GreenTokenWithIntValueAndTrivia, GreenTokenWithStringValue, GreenTokenWithStringValueAndTrivia,
    GreenTrivia, GreenTriviaData, SyntaxKind, syntax::green::NodeOrTokenOrTrivia,
};

/// Concrete green tree child element used in node slot arrays.
pub(crate) type GreenNodeElement = NodeOrTokenOrTrivia<GreenNode, GreenTokenElement, GreenTrivia>;
pub(crate) type GreenNodeElementRef<'a> = NodeOrTokenOrTrivia<&'a GreenNodeData, GreenTokenElementRef<'a>, &'a GreenTriviaData>;

impl GreenNodeElement {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        match self {
            GreenNodeElement::Node(n) => n.kind(),
            GreenNodeElement::Token(t) => t.kind(),
            GreenNodeElement::Trivia(tr) => tr.kind(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        match self {
            GreenNodeElement::Node(n) => n.width(),
            GreenNodeElement::Token(t) => t.width(),
            GreenNodeElement::Trivia(tr) => tr.width().into(),
        }
    }

    #[inline]
    pub fn text(&self) -> Vec<u8> {
        match self {
            GreenNodeElement::Node(n) => n.text(),
            GreenNodeElement::Token(t) => t.text(),
            GreenNodeElement::Trivia(tr) => tr.text().to_vec(),
        }
    }

    #[inline]
    pub fn full_width(&self) -> u32 {
        match self {
            GreenNodeElement::Node(n) => n.full_width(),
            GreenNodeElement::Token(t) => t.full_width(),
            GreenNodeElement::Trivia(tr) => tr.width().into(),
        }
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        match self {
            GreenNodeElement::Node(n) => n.full_text(),
            GreenNodeElement::Token(t) => t.full_text(),
            GreenNodeElement::Trivia(tr) => tr.text().to_vec(),
        }
    }

    #[inline]
    pub fn leading_trivia_width(&self) -> u32 {
        match self {
            GreenNodeElement::Node(n) => n.leading_trivia_width(),
            GreenNodeElement::Token(t) => t.leading_trivia_width(),
            GreenNodeElement::Trivia(_) => 0,
        }
    }

    #[inline]
    pub fn trailing_trivia_width(&self) -> u32 {
        match self {
            GreenNodeElement::Node(n) => n.trailing_trivia_width(),
            GreenNodeElement::Token(t) => t.trailing_trivia_width(),
            GreenNodeElement::Trivia(_) => 0,
        }
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenNodeElement::Node(n) => n.leading_trivia(),
            GreenNodeElement::Token(t) => t.leading_trivia(),
            GreenNodeElement::Trivia(_) => None,
        }
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        match self {
            GreenNodeElement::Node(n) => n.trailing_trivia(),
            GreenNodeElement::Token(t) => t.trailing_trivia(),
            GreenNodeElement::Trivia(_) => None,
        }
    }

    #[inline]
    pub fn contains_diagnostics(&self) -> bool {
        match self {
            GreenNodeElement::Node(n) => n.contains_diagnostics(),
            GreenNodeElement::Token(t) => t.contains_diagnostics(),
            GreenNodeElement::Trivia(tr) => tr.contains_diagnostics(),
        }
    }

    #[inline]
    pub fn diagnostics(&self) -> Option<Vec<GreenDiagnostic>> {
        match self {
            GreenNodeElement::Node(n) => n.diagnostics(),
            GreenNodeElement::Token(t) => t.diagnostics(),
            GreenNodeElement::Trivia(tr) => tr.diagnostics(),
        }
    }

    #[inline]
    pub fn is_token(&self) -> bool {
        matches!(self, GreenNodeElement::Token(_))
    }

    #[inline]
    pub fn is_trivia(&self) -> bool {
        matches!(self, GreenNodeElement::Trivia(_))
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        self.kind() == SyntaxKind::List
    }

    #[inline]
    pub fn is_missing(&self) -> bool {
        match self {
            GreenNodeElement::Node(n) => n.is_missing(),
            GreenNodeElement::Token(t) => t.is_missing(),
            GreenNodeElement::Trivia(tr) => tr.is_missing(),
        }
    }
}

impl From<GreenToken> for GreenNodeElement {
    #[inline]
    fn from(token: GreenToken) -> GreenNodeElement {
        let token_element: GreenTokenElement = token.into();
        token_element.into()
    }
}

impl From<GreenTokenWithIntValue> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenWithIntValue) -> GreenNodeElement {
        let token_element: GreenTokenElement = token.into();
        token_element.into()
    }
}

impl From<GreenTokenWithFloatValue> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenWithFloatValue) -> GreenNodeElement {
        let token_element: GreenTokenElement = token.into();
        token_element.into()
    }
}

impl From<GreenTokenWithStringValue> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenWithStringValue) -> GreenNodeElement {
        let token_element: GreenTokenElement = token.into();
        token_element.into()
    }
}

impl From<GreenTokenWithIntValueAndTrivia> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenWithIntValueAndTrivia) -> GreenNodeElement {
        let token_element: GreenTokenElement = token.into();
        token_element.into()
    }
}

impl From<GreenTokenWithFloatValueAndTrivia> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenWithFloatValueAndTrivia) -> GreenNodeElement {
        let token_element: GreenTokenElement = token.into();
        token_element.into()
    }
}

impl From<GreenTokenWithStringValueAndTrivia> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenWithStringValueAndTrivia) -> GreenNodeElement {
        let token_element: GreenTokenElement = token.into();
        token_element.into()
    }
}

impl From<GreenNode> for GreenNodeElement {
    #[inline]
    fn from(node: GreenNode) -> GreenNodeElement {
        GreenNodeElement::Node(node)
    }
}

impl From<GreenTrivia> for GreenNodeElement {
    #[inline]
    fn from(trivia: GreenTrivia) -> GreenNodeElement {
        GreenNodeElement::Trivia(trivia)
    }
}

impl From<GreenTokenElement> for GreenNodeElement {
    #[inline]
    fn from(token: GreenTokenElement) -> GreenNodeElement {
        GreenNodeElement::Token(token)
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_node_element_memory_layout() {
        // GreenNodeElement is an enum over Arc-backed payloads and should be
        // pointer-sized payload + enum discriminant.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenNodeElement>(), 16);
            assert_eq!(std::mem::align_of::<GreenNodeElement>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenNodeElement>(), 8);
            assert_eq!(std::mem::align_of::<GreenNodeElement>(), 4);
        }
    }

    #[test]
    fn test_green_node_element_ref_memory_layout() {
        // GreenNodeElementRef stores references/borrows to green data payloads.
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenNodeElementRef<'_>>(), 16);
            assert_eq!(std::mem::align_of::<GreenNodeElementRef<'_>>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenNodeElementRef<'_>>(), 8);
            assert_eq!(std::mem::align_of::<GreenNodeElementRef<'_>>(), 4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_when_node_token_trivia_variants_expect_inner_kinds() {
        let node = GreenNode::new(SyntaxKind::List, vec![]);
        let token = GreenToken::new(SyntaxKind::TrueKeyword);
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ");

        assert_eq!(GreenNodeElement::Node(node).kind(), SyntaxKind::List);
        assert_eq!(GreenNodeElement::Token(GreenTokenElement::Token(token)).kind(), SyntaxKind::TrueKeyword);
        assert_eq!(GreenNodeElement::Trivia(trivia).kind(), SyntaxKind::WhitespaceTrivia);
    }

    #[test]
    fn test_width_and_full_width_when_mixed_variants_expect_expected_values() {
        let node = GreenNode::new(SyntaxKind::List, vec![GreenToken::new(SyntaxKind::TrueKeyword).into()]);
        let token = GreenToken::new(SyntaxKind::FalseKeyword);
        let trivia = GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b"\n");

        let node_element = GreenNodeElement::Node(node);
        let token_element = GreenNodeElement::Token(GreenTokenElement::Token(token));
        let trivia_element = GreenNodeElement::Trivia(trivia);

        assert_eq!(node_element.width(), node_element.full_width());
        assert_eq!(token_element.width(), token_element.full_width());
        assert_eq!(trivia_element.width(), trivia_element.full_width());
    }

    #[test]
    fn test_from_when_token_and_trivia_types_expect_matching_variant_construction() {
        let plain = GreenNodeElement::from(GreenToken::new(SyntaxKind::NullKeyword));
        let int_value = GreenNodeElement::from(GreenTokenWithIntValue::new(SyntaxKind::NumericLiteralToken, b"42", 42));
        let float_value = GreenNodeElement::from(GreenTokenWithFloatValue::new(SyntaxKind::NumericLiteralToken, b"3.14", 3.14));
        let string_value = GreenNodeElement::from(GreenTokenWithStringValue::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string()));
        let int_value_trivia = GreenNodeElement::from(GreenTokenWithIntValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 42, None, None));
        let float_value_trivia = GreenNodeElement::from(GreenTokenWithFloatValueAndTrivia::new(
            SyntaxKind::NumericLiteralToken,
            b"3.14",
            3.14,
            None,
            None,
        ));
        let string_value_trivia = GreenNodeElement::from(GreenTokenWithStringValueAndTrivia::new(
            SyntaxKind::NameLiteralToken,
            b"Type",
            "Type".to_string(),
            None,
            None,
        ));
        let trivia = GreenNodeElement::from(GreenTrivia::new(SyntaxKind::CommentTrivia, b"%x"));

        assert!(matches!(plain, GreenNodeElement::Token(GreenTokenElement::Token(_))));
        assert!(matches!(int_value, GreenNodeElement::Token(GreenTokenElement::TokenWithIntValue(_))));
        assert!(matches!(float_value, GreenNodeElement::Token(GreenTokenElement::TokenWithFloatValue(_))));
        assert!(matches!(string_value, GreenNodeElement::Token(GreenTokenElement::TokenWithStringValue(_))));
        assert!(matches!(
            int_value_trivia,
            GreenNodeElement::Token(GreenTokenElement::TokenWithIntValueAndTrivia(_))
        ));
        assert!(matches!(
            float_value_trivia,
            GreenNodeElement::Token(GreenTokenElement::TokenWithFloatValueAndTrivia(_))
        ));
        assert!(matches!(
            string_value_trivia,
            GreenNodeElement::Token(GreenTokenElement::TokenWithStringValueAndTrivia(_))
        ));
        assert!(matches!(trivia, GreenNodeElement::Trivia(_)));
    }

    #[test]
    fn test_from_when_green_token_element_expect_token_variant_preserved() {
        let token_element = GreenTokenElement::Token(GreenToken::new(SyntaxKind::TrueKeyword));
        let node_element: GreenNodeElement = token_element.into();
        assert!(matches!(node_element, GreenNodeElement::Token(GreenTokenElement::Token(_))));
    }
}
