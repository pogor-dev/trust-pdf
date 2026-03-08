use crate::{
    GreenCst, GreenDiagnostic, GreenExpressionSyntax, GreenListSyntax, GreenLiteralExpressionSyntax, GreenNode, GreenNodeElement, GreenNodeSyntax,
    GreenTokenElement, SyntaxKind,
};

/// Cross-reference table: xref sections with entries
/// ISO 32000-2:2020, 7.5.4 — Cross-reference table
#[derive(Clone)]
pub(crate) struct GreenXRefTableExpressionSyntax(GreenExpressionSyntax);

impl GreenXRefTableExpressionSyntax {
    pub(crate) fn new(kind: SyntaxKind, sections: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![sections];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenXRefTableExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn sections(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
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
pub(crate) struct GreenXRefSectionSyntax(GreenExpressionSyntax);

impl GreenXRefSectionSyntax {
    pub(crate) fn new(kind: SyntaxKind, subsections: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![subsections];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenXRefSectionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn subsections(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
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
pub(crate) struct GreenXRefSubSectionSyntax(GreenExpressionSyntax);

impl GreenXRefSubSectionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        start_object_number: GreenNodeElement,
        entry_count: GreenNodeElement,
        entries: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![start_object_number, entry_count, entries];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenXRefSubSectionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn start_object_number(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn entry_count(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn entries(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
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
pub(crate) struct GreenXRefEntryExpressionSyntax(GreenExpressionSyntax);

impl GreenXRefEntryExpressionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        byte_offset: GreenNodeElement,
        generation_number: GreenNodeElement,
        in_use_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![byte_offset, generation_number, in_use_token];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenXRefEntryExpressionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn byte_offset(&self) -> Option<GreenLiteralExpressionSyntax> {
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
    pub(crate) fn in_use_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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
