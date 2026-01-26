use crate::{
    GreenCst, GreenDiagnostics, GreenDirectObjectOrIndirectReferenceExpressionSyntax, GreenElement, GreenExpressionSyntax, GreenListSyntax,
    GreenLiteralExpressionSyntax, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait, SyntaxKind,
};

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
    pub fn elements(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
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
    pub fn entries(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
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

impl GreenCst for GreenDictionaryExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::DictionaryExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenDictionaryExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
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
