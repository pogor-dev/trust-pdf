use crate::{
    GreenCst, GreenDiagnostic, GreenExpressionSyntax, GreenLiteralExpressionSyntax, GreenNode, GreenNodeElement, GreenNodeSyntax, GreenStreamExpressionSyntax,
    GreenTokenElement, SyntaxKind,
};

/// Direct object: a value that is not an indirect reference
/// ISO 32000-2:2020, 7.3.10 — Indirect objects
#[derive(Clone)]
pub(crate) struct GreenDirectObjectExpressionSyntax(GreenExpressionSyntax);

impl GreenDirectObjectExpressionSyntax {
    pub(crate) fn new(kind: SyntaxKind, value: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![value.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenDirectObjectExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn value(&self) -> Option<GreenNode> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => Some(n.clone()),
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

/// Indirect reference: objNumber genNumber R
/// ISO 32000-2:2020, 7.3.10 — Indirect objects
#[derive(Clone)]
pub(crate) struct GreenIndirectReferenceExpressionSyntax(GreenExpressionSyntax);

impl GreenIndirectReferenceExpressionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        object_number: GreenLiteralExpressionSyntax,
        generation_number: GreenLiteralExpressionSyntax,
        r_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![object_number.0.0.clone().into(), generation_number.0.0.clone().into(), r_token.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenIndirectReferenceExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn object_number(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn generation_number(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn r_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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

/// Either a direct object or an indirect reference
/// ISO 32000-2:2020, 7.3.10 — Indirect objects
#[derive(Clone)]
pub(crate) struct GreenDirectObjectOrIndirectReferenceExpressionSyntax(GreenExpressionSyntax);

impl GreenDirectObjectOrIndirectReferenceExpressionSyntax {
    pub(crate) fn new(kind: SyntaxKind, value: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![value];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenDirectObjectOrIndirectReferenceExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn direct_object(&self) -> Option<GreenDirectObjectExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenDirectObjectExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn indirect_reference(&self) -> Option<GreenIndirectReferenceExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenIndirectReferenceExpressionSyntax::cast(n.clone()),
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

/// Indirect object: objNumber genNumber obj ... endobj
/// ISO 32000-2:2020, 7.3.10 — Indirect objects
#[derive(Clone)]
pub(crate) struct IndirectObjectExpressionSyntax(GreenExpressionSyntax);

impl IndirectObjectExpressionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        header: GreenIndirectObjectHeaderExpressionSyntax,
        body: GreenIndirectBodyExpressionSyntax,
        endobj_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![header.0.0.clone().into(), body.0.0.clone().into(), endobj_token.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        IndirectObjectExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn header(&self) -> Option<GreenIndirectObjectHeaderExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenIndirectObjectHeaderExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn body(&self) -> Option<GreenIndirectBodyExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenIndirectBodyExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn endobj_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }
}

impl GreenCst for IndirectObjectExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::IndirectObjectExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(IndirectObjectExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[derive(Clone)]
/// Content of an indirect object: either a direct object or stream
/// ISO 32000-2:2020, 7.3.10 — Indirect objects
pub(crate) struct GreenIndirectBodyExpressionSyntax(GreenExpressionSyntax);

impl GreenIndirectBodyExpressionSyntax {
    pub(crate) fn new(kind: SyntaxKind, value: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![value.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenIndirectBodyExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn direct_object(&self) -> Option<GreenDirectObjectExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenDirectObjectExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn stream_expression(&self) -> Option<GreenStreamExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenStreamExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }
}

impl GreenCst for GreenIndirectBodyExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        matches!(node.kind(), SyntaxKind::DirectObjectExpression | SyntaxKind::StreamExpression) && node.slot_count() == 1
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenIndirectBodyExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// Indirect object header: objNumber genNumber obj
/// ISO 32000-2:2020, 7.3.10 — Indirect objects
#[derive(Clone)]
pub(crate) struct GreenIndirectObjectHeaderExpressionSyntax(GreenExpressionSyntax);

impl GreenIndirectObjectHeaderExpressionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        object_number: GreenLiteralExpressionSyntax,
        generation_number: GreenLiteralExpressionSyntax,
        obj_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![object_number.0.0.clone().into(), generation_number.0.0.clone().into(), obj_token.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenIndirectObjectHeaderExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn object_number(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn generation_number(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn obj_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }
}

impl GreenCst for GreenIndirectObjectHeaderExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::IndirectObjectHeaderExpression && node.slot_count() == 3 // TODO: Validate slot kinds?
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenIndirectObjectHeaderExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
