use crate::{
    GreenCst, GreenDiagnostics, GreenElement, GreenExpressionSyntax, GreenLiteralExpressionSyntax, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait,
    SyntaxKind,
};

#[derive(Clone)]
pub struct GreenPdfVersionSyntax(GreenExpressionSyntax);

impl GreenPdfVersionSyntax {
    pub fn new(kind: SyntaxKind, major_version_token: GreenToken, minor_version_token: GreenToken, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Token(major_version_token), GreenElement::Token(minor_version_token)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenPdfVersionSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn major_version_token(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn minor_version_token(&self) -> Option<GreenLiteralExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenLiteralExpressionSyntax::cast(n),
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
