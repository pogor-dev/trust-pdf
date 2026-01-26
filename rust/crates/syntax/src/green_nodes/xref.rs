use crate::{
    GreenCst, GreenDiagnostics, GreenElement, GreenExpressionSyntax, GreenListSyntax, GreenLiteralExpressionSyntax, GreenNode, GreenNodeSyntax, GreenToken,
    GreenTrait, SyntaxKind,
};

#[derive(Clone)]
pub struct GreenXRefTableExpressionSyntax(GreenExpressionSyntax);

impl GreenXRefTableExpressionSyntax {
    pub fn new(kind: SyntaxKind, sections: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(sections)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenXRefTableExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn sections(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }
}

impl GreenCst for GreenXRefTableExpressionSyntax {
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::XRefTableExpression && node.slot_count() == 1
    }

    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenXRefTableExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenXRefSectionSyntax(GreenExpressionSyntax);

impl GreenXRefSectionSyntax {
    pub fn new(kind: SyntaxKind, subsections: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(subsections)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenXRefSectionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn subsections(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenXRefSubSectionSyntax(GreenExpressionSyntax);

impl GreenXRefSubSectionSyntax {
    pub fn new(
        kind: SyntaxKind,
        start_object_number: GreenLiteralExpressionSyntax,
        entry_count: GreenLiteralExpressionSyntax,
        entries: GreenNode,
        diagnostics: Option<GreenDiagnostics>,
    ) -> Self {
        let slots = vec![
            GreenElement::Node(start_object_number.0.0.clone()),
            GreenElement::Node(entry_count.0.0.clone()),
            GreenElement::Node(entries),
        ];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenXRefSubSectionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn start_object_number(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn entry_count(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn entries(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(2) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct GreenXRefEntryExpressionSyntax(GreenExpressionSyntax);

impl GreenXRefEntryExpressionSyntax {
    pub fn new(
        kind: SyntaxKind,
        byte_offset: GreenLiteralExpressionSyntax,
        generation_number: GreenLiteralExpressionSyntax,
        in_use_token: GreenToken,
        diagnostics: Option<GreenDiagnostics>,
    ) -> Self {
        let slots = vec![
            GreenElement::Node(byte_offset.0.0.clone()),
            GreenElement::Node(generation_number.0.0.clone()),
            GreenElement::Token(in_use_token),
        ];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenXRefEntryExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn byte_offset(&self) -> Option<GreenLiteralExpressionSyntax> {
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
    pub fn in_use_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}
