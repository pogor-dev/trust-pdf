use crate::{GreenCst, GreenDiagnostic, GreenNode, GreenNodeElement, GreenNodeSyntax, SyntaxKind};

#[derive(Clone)]
pub(crate) struct GreenExpressionSyntax(pub GreenNode);

impl GreenNodeSyntax for GreenExpressionSyntax {
    #[inline]
    fn green(&self) -> &GreenNode {
        &self.0
    }
}

#[derive(Clone)]
pub(crate) struct GreenListSyntax(GreenExpressionSyntax);

impl GreenListSyntax {
    pub(crate) fn new(kind: SyntaxKind, elements: Vec<GreenNodeElement>, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let green = GreenNode::new_with_diagnostic(kind, elements, diagnostics);
        GreenListSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn elements(&self) -> Option<GreenNode> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => Some(n.clone()),
            _ => None,
        }
    }
}

impl GreenCst for GreenListSyntax {
    fn can_cast(node: &crate::GreenNode) -> bool {
        node.kind() == SyntaxKind::List
    }

    fn cast(node: crate::GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenListSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
