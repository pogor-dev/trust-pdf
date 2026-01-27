use crate::{
    GreenCst, GreenDiagnostics, GreenElement, GreenExpressionSyntax, GreenLiteralExpressionSyntax, GreenNode, GreenNodeSyntax, GreenStreamExpressionSyntax,
    GreenToken, GreenTrait, SyntaxKind,
};

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

#[derive(Clone)]
pub struct IndirectObjectExpressionSyntax(GreenExpressionSyntax);

impl IndirectObjectExpressionSyntax {
    pub fn new(
        kind: SyntaxKind,
        header: GreenIndirectObjectHeaderExpressionSyntax,
        body: GreenIndirectBodyExpressionSyntax,
        endobj_token: GreenToken,
        diagnostics: Option<GreenDiagnostics>,
    ) -> Self {
        let slots = vec![
            GreenElement::Node(header.0.0.clone()),
            GreenElement::Node(body.0.0.clone()),
            GreenElement::Token(endobj_token),
        ];
        let green = GreenNode::new(kind, slots, diagnostics);
        IndirectObjectExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn header(&self) -> Option<GreenIndirectObjectHeaderExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenIndirectObjectHeaderExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn body(&self) -> Option<GreenIndirectBodyExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenIndirectBodyExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn endobj_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for IndirectObjectExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::IndirectObjectDefinition && node.slot_count() == 3
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
pub struct GreenIndirectBodyExpressionSyntax(GreenExpressionSyntax);

impl GreenIndirectBodyExpressionSyntax {
    pub fn new(kind: SyntaxKind, value: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(value)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenIndirectBodyExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn direct_object(&self) -> Option<GreenDirectObjectExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenDirectObjectExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn stream_expression(&self) -> Option<GreenStreamExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenStreamExpressionSyntax::cast(n),
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

#[derive(Clone)]
pub struct GreenIndirectObjectHeaderExpressionSyntax(GreenExpressionSyntax);

impl GreenIndirectObjectHeaderExpressionSyntax {
    pub fn new(
        kind: SyntaxKind,
        object_number: GreenLiteralExpressionSyntax,
        generation_number: GreenLiteralExpressionSyntax,
        obj_token: GreenToken,
        diagnostics: Option<GreenDiagnostics>,
    ) -> Self {
        let slots = vec![
            GreenElement::Node(object_number.0.0.clone()),
            GreenElement::Node(generation_number.0.0.clone()),
            GreenElement::Token(obj_token),
        ];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenIndirectObjectHeaderExpressionSyntax(GreenExpressionSyntax(green))
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
    pub fn obj_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for GreenIndirectObjectHeaderExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::IndirectObjectDefinition && node.slot_count() == 3 // TODO: Validate slot kinds?
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenIndirectObjectHeaderExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
