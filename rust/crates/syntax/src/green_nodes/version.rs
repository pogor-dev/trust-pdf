use crate::{
    GreenCst, GreenDiagnostics, GreenElement, GreenExpressionSyntax, GreenLiteralExpressionSyntax, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait,
    SyntaxKind,
};

#[derive(Clone)]
pub struct GreenPdfVersionExpressionSyntax(GreenExpressionSyntax);

impl GreenPdfVersionExpressionSyntax {
    pub fn new(kind: SyntaxKind, major_version_token: GreenToken, minor_version_token: GreenToken, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Token(major_version_token), GreenElement::Token(minor_version_token)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenPdfVersionExpressionSyntax(GreenExpressionSyntax(green))
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
