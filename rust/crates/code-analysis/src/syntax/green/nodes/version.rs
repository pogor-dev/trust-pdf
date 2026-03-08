use crate::{GreenCst, GreenDiagnostic, GreenExpressionSyntax, GreenLiteralExpressionSyntax, GreenNode, GreenNodeElement, GreenNodeSyntax, SyntaxKind};

/// PDF version: %PDF-major.minor
/// ISO 32000-2:2020, 7.5.2 — File header
#[derive(Clone)]
pub(crate) struct GreenPdfVersionSyntax(GreenExpressionSyntax);

impl GreenPdfVersionSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        major_version_token: GreenNodeElement,
        minor_version_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![major_version_token, minor_version_token];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenPdfVersionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn major_version_token(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn minor_version_token(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }
}

impl GreenCst for GreenPdfVersionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::PdfVersionExpression && node.slot_count() == 2
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenPdfVersionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
