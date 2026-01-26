use crate::{GreenCst, GreenDiagnostics, GreenElement, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait, SyntaxKind};

// TODO: lex the PDF version separately? Might be false positive inside the document
#[derive(Clone)]
pub struct GreenPdfDocumentSyntax {
    kind: SyntaxKind,
    bodies: GreenNode,
}

pub struct GreenPdfDocumentInnerSyntax {
    kind: SyntaxKind,
    objects: GreenNode,
    xref_table: GreenNode,
    trailer: GreenNode,
}

#[derive(Clone)]
pub struct GreenExpressionSyntax(GreenNode);

impl GreenNodeSyntax for GreenExpressionSyntax {
    #[inline]
    fn green(&self) -> &GreenNode {
        &self.0
    }
}

#[derive(Clone)]
pub struct GreenLiteralExpressionSyntax(GreenExpressionSyntax);

impl GreenLiteralExpressionSyntax {
    pub fn new(kind: SyntaxKind, token: GreenToken, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Token(token)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenLiteralExpressionSyntax(GreenExpressionSyntax(green))
    }

    pub fn token(&self) -> Option<GreenToken> {
        match self.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenNodeSyntax for GreenLiteralExpressionSyntax {
    #[inline]
    fn green(&self) -> &GreenNode {
        &self.0.0
    }
}

impl GreenCst for GreenLiteralExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        // Accept only literal expression node kinds with exactly one token child.
        let is_literal_kind = matches!(
            node.kind(),
            SyntaxKind::TrueLiteralExpression
                | SyntaxKind::FalseLiteralExpression
                | SyntaxKind::NullLiteralExpression
                | SyntaxKind::NumericLiteralExpression
                | SyntaxKind::NameLiteralExpression
                | SyntaxKind::StringLiteralExpression
                | SyntaxKind::HexStringLiteralExpression
        );

        if !is_literal_kind || node.slot_count() != 1 {
            return false;
        }

        match node.slot(0) {
            Some(GreenElement::Token(t)) => matches!(
                t.kind(),
                SyntaxKind::TrueKeyword
                    | SyntaxKind::FalseKeyword
                    | SyntaxKind::NullKeyword
                    | SyntaxKind::NumericLiteralToken
                    | SyntaxKind::NameLiteralToken
                    | SyntaxKind::StringLiteralToken
                    | SyntaxKind::HexStringLiteralToken
            ),
            _ => false,
        }
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenLiteralExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenArrayExpressionSyntax(GreenExpressionSyntax);

impl GreenArrayExpressionSyntax {
    pub fn new(
        kind: SyntaxKind,
        open_bracket_token: GreenToken, // TODO: Create GreenSyntaxToken to accept Missing node?
        elements: GreenNode,
        close_bracket_token: GreenToken,
        diagnostics: Option<GreenDiagnostics>,
    ) -> Self {
        let slots = vec![
            GreenElement::Token(open_bracket_token),
            GreenElement::Node(elements),
            GreenElement::Token(close_bracket_token),
        ];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenArrayExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn open_bracket_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn elements(&self) -> Option<GreenNode> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => Some(n),
            _ => None,
        }
    }

    #[inline]
    pub fn close_bracket_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenArrayElementExpressionSyntax(GreenExpressionSyntax);

impl GreenArrayElementExpressionSyntax {
    pub fn new(kind: SyntaxKind, value: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(value)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenArrayElementExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn value(&self) -> Option<GreenDirectObjectOrIndirectReferenceExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenDirectObjectOrIndirectReferenceExpressionSyntax::cast(n),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenDictionaryExpressionSyntax(GreenExpressionSyntax);

impl GreenDictionaryExpressionSyntax {
    pub fn new(
        kind: SyntaxKind,
        open_angle_token: GreenToken,
        entries: GreenNode,
        close_angle_token: GreenToken,
        diagnostics: Option<GreenDiagnostics>,
    ) -> Self {
        let slots = vec![
            GreenElement::Token(open_angle_token),
            GreenElement::Node(entries),
            GreenElement::Token(close_angle_token),
        ];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenDictionaryExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn open_angle_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn entries(&self) -> Option<GreenNode> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => Some(n),
            _ => None,
        }
    }

    #[inline]
    pub fn close_angle_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenDictionaryElementSyntax(GreenExpressionSyntax);

impl GreenDictionaryElementSyntax {
    pub fn new(kind: SyntaxKind, key: GreenNode, value: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(key), GreenElement::Node(value)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenDictionaryElementSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn key(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn value(&self) -> Option<GreenDirectObjectOrIndirectReferenceExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenDirectObjectOrIndirectReferenceExpressionSyntax::cast(n),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenDirectObjectExpressionSyntax(GreenExpressionSyntax);

impl GreenDirectObjectExpressionSyntax {
    pub fn new(kind: SyntaxKind, value: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(value)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenDirectObjectExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn value(&self) -> Option<GreenNode> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => Some(n),
            _ => None,
        }
    }
}

impl GreenCst for GreenDirectObjectExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::DirectObjectExpression && node.slot_count() == 1
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenDirectObjectExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenIndirectReferenceExpressionSyntax(GreenExpressionSyntax);

impl GreenIndirectReferenceExpressionSyntax {
    pub fn new(
        kind: SyntaxKind,
        object_number: GreenLiteralExpressionSyntax,
        generation_number: GreenLiteralExpressionSyntax,
        r_token: GreenToken,
        diagnostics: Option<GreenDiagnostics>,
    ) -> Self {
        let slots = vec![
            GreenElement::Node(object_number.0.0.clone()),
            GreenElement::Node(generation_number.0.0.clone()),
            GreenElement::Token(r_token),
        ];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenIndirectReferenceExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn object_number(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn generation_number(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn r_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for GreenIndirectReferenceExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::IndirectReferenceExpression && node.slot_count() == 3 // TODO: Validate slot kinds?
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenIndirectReferenceExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenDirectObjectOrIndirectReferenceExpressionSyntax(GreenExpressionSyntax);

impl GreenDirectObjectOrIndirectReferenceExpressionSyntax {
    pub fn new(kind: SyntaxKind, value: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(value)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenDirectObjectOrIndirectReferenceExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn direct_object(&self) -> Option<GreenDirectObjectExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenDirectObjectExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn indirect_reference(&self) -> Option<GreenIndirectReferenceExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenIndirectReferenceExpressionSyntax::cast(n),
            _ => None,
        }
    }
}

impl GreenCst for GreenDirectObjectOrIndirectReferenceExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        matches!(node.kind(), SyntaxKind::DirectObjectExpression | SyntaxKind::IndirectReferenceExpression) && node.slot_count() == 1
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenDirectObjectOrIndirectReferenceExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[cfg(test)]
mod cast_tests {
    use super::*;

    #[test]
    fn test_can_cast_literal_expression_true() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword, b"true", None, None, None);
        let node = GreenNode::new(SyntaxKind::TrueLiteralExpression, vec![GreenElement::Token(token)], None);
        assert!(GreenLiteralExpressionSyntax::can_cast(&node));
        assert!(GreenLiteralExpressionSyntax::cast(node).is_some());
    }

    #[test]
    fn test_can_cast_literal_expression_wrong_kind() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword, b"true", None, None, None);
        let node = GreenNode::new(SyntaxKind::List, vec![GreenElement::Token(token)], None);
        assert!(!GreenLiteralExpressionSyntax::can_cast(&node));
    }
}
