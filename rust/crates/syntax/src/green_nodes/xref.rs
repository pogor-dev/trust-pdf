use crate::{
    GreenCst, GreenDiagnostics, GreenElement, GreenExpressionSyntax, GreenListSyntax, GreenLiteralExpressionSyntax, GreenNode, GreenNodeSyntax, GreenToken,
    GreenTrait, SyntaxKind,
};

/// Cross-reference table: xref sections with entries
/// ISO 32000-2:2020, 7.5.4 — Cross-reference table
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

/// Single xref section with subsections
/// ISO 32000-2:2020, 7.5.4 — Cross-reference table
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

impl GreenCst for GreenXRefSectionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::XRefSectionExpression && node.slot_count() == 1
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenXRefSectionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// xref subsection with contiguous entry range
/// ISO 32000-2:2020, 7.5.4 — Cross-reference table
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

impl GreenCst for GreenXRefSubSectionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::XRefSubSectionExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenXRefSubSectionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// xref entry: byte-offset generation-number in-use-flag
/// ISO 32000-2:2020, 7.5.4 — Cross-reference table
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

impl GreenCst for GreenXRefEntryExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::XRefEntryExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenXRefEntryExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
