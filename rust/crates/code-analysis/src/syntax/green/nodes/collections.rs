use crate::{
    GreenCst, GreenDiagnostic, GreenDirectObjectOrIndirectReferenceExpressionSyntax, GreenExpressionSyntax, GreenListSyntax, GreenLiteralExpressionSyntax,
    GreenNode, GreenNodeElement, GreenNodeSyntax, GreenTokenElement, SyntaxKind,
};

/// Array object: [ element1 element2 ... ]
/// ISO 32000-2:2020, 7.3.6 — Arrays
#[derive(Clone)]
pub(crate) struct GreenArrayExpressionSyntax(GreenExpressionSyntax);

impl GreenArrayExpressionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        open_bracket_token: GreenNodeElement,
        elements: GreenNodeElement,
        close_bracket_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![open_bracket_token, elements, close_bracket_token];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenArrayExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn open_bracket_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn elements(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn close_bracket_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }
}

impl GreenCst for GreenArrayExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::ArrayExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenArrayExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// Element within an array
/// ISO 32000-2:2020, 7.3.6 — Arrays
#[derive(Clone)]
pub(crate) struct GreenArrayElementExpressionSyntax(GreenExpressionSyntax);

impl GreenArrayElementExpressionSyntax {
    pub(crate) fn new(kind: SyntaxKind, value: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![value];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenArrayElementExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn value(&self) -> Option<GreenDirectObjectOrIndirectReferenceExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenDirectObjectOrIndirectReferenceExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }
}

impl GreenCst for GreenArrayElementExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::ArrayElementExpression && node.slot_count() == 1
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenArrayElementExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// Dictionary object: << key1 value1 key2 value2 ... >>
/// ISO 32000-2:2020, 7.3.7 — Dictionaries
#[derive(Clone)]
pub(crate) struct GreenDictionaryExpressionSyntax(GreenExpressionSyntax);

impl GreenDictionaryExpressionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        open_angle_token: GreenNodeElement,
        entries: GreenNodeElement,
        close_angle_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![open_angle_token, entries, close_angle_token];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenDictionaryExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn open_angle_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn entries(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn close_angle_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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

/// Key-value entry within a dictionary
/// ISO 32000-2:2020, 7.3.7 — Dictionaries
#[derive(Clone)]
pub(crate) struct GreenDictionaryElementSyntax(GreenExpressionSyntax);

impl GreenDictionaryElementSyntax {
    pub(crate) fn new(kind: SyntaxKind, key: GreenNodeElement, value: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![key, value];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenDictionaryElementSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn key(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn value(&self) -> Option<GreenDirectObjectOrIndirectReferenceExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenDirectObjectOrIndirectReferenceExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }
}

impl GreenCst for GreenDictionaryElementSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::DictionaryElementExpression && node.slot_count() == 2
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenDictionaryElementSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
